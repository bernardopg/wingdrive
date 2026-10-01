//! Linux-specific event handler
//!
//! Linux inotify provides better rename tracking than macOS FSEvents,
//! but still requires some buffering for reliable handling.

use crate::event::{FsEvent, RawEventKind, RawNotifyEvent};
use crate::platform::EventHandler;
use crate::Result;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::trace;

/// Timeout for event stabilization
const STABILIZATION_TIMEOUT_MS: u64 = 100;

/// Linux event handler
pub struct LinuxHandler {
	/// Files pending stabilization
	pending_updates: RwLock<HashMap<PathBuf, Instant>>,
	/// Unpaired rename sources whose path is gone. inotify reports a rename
	/// as From, To, and a paired Both event; a From that no Both claims
	/// within the timeout moved out of the watched tree.
	pending_moves_out: RwLock<HashMap<PathBuf, Instant>>,
	/// Unpaired rename targets that exist; a To that no Both claims moved in
	/// from outside the watched tree and must be indexed as new.
	pending_moves_in: RwLock<HashMap<PathBuf, Instant>>,
}

impl LinuxHandler {
	/// Create a new Linux handler
	pub fn new() -> Self {
		Self {
			pending_updates: RwLock::new(HashMap::new()),
			pending_moves_out: RwLock::new(HashMap::new()),
			pending_moves_in: RwLock::new(HashMap::new()),
		}
	}

	/// Evict pending updates that have stabilized
	async fn evict_updates(&self, timeout: Duration) -> Vec<FsEvent> {
		let mut events = Vec::new();
		let mut updates = self.pending_updates.write().await;
		let mut to_remove = Vec::new();

		for (path, timestamp) in updates.iter() {
			if timestamp.elapsed() > timeout {
				to_remove.push(path.clone());
				events.push(FsEvent::modify(path.clone()));
				trace!("Evicting update (stabilized): {}", path.display());
			}
		}

		for path in to_remove {
			updates.remove(&path);
		}

		let mut moves = self.pending_moves_out.write().await;
		moves.retain(|path, timestamp| {
			if timestamp.elapsed() <= timeout {
				return true;
			}
			// Still gone after the window: it left the tree
			if std::fs::symlink_metadata(path).is_err() {
				trace!(
					"Unpaired rename source treated as remove: {}",
					path.display()
				);
				events.push(FsEvent::remove(path.clone()));
			}
			false
		});

		let mut moves = self.pending_moves_in.write().await;
		moves.retain(|path, timestamp| {
			if timestamp.elapsed() <= timeout {
				return true;
			}
			if std::fs::symlink_metadata(path).is_ok() {
				trace!(
					"Unpaired rename target treated as create: {}",
					path.display()
				);
				events.push(FsEvent::create(path.clone()));
			}
			false
		});

		events
	}
}

impl Default for LinuxHandler {
	fn default() -> Self {
		Self::new()
	}
}

#[async_trait::async_trait]
impl EventHandler for LinuxHandler {
	async fn process(&self, event: RawNotifyEvent) -> Result<Vec<FsEvent>> {
		let Some(path) = event.primary_path().cloned() else {
			return Ok(vec![]);
		};

		match event.kind {
			RawEventKind::Create => Ok(vec![FsEvent::create(path)]),
			RawEventKind::Remove => Ok(vec![FsEvent::remove(path)]),
			RawEventKind::Modify => {
				// Buffer modifications for stabilization
				let mut updates = self.pending_updates.write().await;
				updates.insert(path, Instant::now());
				Ok(vec![])
			}
			RawEventKind::Rename => {
				// inotify provides rename events with both paths
				if event.paths.len() >= 2 {
					let from = event.paths[0].clone();
					let to = event.paths[1].clone();
					self.pending_moves_out.write().await.remove(&from);
					self.pending_moves_in.write().await.remove(&to);
					Ok(vec![FsEvent::rename(from, to)])
				} else if std::fs::symlink_metadata(&path).is_err() {
					// Rename source whose path is gone: either an in-tree rename
					// whose paired event follows, or a move out of the tree
					// (including to the trash). Decide on tick.
					self.pending_moves_out
						.write()
						.await
						.insert(path, Instant::now());
					Ok(vec![])
				} else {
					// Rename target that exists: either an in-tree rename whose
					// paired event follows, or a move in from outside. A modify
					// would be skipped for an unknown path, so decide on tick.
					self.pending_moves_in
						.write()
						.await
						.insert(path, Instant::now());
					Ok(vec![])
				}
			}
			RawEventKind::Other(ref kind) => {
				trace!("Ignoring unknown event kind: {}", kind);
				Ok(vec![])
			}
		}
	}

	async fn tick(&self) -> Result<Vec<FsEvent>> {
		let timeout = Duration::from_millis(STABILIZATION_TIMEOUT_MS);
		Ok(self.evict_updates(timeout).await)
	}

	async fn reset(&self) {
		self.pending_updates.write().await.clear();
		self.pending_moves_out.write().await.clear();
		self.pending_moves_in.write().await.clear();
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[tokio::test]
	async fn test_handler_creation() {
		let handler = LinuxHandler::new();
		assert!(handler.pending_updates.read().await.is_empty());
	}

	#[tokio::test]
	async fn test_create_event() {
		let handler = LinuxHandler::new();
		let event = RawNotifyEvent {
			kind: RawEventKind::Create,
			paths: vec![PathBuf::from("/test/file.txt")],
			timestamp: std::time::SystemTime::now(),
		};

		let events = handler.process(event).await.unwrap();
		assert_eq!(events.len(), 1);
		assert!(events[0].kind.is_create());
	}

	#[tokio::test]
	async fn test_remove_event() {
		let handler = LinuxHandler::new();
		let event = RawNotifyEvent {
			kind: RawEventKind::Remove,
			paths: vec![PathBuf::from("/test/file.txt")],
			timestamp: std::time::SystemTime::now(),
		};

		let events = handler.process(event).await.unwrap();
		assert_eq!(events.len(), 1);
		assert!(events[0].kind.is_remove());
	}

	#[tokio::test]
	async fn test_rename_event() {
		let handler = LinuxHandler::new();
		let event = RawNotifyEvent {
			kind: RawEventKind::Rename,
			paths: vec![
				PathBuf::from("/test/old.txt"),
				PathBuf::from("/test/new.txt"),
			],
			timestamp: std::time::SystemTime::now(),
		};

		let events = handler.process(event).await.unwrap();
		assert_eq!(events.len(), 1);
		assert!(events[0].kind.is_rename());
	}

	fn rename_event(paths: &[&str]) -> RawNotifyEvent {
		RawNotifyEvent {
			kind: RawEventKind::Rename,
			paths: paths.iter().map(PathBuf::from).collect(),
			timestamp: std::time::SystemTime::now(),
		}
	}

	#[tokio::test]
	async fn test_rename_out_of_tree_is_remove() {
		let handler = LinuxHandler::new();
		let moved = "/nonexistent/wingdrive-test/moved.txt";
		assert!(handler
			.process(rename_event(&[moved]))
			.await
			.unwrap()
			.is_empty());

		let events = handler.evict_updates(Duration::ZERO).await;
		assert_eq!(events.len(), 1);
		assert!(events[0].kind.is_remove());
	}

	#[tokio::test]
	async fn test_rename_into_tree_is_create() {
		let handler = LinuxHandler::new();
		let dir = std::env::temp_dir();
		let moved_in = dir.to_str().unwrap();
		assert!(handler
			.process(rename_event(&[moved_in]))
			.await
			.unwrap()
			.is_empty());

		let events = handler.evict_updates(Duration::ZERO).await;
		assert_eq!(events.len(), 1);
		assert!(events[0].kind.is_create());
	}

	#[tokio::test]
	async fn test_paired_rename_cancels_pending_remove() {
		let handler = LinuxHandler::new();
		let from = "/nonexistent/wingdrive-test/a.txt";
		let to = "/nonexistent/wingdrive-test/b.txt";
		handler.process(rename_event(&[from])).await.unwrap();
		let events = handler.process(rename_event(&[from, to])).await.unwrap();
		assert!(events[0].kind.is_rename());

		let events = handler.evict_updates(Duration::ZERO).await;
		assert!(events.iter().all(|e| !e.kind.is_remove()));
	}
}
