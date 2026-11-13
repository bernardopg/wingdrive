//! Commands that proxy the daemon or read shared state, driven through a mock
//! Tauri app and a one-shot TCP listener standing in for the daemon.

use std::path::PathBuf;
use std::sync::Arc;

use serde_json::json;
use tauri::Manager;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;
use tokio::sync::{oneshot, RwLock};

use super::{
	daemon_request, get_active_subscriptions, get_current_library_id, get_daemon_socket,
	get_daemon_status, get_file_stream_url, get_selected_file_ids, get_server_url,
	is_daemon_running, percent_encode_query, unsubscribe_from_events, AppState,
	DaemonConnectionPool, DaemonState, SubscriptionManager,
};

fn daemon_state(
	socket_addr: &str,
	server_url: Option<&str>,
	server_token: Option<&str>,
) -> Arc<RwLock<DaemonState>> {
	Arc::new(RwLock::new(DaemonState {
		started_by_us: false,
		socket_addr: socket_addr.to_string(),
		data_dir: PathBuf::from("/nonexistent/wingdrive-test"),
		base_data_dir: PathBuf::from("/nonexistent/wingdrive-test"),
		instance: None,
		server_url: server_url.map(str::to_string),
		server_token: server_token.map(str::to_string),
		server_shutdown: None,
		daemon_process: None,
	}))
}

fn app_with_daemon(state: Arc<RwLock<DaemonState>>) -> tauri::App<tauri::test::MockRuntime> {
	let app = tauri::test::mock_app();
	app.manage(state);
	app
}

fn app_state() -> AppState {
	AppState {
		current_library_id: Arc::new(RwLock::new(None)),
		selected_file_ids: Arc::new(RwLock::new(Vec::new())),
		connection_pool: Arc::new(DaemonConnectionPool::new("127.0.0.1:9".into())),
		subscription_manager: SubscriptionManager::new(),
	}
}

#[tokio::test]
async fn resetting_the_pool_discards_connection_state_and_subscriptions() {
	let pool = DaemonConnectionPool::new("invalid-address".into());
	*pool.initialized.lock().await = true;
	pool.subscriptions.write().await.extend([(2, ()), (3, ())]);
	pool.reset().await;
	assert!(!*pool.initialized.lock().await);
	assert!(pool.writer.lock().await.is_none());
	assert!(pool.subscriptions.read().await.is_empty());
	assert_eq!(
		pool.subscribe(5, vec![], None).await.unwrap_err(),
		"Connection not initialized"
	);
	pool.reset().await;
	assert!(pool.subscriptions.read().await.is_empty());
}

/// Accepts one connection, returns the line it received and writes `reply`.
async fn one_shot_daemon(reply: &'static str) -> (String, tokio::task::JoinHandle<String>) {
	let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
	let address = listener.local_addr().unwrap().to_string();
	let task = tokio::spawn(async move {
		let (stream, _) = listener.accept().await.unwrap();
		let (reader, mut writer) = stream.into_split();
		let mut line = String::new();
		BufReader::new(reader).read_line(&mut line).await.unwrap();
		writer.write_all(reply.as_bytes()).await.unwrap();
		line
	});
	(address, task)
}

/// Answers every line of every connection with `reply`, as a daemon would
/// answer a ping and a libraries query alike.
async fn answering_daemon(reply: serde_json::Value) -> String {
	let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
	let address = listener.local_addr().unwrap().to_string();
	tokio::spawn(async move {
		while let Ok((stream, _)) = listener.accept().await {
			let reply = format!("{reply}\n");
			tokio::spawn(async move {
				let (reader, mut writer) = stream.into_split();
				let mut reader = BufReader::new(reader);
				let mut line = String::new();
				while reader.read_line(&mut line).await.unwrap_or(0) > 0 {
					let _ = writer.write_all(reply.as_bytes()).await;
					line.clear();
				}
			});
		}
	});
	address
}

/// An address on which nothing listens.
///
/// A freed ephemeral port can be taken again by a test running in parallel;
/// port 1 needs privileges to bind, so no test ever listens there.
async fn closed_address() -> String {
	"127.0.0.1:1".to_string()
}

#[tokio::test]
async fn daemon_request_sends_one_json_line_and_returns_the_reply() {
	let (address, daemon) = one_shot_daemon("{\"JsonOk\":{\"libraries\":[]}}\n").await;
	let app = app_with_daemon(daemon_state(&address, None, None));
	let request = json!({"Query": {"method": "query:libraries.list", "payload": {}}});

	let reply = daemon_request(request.clone(), app.state()).await.unwrap();

	assert_eq!(reply, json!({"JsonOk": {"libraries": []}}));
	let sent = daemon.await.unwrap();
	assert!(sent.ends_with('\n'));
	assert_eq!(
		serde_json::from_str::<serde_json::Value>(&sent).unwrap(),
		request
	);
}

#[tokio::test]
async fn daemon_request_reports_closed_connections_and_invalid_replies() {
	let (address, _daemon) = one_shot_daemon("").await;
	let app = app_with_daemon(daemon_state(&address, None, None));
	let error = daemon_request(json!("Ping"), app.state())
		.await
		.unwrap_err();
	assert!(error.contains("closed the connection"), "{error}");

	let (address, _daemon) = one_shot_daemon("not json\n").await;
	let app = app_with_daemon(daemon_state(&address, None, None));
	let error = daemon_request(json!("Ping"), app.state())
		.await
		.unwrap_err();
	assert!(error.contains("Failed to parse response"), "{error}");
	assert!(error.contains("not json"), "{error}");

	let app = app_with_daemon(daemon_state(&closed_address().await, None, None));
	let error = daemon_request(json!("Ping"), app.state())
		.await
		.unwrap_err();
	assert!(error.starts_with("Failed to connect to daemon"), "{error}");
}

#[tokio::test]
async fn daemon_ping_requires_a_reply_line() {
	assert!(!is_daemon_running(&closed_address().await).await);

	let (address, daemon) = one_shot_daemon("{\"Pong\":null}\n").await;
	assert!(is_daemon_running(&address).await);
	let sent: serde_json::Value = serde_json::from_str(&daemon.await.unwrap()).unwrap();
	assert_eq!(sent, json!({"Ping": null}));

	// A listener that accepts but never answers is not a running daemon.
	let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
	let address = listener.local_addr().unwrap().to_string();
	let (release, hold) = oneshot::channel::<()>();
	let silent = tokio::spawn(async move {
		let (_stream, _) = listener.accept().await.unwrap();
		let _ = hold.await;
	});
	assert!(!is_daemon_running(&address).await);
	let _ = release.send(());
	silent.await.unwrap();

	// A connection closed without a reply is not a running daemon either.
	let (address, _daemon) = one_shot_daemon("").await;
	assert!(!is_daemon_running(&address).await);
}

#[tokio::test]
async fn daemon_status_reflects_the_ping_and_state() {
	let (address, _daemon) = one_shot_daemon("{\"Pong\":null}\n").await;
	let app = app_with_daemon(daemon_state(
		&address,
		Some("http://127.0.0.1:1"),
		Some("t"),
	));
	let status = get_daemon_status(app.state()).await.unwrap();
	assert!(status.is_running);
	assert_eq!(status.socket_addr, address);
	assert_eq!(status.server_url.as_deref(), Some("http://127.0.0.1:1"));
	assert_eq!(status.server_token.as_deref(), Some("t"));
	assert!(!status.started_by_us);

	let app = app_with_daemon(daemon_state(&closed_address().await, None, None));
	assert!(!get_daemon_status(app.state()).await.unwrap().is_running);
}

#[tokio::test]
async fn server_url_commands_require_a_started_server_and_token() {
	let app = app_with_daemon(daemon_state("127.0.0.1:6969", None, None));
	assert_eq!(
		get_daemon_socket(app.state()).await.unwrap(),
		"127.0.0.1:6969"
	);
	assert_eq!(
		get_server_url(app.state()).await.unwrap_err(),
		"Server not started"
	);
	assert_eq!(
		get_file_stream_url("/tmp/a".into(), app.state())
			.await
			.unwrap_err(),
		"Server not started"
	);

	let app = app_with_daemon(daemon_state(
		"127.0.0.1:6969",
		Some("http://127.0.0.1:4"),
		None,
	));
	assert_eq!(
		get_server_url(app.state()).await.unwrap(),
		"http://127.0.0.1:4"
	);
	assert_eq!(
		get_file_stream_url("/tmp/a".into(), app.state())
			.await
			.unwrap_err(),
		"Server token unavailable"
	);

	let app = app_with_daemon(daemon_state(
		"127.0.0.1:6969",
		Some("http://127.0.0.1:4"),
		Some("secret"),
	));
	assert_eq!(
		get_file_stream_url("/home/me/a b&c=d#é?.mp4".into(), app.state())
			.await
			.unwrap(),
		"http://127.0.0.1:4/file?path=/home/me/a%20b%26c%3Dd%23%C3%A9%3F.mp4&token=secret"
	);
}

#[test]
fn query_encoding_keeps_only_unreserved_bytes_and_slashes() {
	assert_eq!(percent_encode_query("AZaz09-_.~/"), "AZaz09-_.~/");
	assert_eq!(percent_encode_query("%+ \n"), "%25%2B%20%0A");
	assert_eq!(percent_encode_query("ñ"), "%C3%B1");
	assert_eq!(percent_encode_query(""), "");
}

#[tokio::test]
async fn app_state_commands_report_selection_library_and_subscriptions() {
	let app = tauri::test::mock_app();
	app.manage(app_state());

	assert!(get_selected_file_ids(app.state()).await.unwrap().is_empty());
	assert_eq!(
		get_current_library_id(app.state()).await.unwrap_err(),
		"No library selected"
	);
	*app.state::<AppState>().current_library_id.write().await = Some("lib".into());
	assert_eq!(get_current_library_id(app.state()).await.unwrap(), "lib");

	assert_eq!(get_active_subscriptions(app.state()).await.unwrap(), 0);
	let manager = &app.state::<AppState>().subscription_manager;
	let id = manager.next_id();
	let (cancel, cancelled) = oneshot::channel();
	manager.register(id, "main".into(), cancel).await;
	assert_eq!(get_active_subscriptions(app.state()).await.unwrap(), 1);

	unsubscribe_from_events(app.state(), id).await.unwrap();
	assert_eq!(cancelled.await, Ok(()));
	assert_eq!(get_active_subscriptions(app.state()).await.unwrap(), 0);
	assert_eq!(
		unsubscribe_from_events(app.state(), id).await.unwrap_err(),
		format!("Subscription {id} not found")
	);
}

#[cfg(unix)]
mod daemon;
mod events;
mod service;
