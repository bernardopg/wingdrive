use std::process::{Command, Output};

fn run(args: &[&str]) -> Output {
	Command::new(env!("CARGO_BIN_EXE_wing"))
		.args(args)
		.output()
		.unwrap()
}

#[test]
fn unknown_subcommand_exits_with_clap_error() {
	let out = run(&["not-a-command"]);
	assert_eq!(out.status.code(), Some(2));
	assert!(String::from_utf8_lossy(&out.stderr).contains("unrecognized subcommand"));
}

#[test]
fn help_exits_successfully() {
	let out = run(&["--help"]);
	assert!(out.status.success());
	let text = String::from_utf8(out.stdout).unwrap();
	assert!(text.contains("--instance"));
	assert!(text.contains("--data-dir"));
}

#[test]
fn invalid_instance_exits_with_explanation() {
	let dir = tempfile::tempdir().unwrap();
	let out = run(&[
		"--data-dir",
		dir.path().to_str().unwrap(),
		"--instance",
		"../bad",
		"config",
		"show",
	]);
	assert_eq!(out.status.code(), Some(1));
	assert!(String::from_utf8_lossy(&out.stderr).contains("Invalid instance name"));
	assert!(!dir.path().join("cli.json").exists());
}

#[test]
fn config_table_uses_explicit_data_directory() {
	let dir = tempfile::tempdir().unwrap();
	let out = run(&["--data-dir", dir.path().to_str().unwrap(), "config", "show"]);
	assert!(out.status.success());
	let text = String::from_utf8(out.stdout).unwrap();
	assert!(text.contains("current_library_id"));
	assert!(text.contains("update.repo"));
	assert!(text.contains("bernardopg/wingdrive"));
	assert!(dir.path().join("cli.json").is_file());
}

#[test]
fn unknown_config_key_exits_with_error() {
	let dir = tempfile::tempdir().unwrap();
	let out = run(&[
		"--data-dir",
		dir.path().to_str().unwrap(),
		"config",
		"get",
		"invalid.key",
	]);
	assert_eq!(out.status.code(), Some(1));
	assert!(String::from_utf8_lossy(&out.stderr).contains("Unknown config key: invalid.key"));
}

#[test]
fn library_list_json_is_valid_and_instance_reaches_isolated_rpc() {
	use std::io::{Read, Write};
	use std::net::TcpListener;
	use std::time::{Duration, Instant};

	let (instance, listener) = (0..1000)
		.find_map(|n| {
			let instance = format!("cli_fixture_{n}");
			let port = 6970 + instance.bytes().map(u16::from).sum::<u16>() % 1000;
			TcpListener::bind(("127.0.0.1", port))
				.ok()
				.map(|listener| (instance, listener))
		})
		.expect("no available test instance port");
	listener.set_nonblocking(true).unwrap();
	let server = std::thread::spawn(move || {
		let deadline = Instant::now() + Duration::from_secs(15);
		for _ in 0..2 {
			let mut stream = loop {
				match listener.accept() {
					Ok((stream, _)) => break stream,
					Err(error)
						if error.kind() == std::io::ErrorKind::WouldBlock
							&& Instant::now() < deadline =>
					{
						std::thread::sleep(Duration::from_millis(10));
					}
					Err(error) => panic!("mock RPC accept failed: {error}"),
				}
			};
			stream
				.set_read_timeout(Some(Duration::from_secs(5)))
				.unwrap();
			let mut request = Vec::new();
			stream.read_to_end(&mut request).unwrap();
			let request: wing_core::infra::daemon::types::DaemonRequest =
				serde_json::from_slice(&request).unwrap();
			assert!(
				matches!(request, wing_core::infra::daemon::types::DaemonRequest::Query { ref method, .. } if method == "query:libraries.list")
			);
			let response = wing_core::infra::daemon::types::DaemonResponse::JsonOk(
				serde_json::json!([
					{"id": "550e8400-e29b-41d4-a716-446655440000", "name": "fixture", "path": "fixture", "stats": null}
				]),
			);
			stream
				.write_all(&serde_json::to_vec(&response).unwrap())
				.unwrap();
		}
	});
	let dir = tempfile::tempdir().unwrap();
	let out = run(&[
		"--data-dir",
		dir.path().to_str().unwrap(),
		"--instance",
		&instance,
		"--format",
		"json",
		"library",
		"list",
	]);
	server.join().unwrap();
	assert!(
		out.status.success(),
		"{}",
		String::from_utf8_lossy(&out.stderr)
	);
	let value: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
	assert_eq!(value[0]["name"], "fixture");
}
