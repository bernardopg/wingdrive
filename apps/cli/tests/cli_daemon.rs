#![cfg(unix)]

use std::{
	fs,
	io::Read,
	net::TcpStream,
	path::{Path, PathBuf},
	process::{Child, Command, Output, Stdio},
	thread,
	time::{Duration, Instant},
};


fn test_tempdir() -> tempfile::TempDir {
	let base = PathBuf::from("/home/bitter/.cache/opencode/cov10-tmp");
	fs::create_dir_all(&base).expect("create cov10 tmp base");
	tempfile::Builder::new()
		.prefix("cov10-")
		.tempdir_in(&base)
		.expect("tempdir")
}
struct Daemon {
	child: Child,
	data_dir: tempfile::TempDir,
	instance: String,
	_port: u16,
}

impl Daemon {
	fn start() -> Self {
		let data_dir = test_tempdir();
		let instance = unique_instance();
		let port = instance_port(&instance);
		let daemon_bin = daemon_bin();
		let mut child = Command::new(&daemon_bin)
			.arg("--data-dir")
			.arg(data_dir.path())
			.arg("--instance")
			.arg(&instance)
			.stdin(Stdio::null())
			.stdout(Stdio::null())
			.stderr(Stdio::null())
			.spawn()
			.unwrap_or_else(|e| panic!("spawn {}: {e}", daemon_bin.display()));
		wait_for_port(port, &mut child);
		Self {
			child,
			data_dir,
			instance,
			_port: port,
		}
	}

	fn data_dir(&self) -> &Path {
		self.data_dir.path()
	}

	fn run(&self, args: &[&str]) -> Output {
		let mut cmd = Command::new(env!("CARGO_BIN_EXE_wing"));
		cmd.arg("--data-dir")
			.arg(self.data_dir())
			.arg("--instance")
			.arg(&self.instance)
			.args(args);
		cmd.output().expect("run wing")
	}

	fn run_json(&self, args: &[&str]) -> (Output, serde_json::Value) {
		let mut full = vec!["--format", "json"];
		full.extend_from_slice(args);
		let out = self.run(&full);
		let stdout = String::from_utf8_lossy(&out.stdout);
		let mut values: Vec<serde_json::Value> = serde_json::Deserializer::from_str(&stdout)
			.into_iter()
			.filter_map(Result::ok)
			.collect();
		let value = if values.len() == 1 {
			values.pop().unwrap()
		} else if values.is_empty() {
			panic!(
				"json parse failed\nstdout: {}\nstderr: {}",
				stdout,
				String::from_utf8_lossy(&out.stderr)
			);
		} else {
			serde_json::Value::Array(values)
		};
		(out, value)
	}
}

impl Drop for Daemon {
	fn drop(&mut self) {
		let _ = self.child.kill();
		let _ = self.child.wait();
	}
}

fn daemon_bin() -> PathBuf {
	if let Ok(path) = std::env::var("WING_DAEMON_BIN") {
		return PathBuf::from(path);
	}
	let mut path = std::env::current_exe().expect("current exe");
	path.pop();
	if path.ends_with("deps") {
		path.pop();
	}
	path.push("wing-daemon");
	if path.exists() {
		return path;
	}
	let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
	let fallback = manifest.join("../../target/debug/wing-daemon");
	if fallback.exists() {
		return fallback;
	}
	panic!(
		"wing-daemon binary not found at {} or {}; build it or set WING_DAEMON_BIN",
		path.display(),
		fallback.display()
	);
}

fn unique_instance() -> String {
	use std::sync::atomic::{AtomicU64, Ordering};
	static N: AtomicU64 = AtomicU64::new(0);
	let n = N.fetch_add(1, Ordering::Relaxed);
	format!("cov10_{}_{}_{n}", std::process::id(), n)
}

fn instance_port(instance: &str) -> u16 {
	6970 + instance.bytes().map(u16::from).sum::<u16>() % 1000
}

fn wait_for_port(port: u16, child: &mut Child) {
	let deadline = Instant::now() + Duration::from_secs(30);
	while Instant::now() < deadline {
		if let Some(status) = child.try_wait().expect("try_wait") {
			panic!("daemon exited early: {status}");
		}
		if TcpStream::connect(("127.0.0.1", port)).is_ok() {
			return;
		}
		thread::sleep(Duration::from_millis(50));
	}
	panic!("daemon did not listen on 127.0.0.1:{port} within 30s");
}

fn assert_success(out: &Output) -> String {
	assert!(
		out.status.success(),
		"status: {:?}\nstdout: {}\nstderr: {}",
		out.status.code(),
		String::from_utf8_lossy(&out.stdout),
		String::from_utf8_lossy(&out.stderr)
	);
	String::from_utf8_lossy(&out.stdout).into_owned()
}

fn assert_failure(out: &Output) -> String {
	assert!(
		!out.status.success(),
		"expected failure, got success\nstdout: {}\nstderr: {}",
		String::from_utf8_lossy(&out.stdout),
		String::from_utf8_lossy(&out.stderr)
	);
	String::from_utf8_lossy(&out.stderr).into_owned()
}

fn create_library(daemon: &Daemon, name: &str) -> String {
	let out = daemon.run(&["library", "create", name]);
	let stdout = assert_success(&out);
	// Output contains "Created library <name> with ID <uuid> ..."
	for token in stdout.split_whitespace() {
		if uuid::Uuid::parse_str(token).is_ok() {
			return token.to_string();
		}
	}
	panic!("no library uuid in output: {stdout}");
}

fn add_location(daemon: &Daemon, path: &Path) -> String {
	let out = daemon.run(&["location", "add", path.to_str().unwrap()]);
	let stdout = assert_success(&out);
	for token in stdout.split_whitespace() {
		if uuid::Uuid::parse_str(token).is_ok() {
			return token.to_string();
		}
	}
	panic!("no location uuid in output: {stdout}");
}

fn wait_for_job(daemon: &Daemon, job_id: &str) {
	let deadline = Instant::now() + Duration::from_secs(60);
	while Instant::now() < deadline {
		let out = daemon.run(&["--format", "json", "job", "info", job_id]);
		if out.status.success() {
			let value: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap_or_default();
			let status = value
				.get("status")
				.or_else(|| value.get("job").and_then(|j| j.get("status")))
				.and_then(|s| s.as_str())
				.unwrap_or("");
			match status {
				"Completed" | "completed" => return,
				"Failed" | "failed" | "Cancelled" | "cancelled" => {
					panic!("job {job_id} ended with status {status}: {value}");
				}
				_ => {}
			}
		}
		thread::sleep(Duration::from_millis(100));
	}
	panic!("job {job_id} did not complete within 60s");
}

fn extract_job_id(text: &str) -> Option<String> {
	for token in text.split_whitespace() {
		let cleaned = token.trim_matches(|c: char| !c.is_alphanumeric() && c != '-');
		if uuid::Uuid::parse_str(cleaned).is_ok() {
			return Some(cleaned.to_string());
		}
	}
	None
}

#[test]
fn daemon_isolated_library_location_file_flow() {
	let daemon = Daemon::start();
	let lib_id = create_library(&daemon, "cov10-lib");

	let (out, json) = daemon.run_json(&["library", "list"]);
	assert_success(&out);
	assert!(json.as_array().unwrap().iter().any(|l| l["id"] == lib_id));

	let status = daemon.run(&["status"]);
	let status_text = assert_success(&status);
	assert!(!status_text.is_empty());

	let workspace = test_tempdir();
	let root = workspace.path().join("root");
	fs::create_dir(&root).unwrap();
	fs::write(root.join("a.txt"), "alpha").unwrap();
	let _loc_id = add_location(&daemon, &root);

	// mkdir
	let mkdir_out = daemon.run(&["file", "mkdir", root.to_str().unwrap(), "sub"]);
	let mkdir_text = assert_success(&mkdir_out);
	assert!(mkdir_text.to_lowercase().contains("created") || mkdir_text.contains("sub"));
	assert!(root.join("sub").is_dir());

	// mkdir error: existing
	let mkdir_err = daemon.run(&["file", "mkdir", root.to_str().unwrap(), "sub"]);
	assert!(!mkdir_err.status.success());

	// copy
	let dest = root.join("sub").join("a-copy.txt");
	let copy_out = daemon.run(&[
		"file",
		"copy",
		root.join("a.txt").to_str().unwrap(),
		"--destination",
		dest.to_str().unwrap(),
	]);
	let copy_text = assert_success(&copy_out);
	if let Some(job_id) = extract_job_id(&copy_text) {
		wait_for_job(&daemon, &job_id);
	}
	assert_eq!(fs::read_to_string(&dest).unwrap(), "alpha");

	// file list
	let list_out = daemon.run(&["file", "list", root.to_str().unwrap()]);
	let list_text = assert_success(&list_out);
	assert!(list_text.contains("a.txt"), "list: {list_text}");

	// file info
	let info_out = daemon.run(&["file", "info", root.join("a.txt").to_str().unwrap()]);
	let info_text = assert_success(&info_out);
	assert!(info_text.contains("a.txt") || info_text.to_lowercase().contains("file"), "info: {info_text}");

	// search (may return empty if indexing hasn't completed, but command must work)
	let (search_out, search_json) = daemon.run_json(&["search", "files", "a.txt"]);
	assert_success(&search_out);
	assert!(search_json.get("results").is_some() || search_json.get("total_found").is_some(), "search: {search_json}");

	// copy error: missing source
	let bad_copy = daemon.run(&[
		"file",
		"copy",
		root.join("missing.txt").to_str().unwrap(),
		"--destination",
		root.join("sub/missing-copy.txt").to_str().unwrap(),
	]);
	// may fail immediately or as failed job; accept either
	if bad_copy.status.success() {
		let text = String::from_utf8_lossy(&bad_copy.stdout);
		if let Some(job_id) = extract_job_id(&text) {
			let deadline = Instant::now() + Duration::from_secs(30);
			let mut failed = false;
			while Instant::now() < deadline {
				let out = daemon.run(&["--format", "json", "job", "info", &job_id]);
				if out.status.success() {
					let v: serde_json::Value =
						serde_json::from_slice(&out.stdout).unwrap_or_default();
					let status = v
						.get("status")
						.or_else(|| v.get("job").and_then(|j| j.get("status")))
						.and_then(|s| s.as_str())
						.unwrap_or("");
					if matches!(status, "Failed" | "failed") {
						failed = true;
						break;
					}
					if matches!(status, "Completed" | "completed") {
						break;
					}
				}
				thread::sleep(Duration::from_millis(100));
			}
			assert!(failed, "expected copy of missing source to fail");
		} else {
			panic!("copy missing source succeeded unexpectedly: {text}");
		}
	}

	// rename
	let rename_out = daemon.run(&[
		"file",
		"rename",
		dest.to_str().unwrap(),
		"b.txt",
	]);
	let rename_text = assert_success(&rename_out);
	if let Some(job_id) = extract_job_id(&rename_text) {
		wait_for_job(&daemon, &job_id);
	}
	assert!(!dest.exists());
	assert_eq!(fs::read_to_string(root.join("sub/b.txt")).unwrap(), "alpha");

	// rename error: missing target
	let rename_err = daemon.run(&[
		"file",
		"rename",
		root.join("sub/nope.txt").to_str().unwrap(),
		"nope2.txt",
	]);
	if rename_err.status.success() {
		let text = String::from_utf8_lossy(&rename_err.stdout);
		if let Some(job_id) = extract_job_id(&text) {
			let deadline = Instant::now() + Duration::from_secs(30);
			let mut failed = false;
			while Instant::now() < deadline {
				let out = daemon.run(&["--format", "json", "job", "info", &job_id]);
				if out.status.success() {
					let v: serde_json::Value =
						serde_json::from_slice(&out.stdout).unwrap_or_default();
					let status = v
						.get("status")
						.or_else(|| v.get("job").and_then(|j| j.get("status")))
						.and_then(|s| s.as_str())
						.unwrap_or("");
					if matches!(status, "Failed" | "failed") {
						failed = true;
						break;
					}
					if matches!(status, "Completed" | "completed") {
						break;
					}
				}
				thread::sleep(Duration::from_millis(100));
			}
			assert!(failed, "expected rename of missing file to fail");
		}
	} else {
		assert_failure(&rename_err);
	}

	// move (copy --move-files)
	let move_dest = root.join("moved.txt");
	let move_out = daemon.run(&[
		"file",
		"copy",
		root.join("sub/b.txt").to_str().unwrap(),
		"--destination",
		move_dest.to_str().unwrap(),
		"--move-files",
	]);
	let move_text = assert_success(&move_out);
	if let Some(job_id) = extract_job_id(&move_text) {
		wait_for_job(&daemon, &job_id);
	}
	assert_eq!(fs::read_to_string(&move_dest).unwrap(), "alpha");
	assert!(!root.join("sub/b.txt").exists());

	// move error: missing source
	let bad_move = daemon.run(&[
		"file",
		"copy",
		root.join("sub/gone.txt").to_str().unwrap(),
		"--destination",
		root.join("gone2.txt").to_str().unwrap(),
		"--move-files",
	]);
	if bad_move.status.success() {
		let text = String::from_utf8_lossy(&bad_move.stdout);
		if let Some(job_id) = extract_job_id(&text) {
			let deadline = Instant::now() + Duration::from_secs(30);
			let mut failed = false;
			while Instant::now() < deadline {
				let out = daemon.run(&["--format", "json", "job", "info", &job_id]);
				if out.status.success() {
					let v: serde_json::Value =
						serde_json::from_slice(&out.stdout).unwrap_or_default();
					let status = v
						.get("status")
						.or_else(|| v.get("job").and_then(|j| j.get("status")))
						.and_then(|s| s.as_str())
						.unwrap_or("");
					if matches!(status, "Failed" | "failed") {
						failed = true;
						break;
					}
					if matches!(status, "Completed" | "completed") {
						break;
					}
				}
				thread::sleep(Duration::from_millis(100));
			}
			assert!(failed, "expected move of missing source to fail");
		}
	}

	// delete (trash by default) - needs -y? permanent needs confirm; default is trash, no confirm
	let delete_out = daemon.run(&["file", "delete", move_dest.to_str().unwrap()]);
	let delete_text = assert_success(&delete_out);
	if let Some(job_id) = extract_job_id(&delete_text) {
		wait_for_job(&daemon, &job_id);
	}
	assert!(!move_dest.exists());

	// delete error: missing path with permanent + yes
	let delete_err = daemon.run(&[
		"file",
		"delete",
		root.join("already-gone.txt").to_str().unwrap(),
		"--permanent",
		"-y",
	]);
	if delete_err.status.success() {
		let text = String::from_utf8_lossy(&delete_err.stdout);
		if let Some(job_id) = extract_job_id(&text) {
			let deadline = Instant::now() + Duration::from_secs(30);
			let mut failed = false;
			while Instant::now() < deadline {
				let out = daemon.run(&["--format", "json", "job", "info", &job_id]);
				if out.status.success() {
					let v: serde_json::Value =
						serde_json::from_slice(&out.stdout).unwrap_or_default();
					let status = v
						.get("status")
						.or_else(|| v.get("job").and_then(|j| j.get("status")))
						.and_then(|s| s.as_str())
						.unwrap_or("");
					if matches!(status, "Failed" | "failed") {
						failed = true;
						break;
					}
					if matches!(status, "Completed" | "completed") {
						break;
					}
				}
				thread::sleep(Duration::from_millis(100));
			}
			assert!(failed, "expected delete of missing file to fail, job {job_id}");
		} else {
			// No job id - command itself should have signaled error in output
			assert!(
				text.to_lowercase().contains("error") || text.to_lowercase().contains("fail"),
				"expected delete failure signal: {text}"
			);
		}
	} else {
		assert_failure(&delete_err);
	}
}

#[test]
fn daemon_unavailable_reports_error() {
	let dir = test_tempdir();
	// use an instance name that is unlikely to have a daemon; don't start one
	let instance = unique_instance();
	let out = Command::new(env!("CARGO_BIN_EXE_wing"))
		.arg("--data-dir")
		.arg(dir.path())
		.arg("--instance")
		.arg(&instance)
		.args(["library", "list"])
		.output()
		.unwrap();
	assert!(!out.status.success());
	let stderr = String::from_utf8_lossy(&out.stderr);
	assert!(!stderr.trim().is_empty(), "stderr should explain failure");
}

#[test]
fn missing_library_reports_error() {
	let daemon = Daemon::start();
	// Querying a nonexistent library should fail with a clear error.
	let out = daemon.run(&["library", "info", "00000000-0000-0000-0000-000000000001"]);
	assert!(
		!out.status.success(),
		"stdout: {}\nstderr: {}",
		String::from_utf8_lossy(&out.stdout),
		String::from_utf8_lossy(&out.stderr)
	);
	let stderr = String::from_utf8_lossy(&out.stderr).to_lowercase();
	let stdout = String::from_utf8_lossy(&out.stdout).to_lowercase();
	assert!(
		stderr.contains("library") || stderr.contains("not found") || stdout.contains("not found"),
		"stderr: {}\nstdout: {}",
		String::from_utf8_lossy(&out.stderr),
		String::from_utf8_lossy(&out.stdout)
	);
}

#[test]
fn status_and_job_json_work_with_isolated_daemon() {
	let daemon = Daemon::start();
	let _lib_id = create_library(&daemon, "cov10-status");
	let status = daemon.run(&["status"]);
	assert_success(&status);

	let (out, json) = daemon.run_json(&["job", "list"]);
	assert_success(&out);
	assert!(json.is_array() || json.get("jobs").is_some());
}

#[test]
fn daemon_timeout_reports_error() {
	use std::net::TcpListener;
	let instance = unique_instance();
	let port = instance_port(&instance);
	let listener = TcpListener::bind(("127.0.0.1", port)).expect("bind");
	listener.set_nonblocking(false).ok();
	let dir = test_tempdir();
	let handle = std::thread::spawn(move || {
		if let Ok((mut stream, _)) = listener.accept() {
			let mut buf = [0u8; 1024];
			let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
			let _ = stream.read(&mut buf);
			// drop without responding - CLI should report error
			thread::sleep(Duration::from_millis(200));
		}
	});
	let out = Command::new(env!("CARGO_BIN_EXE_wing"))
		.arg("--data-dir")
		.arg(dir.path())
		.arg("--instance")
		.arg(&instance)
		.args(["library", "list"])
		.output()
		.unwrap();
	assert!(!out.status.success());
	let stderr = String::from_utf8_lossy(&out.stderr);
	assert!(!stderr.trim().is_empty(), "timeout should report error: {stderr}");
	let _ = handle.join();
}
