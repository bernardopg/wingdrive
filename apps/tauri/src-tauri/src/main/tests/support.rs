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
