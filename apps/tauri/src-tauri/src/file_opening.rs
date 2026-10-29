use file_opening::{FileOpener, OpenResult, OpenWithApp};
use std::path::PathBuf;
use std::sync::Arc;
use tauri::State;

#[cfg(target_os = "macos")]
use file_opening_macos::MacFileOpener as PlatformOpener;

#[cfg(target_os = "windows")]
use file_opening_windows::WindowsFileOpener as PlatformOpener;

#[cfg(target_os = "linux")]
use file_opening_linux::LinuxFileOpener as PlatformOpener;

pub struct FileOpeningService {
	opener: Arc<dyn FileOpener>,
}

impl FileOpeningService {
	pub fn new() -> Self {
		Self {
			opener: Arc::new(PlatformOpener),
		}
	}

	/// Runs `job` on the blocking pool: listing apps reads many files and
	/// launching spawns processes, which must not stall the async runtime.
	async fn run<T: Send + 'static>(
		&self,
		job: impl FnOnce(&dyn FileOpener) -> Result<T, String> + Send + 'static,
	) -> Result<T, String> {
		let opener = self.opener.clone();
		tauri::async_runtime::spawn_blocking(move || job(opener.as_ref()))
			.await
			.map_err(|e| e.to_string())?
	}
}

/// Get applications that can open the given file paths
/// Returns intersection of compatible apps for multiple files
#[tauri::command]
pub async fn get_apps_for_paths(
	paths: Vec<PathBuf>,
	service: State<'_, FileOpeningService>,
) -> Result<Vec<OpenWithApp>, String> {
	if paths.is_empty() {
		return Ok(vec![]);
	}
	service
		.run(move |opener| opener.get_apps_for_files(&paths))
		.await
}

/// Open file with system default application
#[tauri::command]
pub async fn open_path_default(
	path: PathBuf,
	service: State<'_, FileOpeningService>,
) -> Result<OpenResult, String> {
	service
		.run(move |opener| opener.open_with_default(&path))
		.await
}

/// Open file with specific application
#[tauri::command]
pub async fn open_path_with_app(
	path: PathBuf,
	app_id: String,
	service: State<'_, FileOpeningService>,
) -> Result<OpenResult, String> {
	service
		.run(move |opener| opener.open_with_app(&path, &app_id))
		.await
}

/// Open multiple files with specific application
#[tauri::command]
pub async fn open_paths_with_app(
	paths: Vec<PathBuf>,
	app_id: String,
	service: State<'_, FileOpeningService>,
) -> Result<Vec<OpenResult>, String> {
	service
		.run(move |opener| opener.open_files_with_app(&paths, &app_id))
		.await
}

/// Make `app_id` the default application for files of this file's type
#[tauri::command]
pub async fn set_default_app_for_path(
	path: PathBuf,
	app_id: String,
	service: State<'_, FileOpeningService>,
) -> Result<(), String> {
	service
		.run(move |opener| opener.set_default_app(&path, &app_id))
		.await
}

/// A user-defined command for the context menu, from a Dolphin service menu.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ContextAction {
	pub id: String,
	pub name: String,
	pub icon: Option<String>,
	pub submenu: Option<String>,
}

/// Service menu actions that apply to every one of `paths`; empty off Linux.
#[tauri::command]
pub async fn get_context_actions(paths: Vec<PathBuf>) -> Result<Vec<ContextAction>, String> {
	#[cfg(target_os = "linux")]
	{
		tauri::async_runtime::spawn_blocking(move || {
			file_opening_linux::context_actions(&paths)
				.into_iter()
				.map(|action| ContextAction {
					id: action.id,
					name: action.name,
					icon: action.icon,
					submenu: action.submenu,
				})
				.collect()
		})
		.await
		.map_err(|e| e.to_string())
	}
	#[cfg(not(target_os = "linux"))]
	{
		let _ = paths;
		Ok(Vec::new())
	}
}

/// Runs the service menu action `id` on `paths`.
#[tauri::command]
pub async fn run_context_action(id: String, paths: Vec<PathBuf>) -> Result<(), String> {
	#[cfg(target_os = "linux")]
	{
		tauri::async_runtime::spawn_blocking(move || {
			file_opening_linux::run_context_action(&id, &paths)
		})
		.await
		.map_err(|e| e.to_string())?
	}
	#[cfg(not(target_os = "linux"))]
	{
		let _ = (id, paths);
		Err("Context menu actions are only available on Linux".to_string())
	}
}
