//! # org.freedesktop.FileManager1
//!
//! Browsers, editors and the desktop portal reveal files by calling
//! `org.freedesktop.FileManager1` on the session bus. Owning that name while
//! WingDrive runs makes "Show in folder" from any app open WingDrive with the
//! file selected. The name is requested with replacement and queuing, so a
//! file manager that already holds it either hands it over or passes it on
//! when it exits.
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

/// Serves FileManager1 for the app's lifetime.
///
/// Failure only costs the integration, so it is logged and the app goes on.
pub async fn serve(app: AppHandle) {
	if let Err(error) = try_serve(app).await {
		tracing::warn!(%error, "org.freedesktop.FileManager1 unavailable");
	}
}

async fn try_serve(app: AppHandle) -> zbus::Result<()> {
	let connection = zbus::connection::Builder::session()?
		.serve_at(OBJECT_PATH, FileManager1 { app })?
		.build()
		.await?;
	let reply = connection
		.request_name_with_flags(
			BUS_NAME,
			RequestNameFlags::ReplaceExisting | RequestNameFlags::AllowReplacement,
		)
		.await?;
	tracing::info!(?reply, "Requested {BUS_NAME}");
	// The connection serves requests while alive; hold it until the app exits.
	std::future::pending::<()>().await;
	drop(connection);
	Ok(())
}
