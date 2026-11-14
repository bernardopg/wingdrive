//! # File chooser portal backend
//!
//! Browsers, Flatpak and GTK4 apps ask `xdg-desktop-portal` for a file
//! chooser, and the portal forwards the request to the backend `portals.conf`
//! names for `org.freedesktop.impl.portal.FileChooser`. WingDrive serves that
//! interface on `org.freedesktop.impl.portal.desktop.wingdrive`, so with the
//! Settings switch on, "upload a file" and "Save as" open a WingDrive picker:
//! an Explorer window with a bar for Open, Save and Cancel.
//!
//! Each portal call becomes a pending request in the [`Broker`]. The picker
//! window reads it, and its answer (or its closing) finishes the call with
//! response 0 (chosen), 1 (cancelled or closed) or 2 (error), so the calling
//! app never waits on a window that is gone. Serving the name is harmless
//! while the switch is off: the portal only calls the backend `portals.conf`
//! selects.
//!
//! The name is claimed in `main`, before GTK starts, and the window host is
//! attached later in `setup`. GTK's application startup asks D-Bus to start
//! `org.freedesktop.portal.Desktop` and waits for it, while a portal that is
//! starting waits for every backend it activates. Claiming the name after GTK
//! deadlocked the two until the portal gave up after 25 s and left the
//! session without any file chooser.
//!
//! ## Example
//! ```rust,ignore
//! let broker = file_chooser::start(); // in main, before tauri::Builder
//! // in setup:
//! file_chooser::attach(app.handle(), broker);
//! ```

pub mod options;
pub mod portal;
pub mod portals_conf;

#[cfg(test)]
#[path = "file_chooser/tests/mod.rs"]
mod tests;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Manager, Runtime};
use tokio::sync::{oneshot, watch};
use zbus::OwnedVariant as OwnedValue;

use options::{PickerMode, PickerOptions, Selection, SelectionError};

/// Window labels of pickers start with this; the rest is the request id.
pub const WINDOW_PREFIX: &str = "file-chooser-";

/// Environment override: `0`/`false`/`off`/`no` stops WingDrive from serving
/// the backend name.
pub const OVERRIDE_ENV: &str = "WINGDRIVE_FILE_CHOOSER";

/// A portal call waiting for the user, as the picker window reads it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PickerRequest {
	pub id: String,
	pub mode: PickerMode,
	pub app_id: String,
	pub parent_window: String,
	pub title: String,
	pub options: PickerOptions,
	/// The folder the picker opens in.
	pub start_folder: PathBuf,
	/// The name a save suggests.
	pub suggested_name: Option<String>,
}

/// How a request ended.
#[derive(Debug)]
pub enum Outcome {
	Chosen(HashMap<String, OwnedValue>),
	Cancelled,
	Failed(String),
}

/// Response codes of `org.freedesktop.impl.portal.Request`.
pub mod response {
	pub const SUCCESS: u32 = 0;
	pub const CANCELLED: u32 = 1;
	pub const OTHER: u32 = 2;
}

impl Outcome {
	/// The `(response, results)` pair the portal method returns.
	pub fn into_reply(self) -> (u32, HashMap<String, OwnedValue>) {
		match self {
			Self::Chosen(results) => (response::SUCCESS, results),
			Self::Cancelled => (response::CANCELLED, HashMap::new()),
			Self::Failed(error) => {
				tracing::warn!(%error, "File chooser request failed");
				(response::OTHER, HashMap::new())
			}
		}
	}
}

/// Shows and hides picker windows; tests use a fake one.
pub trait PickerHost: Send + Sync + 'static {
	fn show(&self, request: &PickerRequest) -> Result<(), String>;
	fn dismiss(&self, id: &str);
}

struct Pending {
	request: PickerRequest,
	reply: oneshot::Sender<Outcome>,
}

/// How long a request waits for the window host before it fails.
pub const HOST_WAIT: Duration = Duration::from_secs(60);

/// Pending portal calls, by request id.
pub struct Broker {
	pending: Mutex<HashMap<String, Pending>>,
	next_id: AtomicU64,
	host: OnceLock<Box<dyn PickerHost>>,
	attached: watch::Sender<bool>,
	host_wait: Duration,
}

impl Broker {
	#[cfg(test)]
	pub fn new(host: impl PickerHost) -> Arc<Self> {
		let broker = Self::detached(HOST_WAIT);
		broker.attach(host);
		broker
	}

	/// A broker whose host arrives later through [`Broker::attach`]. Requests
	/// that come first wait up to `host_wait` for it, then fail.
	pub fn detached(host_wait: Duration) -> Arc<Self> {
		Arc::new(Self {
			pending: Mutex::new(HashMap::new()),
			next_id: AtomicU64::new(1),
			host: OnceLock::new(),
			attached: watch::channel(false).0,
			host_wait,
		})
	}

	/// Sets the host once; later calls are ignored and return `false`.
	pub fn attach(&self, host: impl PickerHost) -> bool {
		let attached = self.host.set(Box::new(host)).is_ok();
		if attached {
			self.attached.send_replace(true);
		}
		attached
	}

	async fn host(&self) -> Option<&dyn PickerHost> {
		if let Some(host) = self.host.get() {
			return Some(host.as_ref());
		}
		let mut attached = self.attached.subscribe();
		tokio::time::timeout(self.host_wait, attached.wait_for(|ready| *ready))
			.await
			.ok()?
			.ok()?;
		self.host.get().map(|host| host.as_ref())
	}

	pub fn next_id(&self) -> String {
		self.next_id.fetch_add(1, Ordering::Relaxed).to_string()
	}

	/// Shows a picker for `request` and waits for its outcome.
	pub async fn run(&self, request: PickerRequest) -> Outcome {
		let (reply, outcome) = oneshot::channel();
		let id = request.id.clone();
		self.lock().insert(
			id.clone(),
			Pending {
				request: request.clone(),
				reply,
			},
		);
		let shown = match self.host().await {
			Some(host) => host.show(&request),
			None => Err("WingDrive did not finish starting".to_string()),
		};
		if let Err(error) = shown {
			self.lock().remove(&id);
			return Outcome::Failed(error);
		}
		outcome
			.await
			.unwrap_or_else(|_| Outcome::Failed("the picker went away".to_string()))
	}

	pub fn request(&self, id: &str) -> Option<PickerRequest> {
		self.lock().get(id).map(|pending| pending.request.clone())
	}

	#[cfg(test)]
	pub fn is_pending(&self, id: &str) -> bool {
		self.lock().contains_key(id)
	}

	/// Finishes a request with the user's choice. A selection the request
	/// cannot take, or a save over an existing file not yet confirmed, leaves
	/// the request open and tells the picker why.
	pub fn respond(&self, id: &str, selection: Selection) -> Result<(), SelectionError> {
		let mut pending = self.lock();
		let entry = pending.get(id).ok_or_else(|| SelectionError::Invalid {
			reason: "the request is no longer open".to_string(),
		})?;
		let results = options::results(entry.request.mode, &entry.request.options, selection)?;
		let entry = pending.remove(id).expect("checked above");
		drop(pending);
		entry.reply.send(Outcome::Chosen(results)).ok();
		self.dismiss(id);
		Ok(())
	}

	/// Cancels a request and closes its window.
	pub fn cancel(&self, id: &str) {
		if self.finish(id, Outcome::Cancelled) {
			self.dismiss(id);
		}
	}

	fn dismiss(&self, id: &str) {
		if let Some(host) = self.host.get() {
			host.dismiss(id);
		}
	}

	/// The window is already gone: answer without touching it again.
	pub fn window_closed(&self, id: &str) {
		self.finish(id, Outcome::Cancelled);
	}

	fn finish(&self, id: &str, outcome: Outcome) -> bool {
		let entry = self.lock().remove(id);
		match entry {
			Some(entry) => {
				entry.reply.send(outcome).ok();
				true
			}
			None => false,
		}
	}

	fn lock(&self) -> std::sync::MutexGuard<'_, HashMap<String, Pending>> {
		// A poisoned map still holds valid requests; a panic elsewhere must not
		// leave every app waiting forever.
		self.pending
			.lock()
			.unwrap_or_else(|poison| poison.into_inner())
	}
}

/// The request id of a picker window label.
pub fn request_id(label: &str) -> Option<&str> {
	label
		.strip_prefix(WINDOW_PREFIX)
		.filter(|id| !id.is_empty())
}

/// Whether the environment turns the backend off.
pub fn disabled_by_env(value: Option<&str>) -> bool {
	value.is_some_and(|value| {
		matches!(
			value.trim().to_ascii_lowercase().as_str(),
			"0" | "false" | "off" | "no"
		)
	})
}

struct TauriHost<R: Runtime> {
	app: AppHandle<R>,
}

impl<R: Runtime> PickerHost for TauriHost<R> {
	fn show(&self, request: &PickerRequest) -> Result<(), String> {
		let window = crate::windows::create_file_chooser_window(
			&self.app,
			&format!("{WINDOW_PREFIX}{}", request.id),
			&request.start_folder,
			&request.title,
		)?;
		let app = self.app.clone();
		let id = request.id.clone();
		window.on_window_event(move |event| {
			if let tauri::WindowEvent::Destroyed = event {
				if let Some(broker) = app.try_state::<Arc<Broker>>() {
					broker.window_closed(&id);
				}
			}
		});
		Ok(())
	}

	fn dismiss(&self, id: &str) {
		if let Some(window) = self.app.get_webview_window(&format!("{WINDOW_PREFIX}{id}")) {
			window.destroy().ok();
		}
	}
}

/// Claims the backend name and serves it for the app's lifetime. Call it in
/// `main` before GTK starts (see the module docs), then [`attach`] the app.
///
/// Returns `None` when the environment turns the backend off. Failure to
/// connect only costs the integration, so it is logged and the app goes on.
pub fn start() -> Option<Arc<Broker>> {
	if disabled_by_env(std::env::var(OVERRIDE_ENV).ok().as_deref()) {
		tracing::info!("{OVERRIDE_ENV} turned the file chooser backend off");
		return None;
	}
	let broker = Broker::detached(HOST_WAIT);
	tauri::async_runtime::spawn(serve(broker.clone()));
	Some(broker)
}

/// Gives a started backend its window host and the Tauri commands their broker.
pub fn attach<R: Runtime>(app: &AppHandle<R>, broker: Option<Arc<Broker>>) {
	if let Some(broker) = broker {
		broker.attach(TauriHost { app: app.clone() });
		app.manage(broker);
	}
}

async fn serve(broker: Arc<Broker>) {
	match portal::connect(zbus::connection::Builder::session(), broker).await {
		Ok(connection) => {
			tracing::info!("Serving {}", portal::BUS_NAME);
			// The connection serves requests while alive; hold it until the app exits.
			std::future::pending::<()>().await;
			drop(connection);
		}
		Err(error) => tracing::warn!(%error, "{} unavailable", portal::BUS_NAME),
	}
}

fn broker_for<R: Runtime>(app: &AppHandle<R>) -> Result<Arc<Broker>, String> {
	app.try_state::<Arc<Broker>>()
		.map(|state| state.inner().clone())
		.ok_or_else(|| "the file chooser backend is not running".to_string())
}

fn window_request_id<R: Runtime>(window: &tauri::Window<R>) -> Result<String, String> {
	request_id(window.label())
		.map(str::to_string)
		.ok_or_else(|| "not a file chooser window".to_string())
}

/// The request the calling picker window answers, or `None` when it already ended.
#[tauri::command]
pub fn file_chooser_request<R: Runtime>(
	app: AppHandle<R>,
	window: tauri::Window<R>,
) -> Result<Option<PickerRequest>, String> {
	let id = window_request_id(&window)?;
	Ok(broker_for(&app)?.request(&id))
}

/// Returns the user's choice to the app that asked.
#[tauri::command]
pub fn file_chooser_respond<R: Runtime>(
	app: AppHandle<R>,
	window: tauri::Window<R>,
	selection: Selection,
) -> Result<(), SelectionError> {
	let invalid = |reason: String| SelectionError::Invalid { reason };
	let id = window_request_id(&window).map_err(invalid)?;
	broker_for(&app).map_err(invalid)?.respond(&id, selection)
}

/// Cancels the calling picker's request and closes it.
#[tauri::command]
pub fn file_chooser_cancel<R: Runtime>(
	app: AppHandle<R>,
	window: tauri::Window<R>,
) -> Result<(), String> {
	let id = window_request_id(&window)?;
	broker_for(&app)?.cancel(&id);
	Ok(())
}

/// Where the switch keeps the previous FileChooser value.
pub struct PortalStateFile(pub PathBuf);

fn portal_paths(state: &PortalStateFile) -> portals_conf::PortalPaths {
	portals_conf::PortalPaths::from_env(state.0.clone())
}

/// The Settings switch's state.
#[tauri::command]
pub fn file_chooser_portal_status(
	state: tauri::State<'_, PortalStateFile>,
) -> portals_conf::PortalStatus {
	portal_paths(&state).status()
}

/// Turns the switch on or off, then restarts `xdg-desktop-portal`, which
/// only reads its configuration at start-up.
#[tauri::command]
pub async fn file_chooser_portal_set(
	state: tauri::State<'_, PortalStateFile>,
	enabled: bool,
) -> Result<portals_conf::PortalStatus, String> {
	let status = switch_portal(portal_paths(&state), enabled).await?;
	restart_portal(systemctl_restart()).await;
	Ok(status)
}

/// Edits `portals.conf` off the async runtime, since it reads and writes files.
async fn switch_portal(
	paths: portals_conf::PortalPaths,
	enabled: bool,
) -> Result<portals_conf::PortalStatus, String> {
	tauri::async_runtime::spawn_blocking(move || match enabled {
		true => paths.enable(),
		false => paths.disable(),
	})
	.await
	.map_err(|e| e.to_string())?
}

fn systemctl_restart() -> tokio::process::Command {
	let mut command = tokio::process::Command::new("systemctl");
	command.args(["--user", "try-restart", "xdg-desktop-portal.service"]);
	command
}

/// Best effort: without systemd the change applies at the next login.
/// Returns whether the restart command succeeded.
async fn restart_portal(mut command: tokio::process::Command) -> bool {
	match command.status().await {
		Ok(status) if status.success() => {
			tracing::info!("Restarted xdg-desktop-portal");
			true
		}
		Ok(status) => {
			tracing::warn!(%status, "Could not restart xdg-desktop-portal");
			false
		}
		Err(error) => {
			tracing::warn!(%error, "Could not restart xdg-desktop-portal");
			false
		}
	}
}
