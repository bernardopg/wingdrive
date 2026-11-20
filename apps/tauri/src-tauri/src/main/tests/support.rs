use std::{
	io::{BufRead, BufReader},
	process::{Child, Command, Stdio},
};

pub(crate) struct PrivateBus {
	daemon: Child,
	address: String,
}

impl PrivateBus {
	pub(crate) fn start() -> Option<Self> {
		let mut daemon = Command::new("dbus-daemon")
			.args(["--session", "--nofork", "--print-address=1"])
			.stdout(Stdio::piped())
			.stderr(Stdio::null())
			.spawn()
			.ok()?;
		let mut address = String::new();
		BufReader::new(daemon.stdout.take()?)
			.read_line(&mut address)
			.ok()?;
		Some(Self {
			daemon,
			address: address.trim().to_string(),
		})
	}

	pub(crate) fn builder(&self) -> zbus::connection::Builder<'_> {
		zbus::connection::Builder::address(self.address.as_str())
	}

	pub(crate) async fn connect(&self) -> zbus::Connection {
		self.builder().build().await.unwrap()
	}
}

impl Drop for PrivateBus {
	fn drop(&mut self) {
		self.daemon.kill().ok();
		self.daemon.wait().ok();
	}
}

/// Exercises a command that resolves process-wide environment in an isolated child.
pub(crate) fn run_child_test(name: &str, configure: impl FnOnce(&mut Command)) {
	let mut command = Command::new(std::env::current_exe().unwrap());
	command.args(["--exact", name, "--nocapture"]);
	configure(&mut command);
	let output = command.output().unwrap();
	let stdout = String::from_utf8_lossy(&output.stdout);
	assert!(
		output.status.success() && stdout.contains("1 passed"),
		"{name} [{}]: {stdout}\n{}",
		output.status,
		String::from_utf8_lossy(&output.stderr)
	);
}
