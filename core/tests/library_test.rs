//! Integration tests for the library system

use tempfile::TempDir;
use wing_core::Core;

#[tokio::test]
async fn test_library_lifecycle() {
	// Create temporary directory for test
	let temp_dir = TempDir::new().unwrap();

	// Initialize core with custom data directory
	let core = Core::new(temp_dir.path().to_path_buf()).await.unwrap();

	// Create library (will be created in the libraries directory)
	let library = core
		.libraries
		.create_library("Test Library", None, core.context.clone())
		.await
		.unwrap();

	assert_eq!(library.name().await, "Test Library");

	// Verify directory structure
	let lib_path = library.path();
	assert!(lib_path.exists());
	assert!(lib_path.join("library.json").exists());
	assert!(lib_path.join("library.db").exists());
	assert!(lib_path.join("previews").exists());
	assert!(lib_path.join("exports").exists());

	// Test configuration update
	library
		.update_config(|config| {
			config.description = Some("Test description".to_string());
			config.settings.thumbnail_quality = 90;
		})
		.await
		.unwrap();

	let config = library.config().await;
	assert_eq!(config.description, Some("Test description".to_string()));
	assert_eq!(config.settings.thumbnail_quality, 90);

	// Close library
	let lib_id = library.id();
	let lib_path = library.path().to_path_buf();
	core.libraries.close_library(lib_id).await.unwrap();

	// Drop the library reference to release the lock
	drop(library);

	// Verify can't close again
	assert!(core.libraries.close_library(lib_id).await.is_err());

	// Re-open library
	let reopened = core
		.libraries
		.open_library(&lib_path, core.context.clone())
		.await
		.unwrap();
	assert_eq!(reopened.id(), lib_id);
	assert_eq!(reopened.name().await, "Test Library");

	// Verify data persisted
	// assert!(reopened.has_thumbnail(cas_id).await);
	let config = reopened.config().await;
	assert_eq!(config.description, Some("Test description".to_string()));
}

#[tokio::test]
async fn test_library_locking() {
	let temp_dir = TempDir::new().unwrap();
	let core = Core::new(temp_dir.path().to_path_buf()).await.unwrap();

	// Create library
	let library = core
		.libraries
		.create_library("Lock Test", None, core.context.clone())
		.await
		.unwrap();

	let lib_path = library.path().to_path_buf();

	// Try to open same library again - should fail
	let result = core
		.libraries
		.open_library(&lib_path, core.context.clone())
		.await;
	assert!(result.is_err());

	// Close library
	let lib_id = library.id();
	core.libraries.close_library(lib_id).await.unwrap();

	// Drop the library reference to release the lock
	drop(library);

	// Now should be able to open
	let reopened = core
		.libraries
		.open_library(&lib_path, core.context.clone())
		.await
		.unwrap();
	assert_eq!(reopened.name().await, "Lock Test");
}

#[tokio::test]
async fn test_library_discovery() {
	let temp_dir = TempDir::new().unwrap();
	let core = Core::new(temp_dir.path().to_path_buf()).await.unwrap();

	// Create multiple libraries
	let lib1 = core
		.libraries
		.create_library("Library 1", None, core.context.clone())
		.await
		.unwrap();

	let lib2 = core
		.libraries
		.create_library("Library 2", None, core.context.clone())
		.await
		.unwrap();

	// Close both
	let lib1_id = lib1.id();
	let lib2_id = lib2.id();
	core.libraries.close_library(lib1_id).await.unwrap();
	core.libraries.close_library(lib2_id).await.unwrap();

	// Drop library references to release locks
	drop(lib1);
	drop(lib2);

	// Test auto-loading - reload all libraries
	let loaded_count = core.libraries.load_all(core.context.clone()).await.unwrap();
	assert!(loaded_count >= 2);

	// Verify libraries were loaded
	let open_libraries = core.libraries.list().await;
	let names: Vec<String> =
		futures::future::join_all(open_libraries.iter().map(|lib| lib.name())).await;

	assert!(names.iter().any(|n| n == "Library 1"));
	assert!(names.iter().any(|n| n == "Library 2"));
}

#[tokio::test]
async fn test_library_name_sanitization() {
	let temp_dir = TempDir::new().unwrap();
	let core = Core::new(temp_dir.path().to_path_buf()).await.unwrap();

	// Create library with problematic name
	let library = core
		.libraries
		.create_library("My/Library:Name*", None, core.context.clone())
		.await
		.unwrap();

	// Verify directory name was sanitized
	let dir_name = library.path().file_name().unwrap().to_str().unwrap();
	assert!(dir_name.ends_with(".winglibrary"));
	assert!(!dir_name.contains('/'));
	assert!(!dir_name.contains(':'));
	assert!(!dir_name.contains('*'));
}

#[tokio::test]
async fn test_default_library_creation() {
	let temp_dir = TempDir::new().unwrap();

	// Initialize core with fresh temporary directory (no existing libraries)
	let core = Core::new(temp_dir.path().to_path_buf()).await.unwrap();

	// Check that a default library was created automatically
	let open_libraries = core.libraries.list().await;
	assert_eq!(
		open_libraries.len(),
		1,
		"Expected exactly one default library to be created"
	);

	let default_library = &open_libraries[0];
	assert_eq!(
		default_library.name().await,
		"My Library",
		"Default library should be named 'My Library'"
	);

	// Verify directory structure exists
	let lib_path = default_library.path();
	assert!(lib_path.exists());
	assert!(lib_path.join("library.json").exists());
	assert!(lib_path.join("library.db").exists());
	assert!(lib_path.join("previews").exists());
	assert!(lib_path.join("exports").exists());
}

/// A slug changed through device.update must reach the library's device row,
/// or listings scoped to the stale slug never receive live events.
#[tokio::test]
async fn device_slug_change_reaches_open_library_records() {
	use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
	use wing_core::infra::db::entities::device;

	let temp_dir = TempDir::new().unwrap();
	let core = Core::new(temp_dir.path().to_path_buf()).await.unwrap();
	let library = core
		.libraries
		.create_library("Slug Library", None, core.context.clone())
		.await
		.unwrap();
	let device_id = core.context.device_manager.device_id().unwrap();

	core.context
		.device_manager
		.update(None, Some("renamed-machine".to_string()))
		.unwrap();
	core.libraries.refresh_device_records().await;

	let row = device::Entity::find()
		.filter(device::Column::Uuid.eq(device_id))
		.one(library.db().conn())
		.await
		.unwrap()
		.unwrap();
	assert_eq!(row.slug, "renamed-machine");
}

/// Jobs left running by a previous process must not stay "running" forever.
/// Auto-resume is off, so resumable jobs wait as paused and others fail.
#[tokio::test]
async fn interrupted_jobs_are_settled_on_library_open() {
	use sea_orm::{ActiveModelTrait, Database, Set};
	use wing_core::infra::job::database::jobs;

	let temp_dir = TempDir::new().unwrap();
	let core = Core::new(temp_dir.path().to_path_buf()).await.unwrap();
	let library = core
		.libraries
		.create_library("Jobs Library", None, core.context.clone())
		.await
		.unwrap();

	let db = Database::connect(format!(
		"sqlite://{}?mode=rwc",
		library.path().join("jobs.db").display()
	))
	.await
	.unwrap();
	let row = |id: &str, name: &str, status: &str| jobs::ActiveModel {
		id: Set(id.to_string()),
		name: Set(name.to_string()),
		state: Set(Vec::new()),
		status: Set(status.to_string()),
		priority: Set(0),
		progress_type: Set(None),
		progress_data: Set(None),
		parent_job_id: Set(None),
		created_at: Set(chrono::Utc::now()),
		started_at: Set(Some(chrono::Utc::now())),
		completed_at: Set(None),
		paused_at: Set(None),
		error_message: Set(None),
		warnings: Set(None),
		non_critical_errors: Set(None),
		metrics: Set(None),
		action_context: Set(None),
		action_type: Set(None),
	};
	let resumable = uuid::Uuid::new_v4().to_string();
	let not_resumable = uuid::Uuid::new_v4().to_string();
	// Shutdown pauses running jobs, so a paused row may be one that cannot resume.
	let paused_not_resumable = uuid::Uuid::new_v4().to_string();
	let paused_resumable = uuid::Uuid::new_v4().to_string();
	row(&resumable, "file_copy", "running")
		.insert(&db)
		.await
		.unwrap();
	row(&not_resumable, "archive_compress", "running")
		.insert(&db)
		.await
		.unwrap();
	row(&paused_not_resumable, "wingdrop_send", "paused")
		.insert(&db)
		.await
		.unwrap();
	row(&paused_resumable, "file_copy", "paused")
		.insert(&db)
		.await
		.unwrap();

	library.jobs().reconcile_interrupted_jobs().await.unwrap();

	let statuses: std::collections::HashMap<String, String> = library
		.jobs()
		.list_jobs(None)
		.await
		.unwrap()
		.into_iter()
		.map(|job| (job.id.to_string(), format!("{:?}", job.status)))
		.collect();
	assert_eq!(statuses.get(&resumable).map(String::as_str), Some("Paused"));
	assert_eq!(
		statuses.get(&not_resumable).map(String::as_str),
		Some("Failed")
	);
	assert_eq!(
		statuses.get(&paused_not_resumable).map(String::as_str),
		Some("Failed")
	);
	assert_eq!(
		statuses.get(&paused_resumable).map(String::as_str),
		Some("Paused")
	);
}

/// The library directory watcher debounces a new `.winglibrary` folder for
/// 500 ms. A library its creator closes inside that window must stay closed:
/// the watcher used to reopen it from the folder's own create event, and the
/// reopened copy held the lock against the next open.
#[tokio::test]
async fn closing_a_new_library_inside_the_watcher_debounce_keeps_it_closed() {
	let temp_dir = TempDir::new().unwrap();
	let core = Core::new(temp_dir.path().to_path_buf()).await.unwrap();

	let library = core
		.libraries
		.create_library("Closed Early", None, core.context.clone())
		.await
		.unwrap();
	let id = library.id();
	core.libraries.close_library(id).await.unwrap();
	drop(library);

	tokio::time::sleep(std::time::Duration::from_millis(1200)).await;

	assert!(
		core.libraries.get_library(id).await.is_none(),
		"the watcher reopened a library its creator closed"
	);
}

/// The directory watcher debounces a new or changed `.winglibrary` folder
/// for 500 ms before it opens it. A shutdown inside that window used to let
/// the watcher's last tick reopen the library `close_all` had just closed,
/// and the orphan held the lock file against the next process.
///
/// The tick runs every 100 ms, so the stop is landed at several points
/// near the end of the window; one of them is the last tick's.
///
/// Ported from upstream Spacedrive `393158c23`.
#[tokio::test]
async fn stopping_the_watcher_inside_its_debounce_reopens_nothing() {
	// Windows refuses a plain open of a directory without backup semantics.
	fn touch_directory(path: &std::path::Path) {
		let mut options = std::fs::OpenOptions::new();
		options.read(true);
		#[cfg(windows)]
		{
			use std::os::windows::fs::OpenOptionsExt;
			options.write(true).custom_flags(0x0200_0000);
		}
		options
			.open(path)
			.unwrap()
			.set_modified(std::time::SystemTime::now())
			.unwrap();
	}

	let temp_dir = TempDir::new().unwrap();
	let core = Core::new(temp_dir.path().to_path_buf()).await.unwrap();
	let lib_path = core.libraries.list().await[0].path().to_path_buf();
	let lock_path = lib_path.join(".winglibrary.lock");

	for offset_ms in (400u64..500).step_by(10) {
		touch_directory(&lib_path);
		tokio::time::sleep(std::time::Duration::from_millis(offset_ms)).await;
		core.libraries.stop_watching().await.unwrap();
		core.libraries.close_all().await.unwrap();
		tokio::time::sleep(std::time::Duration::from_millis(300)).await;

		assert!(
			core.libraries.list().await.is_empty(),
			"no library is open after a stop {offset_ms} ms into the debounce"
		);
		assert!(
			!lock_path.exists(),
			"the lock file is released after a stop {offset_ms} ms into the debounce"
		);

		core.libraries.load_all(core.context.clone()).await.unwrap();
		assert_eq!(core.libraries.list().await.len(), 1);
		core.libraries.start_watching().await.unwrap();
	}
	core.shutdown().await.unwrap();
}
