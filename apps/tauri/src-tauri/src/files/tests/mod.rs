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
