//! The session bus side of notifications, against a peer-to-peer connection
//! whose other end serves a fake `org.freedesktop.Notifications`.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};
use zbus::object_server::SignalEmitter;
use zbus::OwnedVariant;

use super::*;

type Call = (String, String, String, String, Vec<String>, Vec<String>);

/// Records `Notify` calls the way a notification server receives them.
#[derive(Clone, Default)]
struct FakeServer {
	calls: Arc<Mutex<Vec<Call>>>,
}

#[zbus::interface(name = "org.freedesktop.Notifications")]
impl FakeServer {
	#[allow(clippy::too_many_arguments)]
	fn notify(
		&self,
		app_name: String,
		_replaces_id: u32,
		app_icon: String,
		summary: String,
		body: String,
		actions: Vec<String>,
		hints: HashMap<String, OwnedVariant>,
		_expire_timeout: i32,
	) -> u32 {
		let mut hints: Vec<String> = hints.into_keys().collect();
		hints.sort();
		let mut calls = self.calls.lock().unwrap();
		calls.push((app_name, app_icon, summary, body, actions, hints));
		calls.len() as u32 + 41
	}

	#[zbus(signal)]
	async fn action_invoked(emitter: &SignalEmitter<'_>, id: u32, action: &str)
		-> zbus::Result<()>;
}

/// A connected client and a server serving `FakeServer`.
async fn peers(server: FakeServer) -> (zbus::Connection, zbus::Connection) {
	let (a, b) = std::os::unix::net::UnixStream::pair().unwrap();
	let server = zbus::connection::Builder::unix_stream(a)
		.p2p()
		.server(zbus::Guid::generate())
		.serve_at(NOTIFICATIONS_PATH, server)
		.build();
	let client = zbus::connection::Builder::unix_stream(b).p2p().build();
	let (server, client) = tokio::join!(server, client);
	(server.unwrap(), client.unwrap())
}

fn request() -> NotificationRequest {
	NotificationRequest {
		title: "Copy finished".into(),
		body: "Copied a.txt".into(),
		kind: NotificationKind::TransferComplete,
		open_label: Some("Show WingDrive".into()),
	}
}

#[tokio::test]
async fn notifications_reach_the_server_and_their_ids_are_remembered() {
	let fake = FakeServer::default();
	let (_server, client) = peers(fake.clone()).await;
	let caps = Capabilities::parse(["body", "actions"]);
	let notifier = Notifier::default();
	assert!(notifier.session.set(Some((client, caps))).is_ok());
	let app = tauri::test::mock_app();

	notifier.send(app.handle(), &request()).await.unwrap();
	let calls = fake.calls.lock().unwrap().clone();
	assert_eq!(calls.len(), 1);
	let (app_name, _, summary, body, actions, hints) = &calls[0];
	assert_eq!(app_name, APP_NAME);
	assert_eq!(summary, "Copy finished");
	assert_eq!(body, "Copied a.txt");
	assert_eq!(actions, &["default", "Show WingDrive"]);
	assert!(hints.contains(&"desktop-entry".to_string()), "{hints:?}");
	assert!(notifier.owns(42));
	assert!(!notifier.owns(7));

	// Only the most recent ids are kept.
	for _ in 0..70 {
		notifier.send(app.handle(), &request()).await.unwrap();
	}
	assert!(!notifier.owns(42));
	assert!(notifier.owns(42 + 70));
}

#[test]
fn only_a_body_click_on_our_notification_opens_the_window() {
	let app = tauri::test::mock_app();
	assert!(!click_opens_window(app.handle(), 7, DEFAULT_ACTION));
	let notifier = Notifier::default();
	notifier.sent.lock().unwrap().push(7);
	app.manage(notifier);
	assert!(click_opens_window(app.handle(), 7, DEFAULT_ACTION));
	assert!(!click_opens_window(app.handle(), 8, DEFAULT_ACTION));
	assert!(!click_opens_window(app.handle(), 7, "other"));
}

#[tokio::test]
async fn click_signals_are_received_over_the_bus() {
	let (server, client) = peers(FakeServer::default()).await;
	let app = tauri::test::mock_app();
	let notifier = Notifier::default();
	notifier.sent.lock().unwrap().push(7);
	app.manage(notifier);
	WebviewWindowBuilder::new(&app, "main", WebviewUrl::default())
		.build()
		.unwrap();
	listen_for_clicks(app.handle().clone(), client.clone());
	tokio::time::sleep(Duration::from_millis(200)).await;
	let emitter = SignalEmitter::new(&server, NOTIFICATIONS_PATH).unwrap();
	for (id, action) in [(8, DEFAULT_ACTION), (7, "other"), (7, DEFAULT_ACTION)] {
		FakeServer::action_invoked(&emitter, id, action)
			.await
			.unwrap();
	}
	// The listener keeps the connection; it ends when the peer goes away.
	tokio::time::sleep(Duration::from_millis(200)).await;
	drop(server);
}

#[test]
fn hint_values_carry_the_spec_types() {
	assert_eq!(hint_value(&Hint::Byte(2)), Value::U8(2));
	assert_eq!(
		hint_value(&Hint::Str("x".into())),
		Value::from("x".to_string())
	);
	let image = RgbaImage {
		width: 1,
		height: 1,
		rgba: vec![1, 2, 3, 4],
	};
	let Value::Structure(structure) = hint_value(&Hint::Image(image)) else {
		panic!("image-data is a structure");
	};
	assert_eq!(structure.fields().len(), 7);
}

#[test]
fn identity_is_detected_once() {
	let app = tauri::test::mock_app();
	let notifier = Notifier::default();
	let first = notifier.identity(app.handle()).clone();
	assert_eq!(notifier.identity(app.handle()), &first);
	assert!(DESKTOP_ENTRY_CANDIDATES.contains(&first.desktop_entry.as_str()));
}
