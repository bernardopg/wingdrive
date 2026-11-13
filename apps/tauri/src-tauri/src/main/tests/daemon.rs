//! Finding, spawning, waiting for and stopping the daemon process, with shell
//! scripts standing in for `wing-daemon` and a TCP listener answering pings.

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;

use super::closed_address;
use crate::{find_daemon_binary_in, spawn_daemon, terminate_daemon, wait_for_daemon_within};

/// Writes an executable `sh` script named `name` into `dir`.
fn script(dir: &Path, name: &str, body: &str) -> PathBuf {
	let path = dir.join(name);
	std::fs::write(&path, format!("#!/bin/sh\n{body}\n")).unwrap();
	std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
	path
}

/// Retries `attempt` while it fails with "Text file busy".
///
/// A script written moments ago cannot be executed while another test thread
/// is between `fork` and `exec`, because the child briefly holds the write
/// handle. The error clears once that child has exec'd.
async fn retry_busy<T>(mut attempt: impl AsyncFnMut() -> Result<T, String>) -> Result<T, String> {
	for _ in 0..50 {
		match attempt().await {
			Err(error) if error.contains("Text file busy") => {
				tokio::time::sleep(Duration::from_millis(20)).await
			}
			result => return result,
		}
	}
	attempt().await
}

/// `Command::spawn` for a just-written script, with the same retry.
fn spawn_script(path: &Path) -> std::process::Child {
	for _ in 0..50 {
		match std::process::Command::new(path).spawn() {
			Err(error) if error.raw_os_error() == Some(libc::ETXTBSY) => {
				std::thread::sleep(Duration::from_millis(20))
			}
			result => return result.unwrap(),
		}
	}
	std::process::Command::new(path).spawn().unwrap()
}

/// Answers every connection's first line with `Pong`, as the daemon does.
async fn ponging_daemon() -> String {
	let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
	let address = listener.local_addr().unwrap().to_string();
	tokio::spawn(async move {
		while let Ok((stream, _)) = listener.accept().await {
			tokio::spawn(async move {
				let (reader, mut writer) = stream.into_split();
				let mut line = String::new();
				if BufReader::new(reader).read_line(&mut line).await.is_ok() {
					let _ = writer.write_all(b"\"Pong\"\n").await;
				}
			});
		}
	});
	address
}

#[test]
fn daemon_binary_is_looked_up_next_to_the_app() {
	let dir = tempfile::tempdir().unwrap();
	let error = find_daemon_binary_in(dir.path()).unwrap_err();
	assert!(error.starts_with("Daemon binary not found"), "{error}");

	let suffixed = dir.path().join(format!(
		"wing-daemon-{}{}",
		env!("WING_TARGET_TRIPLE"),
		std::env::consts::EXE_SUFFIX
	));
	std::fs::write(&suffixed, "").unwrap();
	assert_eq!(find_daemon_binary_in(dir.path()).unwrap(), suffixed);

	// A debug build prefers the plain binary cargo keeps current; a release
	// bundle prefers the suffixed one Tauri ships.
	let plain = dir
		.path()
		.join(format!("wing-daemon{}", std::env::consts::EXE_SUFFIX));
	std::fs::write(&plain, "").unwrap();
	let expected = if cfg!(debug_assertions) {
		&plain
	} else {
		&suffixed
	};
	assert_eq!(&find_daemon_binary_in(dir.path()).unwrap(), expected);
}

#[tokio::test]
async fn a_daemon_that_answers_is_returned_running_and_then_stopped() {
	let dir = tempfile::tempdir().unwrap();
	let daemon = script(dir.path(), "wing-daemon", "exec sleep 30");
	let address = ponging_daemon().await;
	let mut child = retry_busy(async || {
		spawn_daemon(
			&daemon,
			&dir.path().to_path_buf(),
			Some("test"),
			&address,
			Duration::from_secs(10),
		)
		.await
	})
	.await
	.unwrap();
	assert!(child.try_wait().unwrap().is_none());

	// SIGTERM ends it well before the grace period runs out.
	let started = Instant::now();
	terminate_daemon(&mut child, &address);
	assert!(child.try_wait().unwrap().is_some());
	assert!(started.elapsed() < Duration::from_secs(5));
	// Stopping an already reaped daemon is a no-op.
	terminate_daemon(&mut child, &address);
}

#[tokio::test]
async fn a_daemon_that_exits_or_never_answers_is_reported() {
	let dir = tempfile::tempdir().unwrap();
	let crashing = script(dir.path(), "crashing", "exit 3");
	let error = retry_busy(async || {
		spawn_daemon(
			&crashing,
			&dir.path().to_path_buf(),
			None,
			&closed_address().await,
			Duration::from_secs(10),
		)
		.await
	})
	.await
	.unwrap_err();
	assert!(
		error.starts_with("Packaged daemon exited during startup"),
		"{error}"
	);

	let silent = script(dir.path(), "silent", "exec sleep 30");
	let error = retry_busy(async || {
		spawn_daemon(
			&silent,
			&dir.path().to_path_buf(),
			None,
			&closed_address().await,
			Duration::from_millis(300),
		)
		.await
	})
	.await
	.unwrap_err();
	assert!(
		error.starts_with("Daemon failed to start (connection not available"),
		"{error}"
	);

	let error = retry_busy(async || {
		spawn_daemon(
			&dir.path().join("missing"),
			&dir.path().to_path_buf(),
			None,
			&closed_address().await,
			Duration::from_secs(1),
		)
		.await
	})
	.await
	.unwrap_err();
	assert!(error.starts_with("Failed to start daemon"), "{error}");
}

#[tokio::test]
async fn waiting_for_another_daemon_stops_at_the_time_limit() {
	assert!(wait_for_daemon_within(&ponging_daemon().await, Duration::from_secs(5)).await);
	let started = Instant::now();
	assert!(!wait_for_daemon_within(&closed_address().await, Duration::from_millis(200)).await);
	assert!(started.elapsed() < Duration::from_secs(5));
}

#[test]
fn a_daemon_that_ignores_sigterm_is_killed() {
	let dir = tempfile::tempdir().unwrap();
	// Traps SIGTERM, so only the kill after the grace period stops it.
	let stubborn = script(
		dir.path(),
		"stubborn",
		"trap '' TERM; while :; do sleep 1; done",
	);
	let mut child = spawn_script(&stubborn);
	std::thread::sleep(Duration::from_millis(200));
	terminate_daemon(&mut child, "127.0.0.1:9");
	assert!(child.try_wait().unwrap().is_some());
}

#[tokio::test]
async fn starting_the_daemon_refuses_a_running_one_and_reaps_the_old_child() {
	use tauri::Manager;

	use super::{app_with_daemon, daemon_state};
	use crate::{get_daemon_status, start_daemon_process, stop_daemon_process};

	let address = ponging_daemon().await;
	let app = app_with_daemon(daemon_state(&address, None, None));
	assert_eq!(
		start_daemon_process(app.handle().clone(), app.state())
			.await
			.unwrap_err(),
		"Daemon is already running"
	);
	let status = get_daemon_status(app.state()).await.unwrap();
	assert!(status.is_running && !status.started_by_us);
	assert_eq!(
		stop_daemon_process(app.state()).await.unwrap_err(),
		"Cannot stop daemon we didn't start"
	);

	// Nothing answers: the crashed child is reaped before a new start, which
	// fails here because no daemon binary sits next to the test executable.
	let state = daemon_state(&closed_address().await, None, None);
	let mut crashed = std::process::Command::new("sleep")
		.arg("30")
		.spawn()
		.unwrap();
	let pid = crashed.id();
	let _ = crashed.kill();
	state.write().await.daemon_process = Some(Arc::new(tokio::sync::Mutex::new(Some(crashed))));
	let app = app_with_daemon(state);
	let error = start_daemon_process(app.handle().clone(), app.state())
		.await
		.unwrap_err();
	assert!(error.starts_with("Daemon binary not found"), "{error}");
	let state = app.state::<Arc<tokio::sync::RwLock<crate::DaemonState>>>();
	assert!(state.read().await.daemon_process.is_none(), "pid {pid}");
}

#[test]
fn exiting_stops_only_a_daemon_this_app_started() {
	use tauri::Manager;

	use super::daemon_state;
	use crate::stop_owned_daemon;

	// No daemon state at all: nothing to stop.
	stop_owned_daemon(tauri::test::mock_app().handle());

	let state = tauri::async_runtime::block_on(async {
		let state = daemon_state("127.0.0.1:9", None, None);
		let child = std::process::Command::new("sleep")
			.arg("30")
			.spawn()
			.unwrap();
		state.write().await.daemon_process = Some(Arc::new(tokio::sync::Mutex::new(Some(child))));
		state
	});
	let app = tauri::test::mock_app();
	app.manage(state.clone());
	// Not ours: the process is left alone.
	stop_owned_daemon(app.handle());
	let running = tauri::async_runtime::block_on(async {
		state.read().await.daemon_process.clone().unwrap()
	});

	tauri::async_runtime::block_on(async { state.write().await.started_by_us = true });
	stop_owned_daemon(app.handle());
	tauri::async_runtime::block_on(async {
		assert!(!state.read().await.started_by_us);
		assert!(state.read().await.daemon_process.is_none());
		assert!(running.lock().await.is_none());
	});
}

#[tokio::test]
async fn stopping_a_daemon_this_app_started_terminates_it() {
	use tauri::Manager;

	use super::{app_with_daemon, daemon_state};
	use crate::stop_daemon_process;

	let state = daemon_state("127.0.0.1:9", None, None);
	let child = std::process::Command::new("sleep")
		.arg("30")
		.spawn()
		.unwrap();
	let process = Arc::new(tokio::sync::Mutex::new(Some(child)));
	{
		let mut state = state.write().await;
		state.started_by_us = true;
		state.daemon_process = Some(process.clone());
	}
	let app = app_with_daemon(state);
	stop_daemon_process(app.state()).await.unwrap();
	assert!(process.lock().await.is_none());
	let state = app.state::<Arc<tokio::sync::RwLock<crate::DaemonState>>>();
	assert!(!state.read().await.started_by_us);
}

#[test]
fn the_first_ready_main_window_stays_hidden_after_a_hidden_launch() {
	use std::sync::atomic::AtomicBool;

	use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

	use crate::{app_ready, launch::StartHidden};

	let app = tauri::test::mock_app();
	app.manage(StartHidden(AtomicBool::new(true)));
	let main = WebviewWindowBuilder::new(&app, "main", WebviewUrl::default())
		.build()
		.unwrap();
	let other = WebviewWindowBuilder::new(&app, "settings-general", WebviewUrl::default())
		.build()
		.unwrap();
	app_ready(other.as_ref().window(), app.state());
	assert!(app.state::<StartHidden>().is_set());
	app_ready(main.as_ref().window(), app.state());
	assert!(!app.state::<StartHidden>().is_set());
	app_ready(main.as_ref().window(), app.state());
}

/// Drops every connection for its first 1.2 s, then answers like
/// `ponging_daemon`: a daemon that comes up after the app first looked for one
/// and takes longer than a second, as a cold start does.
async fn late_daemon() -> String {
	let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
	let address = listener.local_addr().unwrap().to_string();
	let ready_at = Instant::now() + Duration::from_millis(1200);
	tokio::spawn(async move {
		while let Ok((stream, _)) = listener.accept().await {
			if Instant::now() < ready_at {
				continue;
			}
			tokio::spawn(async move {
				let (reader, mut writer) = stream.into_split();
				let mut line = String::new();
				if BufReader::new(reader).read_line(&mut line).await.is_ok() {
					let _ = writer.write_all(b"\"Pong\"\n").await;
				}
			});
		}
	});
	address
}

fn launch(
	service_active: fn(Option<&str>) -> bool,
	daemon_path: Result<PathBuf, String>,
	startup_timeout: Duration,
) -> crate::DaemonLaunch {
	crate::DaemonLaunch {
		service_active,
		daemon_path,
		startup_timeout,
		validation_delay: Duration::ZERO,
	}
}

#[tokio::test]
async fn startup_waits_for_a_login_service_that_is_still_starting() {
	use crate::acquire_daemon;

	let base = PathBuf::from("/nonexistent");
	let missing = || Err::<PathBuf, String>("not used".into());
	let service = launch(|_| true, missing(), Duration::from_secs(10));
	let (ours, child) = acquire_daemon(&late_daemon().await, &base, None, &service)
		.await
		.unwrap();
	assert!(!ours && child.is_none());

	let service = launch(|_| true, missing(), Duration::from_millis(300));
	assert!(
		acquire_daemon(&closed_address().await, &base, None, &service)
			.await
			.is_none()
	);

	// A named instance never waits for the default instance's service.
	#[cfg(target_os = "linux")]
	assert!(!crate::daemon_service_active(Some("dev")));
}

#[tokio::test]
async fn startup_spawns_its_own_daemon_or_waits_for_one_started_elsewhere() {
	use crate::acquire_daemon;

	let dir = tempfile::tempdir().unwrap();
	let daemon = script(
		dir.path(),
		"wing-daemon",
		"[ \"$1\" = probe ] && exit 0\nexec sleep 30",
	);
	// Once one run succeeds, no thread still holds the script open for writing.
	spawn_probe(&daemon);
	let base = dir.path().to_path_buf();

	let own = launch(|_| false, Ok(daemon.clone()), Duration::from_secs(10));
	let (ours, child) = acquire_daemon(&late_daemon().await, &base, Some("dev"), &own)
		.await
		.unwrap();
	let mut child = child.expect("the spawned daemon");
	assert!(ours);
	assert!(child.try_wait().unwrap().is_none());
	terminate_daemon(&mut child, "127.0.0.1:9");

	// Another launch holds the library: its daemon answers once ready.
	let failing = launch(|_| false, Err("not found".into()), Duration::from_secs(10));
	let (ours, child) = acquire_daemon(&late_daemon().await, &base, None, &failing)
		.await
		.unwrap();
	assert!(!ours && child.is_none());

	let failing = launch(
		|_| false,
		Err("not found".into()),
		Duration::from_millis(300),
	);
	assert!(
		acquire_daemon(&closed_address().await, &base, None, &failing)
			.await
			.is_none()
	);
}

/// Runs `path probe` until the kernel stops reporting the file busy.
fn spawn_probe(path: &Path) {
	for _ in 0..50 {
		match std::process::Command::new(path).arg("probe").status() {
			Err(error) if error.raw_os_error() == Some(libc::ETXTBSY) => {
				std::thread::sleep(Duration::from_millis(20))
			}
			result => {
				assert!(result.unwrap().success());
				return;
			}
		}
	}
	panic!("{path:?} stayed busy");
}

#[tokio::test]
async fn installing_the_service_stops_the_daemon_this_app_started() {
	use tauri::Listener;

	use super::daemon_state;
	use crate::prepare_service_install;

	let state = daemon_state("127.0.0.1:9", None, None);
	let child = std::process::Command::new("sleep")
		.arg("30")
		.spawn()
		.unwrap();
	let process = Arc::new(tokio::sync::Mutex::new(Some(child)));
	state.write().await.daemon_process = Some(process.clone());
	let app = tauri::test::mock_app();
	let starting = Arc::new(std::sync::atomic::AtomicUsize::new(0));
	let seen = starting.clone();
	app.listen_any("daemon-starting", move |_| {
		seen.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
	});

	let (data_dir, socket_addr) = prepare_service_install(app.handle(), &state).await;

	assert_eq!(data_dir, PathBuf::from("/nonexistent/wingdrive-test"));
	assert_eq!(socket_addr, "127.0.0.1:9");
	// The child was stopped and reaped, not merely forgotten.
	assert!(process.lock().await.is_none());
	assert!(state.read().await.daemon_process.is_none());
	assert_eq!(starting.load(std::sync::atomic::Ordering::SeqCst), 1);
}
