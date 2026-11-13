//! # System File Clipboard
//!
//! On Linux, GTK serves URI lists on both X11 and Wayland. GNOME and KDE targets
//! preserve cut semantics when exchanging files with other file managers. On
//! Windows, `file-clipboard-windows` writes the `CF_HDROP` list Explorer reads.

#[cfg(target_os = "linux")]
use gtk::{gdk, Clipboard, TargetEntry, TargetFlags};

/// `file://` URIs for absolute paths, percent-encoded so spaces, `#` and accents survive.
#[cfg(target_os = "linux")]
pub(crate) fn file_uris(paths: &[String]) -> Result<Vec<String>, String> {
	paths
		.iter()
		.map(|path| {
			tauri::Url::from_file_path(path)
				.map(String::from)
				.map_err(|_| format!("Clipboard path must be absolute: {path}"))
		})
		.collect()
}

#[cfg(target_os = "linux")]
fn parse_file_uris(text: &str) -> Result<Vec<String>, String> {
	text.lines()
		.map(str::trim)
		.filter(|line| !line.is_empty() && !line.starts_with('#'))
		.map(|line| {
			tauri::Url::parse(line)
				.map_err(|e| format!("Invalid clipboard URI: {e}"))?
				.to_file_path()
				.map_err(|_| "Clipboard contains a non-local file URI".to_string())?
				.into_os_string()
				.into_string()
				.map_err(|_| "Clipboard file name is not valid UTF-8".to_string())
		})
		.collect()
}

/// Selection targets offered when writing, in the order `selection_text` reads
/// their index: the URI list everyone reads, then GNOME's and KDE's cut markers.
#[cfg(target_os = "linux")]
const WRITE_TARGETS: [&str; 3] = [
	"text/uri-list",
	"x-special/gnome-copied-files",
	"application/x-kde-cutselection",
];

/// The bytes served for the `WRITE_TARGETS` entry at `info`.
#[cfg(target_os = "linux")]
fn selection_text(info: u32, cut: bool, uris: &[String]) -> String {
	match info {
		1 => format!("{}\n{}", if cut { "cut" } else { "copy" }, uris.join("\n")),
		2 => if cut { "1" } else { "0" }.to_string(),
		_ => format!("{}\r\n", uris.join("\r\n")),
	}
}

/// Reads `x-special/gnome-copied-files`: `copy` or `cut`, then one URI per line.
#[cfg(target_os = "linux")]
fn parse_gnome_copied_files(bytes: &[u8]) -> Result<(Vec<String>, bool), String> {
	let text = std::str::from_utf8(bytes).map_err(|e| e.to_string())?;
	let (operation, uris) = text.split_once('\n').ok_or("Invalid file clipboard")?;
	if !matches!(operation, "copy" | "cut") {
		return Err("Invalid clipboard operation".to_string());
	}
	Ok((parse_file_uris(uris)?, operation == "cut"))
}

/// Reads a `text/uri-list` selection; `cut` comes from KDE's marker, if any.
#[cfg(target_os = "linux")]
fn parse_uri_list(bytes: &[u8], cut: bool) -> Result<(Vec<String>, bool), String> {
	let text = std::str::from_utf8(bytes).map_err(|e| e.to_string())?;
	Ok((parse_file_uris(text)?, cut))
}

#[tauri::command]
pub async fn write_file_clipboard<R: tauri::Runtime>(
	app: tauri::AppHandle<R>,
	paths: Vec<String>,
	cut: bool,
) -> Result<(), String> {
	#[cfg(target_os = "linux")]
	{
		let uris = file_uris(&paths)?;
		let (tx, rx) = tokio::sync::oneshot::channel();
		app.run_on_main_thread(move || {
			let clipboard = Clipboard::get(&gdk::SELECTION_CLIPBOARD);
			let targets: Vec<_> = (0..)
				.zip(WRITE_TARGETS)
				.map(|(info, target)| TargetEntry::new(target, TargetFlags::empty(), info))
				.collect();
			let ok = clipboard.set_with_data(&targets, move |_, data, info| {
				let text = selection_text(info, cut, &uris);
				data.set(&data.target(), 8, text.as_bytes());
			});
			let _ = tx.send(if ok {
				Ok(())
			} else {
				Err("Could not own the file clipboard".to_string())
			});
		})
		.map_err(|e| e.to_string())?;
		rx.await.map_err(|e| e.to_string())?
	}
	#[cfg(windows)]
	{
		let _ = app;
		tauri::async_runtime::spawn_blocking(move || file_clipboard_windows::write(&paths, cut))
			.await
			.map_err(|e| e.to_string())?
	}
	#[cfg(not(any(target_os = "linux", windows)))]
	{
		let _ = (app, paths, cut);
		Err("System file clipboard is supported on Linux and Windows".to_string())
	}
}

#[tauri::command]
pub async fn read_file_clipboard<R: tauri::Runtime>(
	app: tauri::AppHandle<R>,
) -> Result<(Vec<String>, bool), String> {
	#[cfg(target_os = "linux")]
	{
		let (tx, rx) = tokio::sync::oneshot::channel();
		app.run_on_main_thread(move || {
			let clipboard = Clipboard::get(&gdk::SELECTION_CLIPBOARD);
			clipboard.request_contents(
				&gdk::Atom::intern("x-special/gnome-copied-files"),
				move |clipboard, data| {
					if data.length() > 0 {
						let _ = tx.send(parse_gnome_copied_files(&data.data()));
					} else {
						clipboard.request_contents(
							&gdk::Atom::intern("application/x-kde-cutselection"),
							move |clipboard, data| {
								let cut = data.length() > 0 && data.data().starts_with(b"1");
								clipboard.request_contents(
									&gdk::Atom::intern("text/uri-list"),
									move |_, data| {
										let result = if data.length() <= 0 {
											Ok((vec![], false))
										} else {
											parse_uri_list(&data.data(), cut)
										};
										let _ = tx.send(result);
									},
								);
							},
						);
					}
				},
			);
		})
		.map_err(|e| e.to_string())?;
		tokio::time::timeout(std::time::Duration::from_secs(3), rx)
			.await
			.map_err(|_| "File clipboard timed out".to_string())?
			.map_err(|e| e.to_string())?
	}
	#[cfg(windows)]
	{
		let _ = app;
		tauri::async_runtime::spawn_blocking(file_clipboard_windows::read)
			.await
			.map_err(|e| e.to_string())?
	}
	#[cfg(not(any(target_os = "linux", windows)))]
	{
		let _ = app;
		Err("System file clipboard is supported on Linux and Windows".to_string())
	}
}

#[cfg(all(test, target_os = "linux"))]
#[path = "clipboard/tests/mod.rs"]
mod tests;
