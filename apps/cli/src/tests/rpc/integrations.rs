use super::*;
use wing_core::domain::addressing::WingPath;

const ACCOUNT: &str = "fixture-account";
const EMAIL: &str = "fixture@example.test";

fn drive() -> Value {
	json!({"fingerprint":"fixture-fingerprint","identifier":EMAIL,"mounted":true})
}

fn account(provider: &str, mode: &str, mount: Value) -> Value {
	let services = if provider == "google" {
		json!(["drive", "photos"])
	} else {
		json!([])
	};
	json!({
		"id":ACCOUNT, "provider":provider, "email":EMAIL, "mode":mode,
		"connected_at":"2026-11-19T12:00:00Z", "services":services,
		"drive":mount, "drive_picker":false
	})
}

fn request(method: &'static str, response: Value) -> Step {
	success(method, Some(library()), response)
}

#[tokio::test]
async fn lists_google_and_microsoft_accounts_and_handles_empty_library() {
	let mut google = account("google", "app", drive());
	google["drive_access"] = json!("app_files");
	let microsoft = account("microsoft", "custom", drive());
	let google_photos = account("google", "custom", Value::Null);
	for accounts in [json!([]), json!([google, microsoft, google_photos])] {
		let (result, requests) = run(
			&["wing", "integrations", "list"],
			OutputFormat::Human,
			vec![
				listing(),
				request("query:integrations.accounts.list", accounts),
			],
		)
		.await;
		result.unwrap();
		assert_eq!(requests[1], json!({}));
	}
}

#[tokio::test]
async fn mount_and_unmount_use_the_accounts_provider() {
	for (provider, method) in [
		("google", "action:integrations.google.drive.mount.input"),
		("microsoft", "action:integrations.onedrive.mount.input"),
	] {
		for (command, mounted) in [("mount", true), ("unmount", false)] {
			let mount = if mounted { drive() } else { Value::Null };
			let (result, requests) = run(
				&["wing", "integrations", "drive", command, ACCOUNT],
				OutputFormat::Human,
				vec![
					listing(),
					request(
						"query:integrations.accounts.list",
						json!([account(provider, "app", drive())]),
					),
					request(method, json!({"drive": mount})),
				],
			)
			.await;
			result.unwrap();
			assert_eq!(requests[2], json!({"account_id":ACCOUNT,"mounted":mounted}));
		}
	}
}

#[tokio::test]
async fn disconnect_can_keep_the_provider_grant() {
	let (result, requests) = run(
		&[
			"wing",
			"integrations",
			"remove",
			ACCOUNT,
			"--keep-grant",
			"--yes",
		],
		OutputFormat::Human,
		vec![
			listing(),
			request(
				"action:integrations.accounts.remove.input",
				json!({"account_id":ACCOUNT}),
			),
		],
	)
	.await;
	result.unwrap();
	assert_eq!(requests[1], json!({"account_id":ACCOUNT,"revoke":false}));
}

#[tokio::test]
async fn google_sign_in_polls_pending_then_prints_connected_account() {
	let (result, requests) = run(
		&[
			"wing",
			"integrations",
			"connect",
			"google",
			"--drive-only",
			"--own-client",
			"--email",
			EMAIL,
			"--no-browser",
		],
		OutputFormat::Human,
		vec![
			listing(),
			request(
				"action:integrations.google.connect.input",
				json!({"session_id":TAG,"auth_url":"https://example.invalid/auth"}),
			),
			request(
				"query:integrations.google.connect_status",
				json!({"status":"pending"}),
			),
			request(
				"query:integrations.google.connect_status",
				json!({"status":"completed","account":account("google","custom",drive())}),
			),
		],
	)
	.await;
	result.unwrap();
	assert_eq!(
		requests[1],
		json!({"mode":"custom","services":["drive"],"login_hint":EMAIL})
	);
	assert_eq!(requests[2], json!({"session_id":TAG}));
	assert_eq!(requests[2], requests[3]);
}

#[tokio::test]
async fn onedrive_sign_in_returns_cancellation() {
	let (result, requests) = run(
		&[
			"wing",
			"integrations",
			"connect",
			"onedrive",
			"--no-browser",
		],
		OutputFormat::Human,
		vec![
			listing(),
			request(
				"action:integrations.microsoft.connect.input",
				json!({"session_id":TAG,"auth_url":"https://example.invalid/auth"}),
			),
			request(
				"query:integrations.google.connect_status",
				json!({"status":"cancelled"}),
			),
		],
	)
	.await;
	assert_eq!(result.unwrap_err().to_string(), "Sign-in cancelled");
	assert_eq!(requests[1], json!({"mode":"app","login_hint":null}));
	assert_eq!(requests[2], json!({"session_id":TAG}));
}

#[tokio::test]
async fn oauth_clients_show_set_and_clear_keep_provider_contracts() {
	for (command, query, action, set_args, set_payload, clear_payload) in [
		(
			"google-client",
			"query:integrations.google.client",
			"action:integrations.google.set_client.input",
			&[
				"--client-id",
				"fake-client",
				"--client-secret",
				"fake-secret",
			][..],
			json!({"client_id":"fake-client","client_secret":"fake-secret"}),
			json!({"client_id":null,"client_secret":null}),
		),
		(
			"microsoft-client",
			"query:integrations.microsoft.client",
			"action:integrations.microsoft.set_client.input",
			&["--client-id", "fake-client"][..],
			json!({"client_id":"fake-client"}),
			json!({"client_id":null}),
		),
	] {
		for (verb, method, extra, expected) in [
			("show", query, &[][..], json!({})),
			("set", action, set_args, set_payload),
			("clear", action, &[][..], clear_payload),
		] {
			let mut args = vec!["wing", "integrations", command, verb];
			args.extend(extra);
			let response = if verb == "set" {
				json!({"app":{"source":"bundled","client_id":"fixture-app"},"custom_client_id":"fake-client"})
			} else {
				json!({"app":null,"custom_client_id":null})
			};
			let (result, requests) = run(
				&args,
				OutputFormat::Human,
				vec![listing(), request(method, response)],
			)
			.await;
			result.unwrap();
			assert_eq!(requests[1], expected, "{command} {verb}");
		}
	}
}

#[tokio::test]
async fn photos_import_waits_for_picker_then_forwards_local_destination() {
	let temp = tempfile::tempdir().unwrap();
	let destination = temp.path().join("Imported");
	let (result, requests) = run(
		&["wing","integrations","photos","import",ACCOUNT,destination.to_str().unwrap(),"--no-browser"],
		OutputFormat::Human,
		vec![
			listing(),
			request("action:integrations.google.photos.picker.start.input", json!({"session_id":TAG,"picker_uri":"https://example.invalid/picker","poll_interval_seconds":0})),
			request("query:integrations.google.photos.picker.status", json!({"ready":false})),
			request("query:integrations.google.photos.picker.status", json!({"ready":true})),
			request("action:integrations.google.photos.picker.import.input", json!({"id":TAG,"job_name":"import"})),
		],
	)
	.await;
	result.unwrap();
	assert_eq!(requests[1], json!({"account_id":ACCOUNT}));
	assert_eq!(requests[2], json!({"account_id":ACCOUNT,"session_id":TAG}));
	assert_eq!(requests[2], requests[3]);
	assert_eq!(
		requests[4],
		json!({"account_id":ACCOUNT,"session_id":TAG,"destination":WingPath::local(destination)})
	);
}

#[tokio::test]
async fn photos_upload_resolves_relative_path_and_preserves_album_title() {
	let (result, requests) = run(
		&[
			"wing",
			"integrations",
			"photos",
			"upload",
			ACCOUNT,
			"sample.jpg",
			"--new-album",
			"Vacation",
		],
		OutputFormat::Human,
		vec![
			listing(),
			request(
				"action:integrations.google.photos.upload.input",
				json!({"id":TAG,"job_name":"upload"}),
			),
		],
	)
	.await;
	result.unwrap();
	let path = std::env::current_dir().unwrap().join("sample.jpg");
	assert_eq!(
		requests[1],
		json!({
			"account_id":ACCOUNT,"paths":[WingPath::local(path)],"album_id":null,"new_album_title":"Vacation"
		})
	);
}

#[tokio::test]
async fn photos_albums_accept_empty_and_populated_responses() {
	for albums in [json!([]), json!([{"id":"album-1","title":"Vacation"}])] {
		let (result, requests) = run(
			&["wing", "integrations", "photos", "albums", ACCOUNT],
			OutputFormat::Human,
			vec![
				listing(),
				request("query:integrations.google.photos.albums", albums),
			],
		)
		.await;
		result.unwrap();
		assert_eq!(requests[1], json!({"account_id": ACCOUNT}));
	}
}

const REMOTE: &str = "gdrive://fixture@example.test/Backup";

fn sync_pair(local: &str) -> Value {
	json!({
		"id":TAG, "name":"Backup", "local_path":local,
		"remote":WingPath::from_uri(REMOTE).unwrap(), "mode":"upload",
		"propagate_deletes":true, "enabled":true, "interval_minutes":30, "running":false
	})
}

#[tokio::test]
async fn cloud_sync_add_forwards_both_paths_and_schedule() {
	let temp = tempfile::tempdir().unwrap();
	let local = temp.path().to_str().unwrap();
	let (result, requests) = run(
		&[
			"wing",
			"integrations",
			"cloud-sync",
			"add",
			"Backup",
			local,
			REMOTE,
			"--mode",
			"upload",
			"--propagate-deletes",
			"--every",
			"30",
			"--no-run",
		],
		OutputFormat::Human,
		vec![
			listing(),
			request("action:cloud_sync.create.input", sync_pair(local)),
		],
	)
	.await;
	result.unwrap();
	assert_eq!(
		requests[1],
		json!({
			"name":"Backup", "local_path":local, "remote":WingPath::from_uri(REMOTE).unwrap(),
			"mode":"upload", "propagate_deletes":true, "interval_minutes":30, "run_now":false
		})
	);
}

#[tokio::test]
async fn cloud_sync_list_renders_empty_and_distinct_pair_states() {
	let mut running = sync_pair("/fixture/active");
	running["running"] = json!(true);
	let mut completed = sync_pair("/fixture/finished");
	completed["last_status"] = json!("success");
	let mut manual = sync_pair("/fixture/manual");
	manual["interval_minutes"] = Value::Null;
	for pairs in [json!([]), json!([running, completed, manual])] {
		let (result, requests) = run(
			&["wing", "integrations", "cloud-sync", "list"],
			OutputFormat::Human,
			vec![listing(), request("query:cloud_sync.list", pairs)],
		)
		.await;
		result.unwrap();
		assert_eq!(requests[1], json!({}));
	}
}

#[tokio::test]
async fn cloud_sync_run_and_remove_forward_pair_id() {
	for (command, method, response) in [
		(
			vec!["run", TAG],
			"action:cloud_sync.run.input",
			json!({"id":TAG,"job_name":"cloud_sync"}),
		),
		(
			vec!["remove", TAG, "--yes"],
			"action:cloud_sync.delete.input",
			json!({"id":TAG}),
		),
	] {
		let mut args = vec!["wing", "integrations", "cloud-sync"];
		args.extend(command);
		let (result, requests) = run(
			&args,
			OutputFormat::Human,
			vec![listing(), request(method, response)],
		)
		.await;
		result.unwrap();
		assert_eq!(requests[1], json!({"id":TAG}));
	}
}

#[tokio::test]
async fn cloud_sync_rejects_malformed_uri_before_sending_action() {
	let (result, requests) = run(
		&[
			"wing",
			"integrations",
			"cloud-sync",
			"add",
			"Backup",
			"local",
			"invalid://fixture/path",
		],
		OutputFormat::Human,
		vec![listing()],
	)
	.await;
	assert!(result
		.unwrap_err()
		.to_string()
		.starts_with("Invalid cloud folder URI:"));
	assert_eq!(requests.len(), 1);
}
