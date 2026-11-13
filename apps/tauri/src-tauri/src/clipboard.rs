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
mod tests {
	use super::*;

	#[test]
	fn malformed_clipboard_entries_reject_the_entire_selection() {
		for invalid in [
			"not a URI",
			"file://remote/tmp/file",
			"file:///tmp/%FF",
			"https://example.invalid/file",
		] {
			assert!(
				parse_file_uris(&format!("file:///tmp/valid\r\n{invalid}\r\n")).is_err(),
				"{invalid}"
			);
		}
		assert!(file_uris(&["/tmp/valid".into(), "relative".into()]).is_err());
	}

	#[test]
	fn clipboard_comments_empty_lists_and_encoded_line_breaks_keep_item_boundaries() {
		assert!(parse_file_uris(" \r\n# comment\r\n\t# another\n")
			.unwrap()
			.is_empty());
		assert!(file_uris(&[]).unwrap().is_empty());
		assert_eq!(
			parse_file_uris("file:///tmp/a%0D%0Ab\r\nfile://localhost/tmp/c%25d\n").unwrap(),
			["/tmp/a\r\nb", "/tmp/c%d"]
		);
	}

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

	/// Each offered target serves the format its readers expect: Nautilus and
	/// Dolphin read the cut marker, everyone else the CRLF URI list.
	#[test]
	fn written_selection_serves_each_target_in_its_own_format() {
		let uris = file_uris(&["/tmp/a b".into(), "/tmp/c".into()]).unwrap();
		assert_eq!(WRITE_TARGETS[0], "text/uri-list");
		assert_eq!(
			selection_text(0, true, &uris),
			"file:///tmp/a%20b\r\nfile:///tmp/c\r\n"
		);
		assert_eq!(WRITE_TARGETS[1], "x-special/gnome-copied-files");
		assert_eq!(
			selection_text(1, true, &uris),
			"cut\nfile:///tmp/a%20b\nfile:///tmp/c"
		);
		assert_eq!(
			selection_text(1, false, &uris),
			"copy\nfile:///tmp/a%20b\nfile:///tmp/c"
		);
		assert_eq!(WRITE_TARGETS[2], "application/x-kde-cutselection");
		assert_eq!(selection_text(2, true, &uris), "1");
		assert_eq!(selection_text(2, false, &uris), "0");
		// An unknown target index falls back to the URI list.
		assert_eq!(
			selection_text(7, false, &uris),
			selection_text(0, false, &uris)
		);
	}

	/// What this app writes, another WingDrive window (or Nautilus) reads back
	/// with the same paths and operation.
	#[test]
	fn written_selections_read_back_as_the_same_paths_and_operation() {
		let paths = vec!["/tmp/a b#c.txt".to_string(), "/tmp/café".to_string()];
		let uris = file_uris(&paths).unwrap();
		for cut in [true, false] {
			assert_eq!(
				parse_gnome_copied_files(selection_text(1, cut, &uris).as_bytes()).unwrap(),
				(paths.clone(), cut)
			);
			assert_eq!(
				parse_uri_list(selection_text(0, cut, &uris).as_bytes(), cut).unwrap(),
				(paths.clone(), cut)
			);
		}
	}

	#[test]
	fn gnome_copied_files_need_a_known_operation_and_valid_uris() {
		assert_eq!(
			parse_gnome_copied_files(b"cut\nfile:///tmp/a\nfile:///tmp/b\n").unwrap(),
			(vec!["/tmp/a".to_string(), "/tmp/b".to_string()], true)
		);
		// An operation with no files yet is an empty copy, not an error.
		assert_eq!(
			parse_gnome_copied_files(b"copy\n").unwrap(),
			(vec![], false)
		);
		assert_eq!(
			parse_gnome_copied_files(b"cut").unwrap_err(),
			"Invalid file clipboard"
		);
		for operation in ["move", "CUT", "", "x-special/nautilus-clipboard"] {
			assert_eq!(
				parse_gnome_copied_files(format!("{operation}\nfile:///tmp/a").as_bytes())
					.unwrap_err(),
				"Invalid clipboard operation",
				"{operation}"
			);
		}
		assert!(parse_gnome_copied_files(b"copy\nhttps://example.invalid/a").is_err());
		assert!(parse_gnome_copied_files(b"copy\n\xff").is_err());
	}

	#[test]
	fn uri_lists_keep_the_cut_marker_and_reject_bad_bytes() {
		assert_eq!(
			parse_uri_list(b"# from a browser\r\nfile:///tmp/a\r\n", true).unwrap(),
			(vec!["/tmp/a".to_string()], true)
		);
		assert!(parse_uri_list(b"file:///tmp/\xff\xfe", false).is_err());
		assert!(parse_uri_list(b"file:///tmp/a\nrelative", false).is_err());
	}

	/// A relative path fails before the main thread or GTK is reached.
	#[tokio::test]
	async fn writing_relative_paths_fails_without_touching_the_clipboard() {
		let app = tauri::test::mock_app();
		let error = write_file_clipboard(app.handle().clone(), vec!["notes.txt".into()], true)
			.await
			.unwrap_err();
		assert_eq!(error, "Clipboard path must be absolute: notes.txt");
	}

	/// Runs `future` on this thread while GTK dispatches the selection
	/// callbacks it waits for.
	fn pump<T>(future: impl std::future::Future<Output = T>) -> T {
		let runtime = tokio::runtime::Builder::new_current_thread()
			.enable_all()
			.build()
			.unwrap();
		let mut future = std::pin::pin!(future);
		runtime.block_on(std::future::poll_fn(|cx| {
			while gtk::events_pending() {
				gtk::main_iteration();
			}
			let poll = future.as_mut().poll(cx);
			if poll.is_pending() {
				cx.waker().wake_by_ref();
			}
			poll
		}))
	}

	/// Writes and reads the clipboard of a private Xvfb server, never the
	/// session's. Run with `--ignored` on a machine with Xvfb.
	#[test]
	#[ignore = "needs a display"]
	fn clipboard_round_trips_and_falls_back_to_kde_and_plain_uri_lists() {
		const CHILD_ENV: &str = "WINGDRIVE_TEST_CLIPBOARD_CHILD";
		if std::env::var_os(CHILD_ENV).is_some() {
			gtk::init().unwrap();
			let app = tauri::test::mock_app();
			let handle = app.handle().clone();
			let paths = vec!["/tmp/a b.txt".to_string(), "/tmp/café".to_string()];

			// Nothing owns the selection yet.
			assert_eq!(
				pump(read_file_clipboard(handle.clone())).unwrap(),
				(vec![], false)
			);
			for cut in [true, false] {
				pump(write_file_clipboard(handle.clone(), paths.clone(), cut)).unwrap();
				assert_eq!(
					pump(read_file_clipboard(handle.clone())).unwrap(),
					(paths.clone(), cut)
				);
			}

			// A KDE app offers no GNOME target: the cut marker and URI list are read.
			let clipboard = Clipboard::get(&gdk::SELECTION_CLIPBOARD);
			let uris = file_uris(&paths).unwrap();
			let targets: Vec<_> = (0..)
				.zip([WRITE_TARGETS[0], WRITE_TARGETS[2]])
				.map(|(info, target)| TargetEntry::new(target, TargetFlags::empty(), info))
				.collect();
			assert!(clipboard.set_with_data(&targets, move |_, data, info| {
				let text = selection_text(if info == 1 { 2 } else { 0 }, true, &uris);
				data.set(&data.target(), 8, text.as_bytes());
			}));
			assert_eq!(
				pump(read_file_clipboard(handle.clone())).unwrap(),
				(paths.clone(), true)
			);

			// A browser offers only a URI list, which reads as a copy.
			clipboard.set_text("ignored text");
			let uris = file_uris(&paths).unwrap();
			assert!(clipboard.set_with_data(
				&[TargetEntry::new(WRITE_TARGETS[0], TargetFlags::empty(), 0)],
				move |_, data, _| {
					data.set(
						&data.target(),
						8,
						selection_text(0, false, &uris).as_bytes(),
					);
				}
			));
			assert_eq!(pump(read_file_clipboard(handle)).unwrap(), (paths, false));
			return;
		}

		let mut xvfb = std::process::Command::new("Xvfb")
			.args([
				"-displayfd",
				"1",
				"-nolisten",
				"tcp",
				"-screen",
				"0",
				"640x480x24",
			])
			.stdout(std::process::Stdio::piped())
			.stderr(std::process::Stdio::null())
			.spawn()
			.expect("Xvfb is installed");
		let mut display = String::new();
		std::io::BufRead::read_line(
			&mut std::io::BufReader::new(xvfb.stdout.take().unwrap()),
			&mut display,
		)
		.unwrap();
		let home = tempfile::tempdir().unwrap();
		let output = std::process::Command::new(std::env::current_exe().unwrap())
			.args([
				"--ignored",
				"--exact",
				"clipboard::tests::clipboard_round_trips_and_falls_back_to_kde_and_plain_uri_lists",
				"--nocapture",
				"--test-threads=1",
			])
			.env(CHILD_ENV, "1")
			.env("DISPLAY", format!(":{}", display.trim()))
			.env("GDK_BACKEND", "x11")
			.env("HOME", home.path())
			.env("XDG_RUNTIME_DIR", home.path())
			.env_remove("WAYLAND_DISPLAY")
			.env_remove("DBUS_SESSION_BUS_ADDRESS")
			.output()
			.unwrap();
		let _ = xvfb.kill();
		let _ = xvfb.wait();
		let stdout = String::from_utf8_lossy(&output.stdout);
		assert!(
			output.status.success() && stdout.contains("1 passed"),
			"child failed: {stdout}{}",
			String::from_utf8_lossy(&output.stderr)
		);
	}
}
