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

fn json_rpc_fixture(
	expected_methods: &[&str],
	responses: Vec<serde_json::Value>,
) -> (
	String,
	tempfile::TempDir,
	std::thread::JoinHandle<Vec<serde_json::Value>>,
) {
	use std::io::{Read, Write};
	use std::net::TcpListener;
	use std::time::{Duration, Instant};

	let (instance, listener) = (0..1000)
		.find_map(|n| {
			let instance = format!("cli_json_fixture_{n}");
			let port = 6970 + instance.bytes().map(u16::from).sum::<u16>() % 1000;
			TcpListener::bind(("127.0.0.1", port))
				.ok()
				.map(|listener| (instance, listener))
		})
		.expect("no available test instance port");
	listener.set_nonblocking(true).unwrap();
	let expected: Vec<String> = expected_methods.iter().map(|s| (*s).to_owned()).collect();
	let server = std::thread::spawn(move || {
		let deadline = Instant::now() + Duration::from_secs(15);
		let mut payloads = Vec::new();
		for (method, response) in expected.into_iter().zip(responses) {
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
			let (got_method, payload) = match request {
				wing_core::infra::daemon::types::DaemonRequest::Query {
					method, payload, ..
				}
				| wing_core::infra::daemon::types::DaemonRequest::Action {
					method, payload, ..
				} => (method, payload),
				other => panic!("unexpected request: {other:?}"),
			};
			assert_eq!(got_method, method);
			payloads.push(payload);
			let response = wing_core::infra::daemon::types::DaemonResponse::JsonOk(response);
			stream
				.write_all(&serde_json::to_vec(&response).unwrap())
				.unwrap();
		}
		payloads
	});
	let dir = tempfile::tempdir().unwrap();
	(instance, dir, server)
}

#[test]
fn library_list_json_is_valid_and_instance_reaches_isolated_rpc() {
	let (instance, dir, server) = json_rpc_fixture(
		&["query:libraries.list", "query:libraries.list"],
		vec![
			serde_json::json!([
				{"id": "550e8400-e29b-41d4-a716-446655440000", "name": "fixture", "path": "fixture", "stats": null}
			]),
			serde_json::json!([
				{"id": "550e8400-e29b-41d4-a716-446655440000", "name": "fixture", "path": "fixture", "stats": null}
			]),
		],
	);
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
	assert!(value.is_array());
	assert_eq!(value[0]["name"], "fixture");
	assert_eq!(value[0]["id"], "550e8400-e29b-41d4-a716-446655440000");
}

#[test]
fn tag_search_json_output_is_structurally_valid() {
	let library = "550e8400-e29b-41d4-a716-446655440000";
	let (instance, dir, server) = json_rpc_fixture(
		&["query:libraries.list", "query:tags.search"],
		vec![
			serde_json::json!([
				{"id": library, "name": "fixture", "path": "fixture", "stats": null}
			]),
			serde_json::json!({
				"tags": [],
				"total_found": 0,
				"disambiguated": false,
				"query": "report",
				"filters": {
					"namespace": "work",
					"tag_type": null,
					"include_archived": true,
					"limit": 5
				}
			}),
		],
	);
	let out = run(&[
		"--data-dir",
		dir.path().to_str().unwrap(),
		"--instance",
		&instance,
		"--format",
		"json",
		"tag",
		"search",
		"report",
		"--namespace",
		"work",
		"--limit",
		"5",
		"--include-archived",
	]);
	let payloads = server.join().unwrap();
	assert!(
		out.status.success(),
		"{}",
		String::from_utf8_lossy(&out.stderr)
	);
	let value: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
	assert_eq!(value["query"], "report");
	assert_eq!(value["total_found"], 0);
	assert_eq!(value["filters"]["namespace"], "work");
	assert_eq!(value["filters"]["include_archived"], true);
	assert_eq!(value["filters"]["limit"], 5);
	assert!(value["tags"].is_array());
	assert_eq!(payloads[1]["query"], "report");
	assert_eq!(payloads[1]["namespace"], "work");
}

#[test]
fn json_error_preserves_daemon_context_on_stderr() {
	use std::io::{Read, Write};
	use std::net::TcpListener;
	use std::time::{Duration, Instant};

	let (instance, listener) = (0..1000)
		.find_map(|n| {
			let instance = format!("cli_json_err_{n}");
			let port = 6970 + instance.bytes().map(u16::from).sum::<u16>() % 1000;
			TcpListener::bind(("127.0.0.1", port))
				.ok()
				.map(|listener| (instance, listener))
		})
		.expect("no available test instance port");
	listener.set_nonblocking(true).unwrap();
	let server = std::thread::spawn(move || {
		let deadline = Instant::now() + Duration::from_secs(15);
		let responses = [
			wing_core::infra::daemon::types::DaemonResponse::JsonOk(serde_json::json!([
				{"id": "550e8400-e29b-41d4-a716-446655440000", "name": "fixture", "path": "fixture", "stats": null}
			])),
			wing_core::infra::daemon::types::DaemonResponse::Error(
				wing_core::infra::daemon::types::DaemonError::ValidationError(
					"duplicate tag: report".into(),
				),
			),
		];
		for response in responses {
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
		"tag",
		"create",
		"report",
	]);
	server.join().unwrap();
	assert_eq!(out.status.code(), Some(1));
	let stderr = String::from_utf8_lossy(&out.stderr);
	assert!(stderr.contains("duplicate tag: report"), "{stderr}");
	// stdout should not contain a fake JSON success payload
	let stdout = String::from_utf8_lossy(&out.stdout);
	assert!(
		!stdout.contains("\"canonical_name\""),
		"unexpected stdout: {stdout}"
	);
}

#[test]
fn stop_reset_honors_wing_cli_yes_without_prompting() {
	let dir = tempfile::tempdir().unwrap();
	std::fs::write(dir.path().join("wingdrive.json"), b"data").unwrap();
	std::fs::create_dir(dir.path().join("libraries")).unwrap();
	std::fs::write(dir.path().join("personal.txt"), b"keep").unwrap();
	let out = Command::new(env!("CARGO_BIN_EXE_wing"))
		.args([
			"--data-dir",
			dir.path().to_str().unwrap(),
			"--instance",
			"cov09-confirm-fixture",
			"stop",
			"--reset",
		])
		.env("WING_CLI_YES", "1")
		.output()
		.unwrap();
	assert!(
		out.status.success(),
		"status={:?}\nstderr={}",
		out.status.code(),
		String::from_utf8_lossy(&out.stderr)
	);
	assert!(!dir.path().join("wingdrive.json").exists());
	assert!(!dir.path().join("libraries").exists());
	assert_eq!(
		std::fs::read(dir.path().join("personal.txt")).unwrap(),
		b"keep"
	);
	let stderr = String::from_utf8_lossy(&out.stderr);
	assert!(!stderr.contains("Aborted by user"), "{stderr}");
}

#[test]
fn adapters_list_renders_states_and_permissions() {
	let card = |id: &str,
	            installed: Option<&str>,
	            update: bool,
	            revoked: Option<&str>,
	            supported: bool,
	            runtime: bool| {
		serde_json::json!({
			"id":id,"name":id,"summary":"Fixture summary","description":"Fixture","translations":{},"version":"2.0",
			"author":"Test","license":"MIT","homepage":null,"categories":[],"trust":"community",
			"permissions":{"read":["/fixture"],"network":["example.invalid"],"secrets":["token"],"tools":["python"]},
			"platforms":[],"supported_here":supported,"runtime_available":runtime,"installed_version":installed,
			"update_available":update,"update_needs_consent":false,"revoked_reason":revoked,"icon_svg":null
		})
	};
	let (instance, dir, server) = json_rpc_fixture(
		&["query:libraries.list", "query:adapters.store.list"],
		vec![
			serde_json::json!([]),
			serde_json::json!({"sequence":42,"isolation":"none","generated_at":null,"updates":2,"adapters":[
				card("available",None,false,None,true,true), card("updated",Some("1.0"),true,None,false,true),
				card("revoked",Some("1.0"),true,Some("unsafe"),true,false), card("current",Some("2.0"),false,None,true,true)
			]}),
		],
	);
	let out = run(&[
		"--data-dir",
		dir.path().to_str().unwrap(),
		"--instance",
		&instance,
		"adapters",
		"list",
	]);
	server.join().unwrap();
	assert!(
		out.status.success(),
		"{}",
		String::from_utf8_lossy(&out.stderr)
	);
	let text = String::from_utf8(out.stdout).unwrap();
	for expected in [
		"available 2.0",
		"installed 1.0, update to 2.0",
		"installed 1.0, REVOKED: unsafe",
		"installed 2.0",
		"reads: /fixture; network: example.invalid; secrets: token; runs: python",
		"does not run on this system",
		"needs Python 3",
		"Index 42",
	] {
		assert!(text.contains(expected), "missing {expected:?} in {text}");
	}
}
