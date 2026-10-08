//! # Launch requests
//!
//! WingDrive is opened by `xdg-open`, desktop launchers and other apps with
//! folder paths, file paths or `file://` URIs. This module turns those
//! arguments into open requests and hands them to the frontend, which opens
//! each folder in a tab. Requests are queued rather than only emitted because
//! the first ones arrive before the webview has registered its listener; the
//! event just tells the frontend to drain the queue.
//!
//! ## Example
//! ```rust,ignore
//! let options = launch::parse_args(std::env::args().skip(1), &std::env::current_dir()?);
//! launch::deliver(&app, options.requests);
//! ```

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

/// Event telling the frontend that `take_open_requests` has new entries.
pub const OPEN_REQUESTS_EVENT: &str = "open-requests";

/// A folder to open, optionally with one of its entries selected.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct OpenRequest {
	pub directory: PathBuf,
	pub select: Option<PathBuf>,
}

/// Parsed command line.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct LaunchOptions {
	/// Start in the tray without showing a window, used by autostart.
	pub hidden: bool,
	pub requests: Vec<OpenRequest>,
}

/// Requests waiting for the frontend.
pub struct PendingOpenRequests(Mutex<Vec<OpenRequest>>);

impl PendingOpenRequests {
	pub fn new(requests: Vec<OpenRequest>) -> Self {
		Self(Mutex::new(requests))
	}
}

/// Set while a `--hidden` launch must keep the main window hidden once.
#[derive(Default)]
pub struct StartHidden(pub AtomicBool);

impl StartHidden {
	/// Whether the launch asked to stay hidden and has not been consumed yet.
	pub fn is_set(&self) -> bool {
		self.0.load(Ordering::SeqCst)
	}

	/// Returns true the first time it is asked after a hidden launch.
	pub fn consume(&self) -> bool {
		self.0.swap(false, Ordering::SeqCst)
	}
}

/// D-Bus base id for the single-instance plugin.
///
/// A named instance (`WINGDRIVE_INSTANCE`) gets its own id, so a test or
/// development run is single-instance among its own launches without handing
/// them to the user's everyday WingDrive.
pub fn single_instance_id(instance: Option<&str>) -> String {
	const BASE: &str = "com.wingdrive.desktop";
	match instance {
		None => BASE.to_string(),
		Some(name) => {
			// Bus name elements allow only ASCII letters, digits and `_`.
			let element: String = name
				.chars()
				.map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
				.collect();
			format!("{BASE}.i{element}")
		}
	}
}

/// Hands this launch to a running WingDrive and returns true when it took it.
///
/// The single-instance plugin does the same, but only after GTK and the
/// window system are up, which costs most of a resident open. Calling its
/// D-Bus method first lets `wingdrive <dir>` return before any of that.
/// When no instance owns the name the call fails at once and start-up goes on;
/// the plugin still catches a launch that races a starting instance.
#[cfg(target_os = "linux")]
pub fn forward_to_running_instance(id: &str, cwd: &Path) -> bool {
	let Ok(connection) = zbus::blocking::Connection::session() else {
		return false;
	};
	let argv: Vec<String> = std::env::args().collect();
	connection
		.call_method(
			Some(format!("{id}.SingleInstance").as_str()),
			// The plugin derives its object path from the name the same way.
			format!("/{}/SingleInstance", id.replace('.', "/")).as_str(),
			Some("org.SingleInstance.DBus"),
			"ExecuteCallback",
			&(argv, cwd.to_string_lossy().into_owned()),
		)
		.is_ok()
}

/// Parses launcher arguments, without the program name.
///
/// Relative paths resolve against `cwd`, which for a forwarded launch is the
/// second process's directory. Arguments that do not name an existing path
/// are skipped, so unknown flags from desktop environments do nothing.
pub fn parse_args(args: impl IntoIterator<Item = String>, cwd: &Path) -> LaunchOptions {
	let mut options = LaunchOptions::default();
	for arg in args {
		if arg == "--hidden" {
			options.hidden = true;
			continue;
		}
		let Some(path) = local_path(&arg, cwd) else {
			continue;
		};
		match std::fs::metadata(&path) {
			Ok(metadata) if metadata.is_dir() => options.requests.push(OpenRequest {
				directory: path,
				select: None,
			}),
			Ok(_) => options.requests.push(reveal_request(path)),
			Err(error) => tracing::warn!(?path, %error, "Ignoring launch path"),
		}
	}
	options
}

/// Opens the parent folder of `path` with `path` selected.
pub fn reveal_request(path: PathBuf) -> OpenRequest {
	let directory = path
		.parent()
		.map(Path::to_path_buf)
		.unwrap_or_else(|| path.clone());
	OpenRequest {
		directory,
		select: Some(path),
	}
}

/// Resolves a path argument or local `file://` URI against `cwd`.
pub fn local_path(arg: &str, cwd: &Path) -> Option<PathBuf> {
	let path = if let Some(rest) = arg.strip_prefix("file://") {
		// Only local URIs: "file:///path" or "file://localhost/path".
		let rest = rest.strip_prefix("localhost").unwrap_or(rest);
		if !rest.starts_with('/') {
			return None;
		}
		PathBuf::from(percent_decode(rest)?)
	} else if arg.starts_with('-') || arg.contains("://") {
		return None;
	} else {
		PathBuf::from(arg)
	};
	let path = if path.is_absolute() {
		path
	} else {
		cwd.join(path)
	};
	Some(normalize(&path))
}

/// Removes `.` and `..` components without touching the file system, so a
/// symlinked folder keeps the path the user typed.
fn normalize(path: &Path) -> PathBuf {
	let mut out = PathBuf::new();
	for component in path.components() {
		match component {
			std::path::Component::CurDir => {}
			std::path::Component::ParentDir => {
				out.pop();
			}
			other => out.push(other),
		}
	}
	out
}

fn percent_decode(input: &str) -> Option<String> {
	let bytes = input.as_bytes();
	let mut out = Vec::with_capacity(bytes.len());
	let mut i = 0;
	while i < bytes.len() {
		if bytes[i] == b'%' {
			let hex = std::str::from_utf8(bytes.get(i + 1..i + 3)?).ok()?;
			out.push(u8::from_str_radix(hex, 16).ok()?);
			i += 3;
		} else {
			out.push(bytes[i]);
			i += 1;
		}
	}
	String::from_utf8(out).ok()
}

/// Queues requests for the frontend and brings the main window forward.
///
/// An empty list still shows the window, because launching WingDrive again
/// while it sits in the tray means the user wants to see it.
pub fn deliver(app: &AppHandle, requests: Vec<OpenRequest>) {
	if !requests.is_empty() {
		tracing::info!(count = requests.len(), "Delivering open requests");
		if let Some(pending) = app.try_state::<PendingOpenRequests>() {
			pending.0.lock().unwrap().extend(requests);
		}
		let _ = app.emit(OPEN_REQUESTS_EVENT, ());
	}
	show_main_window(app);
}

/// Shows, unminimizes and focuses the main window.
pub fn show_main_window(app: &AppHandle) {
	if let Some(window) = app.get_webview_window("main") {
		let _ = window.show();
		let _ = window.unminimize();
		let _ = window.set_focus();
	}
}

/// Takes the first queued request when it only opens a folder.
///
/// The main window calls this before its first render, so a cold launch with
/// a folder paints that folder instead of the overview it would otherwise
/// show while the frontend drains the queue. A request that selects an entry
/// stays queued for the frontend, which also handles the reveal.
#[tauri::command]
pub fn take_initial_open_request(
	state: tauri::State<'_, PendingOpenRequests>,
) -> Option<OpenRequest> {
	let mut queue = state.0.lock().unwrap();
	match queue.first() {
		Some(request) if request.select.is_none() => Some(queue.remove(0)),
		_ => None,
	}
}

/// Returns and clears the queued open requests.
#[tauri::command]
pub fn take_open_requests(state: tauri::State<'_, PendingOpenRequests>) -> Vec<OpenRequest> {
	std::mem::take(&mut *state.0.lock().unwrap())
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn named_instances_get_their_own_bus_name() {
		assert_eq!(single_instance_id(None), "com.wingdrive.desktop");
		assert_eq!(
			single_instance_id(Some("release-181741")),
			"com.wingdrive.desktop.irelease_181741"
		);
		// A leading digit would be an invalid bus name element without the prefix.
		assert!(zbus::names::WellKnownName::try_from(format!(
			"{}.SingleInstance",
			single_instance_id(Some("1.dev"))
		))
		.is_ok());
	}

	fn parse(args: &[&str], cwd: &Path) -> LaunchOptions {
		parse_args(args.iter().map(|a| a.to_string()), cwd)
	}

	#[test]
	fn folders_files_uris_and_flags() {
		let root = tempfile::tempdir().unwrap();
		let dir = root.path().join("My Folder");
		std::fs::create_dir(&dir).unwrap();
		let file = dir.join("notes.txt");
		std::fs::write(&file, "").unwrap();
		let uri = format!("file://{}", file.display()).replace(' ', "%20");

		let options = parse(
			&[
				"--hidden",
				"My Folder",
				&uri,
				"--unknown",
				"missing",
				"https://x",
			],
			root.path(),
		);

		assert!(options.hidden);
		assert_eq!(
			options.requests,
			vec![
				OpenRequest {
					directory: dir.clone(),
					select: None
				},
				OpenRequest {
					directory: dir.clone(),
					select: Some(file)
				},
			]
		);
	}

	#[test]
	fn relative_parent_components_are_normalized() {
		let root = tempfile::tempdir().unwrap();
		let sub = root.path().join("a");
		std::fs::create_dir(&sub).unwrap();
		let options = parse(&["../a/./"], &sub);
		assert_eq!(options.requests[0].directory, sub);
	}

	#[test]
	fn malformed_uris_are_ignored() {
		let options = parse(&["file://host/share", "file:///bad%zz"], Path::new("/"));
		assert!(options.requests.is_empty());
	}
}
