//! Installing the daemon as a login service and handing the connection over to
//! it. `systemctl` is replaced by a recorder, so the user's service manager is
//! never reached.

use std::sync::Arc;

use serde_json::json;

use super::events::record;
use super::{answering_daemon, app_state, closed_address, daemon_state};
use crate::reconnect_to_service_daemon;

#[tokio::test]
async fn handing_over_to_the_service_drops_the_child_and_reconnects() {
	let address = answering_daemon(json!("Pong")).await;
	let app = tauri::test::mock_app();
	let connected = record(&app, "daemon-connected");
	let state = daemon_state(&address, None, None);
	{
		let mut state = state.write().await;
		state.started_by_us = true;
		state.daemon_process = Some(Arc::new(tokio::sync::Mutex::new(None)));
	}
	let app_state = crate::AppState {
		connection_pool: Arc::new(crate::DaemonConnectionPool::new(address.clone())),
		..app_state()
	};
	// A pool that still believes in its old connection would never reconnect.
	*app_state.connection_pool.initialized.lock().await = true;
	app_state
		.connection_pool
		.subscriptions
		.write()
		.await
		.insert(4, ());

	reconnect_to_service_daemon(app.handle(), &state, &app_state, &address, 30)
		.await
		.unwrap();

	let daemon = state.read().await;
	// Exiting the app must now leave the service's daemon running.
	assert!(!daemon.started_by_us && daemon.daemon_process.is_none());
	assert!(*app_state.connection_pool.initialized.lock().await);
	assert!(app_state
		.connection_pool
		.subscriptions
		.read()
		.await
		.is_empty());
	assert_eq!(connected.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn a_service_that_never_answers_is_reported() {
	let app = tauri::test::mock_app();
	let address = closed_address().await;
	let state = daemon_state(&address, None, None);
	let app_state = app_state();
	app_state
		.connection_pool
		.subscriptions
		.write()
		.await
		.insert(4, ());

	let error = reconnect_to_service_daemon(app.handle(), &state, &app_state, &address, 3)
		.await
		.unwrap_err();

	assert_eq!(error, "Daemon failed to start after installing service");
	// The old connection state is gone even though no daemon answered.
	assert!(app_state
		.connection_pool
		.subscriptions
		.read()
		.await
		.is_empty());
}

#[cfg(target_os = "linux")]
mod systemd {
	use std::os::unix::process::ExitStatusExt;
	use std::path::Path;
	use std::process::{ExitStatus, Output};
	use std::sync::Mutex;

	use crate::{
		install_systemd_service, systemd_exec_argument, systemd_service_installed, systemd_unit,
		uninstall_systemd_services,
	};

	/// Records each `systemctl` call and fails the one whose first non-`--user`
	/// argument is `fail_on`.
	fn systemctl<'a>(
		calls: &'a Mutex<Vec<String>>,
		fail_on: Option<&'a str>,
	) -> impl Fn(&[&str]) -> std::io::Result<Output> + 'a {
		move |args| {
			calls.lock().unwrap().push(args.join(" "));
			let failed = fail_on == args.get(1).copied();
			Ok(Output {
				status: ExitStatus::from_raw(if failed { 1 << 8 } else { 0 }),
				stdout: Vec::new(),
				stderr: if failed {
					b"denied".to_vec()
				} else {
					Vec::new()
				},
			})
		}
	}

	#[test]
	fn unit_arguments_survive_spaces_and_systemd_specifiers() {
		assert_eq!(
			systemd_exec_argument(Path::new("/home/me/My Files")),
			"\"/home/me/My Files\""
		);
		assert_eq!(
			systemd_exec_argument(Path::new("/d/100% \"$HOME\" a\\b\nc")),
			"\"/d/100%% \\\"$$HOME\\\" a\\\\b\\nc\""
		);

		let unit = systemd_unit(Path::new("/opt/wing daemon"), Path::new("/data/My Files"));
		assert!(
			unit.contains("ExecStart=\"/opt/wing daemon\" --data-dir \"/data/My Files\"\n"),
			"{unit}"
		);
		assert!(unit.contains("Restart=on-failure"));
		assert!(unit.contains("WantedBy=default.target"));
	}

	#[test]
	fn installing_writes_the_unit_then_reloads_enables_and_starts_it() {
		let home = tempfile::tempdir().unwrap();
		let systemd_dir = home.path().join(".config/systemd/user");
		assert!(!systemd_service_installed(&systemd_dir));
		let calls = Mutex::new(Vec::new());

		install_systemd_service(
			&systemd_dir,
			Path::new("/opt/wing-daemon"),
			Path::new("/data"),
			systemctl(&calls, None),
		)
		.unwrap();

		let unit = std::fs::read_to_string(systemd_dir.join("wingdrive-daemon.service")).unwrap();
		assert!(unit.contains("ExecStart=\"/opt/wing-daemon\" --data-dir \"/data\""));
		assert!(systemd_service_installed(&systemd_dir));
		assert_eq!(
			*calls.lock().unwrap(),
			[
				"--user daemon-reload",
				"--user enable wingdrive-daemon.service",
				"--user start wingdrive-daemon.service",
			]
		);
	}

	#[test]
	fn installing_stops_at_the_first_failing_step() {
		let home = tempfile::tempdir().unwrap();
		let systemd_dir = home.path().join("user");
		for (step, message, made) in [
			("daemon-reload", "Failed to reload systemd: denied", 1),
			("enable", "Failed to enable service: denied", 2),
			("start", "Failed to start daemon service: denied", 3),
		] {
			let calls = Mutex::new(Vec::new());
			let error = install_systemd_service(
				&systemd_dir,
				Path::new("/d"),
				Path::new("/data"),
				systemctl(&calls, Some(step)),
			)
			.unwrap_err();
			assert_eq!(error, message);
			assert_eq!(calls.lock().unwrap().len(), made);
		}

		let error =
			install_systemd_service(&systemd_dir, Path::new("/d"), Path::new("/data"), |_| {
				Err(std::io::Error::from(std::io::ErrorKind::NotFound))
			})
			.unwrap_err();
		assert!(error.starts_with("Failed to reload systemd: "), "{error}");

		// A file where the unit folder should be stops before systemctl runs.
		let blocked = home.path().join("blocked");
		std::fs::write(&blocked, "").unwrap();
		let calls = Mutex::new(Vec::new());
		let error = install_systemd_service(
			&blocked,
			Path::new("/d"),
			Path::new("/data"),
			systemctl(&calls, None),
		)
		.unwrap_err();
		assert!(
			error.starts_with("Failed to create systemd directory"),
			"{error}"
		);
		assert!(calls.lock().unwrap().is_empty());
	}

	#[test]
	fn uninstalling_removes_current_and_legacy_units() {
		let systemd_dir = tempfile::tempdir().unwrap();
		std::fs::write(systemd_dir.path().join("spacedrive-daemon.service"), "").unwrap();
		assert!(systemd_service_installed(systemd_dir.path()));
		let calls = Mutex::new(Vec::new());

		// A unit systemd refuses to stop is still deleted.
		uninstall_systemd_services(systemd_dir.path(), systemctl(&calls, Some("stop"))).unwrap();

		assert!(!systemd_service_installed(systemd_dir.path()));
		assert_eq!(
			*calls.lock().unwrap(),
			[
				"--user stop spacedrive-daemon.service",
				"--user disable spacedrive-daemon.service",
				"--user daemon-reload",
			]
		);

		// Nothing installed: only the reload runs.
		let calls = Mutex::new(Vec::new());
		uninstall_systemd_services(systemd_dir.path(), systemctl(&calls, None)).unwrap();
		assert_eq!(*calls.lock().unwrap(), ["--user daemon-reload"]);

		// A unit that cannot be removed is reported.
		std::fs::create_dir(systemd_dir.path().join("wingdrive-daemon.service")).unwrap();
		let error =
			uninstall_systemd_services(systemd_dir.path(), systemctl(&calls, None)).unwrap_err();
		assert!(
			error.starts_with("Failed to remove service file"),
			"{error}"
		);
	}
}
