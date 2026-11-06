//! # org.freedesktop.FileManager1
//!
//! Browsers, editors and the desktop portal reveal files by calling
//! `org.freedesktop.FileManager1` on the session bus. Owning that name while
//! WingDrive runs makes "Show in folder" from any app open WingDrive with the
//! file selected. The name is requested with replacement and queuing, so a
//! file manager that already holds it either hands it over or passes it on
//! when it exits.
//!
//! The name belongs to whichever file manager the user chose. WingDrive only
//! claims it when it is the default handler for `inode/directory` (or when
//! `WINGDRIVE_FILEMANAGER1=1` forces it); otherwise it would take "Show in
//! folder" away from Dolphin, Nautilus, Thunar, Nemo or PCManFM just by
//! running in the tray. Packages ship no D-Bus activation file for the name
//! either: two activatable providers make dbus-broker drop one of them with a
//! warning on every reload, and the system default file manager should win.
//!
//! ## Example
//! ```rust,ignore
//! tauri::async_runtime::spawn(filemanager1::serve(app.handle().clone()));
//! ```

use std::path::PathBuf;

use tauri::AppHandle;
use zbus::fdo::RequestNameFlags;

use crate::launch::{self, OpenRequest};

const BUS_NAME: &str = "org.freedesktop.FileManager1";
const OBJECT_PATH: &str = "/org/freedesktop/FileManager1";

struct FileManager1 {
	app: AppHandle,
}

#[zbus::interface(name = "org.freedesktop.FileManager1")]
impl FileManager1 {
	/// Opens each folder in a tab.
	#[zbus(name = "ShowFolders")]
	fn show_folders(&self, uris: Vec<String>, _startup_id: String) {
		let requests = local_paths(&uris)
			.into_iter()
			.map(|path| match path.is_dir() {
				true => OpenRequest {
					directory: path,
					select: None,
				},
				false => launch::reveal_request(path),
			})
			.collect();
		launch::deliver(&self.app, requests);
	}

	/// Opens the parent folder of each item with the item selected.
	#[zbus(name = "ShowItems")]
	fn show_items(&self, uris: Vec<String>, _startup_id: String) {
		let requests = local_paths(&uris)
			.into_iter()
			.map(launch::reveal_request)
			.collect();
		launch::deliver(&self.app, requests);
	}

	/// Selects the items; the Inspector shows the selection's properties.
	#[zbus(name = "ShowItemProperties")]
	fn show_item_properties(&self, uris: Vec<String>, startup_id: String) {
		self.show_items(uris, startup_id);
	}
}

/// Keeps only existing local paths; remote URIs have no local folder to open.
fn local_paths(uris: &[String]) -> Vec<PathBuf> {
	uris.iter()
		.filter_map(|uri| launch::local_path(uri, std::path::Path::new("/")))
		.filter(|path| path.exists())
		.collect()
}

/// Environment override: `1`/`true`/`on` always serves, `0`/`false`/`off` never.
pub const OVERRIDE_ENV: &str = "WINGDRIVE_FILEMANAGER1";

/// Whether WingDrive should own the name.
///
/// `default_handler` is the desktop file ID the user set for
/// `inode/directory`; WingDrive serves only when that is one of its own.
pub fn should_serve(override_value: Option<&str>, default_handler: Option<&str>) -> bool {
	match override_value.map(|value| value.trim().to_ascii_lowercase()) {
		Some(value) if matches!(value.as_str(), "1" | "true" | "on" | "yes") => return true,
		Some(value) if matches!(value.as_str(), "0" | "false" | "off" | "no") => return false,
		_ => {}
	}
	default_handler.is_some_and(|handler| {
		crate::notifications::DESKTOP_ENTRY_CANDIDATES
			.iter()
			.any(|id| handler.strip_suffix(".desktop") == Some(id))
	})
}

/// Serves FileManager1 for the app's lifetime when WingDrive is the user's
/// file manager.
///
/// Failure only costs the integration, so it is logged and the app goes on.
pub async fn serve(app: AppHandle) {
	let override_value = std::env::var(OVERRIDE_ENV).ok();
	let default_handler = tauri::async_runtime::spawn_blocking(|| {
		file_opening_linux::default_app_for_mime("inode/directory")
	})
	.await
	.ok()
	.flatten();
	if !should_serve(override_value.as_deref(), default_handler.as_deref()) {
		tracing::info!(
			default = default_handler.as_deref().unwrap_or("none"),
			"Leaving {BUS_NAME} to the default file manager"
		);
		return;
	}
	if let Err(error) = try_serve(app).await {
		tracing::warn!(%error, "org.freedesktop.FileManager1 unavailable");
	}
}

async fn try_serve(app: AppHandle) -> zbus::Result<()> {
	let connection = zbus::connection::Builder::session()
		.serve_at(OBJECT_PATH, FileManager1 { app })
		.build()
		.await?;
	let reply = connection
		.request_name_with_flags(
			BUS_NAME,
			RequestNameFlags::REPLACE_EXISTING | RequestNameFlags::ALLOW_REPLACEMENT,
		)
		.await?;
	tracing::info!(?reply, "Requested {BUS_NAME}");
	// The connection serves requests while alive; hold it until the app exits.
	std::future::pending::<()>().await;
	drop(connection);
	Ok(())
}

#[cfg(test)]
mod tests {
	use super::should_serve;

	#[test]
	fn serves_only_when_wingdrive_is_the_default_file_manager() {
		assert!(should_serve(None, Some("wingdrive.desktop")));
		assert!(should_serve(None, Some("WingDrive.desktop")));
		assert!(!should_serve(None, Some("org.kde.dolphin.desktop")));
		assert!(!should_serve(None, Some("org.gnome.Nautilus.desktop")));
		assert!(!should_serve(None, Some("wingdrive-dev.desktop")));
		assert!(!should_serve(None, None));
	}

	#[test]
	fn only_existing_local_uris_become_paths() {
		let dir = tempfile::tempdir().unwrap();
		let file = dir.path().join("a b.txt");
		std::fs::write(&file, "x").unwrap();
		let uri = |path: &std::path::Path| tauri::Url::from_file_path(path).unwrap().to_string();
		let paths = super::local_paths(&[
			uri(&file),
			uri(dir.path()),
			uri(&dir.path().join("missing")),
			"sftp://host/tmp/file".to_string(),
			"not a uri".to_string(),
		]);
		assert_eq!(paths, vec![file, dir.path().to_path_buf()]);
	}

	#[test]
	fn environment_override_wins_both_ways() {
		assert!(should_serve(Some("1"), Some("thunar.desktop")));
		assert!(should_serve(Some(" ON "), None));
		assert!(!should_serve(Some("0"), Some("wingdrive.desktop")));
		assert!(!should_serve(Some("off"), Some("WingDrive.desktop")));
		assert!(should_serve(Some("maybe"), Some("wingdrive.desktop")));
	}
}
