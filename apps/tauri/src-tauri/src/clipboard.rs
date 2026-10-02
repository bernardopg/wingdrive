//! # Linux File Clipboard
//!
//! GTK serves URI lists on both X11 and Wayland. GNOME and KDE targets
//! preserve cut semantics when exchanging files with other file managers.

#[cfg(target_os = "linux")]
use gtk::{gdk, Clipboard, TargetEntry, TargetFlags};

#[cfg(any(target_os = "linux", test))]
fn file_uris(paths: &[String]) -> Result<Vec<String>, String> {
	paths
		.iter()
		.map(|path| {
			tauri::Url::from_file_path(path)
				.map(String::from)
				.map_err(|_| format!("Clipboard path must be absolute: {path}"))
		})
		.collect()
}

#[cfg(any(target_os = "linux", test))]
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

#[tauri::command]
pub async fn write_file_clipboard(
	app: tauri::AppHandle,
	paths: Vec<String>,
	cut: bool,
) -> Result<(), String> {
	#[cfg(target_os = "linux")]
	{
		let uris = file_uris(&paths)?;
		let (tx, rx) = tokio::sync::oneshot::channel();
		app.run_on_main_thread(move || {
			let clipboard = Clipboard::get(&gdk::SELECTION_CLIPBOARD);
			let targets = [
				TargetEntry::new("text/uri-list", TargetFlags::empty(), 0),
				TargetEntry::new("x-special/gnome-copied-files", TargetFlags::empty(), 1),
				TargetEntry::new("application/x-kde-cutselection", TargetFlags::empty(), 2),
			];
			let ok = clipboard.set_with_data(&targets, move |_, data, info| {
				let text = match info {
					1 => format!("{}\n{}", if cut { "cut" } else { "copy" }, uris.join("\n")),
					2 => if cut { "1" } else { "0" }.to_string(),
					_ => format!("{}\r\n", uris.join("\r\n")),
				};
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
	#[cfg(not(target_os = "linux"))]
	{
		let _ = (app, paths, cut);
		Err("System file clipboard is currently supported on Linux".to_string())
	}
}

#[tauri::command]
pub async fn read_file_clipboard(app: tauri::AppHandle) -> Result<(Vec<String>, bool), String> {
	#[cfg(target_os = "linux")]
	{
		let (tx, rx) = tokio::sync::oneshot::channel();
		app.run_on_main_thread(move || {
			let clipboard = Clipboard::get(&gdk::SELECTION_CLIPBOARD);
			clipboard.request_contents(
				&gdk::Atom::intern("x-special/gnome-copied-files"),
				move |clipboard, data| {
					if data.length() > 0 {
						let bytes = data.data();
						let result = std::str::from_utf8(&bytes)
							.map_err(|e| e.to_string())
							.and_then(|text| {
								let (operation, uris) =
									text.split_once('\n').ok_or("Invalid file clipboard")?;
								if !matches!(operation, "copy" | "cut") {
									return Err("Invalid clipboard operation".to_string());
								}
								Ok((parse_file_uris(uris)?, operation == "cut"))
							});
						let _ = tx.send(result);
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
											let bytes = data.data();
											std::str::from_utf8(&bytes)
												.map_err(|e| e.to_string())
												.and_then(|text| Ok((parse_file_uris(text)?, cut)))
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
	#[cfg(not(target_os = "linux"))]
	{
		let _ = app;
		Err("System file clipboard is currently supported on Linux".to_string())
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn file_clipboard_uri_roundtrip_and_trust_boundary() {
		let paths = vec!["/tmp/a b#c.txt".to_string(), "/tmp/café\nfile".to_string()];
		let uris = file_uris(&paths).unwrap();
		assert_eq!(
			parse_file_uris(&format!("# comment\r\n{}\r\n", uris.join("\r\n"))).unwrap(),
			paths
		);
		assert!(file_uris(&["relative".to_string()]).is_err());
		assert!(parse_file_uris("https://example.com/file").is_err());
		assert!(parse_file_uris("file://remote/tmp/file").is_err());
	}
}
