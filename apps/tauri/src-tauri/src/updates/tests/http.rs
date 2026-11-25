use super::super::*;
use tauri::test::{mock_builder, mock_context, noop_assets, MockRuntime};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

fn app_for(endpoint: String) -> tauri::App<MockRuntime> {
	let mut context = mock_context(noop_assets());
	context.config_mut().plugins.0.insert(
		"updater".into(),
		serde_json::json!({
			"endpoints": [endpoint],
			"dangerousInsecureTransportProtocol": true,
			"pubkey": "fixture"
		}),
	);
	mock_builder()
		.plugin(
			tauri_plugin_updater::Builder::new()
				.target("fixture")
				.build(),
		)
		.build(context)
		.unwrap()
}

async fn endpoint(status: &str, body: &str) -> (String, tokio::task::JoinHandle<()>) {
	endpoint_at("/latest.json", status, body).await
}

async fn endpoint_at(
	path: &str,
	status: &str,
	body: &str,
) -> (String, tokio::task::JoinHandle<()>) {
	let listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
	let url = format!("http://{}{path}", listener.local_addr().unwrap());
	let request_line = format!("GET {path} HTTP/1.1\r\n");
	let response = format!(
		"HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
		body.len()
	);
	let serve = tokio::spawn(async move {
		let (mut stream, _) = listener.accept().await.unwrap();
		let mut request = Vec::new();
		loop {
			let mut chunk = [0; 1024];
			let read = stream.read(&mut chunk).await.unwrap();
			assert!(read > 0, "client closed before sending headers");
			request.extend_from_slice(&chunk[..read]);
			if request.windows(4).any(|bytes| bytes == b"\r\n\r\n") {
				break;
			}
		}
		assert!(request.starts_with(request_line.as_bytes()));
		stream.write_all(response.as_bytes()).await.unwrap();
	});
	(url, serve)
}

async fn check(
	status: &str,
	manifest: &str,
) -> Result<Option<tauri_plugin_updater::Update>, String> {
	let (url, mut server) = endpoint(status, manifest).await;
	let app = app_for(url);
	let result = tokio::time::timeout(std::time::Duration::from_secs(5), find_update(app.handle()))
		.await
		.expect("local updater check timed out");
	tokio::time::timeout(std::time::Duration::from_secs(2), &mut server)
		.await
		.expect("updater never contacted the test endpoint")
		.unwrap();
	result
}

#[tokio::test]
async fn updater_checks_a_fixture_manifest_and_reports_the_release() {
	let update = check(
		"200 OK",
		r#"{"version":"99.0.0","notes":"Fixture release","pub_date":"2026-11-19T12:00:00Z","platforms":{"fixture":{"signature":"fixture","url":"https://example.invalid/download"}}}"#,
	)
	.await
	.unwrap()
	.unwrap();
	assert_eq!(update.version, "99.0.0");
	assert_eq!(update.body.as_deref(), Some("Fixture release"));
	assert!(update.date.is_some());
}

#[tokio::test]
async fn updater_preserves_no_release_and_reports_invalid_manifests() {
	assert!(check("204 No Content", "").await.unwrap().is_none());
	let result = check("200 OK", "{invalid").await;
	assert!(result.is_err());
}

#[test]
fn updater_info_reports_current_version_and_channel() {
	let app = mock_builder().build(mock_context(noop_assets())).unwrap();
	let info = updater_info(app.handle().clone());
	assert_eq!(info.current_version, app.package_info().version.to_string());
	assert_eq!(info.channel, channel());
}

#[cfg(target_os = "linux")]
#[tokio::test]
async fn updater_commands_respect_the_appimage_boundary() {
	use std::time::Duration;

	const FIXTURE: &str = "WINGDRIVE_TEST_UPDATER_CHANNEL";
	const NO_UPDATE: &str = "WingDrive is up to date";
	match std::env::var(FIXTURE).as_deref() {
		Ok("package") => {
			// Package-manager copies learn about new releases but never install them.
			let (url, _response) = endpoint(
				"200 OK",
				r#"{"version":"99.0.0","notes":"Fixture release","pub_date":"2026-11-19T12:00:00Z","platforms":{"fixture":{"signature":"fixture","url":"https://example.invalid/download"}}}"#,
			)
			.await;
			let app = app_for(url);
			let info = updater_info(app.handle().clone());
			assert_eq!(info.channel, UpdateChannel::PackageManager);
			assert_eq!(info.current_version, app.package_info().version.to_string());
			let update = check_for_update(app.handle().clone())
				.await
				.unwrap()
				.unwrap();
			assert_eq!(update.version, "99.0.0");
			assert_eq!(
				install_update(app.handle().clone()).await.unwrap_err(),
				"Updates come from your package manager"
			);
			spawn_startup_check(app.handle().clone(), true);
			assert!(STARTUP_CHECK_DONE.load(Ordering::SeqCst));
		}
		Ok("appimage") => {
			let (url, response) = endpoint(
				"200 OK",
				r#"{"version":"99.0.0","notes":"Fixture release","pub_date":"2026-11-19T12:00:00Z","platforms":{"fixture":{"signature":"fixture","url":"https://example.invalid/download"}}}"#,
			)
			.await;
			let app = app_for(url);
			assert_eq!(
				updater_info(app.handle().clone()).channel,
				UpdateChannel::SelfUpdate
			);
			spawn_startup_check(app.handle().clone(), false);
			assert!(!STARTUP_CHECK_DONE.load(Ordering::SeqCst));
			let update = tokio::time::timeout(
				Duration::from_secs(5),
				check_for_update(app.handle().clone()),
			)
			.await
			.unwrap()
			.unwrap()
			.unwrap();
			assert_eq!(update.version, "99.0.0");
			assert_eq!(update.notes.as_deref(), Some("Fixture release"));
			assert!(update.date.is_some());
			response.await.unwrap();

			let (url, response) = endpoint("204 No Content", "").await;
			let app = app_for(url);
			assert_eq!(
				tokio::time::timeout(Duration::from_secs(5), install_update(app.handle().clone()))
					.await
					.unwrap()
					.unwrap_err(),
				NO_UPDATE
			);
			response.await.unwrap();

			let installed = std::path::PathBuf::from(std::env::var_os("APPIMAGE").unwrap());
			let original = std::fs::read(&installed).unwrap();
			let payload = "an unsigned test update";
			let (asset, served_asset) = endpoint_at("/WingDrive.AppImage", "200 OK", payload).await;
			let manifest = serde_json::json!({
				"version": "99.0.0",
				"platforms": { "fixture": { "signature": "invalid", "url": asset } }
			});
			let (url, served_manifest) = endpoint("200 OK", &manifest.to_string()).await;
			let app = app_for(url);
			let progress = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
			{
				use tauri::Listener;
				let progress = progress.clone();
				app.listen(PROGRESS_EVENT, move |event| {
					let event: serde_json::Value = serde_json::from_str(event.payload()).unwrap();
					progress.lock().unwrap().push(event);
				});
			}
			assert!(tokio::time::timeout(
				Duration::from_secs(5),
				install_update(app.handle().clone())
			)
			.await
			.unwrap()
			.is_err());
			tokio::time::timeout(Duration::from_secs(2), served_manifest)
				.await
				.unwrap()
				.unwrap();
			tokio::time::timeout(Duration::from_secs(2), served_asset)
				.await
				.unwrap()
				.unwrap();
			assert_eq!(
				progress.lock().unwrap().last(),
				Some(&serde_json::json!({ "downloaded": payload.len(), "total": payload.len() }))
			);
			assert_eq!(std::fs::read(installed).unwrap(), original);
		}
		_ => {
			let dir = tempfile::tempdir().unwrap();
			let appimage = dir.path().join("WingDrive.AppImage");
			std::fs::write(&appimage, b"fixture").unwrap();
			for channel in ["package", "appimage"] {
				crate::test_support::run_child_test(
					"updates::tests::http::updater_commands_respect_the_appimage_boundary",
					|command| {
						command
							.env(FIXTURE, channel)
							.env("NO_PROXY", "127.0.0.1")
							.env_remove("APPIMAGE");
						if channel == "appimage" {
							command.env("APPIMAGE", &appimage);
						}
					},
				);
			}
		}
	}
}
