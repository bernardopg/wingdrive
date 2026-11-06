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
