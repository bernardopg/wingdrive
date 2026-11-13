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
	/// What `GetCapabilities` returns; `None` makes the call fail.
	capabilities: Option<Vec<String>>,
}

#[zbus::interface(name = "org.freedesktop.Notifications")]
impl FakeServer {
	fn get_capabilities(&self) -> zbus::fdo::Result<Vec<String>> {
		self.capabilities
			.clone()
			.ok_or_else(|| zbus::fdo::Error::NotSupported("no capabilities".into()))
	}

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

fn server_with(capabilities: Option<&[&str]>) -> FakeServer {
	FakeServer {
		capabilities: capabilities.map(|list| list.iter().map(|c| c.to_string()).collect()),
		..FakeServer::default()
	}
}

#[tokio::test]
async fn the_session_asks_the_server_once_what_it_supports() {
	let fake = server_with(Some(&["body", "body-markup", "actions"]));
	let (_server, client) = peers(fake.clone()).await;
	let app = tauri::test::mock_app();
	let notifier = Notifier::default();

	let (_, caps) = notifier
		.session_with(app.handle(), async { Ok(client) })
		.await
		.expect("connected");
	assert!(caps.body && caps.body_markup && caps.actions);
	// Later calls reuse the first connection instead of connecting again.
	let cached = notifier
		.session_with(app.handle(), async {
			Err(zbus::Error::Failure("must not connect twice".into()))
		})
		.await;
	assert!(cached.is_some());

	let mut request = request();
	request.body = "a & b".into();
	notifier.send(app.handle(), &request).await.unwrap();
	let calls = fake.calls.lock().unwrap().clone();
	assert_eq!(calls[0].3, "a &amp; b");
	assert_eq!(calls[0].4, ["default", "Show WingDrive"]);
}

#[tokio::test]
async fn a_server_without_capabilities_gets_a_plain_notification() {
	let fake = server_with(None);
	let (_server, client) = peers(fake.clone()).await;
	let app = tauri::test::mock_app();
	let notifier = Notifier::default();

	let (_, caps) = notifier
		.session_with(app.handle(), async { Ok(client) })
		.await
		.expect("connected");
	assert_eq!(*caps, Capabilities::minimal());
	notifier.send(app.handle(), &request()).await.unwrap();
	let calls = fake.calls.lock().unwrap().clone();
	// No action button the server never said it could show.
	assert!(calls[0].4.is_empty());
	assert_eq!(calls[0].3, "Copied a.txt");
}

#[tokio::test]
async fn without_a_session_bus_the_command_still_succeeds() {
	let app = tauri::test::mock_app();
	let notifier = Notifier::default();
	let session = notifier
		.session_with(app.handle(), async {
			Err(zbus::Error::Failure("no bus".into()))
		})
		.await;
	assert!(session.is_none());
	assert_eq!(
		notifier.send(app.handle(), &request()).await.unwrap_err(),
		"session bus unavailable"
	);

	// A missing notification server must not turn a finished copy into an error.
	app.manage(notifier);
	show_desktop_notification(app.handle().clone(), app.state(), request())
		.await
		.unwrap();
}

type PortalCall = (String, Vec<String>);

/// Records `AddNotification` calls the way the desktop portal receives them.
#[derive(Clone, Default)]
struct FakePortal {
	calls: Arc<Mutex<Vec<PortalCall>>>,
}

#[zbus::interface(name = "org.freedesktop.portal.Notification")]
impl FakePortal {
	fn add_notification(&self, id: String, notification: HashMap<String, OwnedVariant>) {
		let mut keys: Vec<String> = notification.into_keys().collect();
		keys.sort();
		self.calls.lock().unwrap().push((id, keys));
	}
}

#[tokio::test]
async fn sandboxed_notifications_go_through_the_portal() {
	let portal = FakePortal::default();
	let (a, b) = std::os::unix::net::UnixStream::pair().unwrap();
	let server = zbus::connection::Builder::unix_stream(a)
		.p2p()
		.server(zbus::Guid::generate())
		.serve_at(PORTAL_PATH, portal.clone())
		.build();
	let client = zbus::connection::Builder::unix_stream(b).p2p().build();
	let (_server, client) = tokio::join!(server, client);
	let client = client.unwrap();

	add_portal_notification(&client, &request()).await.unwrap();
	let mut empty = request();
	empty.body.clear();
	add_portal_notification(&client, &empty).await.unwrap();

	let calls = portal.calls.lock().unwrap().clone();
	assert_eq!(calls[0].0, format!("wingdrive-{}", std::process::id()));
	assert_eq!(calls[0].1, ["body", "icon", "priority", "title"]);
	// An empty body is left out rather than shown as a blank line.
	assert_eq!(calls[1].1, ["icon", "priority", "title"]);
}

#[test]
fn identity_prefers_the_theme_icon_and_falls_back_to_window_pixels() {
	let installed = tempfile::tempdir().unwrap();
	let apps = installed.path().join("icons/hicolor/48x48/apps");
	std::fs::create_dir_all(&apps).unwrap();
	std::fs::write(apps.join("WingDrive.png"), b"png").unwrap();
	std::fs::create_dir(installed.path().join("applications")).unwrap();
	std::fs::write(installed.path().join("applications/wingdrive.desktop"), b"").unwrap();

	let mut context = tauri::test::mock_context(tauri::test::noop_assets());
	context.set_default_window_icon(Some(tauri::image::Image::new_owned(vec![9; 16], 2, 2)));
	let with_icon = tauri::test::mock_builder().build(context).unwrap();

	let identity = detect_identity(with_icon.handle(), &[installed.path().to_path_buf()]);
	assert_eq!(identity.desktop_entry, "wingdrive");
	assert_eq!(identity.logo, LogoSource::Themed(ICON_NAME.into()));

	// An AppImage without desktop integration sends the window icon itself.
	let bare = tempfile::tempdir().unwrap();
	let identity = detect_identity(with_icon.handle(), &[bare.path().to_path_buf()]);
	assert_eq!(identity.desktop_entry, PROGRAM_NAME);
	assert_eq!(
		identity.logo,
		LogoSource::Pixels(RgbaImage {
			width: 2,
			height: 2,
			rgba: vec![9; 16],
		})
	);

	let app = tauri::test::mock_app();
	let identity = detect_identity(app.handle(), &[bare.path().to_path_buf()]);
	assert_eq!(identity.logo, LogoSource::None);
}
