use std::path::{Path, PathBuf};
use tracing::error;

/// Reveal a file in the native file manager (Finder on macOS, Explorer on Windows, etc.)
#[tauri::command]
pub async fn reveal_file(path: String) -> Result<(), String> {
	let path = PathBuf::from(path);

	if !path.exists() {
		return Err(format!("Path does not exist: {}", path.display()));
	}

	reveal_path(&path).await.map_err(|e| {
		error!("Failed to reveal file: {:#?}", e);
		format!("Failed to reveal file: {}", e)
	})
}

/// Share files using the native system share sheet (macOS/iOS only)
#[tauri::command]
pub async fn share_files(_paths: Vec<String>) -> Result<(), String> {
	#[cfg(target_os = "macos")]
	{
		// Verify all paths exist
		for path in &_paths {
			let path_buf = PathBuf::from(path);
			if !path_buf.exists() {
				return Err(format!("Path does not exist: {}", path));
			}
		}

		// Join paths with null separator (similar to open_file_paths_with pattern)
		let joined_paths = _paths.join("\0");

		unsafe {
			let success = sd_desktop_macos::share_items(&joined_paths.as_str().into());
			if success {
				Ok(())
			} else {
				Err("Failed to show share sheet".to_string())
			}
		}
	}

	#[cfg(not(target_os = "macos"))]
	{
		Err("Share sheet is only supported on macOS".to_string())
	}
}

/// Get the physical path to a sidecar file
#[tauri::command]
pub async fn get_sidecar_path(
	library_id: String,
	content_uuid: String,
	kind: String,
	variant: String,
	format: String,
) -> Result<String, String> {
	// Get the data directory
	let data_dir = sd_tauri_core::default_data_dir()
		.map_err(|e| format!("Failed to get data directory: {}", e))?;

	// Find the actual library folder (might be named differently than the ID)
	let library_folder = find_library_folder(&data_dir, &library_id)
		.await
		.map_err(|e| format!("Failed to find library folder: {:?}", e))?;

	// Actual path structure: sidecars/content/{first2}/{next2}/{uuid}/{kind}s/{variant}.{format}
	// Example: sidecars/content/35/3c/353c7043-8d28-56ec-a424-7ab8932b1ffe/thumbs/detail@1x.webp
	let first_two = &content_uuid[0..2];
	let next_two = &content_uuid[2..4];

	// Special case: "transcript" stays singular (not "transcripts")
	let kind_dir = if kind == "transcript" {
		kind
	} else {
		format!("{}s", kind) // "thumb" -> "thumbs"
	};

	let sidecar_path = library_folder
		.join("sidecars")
		.join("content")
		.join(first_two)
		.join(next_two)
		.join(&content_uuid)
		.join(&kind_dir)
		.join(format!("{}.{}", variant, format));

	Ok(sidecar_path.to_string_lossy().to_string())
}

/// Find library folder by UUID (reads library.json files to match ID)
async fn find_library_folder(data_dir: &Path, library_id: &str) -> Result<PathBuf, String> {
	let libraries_dir = data_dir.join("libraries");

	// Read all .sdlibrary folders
	let mut entries = tokio::fs::read_dir(&libraries_dir)
		.await
		.map_err(|e| format!("Failed to read libraries directory: {}", e))?;

	while let Some(entry) = entries
		.next_entry()
		.await
		.map_err(|e| format!("Failed to read directory entry: {}", e))?
	{
		let path = entry.path();
		if path.extension().and_then(|s| s.to_str()) == Some("sdlibrary") {
			// Try to read library.json
			let library_json_path = path.join("library.json");
			if let Ok(contents) = tokio::fs::read_to_string(&library_json_path).await {
				if let Ok(json) = serde_json::from_str::<serde_json::Value>(&contents) {
					if let Some(id) = json.get("id").and_then(|v| v.as_str()) {
						if id == library_id {
							return Ok(path);
						}
					}
				}
			}
		}
	}

	Err(format!("Library folder not found for ID: {}", library_id))
}

#[cfg(target_os = "macos")]
async fn reveal_path(path: &Path) -> Result<(), std::io::Error> {
	// tokio keeps the async runtime free while the child process runs; the
	// previous std::process wait() blocked the runtime until the file manager closed.
	let status = tokio::process::Command::new("open")
		.arg("-R")
		.arg(path)
		.status()
		.await?;
	if status.success() {
		Ok(())
	} else {
		Err(std::io::Error::new(
			std::io::ErrorKind::Other,
			"open -R failed",
		))
	}
}

#[cfg(target_os = "windows")]
async fn reveal_path(path: &Path) -> Result<(), std::io::Error> {
	// explorer.exe requires `/select,<path>` as a single argument; passing
	// "/select," and the path separately made it ignore the selection and
	// open the Documents folder instead. explorer.exe exits with 1 even on
	// success, so any exit code is treated as completion.
	let select_arg = format!("/select,{}", path.display());
	tokio::process::Command::new("explorer")
		.arg(select_arg)
		.status()
		.await?;
	Ok(())
}

#[cfg(target_os = "linux")]
async fn reveal_path(path: &Path) -> Result<(), std::io::Error> {
	// Prefer the freedesktop file manager interface so the file is actually
	// selected; fall back to opening the parent directory when no file
	// manager implements ShowItems.
	let uri = format!("file://{}", path.display());
	let dbus_call = tokio::process::Command::new("gdbus")
		.args([
			"call",
			"--session",
			"--dest",
			"org.freedesktop.FileManager1",
			"--object-path",
			"/org/freedesktop/FileManager1",
			"--method",
			"org.freedesktop.FileManager1.ShowItems",
		])
		.arg(&uri)
		.arg("")
		.status()
		.await;

	if matches!(dbus_call, Ok(status) if status.success()) {
		return Ok(());
	}

	if let Some(parent) = path.parent() {
		tokio::process::Command::new("xdg-open")
			.arg(parent)
			.status()
			.await?;
	}
	Ok(())
}

#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
async fn reveal_path(_path: &Path) -> Result<(), std::io::Error> {
	Err(std::io::Error::new(
		std::io::ErrorKind::Unsupported,
		"Reveal is not supported on this platform",
	))
}
