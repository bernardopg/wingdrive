use super::*;
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use std::{
	io::{Read, Write},
	path::PathBuf,
};
use wing_core::domain::volume::{Volume, VolumeFingerprint, VolumeType};

const CHILD: &str = "tests::rpc::cloud_setup::scripted_cloud_setup_child";
const CLOUD_FINGERPRINT: &str = "0123456789abcdef";
const LOCAL_FINGERPRINT: &str = "fedcba9876543210";

#[tokio::test]
async fn scripted_cloud_setup_child() {
	let Some(addr) = std::env::var_os("WING_TEST_CLOUD_RPC_ADDR") else {
		return;
	};
	let dir = PathBuf::from(std::env::var_os("WING_TEST_CLOUD_DATA_DIR").unwrap());
	let cli = Cli::try_parse_from(["wing", "cloud"]).unwrap();
	run_client_command(
		cli.command,
		OutputFormat::Human,
		dir,
		None,
		addr.into_string().unwrap(),
	)
	.await
	.unwrap();
}

async fn run_cloud(input: &[(&str, &str)], steps: Vec<Step>) -> (bool, String, Vec<Value>) {
	let root = tempfile::tempdir().unwrap();
	let (addr, server) = fixture(steps).await;
	let mut command = CommandBuilder::new(std::env::current_exe().unwrap());
	command.args(["--exact", CHILD, "--nocapture"]);
	command.env("WING_TEST_CLOUD_RPC_ADDR", addr);
	command.env("WING_TEST_CLOUD_DATA_DIR", root.path());
	command.env_remove("WING_CLI_YES");

	let pty = native_pty_system().openpty(PtySize::default()).unwrap();
	let mut child = pty.slave.spawn_command(command).unwrap();
	drop(pty.slave);
	let mut reader = pty.master.try_clone_reader().unwrap();
	let mut writer = pty.master.take_writer().unwrap();
	let (send, mut receive) = tokio::sync::mpsc::unbounded_channel();
	let read_task = tokio::task::spawn_blocking(move || {
		let mut buffer = [0u8; 4096];
		while let Ok(length @ 1..) = reader.read(&mut buffer) {
			if send.send(buffer[..length].to_vec()).is_err() {
				break;
			}
		}
	});

	let mut output = Vec::new();
	let result = tokio::time::timeout(Duration::from_secs(12), async {
		let mut cursor = 0;
		for (prompt, response) in input {
			loop {
				if let Some(position) = output[cursor..]
					.windows(prompt.len())
					.position(|window| window == prompt.as_bytes())
				{
					cursor += position + prompt.len();
					break;
				}
				let Some(bytes) = receive.recv().await else {
					return Err(anyhow::anyhow!("child exited before prompt {prompt:?}"));
				};
				output.extend(bytes);
			}
			writer.write_all(response.as_bytes())?;
			writer.flush()?;
		}
		while let Some(bytes) = receive.recv().await {
			output.extend(bytes);
		}
		Ok::<_, anyhow::Error>(child.wait()?.success())
	})
	.await;
	if !matches!(result, Ok(Ok(_))) {
		let _ = child.kill();
		let _ = child.wait();
	}
	drop(writer);
	drop(pty.master);
	read_task.await.unwrap();
	let output = String::from_utf8_lossy(&output).into_owned();
	let succeeded = result
		.expect("cloud setup timed out")
		.unwrap_or_else(|error| panic!("{error}: {output}"));
	let requests = tokio::time::timeout(Duration::from_secs(8), server)
		.await
		.expect("cloud RPC fixture timed out")
		.expect("cloud RPC fixture failed");
	(succeeded, output, requests)
}

const MENU: &str = "What would you like to do?";
const EXIT: &str = "\x1b[B\x1b[B\x1b[B\r";
const REMOVE: &str = "\x1b[B\x1b[B\r";
const REMOVE_INPUT: &[(&str, &str)] = &[
	(MENU, REMOVE),
	("Select volume to remove", "\r"),
	("Are you sure?", "y\n"),
	(MENU, EXIT),
];

fn tracked(kind: VolumeType, name: &str, fingerprint: &str) -> Volume {
	let mut volume = Volume::new(
		library(),
		VolumeFingerprint::from_hex(fingerprint),
		name.into(),
		PathBuf::from("fixture"),
	);
	volume.track(Some(library()));
	volume.volume_type = kind;
	volume
}

fn mixed_volumes() -> Step {
	success(
		"query:volumes.list",
		Some(library()),
		json!({"volumes":[
			tracked(VolumeType::Primary, "Fixture local disk", LOCAL_FINGERPRINT),
			tracked(VolumeType::Cloud, "Fixture cloud volume", CLOUD_FINGERPRINT)
		]}),
	)
}

#[tokio::test]
async fn cloud_setup_list_does_not_show_local_volumes() {
	let (succeeded, output, requests) = run_cloud(
		&[(MENU, "\x1b[B\r"), ("Cloud Volumes:", ""), (MENU, EXIT)],
		vec![listing(), mixed_volumes()],
	)
	.await;
	assert!(succeeded, "{output}");
	assert!(output.contains("Fixture cloud volume"));
	assert!(!output.contains("Fixture local disk"));
	assert_eq!(requests[1], json!({"filter":"TrackedOnly"}));
}

#[tokio::test]
async fn cloud_setup_removes_the_selected_cloud_volume_and_reads_the_response() {
	let (succeeded, output, requests) = run_cloud(
		REMOVE_INPUT,
		vec![
			listing(),
			mixed_volumes(),
			success(
				"action:volumes.remove_cloud.input",
				Some(library()),
				json!({"fingerprint": CLOUD_FINGERPRINT}),
			),
		],
	)
	.await;
	assert_eq!(requests[2], json!({"fingerprint": CLOUD_FINGERPRINT}));
	assert!(succeeded, "{output}");
	assert!(output.contains("Removed cloud volume 'Fixture cloud volume'"));
	assert!(!output.contains("Fixture local disk"));
}

#[tokio::test]
async fn cloud_setup_removal_accepts_the_daemons_structured_response() {
	let volume = tracked(VolumeType::Cloud, "Fixture cloud volume", CLOUD_FINGERPRINT);
	let (succeeded, output, requests) = run_cloud(
		REMOVE_INPUT,
		vec![
			listing(),
			success(
				"query:volumes.list",
				Some(library()),
				json!({"volumes":[volume]}),
			),
			success(
				"action:volumes.remove_cloud.input",
				Some(library()),
				json!({"fingerprint": CLOUD_FINGERPRINT}),
			),
		],
	)
	.await;
	assert_eq!(requests[2], json!({"fingerprint": CLOUD_FINGERPRINT}));
	assert!(succeeded, "{output}");
}

#[tokio::test]
async fn cloud_setup_returns_to_menu_after_adding_storage() {
	let credentials = tempfile::NamedTempFile::new().unwrap();
	let document = r#"{"client_email":"fixture@example.test"}"#;
	std::fs::write(credentials.path(), document).unwrap();
	let path = format!("{}\r", credentials.path().display());
	let (succeeded, output, requests) = run_cloud(
		&[
			(MENU, "\r"),
			(
				"Select cloud storage category",
				"\x1b[B\x1b[B\x1b[B\x1b[B\x1b[B\r",
			),
			("Volume name", "Fixture bucket\r"),
			("Bucket name", "bucket\r"),
			("Root path", "backup\r"),
			("Custom endpoint", "\r"),
			("Path to service account JSON file", &path),
			("Add this cloud volume?", "y\n"),
			(MENU, EXIT),
		],
		vec![
			listing(),
			success(
				"action:volumes.add_cloud.input",
				Some(library()),
				json!({"fingerprint":CLOUD_FINGERPRINT,"volume_name":"Fixture bucket","service":"gcs"}),
			),
		],
	)
	.await;
	assert!(succeeded, "{output}");
	assert!(output.contains("Successfully added cloud volume 'Fixture bucket'"));
	assert!(output.contains("Goodbye!"));
	assert_eq!(requests[1]["display_name"], "Fixture bucket");
	assert_eq!(requests[1]["config"]["bucket"], "bucket");
	assert_eq!(requests[1]["config"]["root"], "backup");
	assert_eq!(requests[1]["config"]["credential"], document);
}

#[tokio::test]
async fn cloud_setup_preserves_s3_provider_regions_endpoints_and_credentials() {
	for (index, service, custom_region, endpoint_prompt) in [
		(0, "s3", false, None),
		(1, "s3", true, Some("Cloudflare R2 account endpoint")),
		(2, "b2", false, None),
		(3, "wasabi", false, None),
		(4, "spaces", false, None),
		(5, "s3", true, Some("MinIO server endpoint")),
		(6, "cloud", true, Some("Custom endpoint URL")),
	] {
		let choice = "\x1b[B".repeat(index) + "\r";
		let mut input = vec![
			(MENU, "\r"),
			("Select cloud storage category", "\r"),
			("Select S3-compatible storage provider", choice.as_str()),
			("Volume name", "Fixture S3\r"),
			("Bucket name", "fixture-bucket\r"),
			(
				"Region",
				if custom_region {
					"\r"
				} else {
					"fixture-region\r"
				},
			),
		];
		if let Some(prompt) = endpoint_prompt {
			input.push((prompt, "https://example.invalid\r"));
		}
		input.extend([
			("Access Key ID", "fixture-id\r"),
			("Secret Access Key", "fixture-key\r"),
			("Add this cloud volume?", "y\n"),
			(MENU, EXIT),
		]);
		let (succeeded, output, requests) = run_cloud(
			&input,
			vec![
				listing(),
				success(
					"action:volumes.add_cloud.input",
					Some(library()),
					json!({"fingerprint":CLOUD_FINGERPRINT,"volume_name":"Fixture S3","service":service}),
				),
			],
		)
		.await;
		assert!(succeeded, "{output}");
		assert_eq!(requests[1]["service"], service);
		let region = match index {
			1 => "auto",
			5 | 6 => "us-east-1",
			_ => "fixture-region",
		};
		assert_eq!(requests[1]["config"]["region"], region);
		assert_eq!(requests[1]["config"]["bucket"], "fixture-bucket");
		assert_eq!(requests[1]["config"]["access_key_id"], "fixture-id");
		assert_eq!(requests[1]["config"]["secret_access_key"], "fixture-key");
		assert_eq!(
			requests[1]["config"]["endpoint"],
			json!(endpoint_prompt.map(|_| "https://example.invalid"))
		);
	}
}

#[tokio::test]
async fn cloud_setup_preserves_azure_container_and_optional_endpoint() {
	let (succeeded, output, requests) = run_cloud(
		&[
			(MENU, "\r"),
			(
				"Select cloud storage category",
				"\x1b[B\x1b[B\x1b[B\x1b[B\r",
			),
			("Volume name", "Fixture Azure\r"),
			("Container name", "fixture-container\r"),
			("Storage account name", "fixture-account\r"),
			("Custom endpoint", "\r"),
			("Storage account key", "fixture-key\r"),
			("Add this cloud volume?", "y\n"),
			(MENU, EXIT),
		],
		vec![
			listing(),
			success(
				"action:volumes.add_cloud.input",
				Some(library()),
				json!({"fingerprint":CLOUD_FINGERPRINT,"volume_name":"Fixture Azure","service":"azblob"}),
			),
		],
	)
	.await;
	assert!(succeeded, "{output}");
	assert_eq!(requests[1]["service"], "azblob");
	assert_eq!(requests[1]["config"]["container"], "fixture-container");
	assert_eq!(requests[1]["config"]["account_name"], "fixture-account");
	assert_eq!(requests[1]["config"]["account_key"], "fixture-key");
	assert!(requests[1]["config"]["endpoint"].is_null());
}

#[tokio::test]
async fn cloud_setup_preserves_dropbox_optional_folder_and_oauth_fields() {
	let (succeeded, output, requests) = run_cloud(
		&[
			(MENU, "\r"),
			("Select cloud storage category", "\x1b[B\x1b[B\x1b[B\r"),
			("Volume name", "Fixture Dropbox\r"),
			("Root folder path", "fixture-root\r"),
			("App Key (Client ID)", "fixture-id\r"),
			("App Secret (Client Secret)", "fixture-secret\r"),
			("Refresh Token", "fixture-refresh\r"),
			("Add this cloud volume?", "y\n"),
			(MENU, EXIT),
		],
		vec![
			listing(),
			success(
				"action:volumes.add_cloud.input",
				Some(library()),
				json!({"fingerprint":CLOUD_FINGERPRINT,"volume_name":"Fixture Dropbox","service":"dropbox"}),
			),
		],
	)
	.await;
	assert!(succeeded, "{output}");
	assert_eq!(requests[1]["service"], "dropbox");
	assert_eq!(requests[1]["config"]["root"], "fixture-root");
	assert_eq!(requests[1]["config"]["client_id"], "fixture-id");
	assert_eq!(requests[1]["config"]["client_secret"], "fixture-secret");
	assert_eq!(requests[1]["config"]["refresh_token"], "fixture-refresh");
}
