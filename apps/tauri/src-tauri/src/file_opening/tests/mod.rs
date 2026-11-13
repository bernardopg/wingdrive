use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use file_opening::{FileOpener, OpenResult, OpenWithApp};
use tauri::Manager;

use super::{
	get_apps_for_paths, open_path_default, open_path_with_app, open_paths_with_app,
	set_default_app_for_path, FileOpeningService,
};

/// Records every call and panics for the app id `crash`.
#[derive(Default)]
struct RecordingOpener {
	calls: Mutex<Vec<String>>,
}

impl RecordingOpener {
	fn record(&self, call: String) {
		self.calls.lock().unwrap().push(call);
	}
}

impl FileOpener for RecordingOpener {
	fn get_apps_for_file(&self, path: &Path) -> Result<Vec<OpenWithApp>, String> {
		self.record(format!("apps {}", path.display()));
		Ok(vec![OpenWithApp {
			id: "editor".into(),
			name: "Editor".into(),
			icon: None,
			is_default: true,
		}])
	}

	fn set_default_app(&self, path: &Path, app_id: &str) -> Result<(), String> {
		self.record(format!("default {} {app_id}", path.display()));
		Ok(())
	}

	fn open_with_default(&self, path: &Path) -> Result<OpenResult, String> {
		self.record(format!("open {}", path.display()));
		Ok(OpenResult::Success)
	}

	fn open_with_app(&self, path: &Path, app_id: &str) -> Result<OpenResult, String> {
		if app_id == "crash" {
			panic!("opener crashed");
		}
		self.record(format!("open {} with {app_id}", path.display()));
		Ok(OpenResult::AppNotFound {
			app_id: app_id.into(),
		})
	}
}

fn app_with(opener: Arc<RecordingOpener>) -> tauri::App<tauri::test::MockRuntime> {
	let app = tauri::test::mock_app();
	app.manage(FileOpeningService { opener });
	app
}

#[tokio::test]
async fn commands_forward_paths_and_app_ids_to_the_platform_opener() {
	let opener = Arc::new(RecordingOpener::default());
	let app = app_with(opener.clone());

	assert!(get_apps_for_paths(vec![], app.state())
		.await
		.unwrap()
		.is_empty());
	let apps = get_apps_for_paths(vec![PathBuf::from("/a.txt")], app.state())
		.await
		.unwrap();
	assert_eq!(apps.len(), 1);
	assert!(apps[0].is_default);

	assert!(matches!(
		open_path_default("/a.txt".into(), app.state())
			.await
			.unwrap(),
		OpenResult::Success
	));
	assert!(matches!(
		open_path_with_app("/a.txt".into(), "viewer".into(), app.state())
			.await
			.unwrap(),
		OpenResult::AppNotFound { app_id } if app_id == "viewer"
	));
	let results = open_paths_with_app(
		vec!["/a.txt".into(), "/b.txt".into()],
		"viewer".into(),
		app.state(),
	)
	.await
	.unwrap();
	assert_eq!(results.len(), 2);
	set_default_app_for_path("/a.txt".into(), "viewer".into(), app.state())
		.await
		.unwrap();

	assert_eq!(
		*opener.calls.lock().unwrap(),
		[
			"apps /a.txt",
			"open /a.txt",
			"open /a.txt with viewer",
			"open /a.txt with viewer",
			"open /b.txt with viewer",
			"default /a.txt viewer",
		]
	);
}

#[tokio::test]
async fn a_panicking_opener_returns_an_error_instead_of_unwinding() {
	let opener = Arc::new(RecordingOpener::default());
	let app = app_with(opener);
	let error = open_path_with_app("/a.txt".into(), "crash".into(), app.state())
		.await
		.unwrap_err();
	assert!(!error.is_empty());
	// The service keeps working after the failed job.
	assert!(open_path_default("/a.txt".into(), app.state())
		.await
		.is_ok());
}

#[test]
fn context_action_serializes_the_fields_the_menu_reads() {
	let action = super::ContextAction {
		id: "svc:compress".into(),
		name: "Compress".into(),
		icon: None,
		submenu: Some("Archive".into()),
	};
	assert_eq!(
		serde_json::to_value(action).unwrap(),
		serde_json::json!({"id": "svc:compress", "name": "Compress", "icon": null, "submenu": "Archive"})
	);
}

#[cfg(not(target_os = "linux"))]
#[tokio::test]
async fn context_actions_are_empty_and_unavailable_off_linux() {
	assert!(super::get_context_actions(vec!["/a".into()])
		.await
		.unwrap()
		.is_empty());
	assert!(super::run_context_action("id".into(), vec![])
		.await
		.is_err());
}

/// The real Linux opener and Dolphin service menus, with every XDG directory
/// and `PATH` pointing into a temporary folder. The checks run in a child
/// process because the environment is process-wide.
#[cfg(target_os = "linux")]
#[test]
fn linux_service_and_context_actions_use_the_user_desktop_files() {
	const CHILD_ENV: &str = "WINGDRIVE_TEST_CONTEXT_ACTIONS_CHILD";
	if let Some(root) = std::env::var_os(CHILD_ENV) {
		let root = Path::new(&root);
		let file = root.join("notes.txt");
		let other = root.join("other.txt");
		let runtime = tokio::runtime::Runtime::new().unwrap();
		runtime.block_on(async {
			let app = tauri::test::mock_app();
			app.manage(FileOpeningService::new());
			let apps = get_apps_for_paths(vec![file.clone()], app.state())
				.await
				.unwrap();
			assert_eq!(
				apps.iter().map(|a| a.id.as_str()).collect::<Vec<_>>(),
				["test-editor.desktop"]
			);

			let actions = super::get_context_actions(vec![file.clone(), other.clone()])
				.await
				.unwrap();
			assert_eq!(actions.len(), 1, "{actions:?}");
			assert_eq!(actions[0].id, "record.desktop#record");
			assert_eq!(actions[0].name, "Record selection");
			assert_eq!(actions[0].submenu.as_deref(), Some("Tools"));
			// Text-only actions are not offered for a folder.
			assert!(super::get_context_actions(vec![root.to_path_buf()])
				.await
				.unwrap()
				.is_empty());

			super::run_context_action(actions[0].id.clone(), vec![file.clone(), other.clone()])
				.await
				.unwrap();
			let recorded = root.join("recorded");
			let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
			while std::fs::read_to_string(&recorded).map_or(true, |s| s.lines().count() < 2) {
				assert!(std::time::Instant::now() < deadline, "action never ran");
				std::thread::sleep(std::time::Duration::from_millis(20));
			}
			assert_eq!(
				std::fs::read_to_string(&recorded).unwrap(),
				format!("{}\n{}\n", file.display(), other.display())
			);

			let error =
				super::run_context_action("record.desktop#record".into(), vec![root.join("gone")])
					.await
					.unwrap_err();
			assert!(error.ends_with("no longer exists"), "{error}");
			let error = super::run_context_action("record.desktop#missing".into(), vec![file])
				.await
				.unwrap_err();
			assert!(error.contains("does not apply"), "{error}");
		});
		return;
	}

	use std::os::unix::fs::PermissionsExt;
	let dir = tempfile::tempdir().unwrap();
	let root = dir.path();
	let config = root.join("config");
	let data = root.join("data");
	let system = root.join("system");
	let menus = data.join("kio/servicemenus");
	for folder in [
		&config,
		&system,
		&menus,
		&data.join("mime"),
		&data.join("applications"),
	] {
		std::fs::create_dir_all(folder).unwrap();
	}
	std::fs::create_dir_all(root.join("bin")).unwrap();
	for name in ["notes.txt", "other.txt"] {
		std::fs::write(root.join(name), "fixture").unwrap();
	}
	std::fs::write(data.join("mime/globs2"), "50:text/plain:*.txt\n").unwrap();
	std::fs::write(
		data.join("applications/test-editor.desktop"),
		"[Desktop Entry]\nType=Application\nName=Test Editor\nExec=/bin/true %f\nMimeType=text/plain;\n",
	)
	.unwrap();
	std::fs::write(
		config.join("mimeapps.list"),
		"[Default Applications]\ntext/plain=test-editor.desktop;\n",
	)
	.unwrap();
	let script = root.join("bin/record");
	std::fs::write(
		&script,
		format!(
			"#!/bin/sh\nfor f in \"$@\"; do printf '%s\\n' \"$f\" >> '{}/recorded'; done\n",
			root.display()
		),
	)
	.unwrap();
	std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
	let menu = menus.join("record.desktop");
	std::fs::write(
		&menu,
		format!(
			"[Desktop Entry]\nType=Service\nMimeType=text/plain;\nActions=record\nX-KDE-Submenu=Tools\n\n[Desktop Action record]\nName=Record selection\nExec={} %F\n",
			script.display()
		),
	)
	.unwrap();
	// Dolphin only trusts executable service menus in the user's folder.
	std::fs::set_permissions(&menu, std::fs::Permissions::from_mode(0o755)).unwrap();

	let output = std::process::Command::new(std::env::current_exe().unwrap())
		.args([
			"--exact",
			"file_opening::tests::linux_service_and_context_actions_use_the_user_desktop_files",
			"--nocapture",
		])
		.env(CHILD_ENV, root)
		.env("HOME", root)
		.env("XDG_CONFIG_HOME", &config)
		.env("XDG_DATA_HOME", &data)
		.env("XDG_CONFIG_DIRS", &system)
		.env("XDG_DATA_DIRS", &system)
		.env("XDG_CURRENT_DESKTOP", "")
		.env(
			"PATH",
			format!("{}:/usr/bin:/bin", root.join("bin").display()),
		)
		.env_remove("APPDIR")
		.env_remove("DBUS_SESSION_BUS_ADDRESS")
		.output()
		.unwrap();
	assert!(
		output.status.success(),
		"child failed: {}{}",
		String::from_utf8_lossy(&output.stdout),
		String::from_utf8_lossy(&output.stderr)
	);
}
