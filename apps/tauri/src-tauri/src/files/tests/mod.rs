use super::{canonical_sidecar_uuid, find_library_folder, sidecar_path_in};
use std::path::Path;

#[tokio::test]
async fn invalid_sidecar_uuid_returns_error_before_library_lookup() {
	for value in ["", "a", "éé", "not-a-uuid"] {
		let result = sidecar_path_in(
			Path::new("/unused"),
			"unused",
			value,
			"thumb".into(),
			"grid",
			"webp",
		)
		.await;
		assert_eq!(result.unwrap_err(), "Invalid content UUID", "{value}");
	}
	assert_eq!(
		canonical_sidecar_uuid("550E8400E29B41D4A716446655440000").unwrap(),
		"550e8400-e29b-41d4-a716-446655440000"
	);
}

#[tokio::test]
async fn sidecar_segments_that_could_leave_the_content_folder_return_an_error() {
	let uuid = "550e8400-e29b-41d4-a716-446655440000";
	for (kind, variant, format) in [
		("thumb", "..", "webp"),
		("thumb", "../../etc/passwd", "webp"),
		("thumb", "grid", "../webp"),
		("../thumb", "grid", "webp"),
		("thumb/x", "grid", "webp"),
		("", "grid", "webp"),
	] {
		let result = sidecar_path_in(
			Path::new("/unused"),
			"unused",
			uuid,
			kind.into(),
			variant,
			format,
		)
		.await;
		assert_eq!(
			result.unwrap_err(),
			"Invalid sidecar path segment",
			"{kind}/{variant}.{format}"
		);
	}
}

/// The path comes from the data directory the window's daemon uses, so a
/// named instance finds its own libraries.
#[tokio::test]
async fn sidecar_path_uses_the_given_data_directory() {
	let dir = tempfile::tempdir().unwrap();
	let instance = dir.path().join("instances/work");
	let library = instance.join("libraries/work.winglibrary");
	tokio::fs::create_dir_all(&library).await.unwrap();
	tokio::fs::write(library.join("library.json"), r#"{"id":"lib-id"}"#)
		.await
		.unwrap();
	let path = sidecar_path_in(
		&instance,
		"lib-id",
		"0CC0B48FA47553ECA580BC7D47B486A9",
		"thumb".into(),
		"grid",
		"webp",
	)
	.await
	.unwrap();
	assert_eq!(
		Path::new(&path),
		library
			.join("sidecars/content/0c/c0/0cc0b48f-a475-53ec-a580-bc7d47b486a9/thumbs/grid.webp")
	);
}

#[tokio::test]
async fn finds_library_by_json_id_in_current_and_legacy_folders() {
	let dir = tempfile::tempdir().unwrap();
	let libraries = dir.path().join("libraries");
	tokio::fs::create_dir(&libraries).await.unwrap();
	for (folder, contents) in [
		("broken.winglibrary", "{invalid"),
		("current.winglibrary", r#"{"id":"current-id"}"#),
		("legacy.sdlibrary", r#"{"id":"legacy-id"}"#),
		("ignored.txt", r#"{"id":"ignored-id"}"#),
	] {
		let path = libraries.join(folder);
		tokio::fs::create_dir(&path).await.unwrap();
		tokio::fs::write(path.join("library.json"), contents)
			.await
			.unwrap();
	}
	for (id, folder) in [
		("current-id", "current.winglibrary"),
		("legacy-id", "legacy.sdlibrary"),
	] {
		assert_eq!(
			find_library_folder(dir.path(), id).await.unwrap(),
			libraries.join(folder)
		);
	}
	assert!(find_library_folder(dir.path(), "ignored-id").await.is_err());
	assert!(find_library_folder(dir.path(), "missing-id").await.is_err());
}

#[tokio::test]
async fn missing_libraries_directory_returns_an_error() {
	let dir = tempfile::tempdir().unwrap();
	let error = find_library_folder(dir.path(), "missing")
		.await
		.unwrap_err();
	assert!(error.contains("libraries directory"), "{error}");
}

/// FileManager1 decodes the URI, so the path has to survive the round trip.
#[cfg(target_os = "linux")]
#[test]
fn reveal_uri_round_trips_paths_with_reserved_characters() {
	for path in [
		"/tmp/plain.txt",
		"/tmp/a b.txt",
		"/tmp/100%20off.txt",
		"/tmp/notes#1.txt",
		"/tmp/what?.txt",
		"/tmp/café.txt",
	] {
		let uri = super::reveal_uri(std::path::Path::new(path));
		let parsed = tauri::Url::parse(&uri).unwrap();
		assert_eq!(parsed.fragment(), None, "{uri}");
		assert_eq!(parsed.query(), None, "{uri}");
		assert_eq!(
			parsed.to_file_path().unwrap(),
			std::path::PathBuf::from(path),
			"{uri}"
		);
	}
}

/// `gdbus call` parses each argument as GVariant text, and ShowItems takes an
/// array of strings.
#[cfg(target_os = "linux")]
#[test]
fn show_items_argument_is_a_gvariant_string_array() {
	assert_eq!(
		super::show_items_argument("file:///tmp/a%20b.txt"),
		"['file:///tmp/a%20b.txt']"
	);
	// `'` may stay unencoded in a URI path, so it is escaped for the parser.
	assert_eq!(
		super::show_items_argument(r"file:///tmp/it's\x"),
		r"['file:///tmp/it\'s\\x']"
	);
}

#[tokio::test]
async fn reveal_and_terminal_reject_missing_folders() {
	let dir = tempfile::tempdir().unwrap();
	let missing = dir.path().join("missing");
	let error = super::reveal_file(missing.to_string_lossy().into_owned())
		.await
		.unwrap_err();
	assert!(error.starts_with("Path does not exist"), "{error}");

	use tauri::Manager;
	let app = tauri::test::mock_app();
	app.manage(crate::background::DesktopSettingsState::load(dir.path()));
	let file = dir.path().join("file");
	std::fs::write(&file, "").unwrap();
	for path in [missing, file] {
		let error = super::open_terminal(path.to_string_lossy().into_owned(), app.state())
			.await
			.unwrap_err();
		assert!(error.starts_with("Not a folder"), "{error}");
	}
}

#[cfg(not(target_os = "macos"))]
#[tokio::test]
async fn share_sheet_is_unavailable_off_macos() {
	assert!(super::share_files(vec!["/tmp".into()]).await.is_err());
}

/// Without a FileManager1 service, reveal opens the parent folder. The opener
/// can stay in the foreground (a terminal does), so reveal must not wait for it.
#[cfg(target_os = "linux")]
#[test]
fn reveal_fallback_does_not_wait_for_the_folder_opener() {
	const CHILD_ENV: &str = "WINGDRIVE_TEST_REVEAL_CHILD";
	if let Some(root) = std::env::var_os(CHILD_ENV) {
		let root = Path::new(&root);
		let runtime = tokio::runtime::Runtime::new().unwrap();
		let started = std::time::Instant::now();
		runtime
			.block_on(super::reveal_path(&root.join("file.txt")))
			.unwrap();
		assert!(
			started.elapsed() < std::time::Duration::from_secs(5),
			"reveal waited {:?} for the opener",
			started.elapsed()
		);
		// The opener still ran with the parent folder.
		let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
		while !root.join("opened").exists() {
			assert!(std::time::Instant::now() < deadline, "opener never ran");
			std::thread::sleep(std::time::Duration::from_millis(50));
		}
		assert_eq!(
			std::fs::read_to_string(root.join("opened")).unwrap().trim(),
			root.display().to_string()
		);
		runtime.shutdown_background();
		return;
	}

	use std::os::unix::fs::PermissionsExt;
	let dir = tempfile::tempdir().unwrap();
	let root = dir.path();
	let bin = root.join("bin");
	std::fs::create_dir(&bin).unwrap();
	std::fs::write(root.join("file.txt"), "x").unwrap();
	// gdbus fails as if no file manager owned FileManager1; the opener records its
	// argument and then stays running like a terminal window.
	for (name, script) in [
		("gdbus", "#!/bin/sh\nexit 1\n".to_string()),
		(
			"xdg-open",
			format!(
				"#!/bin/sh\nprintf '%s\\n' \"$1\" > '{}/opened'\nexec sleep 30\n",
				root.display()
			),
		),
	] {
		let path = bin.join(name);
		std::fs::write(&path, script).unwrap();
		std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
	}
	let output = std::process::Command::new(std::env::current_exe().unwrap())
		.args([
			"--exact",
			"files::tests::reveal_fallback_does_not_wait_for_the_folder_opener",
			"--nocapture",
		])
		.env(CHILD_ENV, root)
		.env("PATH", format!("{}:/usr/bin:/bin", bin.display()))
		.env_remove("APPDIR")
		.output()
		.unwrap();
	assert!(
		output.status.success(),
		"child failed: {}",
		String::from_utf8_lossy(&output.stderr)
	);
}

/// The command reads the data directory from the window's daemon state, so a
/// named instance resolves sidecars inside its own libraries.
#[tokio::test]
async fn sidecar_command_resolves_against_the_daemon_data_directory() {
	use tauri::Manager;

	let dir = tempfile::tempdir().unwrap();
	let library = dir.path().join("libraries/photos.winglibrary");
	tokio::fs::create_dir_all(&library).await.unwrap();
	tokio::fs::write(library.join("library.json"), r#"{"id":"photos"}"#)
		.await
		.unwrap();
	let app = tauri::test::mock_app();
	app.manage(std::sync::Arc::new(tokio::sync::RwLock::new(
		crate::DaemonState {
			started_by_us: false,
			socket_addr: "127.0.0.1:9".into(),
			data_dir: dir.path().to_path_buf(),
			base_data_dir: dir.path().join("base"),
			instance: Some("work".into()),
			server_url: None,
			server_token: None,
			server_shutdown: None,
			daemon_process: None,
		},
	)));

	let uuid = "353c7043-8d28-56ec-a424-7ab8932b1ffe";
	for (kind, variant, format, expected) in [
		("thumb", "detail@1x", "webp", "thumbs/detail@1x.webp"),
		// Transcripts keep a singular folder name.
		("transcript", "text", "srt", "transcript/text.srt"),
	] {
		let path = super::get_sidecar_path(
			app.state(),
			"photos".into(),
			uuid.into(),
			kind.into(),
			variant.into(),
			format.into(),
		)
		.await
		.unwrap();
		assert_eq!(
			Path::new(&path),
			library
				.join("sidecars/content/35/3c")
				.join(uuid)
				.join(expected)
		);
	}
	let error = super::get_sidecar_path(
		app.state(),
		"other".into(),
		uuid.into(),
		"thumb".into(),
		"grid".into(),
		"webp".into(),
	)
	.await
	.unwrap_err();
	assert!(
		error.contains("Library folder not found for ID: other"),
		"{error}"
	);
}

/// "Open Terminal Here" starts the configured terminal in the folder, and
/// reports a terminal that cannot start.
#[cfg(target_os = "linux")]
#[tokio::test]
async fn open_terminal_starts_the_configured_terminal_in_the_folder() {
	use std::os::unix::fs::PermissionsExt;
	use tauri::Manager;

	let dir = tempfile::tempdir().unwrap();
	let folder = dir.path().join("pasta com espaço");
	std::fs::create_dir(&folder).unwrap();
	let physical = folder.canonicalize().unwrap();
	let marker = dir.path().join("cwd");
	let script = dir.path().join("fake-terminal");
	std::fs::write(
		&script,
		format!(
			"#!/bin/sh\nprintf '%s|%s\\n' \"$(pwd -P)\" \"$*\" > '{}'\n",
			marker.display()
		),
	)
	.unwrap();
	std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();

	for (terminal, expected) in [
		(
			script.display().to_string(),
			Ok(format!("{}|", physical.display())),
		),
		(
			format!("{} --title wing", script.display()),
			Ok(format!("{}|--title wing", physical.display())),
		),
		(
			dir.path().join("missing-terminal").display().to_string(),
			Err("Failed to start"),
		),
	] {
		let _ = std::fs::remove_file(&marker);
		let settings_dir = tempfile::tempdir().unwrap();
		std::fs::write(
			settings_dir.path().join("desktop_settings.json"),
			serde_json::json!({ "terminal_command": terminal }).to_string(),
		)
		.unwrap();
		let app = tauri::test::mock_app();
		app.manage(crate::background::DesktopSettingsState::load(
			settings_dir.path(),
		));
		let result = super::open_terminal(folder.to_string_lossy().into_owned(), app.state()).await;
		match expected {
			Ok(expected) => {
				result.unwrap();
				let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
				while !marker.exists() {
					assert!(std::time::Instant::now() < deadline, "terminal never ran");
					std::thread::sleep(std::time::Duration::from_millis(20));
				}
				// The script may still be writing; wait for the line to end.
				let mut written = String::new();
				while !written.ends_with('\n') {
					assert!(
						std::time::Instant::now() < deadline,
						"terminal output incomplete"
					);
					written = std::fs::read_to_string(&marker).unwrap();
				}
				assert_eq!(written.trim_end(), expected);
			}
			Err(prefix) => {
				let error = result.unwrap_err();
				assert!(error.starts_with(prefix), "{error}");
			}
		}
	}
}

/// With a FileManager1 service, reveal selects the item and opens nothing
/// else; without one, a path with no parent has nothing to open.
#[cfg(target_os = "linux")]
#[test]
fn reveal_selects_through_file_manager1_and_terminal_lookup_needs_a_terminal() {
	const CHILD_ENV: &str = "WINGDRIVE_TEST_REVEAL_SELECT_CHILD";
	if let Some(root) = std::env::var_os(CHILD_ENV) {
		let root = Path::new(&root);
		let runtime = tokio::runtime::Runtime::new().unwrap();
		runtime.block_on(async {
			let file = root.join("it's here.txt");
			super::reveal_file(file.to_string_lossy().into_owned())
				.await
				.unwrap();
			let args = std::fs::read_to_string(root.join("gdbus-args")).unwrap();
			let lines: Vec<&str> = args.lines().collect();
			let items = super::show_items_argument(&super::reveal_uri(&file));
			// The last argument is ShowItems' empty startup id.
			assert_eq!(
				lines,
				[
					"call",
					"--session",
					"--dest",
					"org.freedesktop.FileManager1",
					"--object-path",
					"/org/freedesktop/FileManager1",
					"--method",
					"org.freedesktop.FileManager1.ShowItems",
					items.as_str(),
					"",
				]
			);

			// gdbus now fails; "/" has no parent folder to fall back to.
			std::fs::write(root.join("gdbus-fail"), "").unwrap();
			super::reveal_path(Path::new("/")).await.unwrap();
			std::thread::sleep(std::time::Duration::from_millis(200));
			assert!(!root.join("opened").exists());
		});

		// No override, no xdg-terminal-exec, no $TERMINAL and no known emulator.
		let error = super::spawn_terminal(root, None).unwrap_err();
		assert_eq!(
			error,
			"No terminal emulator found. Set one in Settings > General."
		);
		runtime.shutdown_background();
		return;
	}

	use std::os::unix::fs::PermissionsExt;
	let dir = tempfile::tempdir().unwrap();
	let root = dir.path();
	let bin = root.join("bin");
	std::fs::create_dir(&bin).unwrap();
	std::fs::write(root.join("it's here.txt"), "x").unwrap();
	for (name, script) in [
		(
			"gdbus",
			format!(
				"#!/bin/sh\n[ -e '{0}/gdbus-fail' ] && exit 1\nfor a in \"$@\"; do printf '%s\\n' \"$a\"; done > '{0}/gdbus-args'\n",
				root.display()
			),
		),
		(
			"xdg-open",
			format!("#!/bin/sh\ntouch '{}/opened'\n", root.display()),
		),
	] {
		let path = bin.join(name);
		std::fs::write(&path, script).unwrap();
		std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
	}
	// The scripts need a shell, so PATH holds only the fixture folder plus the
	// shell's own, which has no terminal emulators.
	let sh_dir = root.join("sh");
	std::fs::create_dir(&sh_dir).unwrap();
	for tool in ["sh", "touch"] {
		let real = ["/usr/bin", "/bin"]
			.iter()
			.map(|d| Path::new(d).join(tool))
			.find(|p| p.exists())
			.unwrap();
		std::os::unix::fs::symlink(real, sh_dir.join(tool)).unwrap();
	}
	let output = std::process::Command::new(std::env::current_exe().unwrap())
		.args([
			"--exact",
			"files::tests::reveal_selects_through_file_manager1_and_terminal_lookup_needs_a_terminal",
			"--nocapture",
		])
		.env(CHILD_ENV, root)
		.env("PATH", format!("{}:{}", bin.display(), sh_dir.display()))
		.env_remove("TERMINAL")
		.env_remove("APPDIR")
		.output()
		.unwrap();
	assert!(
		output.status.success(),
		"child failed: {}{}",
		String::from_utf8_lossy(&output.stdout),
		String::from_utf8_lossy(&output.stderr)
	);
}
