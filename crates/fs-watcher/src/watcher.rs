//! Main filesystem watcher implementation
//!
//! `FsWatcher` is the primary interface for watching filesystem changes.
//! It's storage-agnostic - it only knows about paths and events, not
//! about locations, libraries, or databases.

use crate::config::{WatchConfig, WatcherConfig};
use crate::error::{Result, WatcherError};
#[cfg(target_os = "linux")]
use crate::event::RawEventKind;
use crate::event::{FsEvent, RawNotifyEvent};
use crate::platform::PlatformHandler;
#[cfg(target_os = "linux")]
use crate::prune;
use notify::{RecommendedWatcher, RecursiveMode, Watcher as NotifyWatcher};
/// The mode to hand notify for a watch.
///
/// macOS is the exception and it fails silently. FSEvents has no non-recursive
/// subscription, and notify's backend does not emulate one: a watch registered
/// `NonRecursive` there is accepted, reports no error, and then delivers
/// nothing at all. Measured, not inferred — the same directory watched
/// `Recursive` delivers immediately.
///
/// So macOS always subscribes recursively and the shallow contract is kept
/// above, by callers that already compare an event's parent against the root
/// they asked for. Watching wider than asked costs events that get filtered;
/// watching non-recursively costs every event.
fn notify_mode(recursive: bool) -> RecursiveMode {
	if recursive || cfg!(target_os = "macos") {
		RecursiveMode::Recursive
	} else {
		RecursiveMode::NonRecursive
	}
}

/// Collect the watchable directories of a tree with generated trees omitted.
///
/// Runs on a blocking thread: `read_dir` over hundreds of thousands of
/// entries would stall the async runtime. Symlinked directories are never
/// entered, so a target outside the tree cannot multiply registration, and
/// names are compared exactly so `target` never matches `target_platform`.
#[cfg(target_os = "linux")]
fn walk_pruned(root: PathBuf, skip_paths: Vec<PathBuf>) -> Vec<PathBuf> {
	let mut dirs: Vec<PathBuf> = Vec::new();
	let mut stack: Vec<PathBuf> = vec![root];

	while let Some(dir) = stack.pop() {
		dirs.push(dir.clone());

		let entries = match std::fs::read_dir(&dir) {
			Ok(entries) => entries,
			Err(e) => {
				warn!("Tree walk could not read {}: {}", dir.display(), e);
				continue;
			}
		};

		for entry in entries.flatten() {
			let Ok(file_type) = entry.file_type() else {
				continue;
			};
			if !file_type.is_dir() || file_type.is_symlink() {
				continue;
			}

			let name = entry.file_name();
			if prune::is_generated_dir(&name) {
				continue;
			}

			let path = entry.path();
			if skip_paths.iter().any(|skip| path.starts_with(skip)) {
				continue;
			}

			stack.push(path);
		}
	}

	dirs
}

/// Run [`walk_pruned`] for a root on a blocking thread.
#[cfg(target_os = "linux")]
async fn walk_tree(root: &Path, skip_paths: &[PathBuf]) -> Result<Vec<PathBuf>> {
	let walk_root = root.to_path_buf();
	let skip = skip_paths.to_vec();
	let dirs = tokio::task::spawn_blocking(move || walk_pruned(walk_root, skip))
		.await
		.map_err(|e| WatcherError::WatchFailed {
			path: root.to_path_buf(),
			reason: format!("tree walk task failed: {}", e),
		})?;
	debug!(
		"Pruned tree walk of {} found {} watchable directories",
		root.display(),
		dirs.len()
	);
	Ok(dirs)
}

#[cfg(not(target_os = "linux"))]
use std::collections::HashMap;
#[cfg(target_os = "linux")]
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, RwLock};
use tracing::{debug, error, info, trace, warn};

/// Handle returned when watching a path
///
/// When dropped, the path is automatically unwatched (if no other handles exist).
pub struct WatchHandle {
	path: PathBuf,
	watcher: Arc<FsWatcherInner>,
}

impl std::fmt::Debug for WatchHandle {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("WatchHandle")
			.field("path", &self.path)
			.finish()
	}
}

impl Drop for WatchHandle {
	fn drop(&mut self) {
		// Decrement reference count and unwatch if zero
		let path = self.path.clone();
		let inner = self.watcher.clone();

		// Spawn a task to handle the async unwatch
		tokio::spawn(async move {
			if let Err(e) = inner.release_watch(&path).await {
				warn!("Failed to release watch for {}: {}", path.display(), e);
			}
		});
	}
}

/// Watch state for a path
struct WatchState {
	config: WatchConfig,
	ref_count: usize,
}

/// Internal watcher state
struct FsWatcherInner {
	/// Configuration
	config: WatcherConfig,
	/// Watched paths with reference counting
	watched_paths: RwLock<HashMap<PathBuf, WatchState>>,
	/// Per-directory OS watches registered for a recursive root (Linux).
	///
	/// On Linux a recursive root is watched directory-by-directory through a
	/// pruned walk instead of notify's full recursive walk, so the set keeps
	/// which directories belong to which root for expansion when new
	/// directories appear and for release when the root goes away.
	#[cfg(target_os = "linux")]
	watched_dirs: RwLock<HashMap<PathBuf, HashSet<PathBuf>>>,
	/// The notify watcher instance
	notify_watcher: RwLock<Option<RecommendedWatcher>>,
	/// Platform-specific event handler
	platform_handler: PlatformHandler,
	/// Whether the watcher is running
	is_running: AtomicBool,
	/// Event sender for broadcasts
	event_tx: broadcast::Sender<FsEvent>,
	/// Metrics
	events_received: AtomicU64,
	events_emitted: AtomicU64,
}

impl FsWatcherInner {
	/// Add a watch with reference counting
	async fn add_watch(&self, path: PathBuf, config: WatchConfig) -> Result<()> {
		if Self::increment_ref(&mut *self.watched_paths.write().await, &path) {
			return Ok(());
		}

		// Validate path exists
		if !path.exists() {
			return Err(WatcherError::PathNotFound(path));
		}

		// The tree walk of a recursive root reads every directory under it,
		// so it runs before the lock is taken: event routing and other
		// registrations keep reading `watched_paths` meanwhile.
		#[cfg(target_os = "linux")]
		let tree = if config.recursive && self.is_running.load(Ordering::SeqCst) {
			Some(walk_tree(&path, &config.filters.skip_paths).await?)
		} else {
			None
		};

		let mut watched = self.watched_paths.write().await;
		if Self::increment_ref(&mut watched, &path) {
			return Ok(());
		}

		// Register with notify if we're running
		if self.is_running.load(Ordering::SeqCst) {
			#[cfg(target_os = "linux")]
			match tree {
				Some(dirs) => self.register_tree_dirs(&path, dirs).await,
				None => self.register_os_watch(&path, &config).await?,
			}
			#[cfg(not(target_os = "linux"))]
			self.register_os_watch(&path, &config).await?;
		}

		watched.insert(
			path.clone(),
			WatchState {
				config,
				ref_count: 1,
			},
		);

		debug!("Started watching: {}", path.display());
		Ok(())
	}

	/// Count one more holder of an already watched path.
	///
	/// Returns false when the path is not watched yet.
	fn increment_ref(watched: &mut HashMap<PathBuf, WatchState>, path: &Path) -> bool {
		let Some(state) = watched.get_mut(path) else {
			return false;
		};
		state.ref_count += 1;
		debug!(
			"Incremented ref count for {}: {}",
			path.display(),
			state.ref_count
		);
		true
	}

	/// Register a watched root with the OS-level watcher.
	///
	/// On Linux, a recursive root is registered directory-by-directory
	/// through a pruned walk rather than by notify's own recursive walk:
	/// notify registers every directory under the root, which burns through
	/// the kernel's inotify watch limit inside build outputs and caches and
	/// can take minutes on a large tree, blocking everything that waits for
	/// the registration to finish.
	async fn register_os_watch(&self, path: &Path, config: &WatchConfig) -> Result<()> {
		#[cfg(target_os = "linux")]
		if config.recursive {
			let dirs = walk_tree(path, &config.filters.skip_paths).await?;
			self.register_tree_dirs(path, dirs).await;
			return Ok(());
		}
		#[cfg(not(target_os = "linux"))]
		let _ = config;

		let mut watcher = self.notify_watcher.write().await;
		if let Some(watcher) = watcher.as_mut() {
			watcher
				.watch(path, notify_mode(config.recursive))
				.map_err(|e| WatcherError::WatchFailed {
					path: path.to_path_buf(),
					reason: e.to_string(),
				})?;
		}
		Ok(())
	}

	/// Watch the directories of a recursive root one by one on Linux.
	///
	/// `dirs` comes from [`walk_tree`]. Each directory gets its own
	/// non-recursive watch; directories that appear later are picked up by
	/// the event loop (see `maintain_tree_watch`) the same way notify would
	/// have done with its recursive auto-add.
	#[cfg(target_os = "linux")]
	async fn register_tree_dirs(&self, root: &Path, dirs: Vec<PathBuf>) {
		let mut registered: HashSet<PathBuf> = HashSet::new();
		{
			let mut watcher = self.notify_watcher.write().await;
			if let Some(watcher) = watcher.as_mut() {
				for dir in &dirs {
					// Duplicate registrations merge masks instead of failing,
					// so a directory racing with the walk through the event
					// loop is already watched either way.
					if let Err(e) = watcher.watch(dir, RecursiveMode::NonRecursive) {
						warn!("Failed to create watch for {}: {}", dir.display(), e);
					} else {
						registered.insert(dir.clone());
					}
				}
			}
		}

		self.watched_dirs
			.write()
			.await
			.entry(root.to_path_buf())
			.or_default()
			.extend(registered);
	}

	/// Release a watch (decrement ref count, unwatch if zero)
	async fn release_watch(&self, path: &Path) -> Result<()> {
		let mut watched = self.watched_paths.write().await;

		let should_unwatch = if let Some(state) = watched.get_mut(path) {
			state.ref_count -= 1;
			debug!(
				"Decremented ref count for {}: {}",
				path.display(),
				state.ref_count
			);
			state.ref_count == 0
		} else {
			return Ok(()); // Not watched
		};

		if should_unwatch {
			watched.remove(path);

			// Drop the per-directory watches of a pruned recursive root.
			#[cfg(target_os = "linux")]
			let tree_dirs: Vec<PathBuf> = {
				let mut dirs = self.watched_dirs.write().await;
				dirs.remove(&path.to_path_buf())
					.map(|set| set.into_iter().collect())
					.unwrap_or_default()
			};

			if self.is_running.load(Ordering::SeqCst) {
				let mut watcher = self.notify_watcher.write().await;
				if let Some(watcher) = watcher.as_mut() {
					if let Err(e) = watcher.unwatch(path) {
						warn!("Failed to unwatch {}: {}", path.display(), e);
					}
					#[cfg(target_os = "linux")]
					for dir in tree_dirs {
						if let Err(e) = watcher.unwatch(&dir) {
							trace!("Stale tree watch {}: {}", dir.display(), e);
						}
					}
				}
			}

			debug!("Stopped watching: {}", path.display());
		}

		Ok(())
	}

	/// Find the recursive root that contains a path, if any.
	#[cfg(target_os = "linux")]
	async fn covering_recursive_root(&self, path: &Path) -> Option<(PathBuf, WatchConfig)> {
		let watched = self.watched_paths.read().await;
		watched
			.iter()
			.find(|(root, _)| path.starts_with(*root) && root.as_path() != path)
			.filter(|(_, state)| state.config.recursive)
			.map(|(root, state)| (root.clone(), state.config.clone()))
	}

	/// Keep the OS watch set in sync with raw filesystem events (Linux).
	///
	/// A created or moved-in directory must be watched right away or events
	/// inside it are lost; notify's recursive auto-add only fires for parents
	/// registered `Recursive`, which the pruned walk never does. The removal
	/// side needs no bookkeeping here: notify releases a watch itself when
	/// its directory is deleted (DELETE_SELF) or moved out (MOVE_SELF), and
	/// the old path entry in `watched_dirs` is a stale no-op.
	#[cfg(target_os = "linux")]
	async fn maintain_tree_watch(self: &Arc<Self>, raw: &RawNotifyEvent) {
		if !matches!(raw.kind, RawEventKind::Create | RawEventKind::Rename) {
			return;
		}

		let candidates: Vec<PathBuf> = raw.paths.to_vec();
		let inner = self.clone();
		tokio::spawn(async move {
			for path in candidates {
				// A watch only follows a real directory: symlinked names are
				// refused so their targets cannot multiply the walk.
				// A rename source, or a directory removed right after its
				// creation, is gone; its watch died with it.
				let Ok(metadata) = std::fs::symlink_metadata(&path) else {
					continue;
				};
				if !metadata.is_dir() || metadata.is_symlink() {
					continue;
				}
				inner.expand_tree_watch(&path).await;
			}
		});
	}

	/// Watch a directory that appeared under a recursive root.
	#[cfg(target_os = "linux")]
	async fn expand_tree_watch(&self, path: &Path) {
		let Some((root, config)) = self.covering_recursive_root(path).await else {
			return;
		};

		let Some(name) = path.file_name() else {
			return;
		};
		if prune::is_generated_dir(name) {
			trace!("Refusing to watch generated directory {}", path.display());
			return;
		}
		if config
			.filters
			.skip_paths
			.iter()
			.any(|skip| path.starts_with(skip))
		{
			return;
		}

		// A directory moved in, or created by `mkdir -p` or an extraction,
		// already holds subdirectories whose creation no watch saw, so the
		// whole new tree is walked like notify's recursive auto-add does.
		// Duplicate registrations merge masks in notify instead of failing,
		// so racing with the initial walk registers once either way.
		let dirs = match walk_tree(path, &config.filters.skip_paths).await {
			Ok(dirs) => dirs,
			Err(e) => {
				warn!("Failed to watch new directory {}: {}", path.display(), e);
				return;
			}
		};
		self.register_tree_dirs(&root, dirs).await;
	}
}

/// Platform-agnostic filesystem watcher
///
/// Watches filesystem paths and emits normalized events. Handles platform-specific
/// quirks like macOS rename detection internally.
///
/// # Example
///
/// ```ignore
/// use wing_fs_watcher::{FsWatcher, WatchConfig};
///
/// let watcher = FsWatcher::new(Default::default());
/// watcher.start().await?;
///
/// // Subscribe to events
/// let mut rx = watcher.subscribe();
///
/// // Watch a path
/// let handle = watcher.watch("/path/to/watch", WatchConfig::recursive()).await?;
///
/// // Receive events
/// while let Ok(event) = rx.recv().await {
///     println!("Event: {:?}", event);
/// }
/// ```
pub struct FsWatcher {
	inner: Arc<FsWatcherInner>,
}

impl FsWatcher {
	/// Create a new filesystem watcher
	pub fn new(config: WatcherConfig) -> Self {
		let (event_tx, _) = broadcast::channel(config.event_buffer_size);

		Self {
			inner: Arc::new(FsWatcherInner {
				config,
				watched_paths: RwLock::new(HashMap::new()),
				#[cfg(target_os = "linux")]
				watched_dirs: RwLock::new(HashMap::new()),
				notify_watcher: RwLock::new(None),
				platform_handler: PlatformHandler::new(),
				is_running: AtomicBool::new(false),
				event_tx,
				events_received: AtomicU64::new(0),
				events_emitted: AtomicU64::new(0),
			}),
		}
	}

	/// Start the watcher
	pub async fn start(&self) -> Result<()> {
		if self.inner.is_running.swap(true, Ordering::SeqCst) {
			return Err(WatcherError::AlreadyRunning);
		}

		info!("Starting filesystem watcher");

		// Create channel for raw events from notify
		let (raw_tx, raw_rx) = mpsc::channel(self.inner.config.event_buffer_size);

		// Create the notify watcher
		let raw_tx_clone = raw_tx.clone();
		let inner_clone = self.inner.clone();

		let watcher = RecommendedWatcher::new(
			move |res: std::result::Result<notify::Event, notify::Error>| match res {
				Ok(event) => {
					inner_clone.events_received.fetch_add(1, Ordering::Relaxed);
					let raw_event = RawNotifyEvent::from_notify(event);

					if let Err(e) = raw_tx_clone.try_send(raw_event) {
						error!("Failed to send raw event: {}", e);
					}
				}
				Err(e) => {
					error!("Notify watcher error: {}", e);
				}
			},
			// Match indexing: symlink targets can escape the root and multiply startup traversal.
			notify::Config::default().with_follow_symlinks(false),
		)
		.map_err(|e| WatcherError::StartFailed(e.to_string()))?;

		*self.inner.notify_watcher.write().await = Some(watcher);

		// Register all existing watched paths
		self.register_existing_watches().await?;

		// Start the event processing loop
		self.start_event_loop(raw_rx).await;

		info!("Filesystem watcher started");
		Ok(())
	}

	/// Stop the watcher
	pub async fn stop(&self) -> Result<()> {
		if !self.inner.is_running.swap(false, Ordering::SeqCst) {
			return Ok(()); // Already stopped
		}

		info!("Stopping filesystem watcher");

		// Clear the notify watcher
		*self.inner.notify_watcher.write().await = None;

		// Reset platform handler state
		self.inner.platform_handler.reset().await;

		// Drop the per-directory bookkeeping of pruned tree roots (Linux)
		#[cfg(target_os = "linux")]
		self.inner.watched_dirs.write().await.clear();

		info!("Filesystem watcher stopped");
		Ok(())
	}

	/// Check if the watcher is running
	pub fn is_running(&self) -> bool {
		self.inner.is_running.load(Ordering::SeqCst)
	}

	/// Watch a path
	///
	/// Returns a handle that automatically unwatches when dropped.
	pub async fn watch(&self, path: impl AsRef<Path>, config: WatchConfig) -> Result<WatchHandle> {
		let path = path.as_ref().to_path_buf();
		self.inner.add_watch(path.clone(), config).await?;

		Ok(WatchHandle {
			path,
			watcher: self.inner.clone(),
		})
	}

	/// Watch a path without returning a handle
	///
	/// Use this when you want to manually manage watch lifecycle via `unwatch()`.
	pub async fn watch_path(&self, path: impl AsRef<Path>, config: WatchConfig) -> Result<()> {
		let path = path.as_ref().to_path_buf();
		self.inner.add_watch(path, config).await
	}

	/// Unwatch a path
	pub async fn unwatch(&self, path: impl AsRef<Path>) -> Result<()> {
		self.inner.release_watch(path.as_ref()).await
	}

	/// Get all watched paths
	pub async fn watched_paths(&self) -> Vec<PathBuf> {
		self.inner
			.watched_paths
			.read()
			.await
			.keys()
			.cloned()
			.collect()
	}

	/// Subscribe to filesystem events
	pub fn subscribe(&self) -> broadcast::Receiver<FsEvent> {
		self.inner.event_tx.subscribe()
	}

	/// Get the number of events received
	pub fn events_received(&self) -> u64 {
		self.inner.events_received.load(Ordering::Relaxed)
	}

	/// Get the number of events emitted
	pub fn events_emitted(&self) -> u64 {
		self.inner.events_emitted.load(Ordering::Relaxed)
	}

	/// Register existing watches with notify
	async fn register_existing_watches(&self) -> Result<()> {
		let configs: Vec<(PathBuf, WatchConfig)> = {
			let watched = self.inner.watched_paths.read().await;
			watched
				.iter()
				.map(|(path, state)| (path.clone(), state.config.clone()))
				.collect()
		};

		for (path, config) in configs {
			if let Err(e) = self.inner.register_os_watch(&path, &config).await {
				warn!("Failed to register watch for {}: {}", path.display(), e);
			} else {
				debug!("Registered watch for: {}", path.display());
			}
		}

		Ok(())
	}

	/// Start the event processing loop
	async fn start_event_loop(&self, mut raw_rx: mpsc::Receiver<RawNotifyEvent>) {
		let inner = self.inner.clone();
		let tick_interval = self.inner.config.tick_interval;

		tokio::spawn(async move {
			info!("Event processing loop started");

			// Use interval instead of sleep to ensure periodic ticking even when events are flowing
			let mut tick_timer = tokio::time::interval(tick_interval);
			tick_timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

			loop {
				if !inner.is_running.load(Ordering::SeqCst) {
					break;
				}

				tokio::select! {
					// Process incoming raw events
					Some(raw_event) = raw_rx.recv() => {
						// Watch-set maintenance for Linux trees: expand the
						// pruned watch set when a real directory appears.
						#[cfg(target_os = "linux")]
						inner.maintain_tree_watch(&raw_event).await;

						// Check if path should be filtered
						let should_process = if let Some(path) = raw_event.primary_path() {
							let watched = inner.watched_paths.read().await;
							// Find the watch config for this path
							let config = watched.iter().find(|(watched_path, _)| {
								path.starts_with(watched_path)
							}).map(|(_, state)| &state.config);

							if let Some(config) = config {
								!config.filters.should_skip(path)
							} else {
								true // No filter config, process anyway
							}
						} else {
							false
						};

						if should_process {
							// Process through platform handler
							match inner.platform_handler.process(raw_event).await {
								Ok(events) => {
									for event in events {
										inner.events_emitted.fetch_add(1, Ordering::Relaxed);
										if let Err(e) = inner.event_tx.send(event) {
											trace!("No event subscribers: {}", e);
										}
									}
								}
								Err(e) => {
									error!("Error processing event: {}", e);
								}
							}
						}
					}

					// Periodic tick for buffered event eviction
					_ = tick_timer.tick() => {
						match inner.platform_handler.tick().await {
							Ok(events) => {
								for event in events {
									inner.events_emitted.fetch_add(1, Ordering::Relaxed);
									if let Err(e) = inner.event_tx.send(event) {
										trace!("No event subscribers: {}", e);
									}
								}
							}
							Err(e) => {
								error!("Error during tick: {}", e);
							}
						}
					}
				}
			}

			info!("Event processing loop stopped");
		});
	}
}

impl Default for FsWatcher {
	fn default() -> Self {
		Self::new(WatcherConfig::default())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::time::Duration;
	use tempfile::TempDir;

	#[tokio::test]
	async fn test_watcher_creation() {
		let watcher = FsWatcher::new(WatcherConfig::default());
		assert!(!watcher.is_running());
	}

	#[tokio::test]
	async fn test_watcher_start_stop() {
		let watcher = FsWatcher::new(WatcherConfig::default());

		watcher.start().await.unwrap();
		assert!(watcher.is_running());

		watcher.stop().await.unwrap();
		assert!(!watcher.is_running());
	}

	#[tokio::test]
	async fn test_watch_path() {
		let watcher = FsWatcher::new(WatcherConfig::default());
		watcher.start().await.unwrap();

		let temp_dir = TempDir::new().unwrap();

		watcher
			.watch_path(temp_dir.path(), WatchConfig::recursive())
			.await
			.unwrap();

		let paths = watcher.watched_paths().await;
		assert_eq!(paths.len(), 1);
		assert_eq!(paths[0], temp_dir.path());

		watcher.stop().await.unwrap();
	}

	/// A shallow watch has to actually deliver.
	///
	/// This is the regression that mattered: on macOS a `NonRecursive`
	/// subscription is accepted, reports no error, and silently delivers
	/// nothing, so every ephemeral browse watch was registered and dead. The
	/// assertion is deliberately about arrival rather than about the mode we
	/// hand notify, since the mode is the workaround and arrival is the
	/// contract.
	#[tokio::test]
	async fn a_shallow_watch_delivers_events_for_immediate_children() {
		let watcher = FsWatcher::new(WatcherConfig::default());
		watcher.start().await.unwrap();

		// Default filters exclude paths containing `.tmp`, including TempDir's default prefix.
		let temp_dir = tempfile::Builder::new()
			.prefix("wing-watch-")
			.tempdir()
			.unwrap();
		let mut events = watcher.subscribe();

		let _handle = watcher
			.watch(temp_dir.path(), WatchConfig::shallow())
			.await
			.unwrap();

		// The backend needs a moment before it is actually subscribed.
		tokio::time::sleep(Duration::from_millis(300)).await;
		let target = temp_dir.path().join("appeared.txt");
		std::fs::write(&target, b"hello").unwrap();
		// The reported path is canonical, and a macOS temp dir reaches its
		// contents through a symlink, so the two forms have to be reconciled
		// before they can be compared.
		let target = target.canonicalize().unwrap();

		let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
		loop {
			let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
			assert!(
				!remaining.is_zero(),
				"a shallow watch delivered no event for a file created directly under it"
			);
			match tokio::time::timeout(remaining, events.recv()).await {
				Ok(Ok(event)) if event.path == target => break,
				Ok(Ok(_)) => continue,
				Ok(Err(_)) | Err(_) => panic!(
					"a shallow watch delivered no event for a file created directly under it"
				),
			}
		}

		watcher.stop().await.unwrap();
	}

	#[cfg(target_os = "linux")]
	#[tokio::test]
	async fn recursive_watch_does_not_follow_directory_symlinks() {
		let root = TempDir::new().unwrap();
		let external = TempDir::new().unwrap();
		std::os::unix::fs::symlink(external.path(), root.path().join("linked")).unwrap();
		let child = root.path().join("child");
		std::fs::create_dir(&child).unwrap();

		let watcher = FsWatcher::default();
		watcher.start().await.unwrap();
		let mut events = watcher.subscribe();
		watcher
			.watch_path(
				root.path(),
				WatchConfig::recursive().with_filters(crate::config::EventFilters::allow_all()),
			)
			.await
			.unwrap();

		std::fs::write(external.path().join("external.txt"), b"outside").unwrap();
		let local = child.join("local.txt");
		std::fs::write(&local, b"inside").unwrap();
		let mut saw_local = false;
		let deadline = tokio::time::Instant::now() + Duration::from_secs(1);
		while let Ok(event) = tokio::time::timeout_at(deadline, events.recv()).await {
			let event = event.unwrap();
			assert_ne!(
				event.path.file_name().unwrap(),
				"external.txt",
				"followed a directory symlink: {event:?}"
			);
			saw_local |= event.path == local;
		}
		assert!(saw_local, "recursive watch missed its real child directory");
		watcher.stop().await.unwrap();
	}

	/// Generated build trees are never watched: their contents would
	/// multiply inotify registrations and flood the event stream, and the
	/// indexer excludes them by default anyway. New directories that appear
	/// after the watch was registered must still be picked up, or every
	/// create inside them would go unnoticed.
	#[cfg(target_os = "linux")]
	#[tokio::test]
	async fn recursive_watch_prunes_generated_dirs_and_expands_new_dirs() {
		let root = TempDir::new().unwrap();
		let generated = root.path().join("node_modules");
		std::fs::create_dir_all(generated.join("package")).unwrap();

		let watcher = FsWatcher::default();
		watcher.start().await.unwrap();
		let mut events = watcher.subscribe();
		watcher
			.watch_path(
				root.path(),
				WatchConfig::recursive().with_filters(crate::config::EventFilters::allow_all()),
			)
			.await
			.unwrap();

		// Nothing inside the generated tree is watched, so its events never
		// reach a subscriber.
		std::fs::write(generated.join("package").join("index.js"), b"").unwrap();

		// A real directory that exists at registration is watched; a real
		// directory created after registration expands the watch set.
		let existing = root.path().join("existing");
		std::fs::create_dir(&existing).unwrap();
		let fresh = root.path().join("fresh");
		std::fs::create_dir(&fresh).unwrap();

		// Let the created-directory event reach the event loop and expand.
		tokio::time::sleep(Duration::from_millis(300)).await;

		std::fs::write(existing.join("one.txt"), b"1").unwrap();
		std::fs::write(fresh.join("two.txt"), b"2").unwrap();

		let deadline = tokio::time::Instant::now() + Duration::from_secs(2);
		let mut saw_one = false;
		let mut saw_two = false;
		while let Ok(event) = tokio::time::timeout_at(deadline, events.recv()).await {
			let event = event.unwrap();
			assert!(
				!event.path.starts_with(&generated),
				"generated tree produced events: {event:?}"
			);
			saw_one |= event.path == existing.join("one.txt");
			saw_two |= event.path == fresh.join("two.txt");
		}
		assert!(saw_one, "watch missed a pre-existing directory");
		assert!(saw_two, "watch missed an expanded directory");
		watcher.stop().await.unwrap();
	}

	/// A directory moved into a watched tree arrives with subdirectories
	/// whose creation no watch saw; files created in them later must still
	/// produce events.
	#[cfg(target_os = "linux")]
	#[tokio::test]
	async fn recursive_watch_covers_nested_dirs_of_a_moved_in_tree() {
		let root = TempDir::new().unwrap();
		let outside = TempDir::new().unwrap();
		let tree = outside.path().join("album");
		std::fs::create_dir_all(tree.join("nested").join("deeper")).unwrap();

		let watcher = FsWatcher::default();
		watcher.start().await.unwrap();
		let mut events = watcher.subscribe();
		watcher
			.watch_path(
				root.path(),
				WatchConfig::recursive().with_filters(crate::config::EventFilters::allow_all()),
			)
			.await
			.unwrap();

		let moved = root.path().join("album");
		std::fs::rename(&tree, &moved).unwrap();
		// Let the rename reach the event loop and expand the watch set.
		tokio::time::sleep(Duration::from_millis(300)).await;
		let file = moved.join("nested").join("deeper").join("photo.jpg");
		std::fs::write(&file, b"x").unwrap();

		let deadline = tokio::time::Instant::now() + Duration::from_secs(2);
		let mut saw_file = false;
		while let Ok(event) = tokio::time::timeout_at(deadline, events.recv()).await {
			saw_file |= event.unwrap().path == file;
		}
		assert!(
			saw_file,
			"watch missed a nested directory of a moved-in tree"
		);
		watcher.stop().await.unwrap();
	}

	#[tokio::test]
	async fn test_watch_handle_drops() {
		let watcher = FsWatcher::new(WatcherConfig::default());
		watcher.start().await.unwrap();

		let temp_dir = TempDir::new().unwrap();

		{
			let _handle = watcher
				.watch(temp_dir.path(), WatchConfig::recursive())
				.await
				.unwrap();

			let paths = watcher.watched_paths().await;
			assert_eq!(paths.len(), 1);
		}

		// Give time for the async drop to complete
		tokio::time::sleep(Duration::from_millis(100)).await;

		let paths = watcher.watched_paths().await;
		assert_eq!(paths.len(), 0);

		watcher.stop().await.unwrap();
	}

	#[tokio::test]
	async fn test_reference_counting() {
		let watcher = FsWatcher::new(WatcherConfig::default());
		watcher.start().await.unwrap();

		let temp_dir = TempDir::new().unwrap();

		// Watch the same path twice
		let _handle1 = watcher
			.watch(temp_dir.path(), WatchConfig::recursive())
			.await
			.unwrap();

		let _handle2 = watcher
			.watch(temp_dir.path(), WatchConfig::recursive())
			.await
			.unwrap();

		let paths = watcher.watched_paths().await;
		assert_eq!(paths.len(), 1); // Only one path in the map

		drop(_handle1);
		tokio::time::sleep(Duration::from_millis(100)).await;

		// Should still be watched (handle2 exists)
		let paths = watcher.watched_paths().await;
		assert_eq!(paths.len(), 1);

		drop(_handle2);
		tokio::time::sleep(Duration::from_millis(100)).await;

		// Now should be unwatched
		let paths = watcher.watched_paths().await;
		assert_eq!(paths.len(), 0);

		watcher.stop().await.unwrap();
	}

	#[tokio::test]
	async fn test_file_deletion_events() {
		let _ = tracing_subscriber::fmt()
			.with_env_filter("wing_fs_watcher=debug")
			.try_init();

		let watcher = FsWatcher::new(WatcherConfig::default());
		watcher.start().await.unwrap();

		// Use home directory instead of temp - macOS FSEvents doesn't watch temp dirs
		let home = std::env::var("HOME").unwrap();
		let test_dir = PathBuf::from(home).join("WING_FS_WATCHER_TEST");
		if test_dir.exists() {
			std::fs::remove_dir_all(&test_dir).unwrap();
		}
		std::fs::create_dir_all(&test_dir).unwrap();
		let test_file = test_dir.join("test.txt");

		// Subscribe BEFORE watching to ensure we get all events
		let mut rx = watcher.subscribe();

		// Watch the directory
		watcher
			.watch_path(&test_dir, WatchConfig::recursive())
			.await
			.unwrap();

		// Give the watcher time to settle
		tokio::time::sleep(Duration::from_millis(200)).await;

		// Drain any startup events
		while let Ok(_) = rx.try_recv() {}

		// Create a file
		std::fs::write(&test_file, "initial content").unwrap();
		println!("Created file: {}", test_file.display());

		// macOS buffers creates for 500ms for rename detection, plus some processing time
		// Wait for create event with generous timeout
		let create_event = tokio::time::timeout(Duration::from_secs(3), async {
			loop {
				match rx.recv().await {
					Ok(event) => {
						println!(
							"Received event: {:?} for path: {}",
							event.kind,
							event.path.display()
						);
						// Match by filename to handle /var vs /private/var differences
						if event.path.file_name() == test_file.file_name() {
							return event;
						}
					}
					Err(e) => {
						println!("Event recv error: {}", e);
						panic!("Broadcast channel closed: {}", e);
					}
				}
			}
		})
		.await
		.expect("Timeout waiting for create event");

		println!("Got create event: {:?}", create_event.kind);
		assert!(
			matches!(create_event.kind, crate::event::FsEventKind::Create),
			"Expected Create event, got {:?}",
			create_event.kind
		);

		// Give a moment for any additional events to settle, then drain them. Writing the
		// file emits IN_CREATE *and* IN_MODIFY; leaving the latter queued would make the
		// post-delete assertion below read the stale write event instead of the deletion.
		tokio::time::sleep(Duration::from_millis(200)).await;
		while let Ok(event) = rx.try_recv() {
			println!("Draining pre-delete event: {:?}", event.kind);
		}

		// Delete the file
		std::fs::remove_file(&test_file).unwrap();
		println!("Deleted file: {}", test_file.display());

		// Wait for remove event (NOT create!)
		let delete_event = tokio::time::timeout(Duration::from_secs(5), async {
			loop {
				match rx.recv().await {
					Ok(event) if event.path == test_file => {
						println!(
							"Received event after delete: {:?} for {}",
							event.kind,
							event.path.display()
						);
						return event;
					}
					Ok(event) => {
						println!(
							"Ignoring unrelated event after delete: {:?} for {}",
							event.kind,
							event.path.display()
						);
					}
					Err(e) => {
						println!("Event recv error after delete: {}", e);
						break;
					}
				}
			}
			panic!("No delete event received within timeout");
		})
		.await
		.expect("Timeout waiting for delete event");

		println!("Got delete event: {:?}", delete_event.kind);

		// This is the critical assertion - we should get a Remove event, not a Create event
		assert!(
			matches!(delete_event.kind, crate::event::FsEventKind::Remove),
			"BUG: Expected Remove event after file deletion, but got {:?}. \
			This indicates the watcher is misreporting deletions as creates.",
			delete_event.kind
		);

		watcher.stop().await.unwrap();

		// Cleanup
		let _ = std::fs::remove_dir_all(&test_dir);
	}

	#[tokio::test]
	async fn test_file_modify_then_delete() {
		let _ = tracing_subscriber::fmt()
			.with_env_filter("wing_fs_watcher=debug")
			.try_init();

		let watcher = FsWatcher::new(WatcherConfig::default());
		watcher.start().await.unwrap();

		// Use home directory instead of temp - macOS FSEvents doesn't watch temp dirs
		let home = std::env::var("HOME").unwrap();
		let test_dir = PathBuf::from(home).join("WING_FS_WATCHER_TEST_2");
		if test_dir.exists() {
			std::fs::remove_dir_all(&test_dir).unwrap();
		}
		std::fs::create_dir_all(&test_dir).unwrap();
		let test_file = test_dir.join("document.txt");

		let mut rx = watcher.subscribe();

		watcher
			.watch_path(&test_dir, WatchConfig::recursive())
			.await
			.unwrap();

		tokio::time::sleep(Duration::from_millis(200)).await;

		// Drain any startup events
		while let Ok(_) = rx.try_recv() {}

		// Create file
		std::fs::write(&test_file, "Hello World").unwrap();
		println!("Created file: {}", test_file.display());

		// Wait for create event
		let _create = tokio::time::timeout(Duration::from_secs(2), async {
			loop {
				if let Ok(event) = rx.recv().await {
					if event.path == test_file
						&& matches!(event.kind, crate::event::FsEventKind::Create)
					{
						println!("Got create event");
						return;
					}
				}
			}
		})
		.await
		.expect("Timeout waiting for create");

		tokio::time::sleep(Duration::from_millis(200)).await;

		// Modify file using tokio::fs::write (like the failing test does)
		tokio::fs::write(&test_file, "Hello World - Updated!")
			.await
			.unwrap();
		println!("Modified file: {}", test_file.display());

		// Collect modify events (could be Create or Modify depending on platform)
		tokio::time::sleep(Duration::from_millis(500)).await;
		while let Ok(event) = rx.try_recv() {
			if event.path == test_file {
				println!("Got modify-related event: {:?}", event.kind);
			}
		}

		// Delete file
		tokio::fs::remove_file(&test_file).await.unwrap();
		println!("Deleted file: {}", test_file.display());

		// Wait for delete event
		let delete_event = tokio::time::timeout(Duration::from_secs(5), async {
			loop {
				match rx.recv().await {
					Ok(event) if event.path == test_file => {
						println!(
							"Received event after delete: {:?} for {}",
							event.kind,
							event.path.display()
						);
						return event;
					}
					Ok(event) => {
						println!(
							"Ignoring event: {:?} for {}",
							event.kind,
							event.path.display()
						);
					}
					Err(_) => break,
				}
			}
			panic!("No delete event received");
		})
		.await
		.expect("Timeout waiting for delete event");

		// Critical assertion - this mimics what the integration test does
		assert!(
			matches!(delete_event.kind, crate::event::FsEventKind::Remove),
			"BUG: After create->modify->delete sequence, expected Remove event but got {:?}. \
			This reproduces the integration test failure where deletions are reported as Creates.",
			delete_event.kind
		);

		watcher.stop().await.unwrap();

		// Cleanup
		let _ = std::fs::remove_dir_all(&test_dir);
	}
}
