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
	let listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
	let url = format!("http://{}/latest.json", listener.local_addr().unwrap());
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
		assert!(request.starts_with(b"GET /latest.json HTTP/1.1\r\n"));
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
