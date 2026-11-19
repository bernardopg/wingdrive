#![cfg(target_os = "linux")]

use std::{
	fs,
	os::unix::fs::PermissionsExt,
	path::{Path, PathBuf},
	process::{Command, Output},
};
use tempfile::TempDir;

const SERVICE: &str = "wingdrive-daemon@fixture.service";

struct Fixture {
	root: TempDir,
	cli: PathBuf,
	bin: PathBuf,
	home: PathBuf,
	data: PathBuf,
	log: PathBuf,
}

impl Fixture {
	fn new() -> Self {
		let root = TempDir::new().unwrap();
		let bin = root.path().join("program files");
		let home = root.path().join("isolated-home");
		let data = root
			.path()
			.join("My Data 100% with \"quotes\" and $dollars");
		let log = root.path().join("systemctl.log");
		fs::create_dir(&bin).unwrap();
		fs::create_dir(&home).unwrap();
		let cli = bin.join("wing");
		let built_cli = env!("CARGO_BIN_EXE_wing");
		if fs::hard_link(built_cli, &cli).is_err() {
			fs::copy(built_cli, &cli).unwrap();
		}
		Self::executable(&bin.join("wing-daemon"), "#!/bin/sh\nexit 0\n");
		Self::executable(
			&bin.join("systemctl"),
			r#"#!/bin/sh
printf '%s\n' "$*" >> "$WING_TEST_SYSTEMCTL_LOG"
if [ "$2" = "$WING_TEST_FAIL_SYSTEMCTL_STEP" ]; then
  printf '%s\n' "fixture rejected $2" >&2
  exit 1
fi
if [ "$2" = "is-active" ]; then
  printf '%s\n' "$WING_TEST_SYSTEMCTL_STATUS"
fi
"#,
		);
		Self {
			root,
			cli,
			bin,
			home,
			data,
			log,
		}
	}

	fn executable(path: &Path, script: &str) {
		fs::write(path, script).unwrap();
		fs::set_permissions(path, fs::Permissions::from_mode(0o700)).unwrap();
	}

	fn daemon(&self, operation: &str) -> Command {
		let mut command = Command::new(&self.cli);
		command
			.arg("--data-dir")
			.arg(&self.data)
			.args(["--instance", "fixture", "daemon", operation])
			.current_dir(self.root.path())
			.env("HOME", &self.home)
			.env("XDG_CONFIG_HOME", self.home.join(".config"))
			.env("PATH", &self.bin)
			.env("WING_TEST_SYSTEMCTL_LOG", &self.log)
			.env_remove("WING_TEST_FAIL_SYSTEMCTL_STEP")
			.env("WING_TEST_SYSTEMCTL_STATUS", "active");
		command
	}

	fn unit(&self) -> PathBuf {
		self.home.join(".config/systemd/user").join(SERVICE)
	}
}

fn successful(output: Output) -> String {
	assert!(
		output.status.success(),
		"{}",
		String::from_utf8_lossy(&output.stderr)
	);
	String::from_utf8(output.stdout).unwrap()
}

#[test]
fn systemd_unit_quotes_both_executable_and_data_paths() {
	let fixture = Fixture::new();
	successful(fixture.daemon("install").output().unwrap());

	let daemon = fixture.bin.join("wing-daemon");
	let unescaped = fixture.data.to_string_lossy();
	let escaped = unescaped
		.replace('%', "%%")
		.replace('$', "$$")
		.replace('"', "\\\"");
	let expected = format!(
		"ExecStart=\"{}\" --data-dir \"{escaped}\" --instance fixture",
		daemon.display()
	);
	let unit = fs::read_to_string(fixture.unit()).unwrap();
	assert!(unit.lines().any(|line| line == expected), "{unit}");
}

#[test]
fn systemd_install_status_and_uninstall_target_the_named_service() {
	let fixture = Fixture::new();
	let status = successful(fixture.daemon("status").output().unwrap());
	assert!(status.contains("Not installed"));
	assert!(!fixture.log.exists());

	let install = successful(fixture.daemon("install").output().unwrap());
	assert!(install.contains("installed and started successfully"));
	assert!(fixture.unit().is_file());

	for (state, expected) in [
		("active", "Active (running)"),
		("inactive", "Inactive (stopped)"),
		("failed", "Service status: Failed"),
		("activating", "Service status: activating"),
	] {
		let status = successful(
			fixture
				.daemon("status")
				.env("WING_TEST_SYSTEMCTL_STATUS", state)
				.output()
				.unwrap(),
		);
		assert!(status.contains(expected), "{status}");
	}

	let uninstall = successful(fixture.daemon("uninstall").output().unwrap());
	assert!(uninstall.contains("uninstalled successfully"));
	assert!(!fixture.unit().exists());
	let status = successful(fixture.daemon("uninstall").output().unwrap());
	assert!(status.contains("is not installed"));
	let calls = fs::read_to_string(fixture.log).unwrap();
	let expected = [
		"--user daemon-reload".to_string(),
		format!("--user enable {SERVICE}"),
		format!("--user start {SERVICE}"),
		format!("--user is-active {SERVICE}"),
		format!("--user is-active {SERVICE}"),
		format!("--user is-active {SERVICE}"),
		format!("--user is-active {SERVICE}"),
		format!("--user stop {SERVICE}"),
		format!("--user disable {SERVICE}"),
		"--user daemon-reload".to_string(),
	];
	assert_eq!(calls.lines().collect::<Vec<_>>(), expected);
}

#[test]
fn systemd_install_reports_enable_and_start_failures() {
	for step in ["enable", "start"] {
		let fixture = Fixture::new();
		let output = fixture
			.daemon("install")
			.env("WING_TEST_FAIL_SYSTEMCTL_STEP", step)
			.output()
			.unwrap();
		assert!(!output.status.success());
		assert!(
			String::from_utf8_lossy(&output.stderr).contains(&format!("fixture rejected {step}"))
		);
	}
}

#[test]
fn systemd_install_reports_a_missing_bundled_daemon() {
	let fixture = Fixture::new();
	fs::remove_file(fixture.bin.join("wing-daemon")).unwrap();
	let output = fixture.daemon("install").output().unwrap();
	assert!(!output.status.success());
	assert!(String::from_utf8_lossy(&output.stderr).contains("Daemon binary not found"));
	assert!(!fixture.unit().exists());
	assert!(!fixture.log.exists());
}
