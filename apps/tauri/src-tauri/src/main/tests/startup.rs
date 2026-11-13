//! App startup: the runtime configuration, the state handed to the commands,
//! the background daemon connection and the window and exit handlers.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde_json::{json, Value};
use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;
use tokio::sync::oneshot;

use super::events::{record, wait_for};
use super::{answering_daemon, app_state, closed_address, daemon_state, one_shot_daemon};
use crate::{
	background, connect_daemon, daemon_runtime_config_from_env, handle_run_event,
	handle_window_event, load_persisted_library_id, manage_app_state, open_library, setup_menus,
	terminal, AppState, DaemonLaunch, DaemonRuntimeConfig, DaemonState, MenuState,
};

#[test]
fn runtime_config_comes_from_the_data_dir_and_instance_values() {
	assert_eq!(
		daemon_runtime_config_from_env(Some("".into()), None)
			.err()
			.as_deref(),
		Some("WINGDRIVE_DATA_DIR cannot be empty")
	);

	let config = daemon_runtime_config_from_env(Some("/srv/wing".into()), None).unwrap();
	assert_eq!(config.base_data_dir, PathBuf::from("/srv/wing"));
	assert_eq!(config.data_dir, PathBuf::from("/srv/wing"));
	assert_eq!(config.socket_addr, "127.0.0.1:6969");
	assert!(config.instance.is_none());

	let config =
		daemon_runtime_config_from_env(Some("/srv/wing".into()), Some("dev".into())).unwrap();
	assert_eq!(config.data_dir, PathBuf::from("/srv/wing/instances/dev"));
	assert_ne!(config.socket_addr, "127.0.0.1:6969");

	let error = daemon_runtime_config_from_env(Some("/srv/wing".into()), Some("a b".into()))
		.err()
		.unwrap();
	assert!(error.contains("letters, numbers"), "{error}");

	// Without the variable the platform's data folder is used, if it resolves.
	match daemon_runtime_config_from_env(None, None) {
		Ok(config) => {
			assert_eq!(config.data_dir, config.base_data_dir);
			assert_eq!(config.socket_addr, "127.0.0.1:6969");
		}
		Err(error) => assert!(error.starts_with("Failed to resolve data directory")),
	}
}

#[test]
fn instance_names_are_limited_in_length() {
	let longest = "a".repeat(64);
	assert!(crate::daemon_runtime_config_for("/d".into(), Some(longest)).is_ok());
	let error = crate::daemon_runtime_config_for("/d".into(), Some("a".repeat(65)))
		.err()
		.unwrap();
	assert!(error.contains("too long"), "{error}");
	let error = crate::daemon_runtime_config_for("/d".into(), Some(String::new()))
		.err()
		.unwrap();
	assert!(error.contains("cannot be empty"), "{error}");
}

#[cfg(target_os = "linux")]
#[test]
fn appimage_scanner_and_gdk_backend_follow_the_environment() {
	use std::ffi::OsStr;

	use crate::{bundled_gst_plugin_scanner, forced_gdk_backend};

	let app_dir = tempfile::tempdir().unwrap();
	assert_eq!(bundled_gst_plugin_scanner(None), None);
	assert_eq!(
		bundled_gst_plugin_scanner(Some(app_dir.path().into())),
		None
	);
	let scanner = app_dir
		.path()
		.join("usr/lib/gstreamer-1.0/gst-plugin-scanner");
	// A folder with the scanner's name is not a scanner.
	std::fs::create_dir_all(&scanner).unwrap();
	assert_eq!(
		bundled_gst_plugin_scanner(Some(app_dir.path().into())),
		None
	);
	std::fs::remove_dir(&scanner).unwrap();
	std::fs::write(&scanner, "").unwrap();
	assert_eq!(
		bundled_gst_plugin_scanner(Some(app_dir.path().into())),
		Some(scanner)
	);

	let display = Some(OsStr::new(":0"));
	assert_eq!(forced_gdk_backend(None, display), Some("x11,wayland"));
	// A backend the user chose is kept, and without X there is no XWayland.
	assert_eq!(
		forced_gdk_backend(Some(OsStr::new("wayland")), display),
		None
	);
	assert_eq!(forced_gdk_backend(None, None), None);
}

#[test]
fn the_saved_library_is_read_trimmed_and_optional() {
	let data = tempfile::tempdir().unwrap();
	assert_eq!(load_persisted_library_id(data.path()), None);

	let file = data.path().join("current_library_id.txt");
	std::fs::write(&file, "lib-1\n").unwrap();
	assert_eq!(
		load_persisted_library_id(data.path()).as_deref(),
		Some("lib-1")
	);

	// An unreadable entry is treated as no selection rather than failing startup.
	std::fs::remove_file(&file).unwrap();
	std::fs::create_dir(&file).unwrap();
	assert_eq!(load_persisted_library_id(data.path()), None);
}

fn runtime(data_dir: &Path, socket_addr: &str) -> DaemonRuntimeConfig {
	DaemonRuntimeConfig {
		base_data_dir: data_dir.to_path_buf(),
		data_dir: data_dir.to_path_buf(),
		socket_addr: socket_addr.to_string(),
		instance: None,
	}
}

#[tokio::test]
async fn startup_state_follows_the_instance_directory() {
	let base = tempfile::tempdir().unwrap();
	let config = crate::daemon_runtime_config_for(base.path().into(), Some("dev".into())).unwrap();
	std::fs::create_dir_all(&config.data_dir).unwrap();
	std::fs::write(config.data_dir.join("current_library_id.txt"), "lib\n").unwrap();
	std::fs::write(
		config.data_dir.join("desktop_settings.json"),
		r#"{"keep_in_background": false}"#,
	)
	.unwrap();
	let data_dir = config.data_dir.clone();
	let socket_addr = config.socket_addr.clone();

	let app = tauri::test::mock_app();
	let daemon = manage_app_state(app.handle(), config);

	{
		let state = daemon.read().await;
		assert_eq!(state.data_dir, data_dir);
		assert_eq!(state.base_data_dir, base.path());
		assert_eq!(state.instance.as_deref(), Some("dev"));
		assert_eq!(state.socket_addr, socket_addr);
		assert!(!state.started_by_us && state.server_url.is_none());
	}
	// Commands reach the same state through the app.
	let managed = app.state::<Arc<tokio::sync::RwLock<DaemonState>>>();
	assert!(Arc::ptr_eq(&managed, &daemon));
	let app_state = app.state::<AppState>();
	assert_eq!(
		app_state.current_library_id.read().await.as_deref(),
		Some("lib")
	);
	assert_eq!(app_state.connection_pool.socket_addr, socket_addr);
	// Settings come from the instance, not from the base directory.
	assert!(!background::keeps_running(app.handle()));
	assert!(app.try_state::<crate::drag::DragCoordinator>().is_some());
	assert!(app.try_state::<crate::keybinds::KeybindState>().is_some());
	assert!(app
		.try_state::<crate::file_opening::FileOpeningService>()
		.is_some());
	#[cfg(target_os = "linux")]
	assert_eq!(
		app.state::<crate::file_chooser::PortalStateFile>().0,
		data_dir.join("file-chooser-portal.json")
	);

	// Files extracted from archives may be shown; the rest of the data is not.
	let scope = app.asset_protocol_scope();
	assert!(scope.is_allowed(data_dir.join("cache/archives/a.zip/photo.jpg")));
	assert!(!scope.is_allowed(data_dir.join("library.db")));
}

#[test]
fn the_menu_keeps_its_items_without_a_menu_bar() {
	let app = tauri::test::mock_app();
	setup_menus(app.handle());

	let menu_state = app.state::<MenuState<tauri::test::MockRuntime>>();
	let items = tauri::async_runtime::block_on(menu_state.items.read());
	for id in ["duplicate", "rename", "delete", "cut", "copy"] {
		assert!(items.contains_key(id), "{id}");
	}
	let bar = app.menu().expect("a menu is set");
	if cfg!(any(target_os = "windows", target_os = "linux")) {
		assert!(bar.items().unwrap().is_empty());
	} else {
		assert!(!bar.items().unwrap().is_empty());
	}
}

fn quick_launch() -> DaemonLaunch {
	DaemonLaunch {
		service_active: |_| false,
		daemon_path: Err("no daemon binary in tests".into()),
		startup_timeout: Duration::from_millis(300),
		validation_delay: Duration::ZERO,
	}
}

#[tokio::test]
async fn connecting_serves_files_and_forgets_a_library_that_is_gone() {
	let data = tempfile::tempdir().unwrap();
	let saved = data.path().join("current_library_id.txt");
	std::fs::write(&saved, "gone").unwrap();
	let address = answering_daemon(json!({ "JsonOk": [{ "id": "other" }] })).await;

	let app = tauri::test::mock_app();
	let connected = record(&app, "daemon-connected");
	let changes = record(&app, "library-changed");
	let daemon = manage_app_state(app.handle(), runtime(data.path(), &address));
	connect_daemon(app.handle().clone(), daemon.clone(), quick_launch()).await;

	{
		let state = daemon.read().await;
		let url = state.server_url.as_deref().unwrap();
		assert!(url.starts_with("http://127.0.0.1:"), "{url}");
		assert!(state.server_token.is_some() && state.server_shutdown.is_some());
		// The daemon was already running, so exiting must leave it alone.
		assert!(!state.started_by_us && state.daemon_process.is_none());
	}
	assert_eq!(connected.lock().unwrap().len(), 1);

	wait_for(|| !changes.lock().unwrap().is_empty()).await;
	assert_eq!(*changes.lock().unwrap(), ["\"\""]);
	assert!(!saved.exists());
	assert!(app
		.state::<AppState>()
		.current_library_id
		.read()
		.await
		.is_none());
}

#[tokio::test]
async fn without_a_daemon_the_windows_are_never_told_it_connected() {
	let data = tempfile::tempdir().unwrap();
	let app = tauri::test::mock_app();
	let connected = record(&app, "daemon-connected");
	let daemon = manage_app_state(app.handle(), runtime(data.path(), &closed_address().await));

	connect_daemon(app.handle().clone(), daemon.clone(), quick_launch()).await;

	let state = daemon.read().await;
	// The file server starts before the daemon is looked for.
	assert!(state.server_url.is_some());
	assert!(!state.started_by_us && state.daemon_process.is_none());
	assert!(connected.lock().unwrap().is_empty());
}

#[tokio::test]
async fn a_daemon_that_cannot_list_libraries_keeps_the_saved_one() {
	let data = tempfile::tempdir().unwrap();
	let saved = data.path().join("current_library_id.txt");
	std::fs::write(&saved, "lib").unwrap();
	// Answers every line with `Pong` and reports when the connection that
	// asked for the libraries has been closed by the app.
	let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
	let address = listener.local_addr().unwrap().to_string();
	let (asked, mut done) = tokio::sync::mpsc::unbounded_channel();
	tokio::spawn(async move {
		while let Ok((stream, _)) = listener.accept().await {
			let asked = asked.clone();
			tokio::spawn(async move {
				let (reader, mut writer) = stream.into_split();
				let mut reader = BufReader::new(reader);
				let mut line = String::new();
				let mut listed = false;
				while reader.read_line(&mut line).await.unwrap_or(0) > 0 {
					listed |= line.contains("query:libraries.list");
					let _ = writer.write_all(b"\"Pong\"\n").await;
					line.clear();
				}
				if listed {
					let _ = asked.send(());
				}
			});
		}
	});
	let app = tauri::test::mock_app();
	let changes = record(&app, "library-changed");
	let daemon = manage_app_state(app.handle(), runtime(data.path(), &address));

	connect_daemon(app.handle().clone(), daemon, quick_launch()).await;
	tokio::time::timeout(Duration::from_secs(5), done.recv())
		.await
		.unwrap()
		.unwrap();

	// The library may still exist, so an unusable answer changes nothing.
	assert!(saved.exists());
	assert_eq!(
		app.state::<AppState>()
			.current_library_id
			.read()
			.await
			.as_deref(),
		Some("lib")
	);
	assert!(changes.lock().unwrap().is_empty());
}

#[tokio::test]
async fn opening_a_library_from_the_menu_tells_the_windows_or_returns_the_refusal() {
	let (address, daemon) =
		one_shot_daemon("{\"JsonOk\":{\"id\":\"lib\",\"name\":\"Photos\"}}\n").await;
	let app = tauri::test::mock_app();
	app.manage(daemon_state(&address, None, None));
	let opened = record(&app, "library-opened");

	open_library(app.handle(), Path::new("/media/Photos.wing"))
		.await
		.unwrap();
	let sent: Value = serde_json::from_str(&daemon.await.unwrap()).unwrap();
	assert_eq!(sent["Action"]["payload"]["path"], "/media/Photos.wing");
	assert_eq!(
		serde_json::from_str::<Value>(&opened.lock().unwrap()[0]).unwrap(),
		json!({"id": "lib", "name": "Photos"})
	);

	let (address, _daemon) =
		one_shot_daemon("{\"Error\":{\"NotFound\":\"no library here\"}}\n").await;
	let app = tauri::test::mock_app();
	app.manage(daemon_state(&address, None, None));
	let opened = record(&app, "library-opened");
	let message = open_library(app.handle(), Path::new("/tmp/empty"))
		.await
		.unwrap_err();
	assert_eq!(message, "Failed to open library: no library here");
	assert!(opened.lock().unwrap().is_empty());
}

#[tokio::test]
async fn a_destroyed_window_releases_only_its_own_streams() {
	let app = tauri::test::mock_app();
	app.manage(app_state());
	app.manage(terminal::TerminalSessions::default());
	let manager = &app.state::<AppState>().subscription_manager;
	let (closing, closed) = oneshot::channel();
	let (staying, _stays) = oneshot::channel();
	manager.register(1, "settings".into(), closing).await;
	manager.register(2, "main".into(), staying).await;

	let settings = WebviewWindowBuilder::new(&app, "settings", WebviewUrl::default())
		.build()
		.unwrap();
	handle_window_event(&settings.as_ref().window(), &tauri::WindowEvent::Destroyed);

	assert_eq!(
		tokio::time::timeout(Duration::from_secs(5), closed).await,
		Ok(Ok(()))
	);
	assert_eq!(manager.get_active_count().await, 1);
}

#[test]
fn closing_main_hides_it_while_the_app_runs_in_the_background() {
	let data = tempfile::tempdir().unwrap();
	let app = tauri::test::mock_app();
	// Default settings keep the app running when its main window closes.
	app.manage(background::DesktopSettingsState::load(data.path()));
	let state = daemon_state("127.0.0.1:9", None, None);
	tauri::async_runtime::block_on(async { state.write().await.started_by_us = true });
	app.manage(state.clone());
	let main = WebviewWindowBuilder::new(&app, "main", WebviewUrl::default())
		.build()
		.unwrap();
	let other = WebviewWindowBuilder::new(&app, "settings", WebviewUrl::default())
		.build()
		.unwrap();

	let requests = Arc::new(Mutex::new(Vec::new()));
	let seen = requests.clone();
	// The mock runtime handles one request per second, in order. Each close of
	// a window it still has produces a close request; a destroyed or closed
	// window produces none. It does not call window event handlers itself, so
	// the loop hands each window event over the way the real runtime does.
	app.run(move |handle, event| {
		match &event {
			tauri::RunEvent::Ready => {
				other.close().unwrap();
				main.close().unwrap();
				main.close().unwrap();
				main.destroy().unwrap();
			}
			tauri::RunEvent::WindowEvent { label, event, .. } => {
				if let tauri::WindowEvent::CloseRequested { .. } = event {
					seen.lock().unwrap().push(label.clone());
				}
				let window = handle.get_webview_window(label).unwrap();
				handle_window_event(&window.as_ref().window(), event);
			}
			_ => {}
		}
		handle_run_event(handle, event);
	});

	// Main survived its first close, so the second one reached it again.
	assert_eq!(*requests.lock().unwrap(), ["settings", "main", "main"]);
	// Exiting stopped the daemon this app had started.
	assert!(!tauri::async_runtime::block_on(state.read()).started_by_us);
}

#[test]
fn the_app_gives_a_cold_daemon_ten_minutes() {
	let launch = DaemonLaunch::for_this_app();
	// A daemon binds its socket only after opening every library.
	assert_eq!(launch.startup_timeout, Duration::from_secs(600));
	assert_eq!(launch.validation_delay, Duration::from_secs(1));
	// Tests run without a daemon next to their executable.
	let error = launch.daemon_path.unwrap_err();
	assert!(error.starts_with("Daemon binary not found"), "{error}");
	assert!(!(launch.service_active)(Some("dev")));
}
