//! The D-Bus side: `org.freedesktop.impl.portal.FileChooser` and one
//! `org.freedesktop.impl.portal.Request` object per call, at the handle the
//! portal passes, so the portal can close a picker the app gave up on.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use zbus::object_server::ObjectServer;
use zbus::{OwnedObjectPath, OwnedVariant as OwnedValue};

use super::options::{self, PickerMode};
use super::{Broker, Outcome, PickerRequest};

/// The backend's bus name; `wingdrive.portal` names it in `DBusName`.
pub const BUS_NAME: &str = "org.freedesktop.impl.portal.desktop.wingdrive";
/// Every portal backend serves its interfaces here.
pub const OBJECT_PATH: &str = "/org/freedesktop/portal/desktop";

/// Where shared-mime-info keeps its glob table, user data first.
fn globs2_paths() -> Vec<PathBuf> {
	let home = std::env::var_os("HOME")
		.map(PathBuf::from)
		.unwrap_or_default();
	let data_home = std::env::var_os("XDG_DATA_HOME")
		.map(PathBuf::from)
		.unwrap_or_else(|| home.join(".local/share"));
	let data_dirs = std::env::var("XDG_DATA_DIRS")
		.ok()
		.filter(|value| !value.is_empty())
		.unwrap_or_else(|| "/usr/local/share:/usr/share".to_string());
	std::iter::once(data_home)
		.chain(data_dirs.split(':').map(PathBuf::from))
		.map(|dir| dir.join("mime/globs2"))
		.collect()
}

async fn read_globs2() -> String {
	let mut table = String::new();
	for path in globs2_paths() {
		if let Ok(text) = tokio::fs::read_to_string(&path).await {
			table.push_str(&text);
			table.push('\n');
		}
	}
	table
}

pub struct FileChooser {
	broker: Arc<Broker>,
	home: PathBuf,
}

impl FileChooser {
	pub fn new(broker: Arc<Broker>, home: PathBuf) -> Self {
		Self { broker, home }
	}

	#[allow(clippy::too_many_arguments)]
	async fn handle(
		&self,
		server: &ObjectServer,
		mode: PickerMode,
		handle: OwnedObjectPath,
		app_id: String,
		parent_window: String,
		title: String,
		options: HashMap<String, OwnedValue>,
	) -> (u32, HashMap<String, OwnedValue>) {
		let mut parsed = match options::parse(mode, &options) {
			Ok(parsed) => parsed,
			Err(error) => return Outcome::Failed(error.to_string()).into_reply(),
		};
		if parsed.filters.iter().any(|filter| {
			filter
				.rules
				.iter()
				.any(|rule| matches!(rule, options::FilterRule::MimeType(_)))
		}) {
			options::resolve_mime_globs(&mut parsed.filters, &read_globs2().await);
		} else {
			options::resolve_mime_globs(&mut parsed.filters, "");
		}
		let id = self.broker.next_id();
		let request = PickerRequest {
			id: id.clone(),
			mode,
			app_id,
			parent_window,
			title,
			start_folder: parsed.start_folder(&self.home),
			suggested_name: parsed.suggested_name(),
			options: parsed,
		};
		let registered = server
			.at(
				&handle,
				Request {
					broker: self.broker.clone(),
					id: id.clone(),
				},
			)
			.await;
		if let Err(error) = &registered {
			// The portal can still cancel by dropping the call; the picker works.
			tracing::warn!(%error, %handle, "Could not export the request object");
		}
		let reply = self.broker.run(request).await.into_reply();
		// `None` means this call added the object; `Some` means one was already there.
		if matches!(registered, Ok(None)) {
			server.remove::<Request, _>(&handle).await.ok();
		}
		reply
	}
}

#[zbus::interface(name = "org.freedesktop.impl.portal.FileChooser")]
impl FileChooser {
	/// Asks for one or more files or folders to open.
	#[zbus(name = "OpenFile")]
	async fn open_file(
		&self,
		#[zbus(object_server)] server: &ObjectServer,
		handle: OwnedObjectPath,
		app_id: String,
		parent_window: String,
		title: String,
		options: HashMap<String, OwnedValue>,
	) -> (u32, HashMap<String, OwnedValue>) {
		self.handle(
			server,
			PickerMode::Open,
			handle,
			app_id,
			parent_window,
			title,
			options,
		)
		.await
	}

	/// Asks where to save one file.
	#[zbus(name = "SaveFile")]
	async fn save_file(
		&self,
		#[zbus(object_server)] server: &ObjectServer,
		handle: OwnedObjectPath,
		app_id: String,
		parent_window: String,
		title: String,
		options: HashMap<String, OwnedValue>,
	) -> (u32, HashMap<String, OwnedValue>) {
		self.handle(
			server,
			PickerMode::Save,
			handle,
			app_id,
			parent_window,
			title,
			options,
		)
		.await
	}

	/// Asks for a folder that receives several named files.
	#[zbus(name = "SaveFiles")]
	async fn save_files(
		&self,
		#[zbus(object_server)] server: &ObjectServer,
		handle: OwnedObjectPath,
		app_id: String,
		parent_window: String,
		title: String,
		options: HashMap<String, OwnedValue>,
	) -> (u32, HashMap<String, OwnedValue>) {
		self.handle(
			server,
			PickerMode::SaveFiles,
			handle,
			app_id,
			parent_window,
			title,
			options,
		)
		.await
	}
}

/// The handle of one call; `Close` dismisses its picker.
struct Request {
	broker: Arc<Broker>,
	id: String,
}

#[zbus::interface(name = "org.freedesktop.impl.portal.Request")]
impl Request {
	#[zbus(name = "Close")]
	fn close(&self) {
		self.broker.cancel(&self.id);
	}
}

/// Builds a connection that serves the backend and owns [`BUS_NAME`].
pub async fn connect(
	builder: zbus::connection::Builder<'_>,
	broker: Arc<Broker>,
) -> zbus::Result<zbus::Connection> {
	let home = std::env::var_os("HOME")
		.map(PathBuf::from)
		.unwrap_or_else(|| PathBuf::from("/"));
	builder
		.serve_at(OBJECT_PATH, FileChooser::new(broker, home))
		.name(BUS_NAME)
		.build()
		.await
}
