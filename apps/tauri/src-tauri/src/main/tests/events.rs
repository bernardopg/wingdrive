//! Menu, library selection and event subscription commands, driven through a
//! mock Tauri app and TCP listeners standing in for the daemon.

use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde_json::{json, Value};
use tauri::{Listener, Manager, WebviewUrl, WebviewWindowBuilder};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;

use super::{app_state, app_with_daemon, closed_address, daemon_state, one_shot_daemon};
use crate::{
	cleanup_all_connections, get_default_event_subscription, globals_script, handle_menu_event,
	open_library_request, set_current_library_id, set_library_id, set_selected_file_ids,
	setup_menu, subscribe_to_events, update_menu_items, validate_and_reset_library_if_needed,
	AppState, DaemonConnectionPool, MenuItemState, MenuState,
};

/// Collects the payloads emitted on `event` anywhere in the app.
fn record<R: tauri::Runtime>(app: &tauri::App<R>, event: &str) -> Arc<Mutex<Vec<String>>> {
	let seen = Arc::new(Mutex::new(Vec::new()));
	let sink = seen.clone();
	app.listen_any(event, move |event| {
		sink.lock().unwrap().push(event.payload().to_string())
	});
	seen
}

async fn wait_for(mut done: impl FnMut() -> bool) {
	tokio::time::timeout(Duration::from_secs(5), async {
		while !done() {
			tokio::time::sleep(Duration::from_millis(10)).await;
		}
	})
	.await
	.expect("condition not reached in time");
}

#[test]
fn default_subscription_matches_the_typescript_client() {
	let source = include_str!("../../../../../../packages/ts-client/src/event-filter.ts");
	let start = source
		.find("DEFAULT_EVENT_SUBSCRIPTION")
		.expect("default list in event-filter.ts");
	let list = &source[start..source[start..].find("];").unwrap() + start];
	let typescript: Vec<&str> = list
		.lines()
		.filter_map(|line| {
			let line = line.trim();
			line.strip_prefix('"')
				.and_then(|rest| rest.strip_suffix("\","))
		})
		.collect();
	assert_eq!(get_default_event_subscription(), typescript);
	// Progress events of the indexer and log lines would flood every window.
	for noisy in ["LogMessage", "IndexingProgress"] {
		assert!(!typescript.contains(&noisy), "{noisy}");
	}
}

#[test]
fn injected_globals_are_json_string_literals() {
	assert_eq!(
		globals_script("http://127.0.0.1:1", "lib"),
		r#"window.__WINGDRIVE_SERVER_URL__ = "http://127.0.0.1:1"; window.__WINGDRIVE_LIBRARY_ID__ = "lib";"#
	);
	// A quote cannot close the literal and run the rest as script.
	let script = globals_script("u", "x\"; alert(1); \"");
	assert!(script.ends_with(r#"= "x\"; alert(1); \"";"#), "{script}");
	assert!(!globals_script("u", "a\nb").contains('\n'));
}

#[tokio::test]
async fn opening_a_library_returns_the_daemon_answer_or_its_error() {
	let (address, daemon) = one_shot_daemon("{\"JsonOk\":{\"library_id\":\"l1\"}}\n").await;
	let library = open_library_request(&address, Path::new("/libraries/Work"))
		.await
		.unwrap();
	assert_eq!(library, json!({"library_id": "l1"}));
	let sent: Value = serde_json::from_str(&daemon.await.unwrap()).unwrap();
	assert_eq!(sent["Action"]["method"], "action:libraries.open.input");
	assert_eq!(sent["Action"]["payload"]["path"], "/libraries/Work");

	// A refusal is an error, not an opened library.
	let (address, _daemon) =
		one_shot_daemon("{\"Error\":{\"ValidationError\":\"no library in folder\"}}\n").await;
	let error = open_library_request(&address, Path::new("/tmp"))
		.await
		.unwrap_err();
	assert_eq!(error, "Failed to open library: no library in folder");

	let (address, _daemon) = one_shot_daemon("{\"Error\":\"InvalidRequest\"}\n").await;
	let error = open_library_request(&address, Path::new("/tmp"))
		.await
		.unwrap_err();
	assert!(error.contains("InvalidRequest"), "{error}");

	let (address, _daemon) = one_shot_daemon("").await;
	let error = open_library_request(&address, Path::new("/tmp"))
		.await
		.unwrap_err();
	assert!(error.contains("without answering"), "{error}");

	let (address, _daemon) = one_shot_daemon("not json\n").await;
	let error = open_library_request(&address, Path::new("/tmp"))
		.await
		.unwrap_err();
	assert!(error.contains("invalid response"), "{error}");

	let error = open_library_request(&closed_address().await, Path::new("/tmp"))
		.await
		.unwrap_err();
	assert!(error.starts_with("Failed to connect to daemon"), "{error}");
}

#[tokio::test]
async fn menu_items_forward_file_and_clipboard_actions_to_the_frontend() {
	let app = tauri::test::mock_app();
	let actions = record(&app, "menu-action");
	let clipboard = record(&app, "clipboard-action");
	let handle = app.handle().clone();
	for id in [
		"duplicate",
		"rename",
		"delete",
		"cut",
		"copy",
		"unknown-item",
	] {
		handle_menu_event(&handle, id);
	}
	assert_eq!(
		*actions.lock().unwrap(),
		["\"duplicate\"", "\"rename\"", "\"delete\""]
	);
	assert_eq!(*clipboard.lock().unwrap(), ["\"cut\"", "\"copy\""]);

	handle_menu_event(&handle, "wingdrop");
	handle_menu_event(&handle, "drag-demo");
	wait_for(|| {
		handle.get_webview_window("wingdrop").is_some()
			&& handle.get_webview_window("drag-demo").is_some()
	})
	.await;
}

#[tokio::test]
async fn menu_is_built_and_its_file_items_follow_the_selection() {
	let app = tauri::test::mock_app();
	let handle = app.handle().clone();
	assert_eq!(
		update_menu_items(handle.clone(), vec![]).await.unwrap_err(),
		"Menu state not initialized"
	);
	setup_menu(&handle).unwrap();
	let items = app
		.state::<MenuState<tauri::test::MockRuntime>>()
		.items
		.clone();
	let mut ids: Vec<String> = items.read().await.keys().cloned().collect();
	ids.sort();
	assert_eq!(ids, ["copy", "cut", "delete", "duplicate", "rename"]);
	assert!(!items.read().await["rename"].is_enabled().unwrap());
	assert!(items.read().await["copy"].is_enabled().unwrap());

	update_menu_items(
		handle.clone(),
		vec![
			MenuItemState {
				id: "rename".into(),
				enabled: true,
			},
			// Items the menu does not have are ignored.
			MenuItemState {
				id: "missing".into(),
				enabled: true,
			},
		],
	)
	.await
	.unwrap();
	assert!(items.read().await["rename"].is_enabled().unwrap());
}

/// Accepts one connection, answers the libraries query with `libraries`.
async fn libraries_daemon(libraries: Value) -> String {
	let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
	let address = listener.local_addr().unwrap().to_string();
	tokio::spawn(async move {
		let (stream, _) = listener.accept().await.unwrap();
		let (reader, mut writer) = stream.into_split();
		let mut line = String::new();
		BufReader::new(reader).read_line(&mut line).await.unwrap();
		let request: Value = serde_json::from_str(&line).unwrap();
		assert_eq!(request["Query"]["method"], "query:libraries.list");
		let reply = json!({ "JsonOk": libraries });
		writer
			.write_all(format!("{reply}\n").as_bytes())
			.await
			.unwrap();
	});
	address
}

#[tokio::test]
async fn a_library_that_no_longer_exists_is_forgotten() {
	let data = tempfile::tempdir().unwrap();
	let persisted = data.path().join("current_library_id.txt");
	let app = tauri::test::mock_app();
	let changes = record(&app, "library-changed");
	let current = Arc::new(tokio::sync::RwLock::new(None));

	// Nothing selected: nothing to ask the daemon.
	let state = daemon_state(&closed_address().await, None, None);
	validate_and_reset_library_if_needed(app.handle().clone(), &current, &state, data.path())
		.await
		.unwrap();

	*current.write().await = Some("kept".to_string());
	std::fs::write(&persisted, "kept").unwrap();
	let address = libraries_daemon(json!([{"id": "kept"}, {"id": "other"}])).await;
	let state = daemon_state(&address, None, None);
	validate_and_reset_library_if_needed(app.handle().clone(), &current, &state, data.path())
		.await
		.unwrap();
	assert_eq!(current.read().await.as_deref(), Some("kept"));
	assert!(persisted.exists());
	assert!(changes.lock().unwrap().is_empty());

	let address = libraries_daemon(json!([{"id": "other"}])).await;
	let state = daemon_state(&address, None, None);
	validate_and_reset_library_if_needed(app.handle().clone(), &current, &state, data.path())
		.await
		.unwrap();
	assert!(current.read().await.is_none());
	assert!(!persisted.exists());
	assert_eq!(*changes.lock().unwrap(), ["\"\""]);
}

#[tokio::test]
async fn library_validation_reports_unreachable_and_refusing_daemons() {
	let data = tempfile::tempdir().unwrap();
	let app = tauri::test::mock_app();
	let current = Arc::new(tokio::sync::RwLock::new(Some("lib".to_string())));
	let state = daemon_state(&closed_address().await, None, None);
	let error =
		validate_and_reset_library_if_needed(app.handle().clone(), &current, &state, data.path())
			.await
			.unwrap_err();
	assert!(error.starts_with("Failed to connect"), "{error}");

	let (address, _daemon) = one_shot_daemon("{\"Error\":\"CoreUnavailable\"}\n").await;
	let state = daemon_state(&address, None, None);
	let error =
		validate_and_reset_library_if_needed(app.handle().clone(), &current, &state, data.path())
			.await
			.unwrap_err();
	assert!(error.contains("Invalid response format"), "{error}");
	// An error keeps the selection: the library may still exist.
	assert_eq!(current.read().await.as_deref(), Some("lib"));
}

#[tokio::test]
async fn selecting_a_library_persists_it_and_tells_every_window() {
	let data = tempfile::tempdir().unwrap();
	let state = daemon_state("127.0.0.1:9", Some("http://127.0.0.1:4"), None);
	state.write().await.data_dir = data.path().to_path_buf();
	let app = app_with_daemon(state);
	app.manage(app_state());
	WebviewWindowBuilder::new(&app, "main", WebviewUrl::default())
		.build()
		.unwrap();
	let changes = record(&app, "library-changed");

	set_current_library_id(
		app.handle().clone(),
		"lib-2".into(),
		app.state(),
		app.state(),
	)
	.await
	.unwrap();
	assert_eq!(
		app.state::<AppState>()
			.current_library_id
			.read()
			.await
			.as_deref(),
		Some("lib-2")
	);
	assert_eq!(
		std::fs::read_to_string(data.path().join("current_library_id.txt")).unwrap(),
		"lib-2"
	);
	assert_eq!(*changes.lock().unwrap(), ["\"lib-2\""]);
	set_library_id(app.handle().clone(), "lib-2".into(), app.state())
		.await
		.unwrap();

	let selection = record(&app, "selected-files-changed");
	set_selected_file_ids(
		app.handle().clone(),
		vec!["a".into(), "b".into()],
		app.state(),
	)
	.await
	.unwrap();
	assert_eq!(
		*app.state::<AppState>().selected_file_ids.read().await,
		["a", "b"]
	);
	assert_eq!(*selection.lock().unwrap(), ["[\"a\",\"b\"]"]);
}

#[tokio::test]
async fn a_library_selection_that_cannot_be_saved_still_takes_effect() {
	let state = daemon_state("127.0.0.1:9", None, None);
	let app = app_with_daemon(state);
	app.manage(app_state());
	// The data directory does not exist, so the file write fails.
	set_current_library_id(app.handle().clone(), "lib".into(), app.state(), app.state())
		.await
		.unwrap();
	assert_eq!(
		app.state::<AppState>()
			.current_library_id
			.read()
			.await
			.as_deref(),
		Some("lib")
	);
}

/// Accepts one subscription, sends `events` and closes the connection.
async fn event_daemon(events: Vec<Value>) -> (String, tokio::task::JoinHandle<Vec<Value>>) {
	let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
	let address = listener.local_addr().unwrap().to_string();
	let task = tokio::spawn(async move {
		let (stream, _) = listener.accept().await.unwrap();
		let (reader, mut writer) = stream.into_split();
		let mut reader = BufReader::new(reader);
		let mut line = String::new();
		reader.read_line(&mut line).await.unwrap();
		let mut received = vec![serde_json::from_str(&line).unwrap()];
		for event in events {
			writer
				.write_all(format!("{event}\n").as_bytes())
				.await
				.unwrap();
		}
		writer.write_all(b"not json\n\n").await.unwrap();
		drop(writer);
		line.clear();
		if reader.read_line(&mut line).await.unwrap_or(0) > 0 {
			received.push(serde_json::from_str(&line).unwrap());
		}
		received
	});
	(address, task)
}

#[tokio::test]
async fn subscriptions_forward_events_to_their_window_and_report_the_end() {
	let (address, daemon) = event_daemon(vec![
		json!({"Event": {"LibraryCreated": {"id": "l"}}}),
		json!("Subscribed"),
	])
	.await;
	let app = app_with_daemon(daemon_state(&address, None, None));
	app.manage(app_state());
	let window = WebviewWindowBuilder::new(&app, "main", WebviewUrl::default())
		.build()
		.unwrap();
	let events = record(&app, "events:1");
	let closed = record(&app, "events:1:closed");

	let id = subscribe_to_events(
		app.handle().clone(),
		window,
		app.state(),
		app.state(),
		Some(vec!["LibraryCreated".into()]),
		Some(json!({"library_id": "l"})),
		Some("events:1".into()),
	)
	.await
	.unwrap();

	wait_for(|| !closed.lock().unwrap().is_empty()).await;
	assert_eq!(
		*events.lock().unwrap(),
		[json!({"LibraryCreated": {"id": "l"}}).to_string()]
	);
	assert_eq!(*closed.lock().unwrap(), ["\"event stream closed\""]);
	let received = daemon.await.unwrap();
	assert_eq!(
		received[0],
		json!({"Subscribe": {"event_types": ["LibraryCreated"], "filter": {"library_id": "l"}}})
	);
	assert_eq!(received.get(1), Some(&json!("Unsubscribe")));
	// The ended stream no longer counts as active.
	let subscriptions = app
		.state::<AppState>()
		.subscription_manager
		.subscriptions
		.clone();
	tokio::time::timeout(Duration::from_secs(5), async {
		while subscriptions.read().await.contains_key(&id) {
			tokio::time::sleep(Duration::from_millis(10)).await;
		}
	})
	.await
	.expect("subscription still registered");
}

#[tokio::test]
async fn subscriptions_without_types_use_the_default_list_and_shared_channel() {
	let (address, daemon) = event_daemon(vec![json!({"Event": "CoreStarted"})]).await;
	let app = app_with_daemon(daemon_state(&address, None, None));
	app.manage(app_state());
	let window = WebviewWindowBuilder::new(&app, "main", WebviewUrl::default())
		.build()
		.unwrap();
	let events = record(&app, "core-event");
	subscribe_to_events(
		app.handle().clone(),
		window,
		app.state(),
		app.state(),
		None,
		None,
		None,
	)
	.await
	.unwrap();
	let received = daemon.await.unwrap();
	assert_eq!(
		received[0]["Subscribe"]["event_types"],
		json!(get_default_event_subscription())
	);
	wait_for(|| !events.lock().unwrap().is_empty()).await;
	assert_eq!(*events.lock().unwrap(), ["\"CoreStarted\""]);
}

#[tokio::test]
async fn subscriptions_reject_bad_channels_and_report_unreachable_daemons() {
	let app = app_with_daemon(daemon_state(&closed_address().await, None, None));
	app.manage(app_state());
	let window = WebviewWindowBuilder::new(&app, "main", WebviewUrl::default())
		.build()
		.unwrap();
	for channel in ["", "a b", "x;y", "ç"] {
		let error = subscribe_to_events(
			app.handle().clone(),
			window.clone(),
			app.state(),
			app.state(),
			None,
			None,
			Some(channel.into()),
		)
		.await
		.unwrap_err();
		assert_eq!(error, format!("Invalid event channel: {channel}"));
	}
	let closed = record(&app, "events:2:closed");
	subscribe_to_events(
		app.handle().clone(),
		window.clone(),
		app.state(),
		app.state(),
		None,
		None,
		Some("events:2".into()),
	)
	.await
	.unwrap();
	wait_for(|| !closed.lock().unwrap().is_empty()).await;
	assert!(closed.lock().unwrap()[0].contains("connect failed"));

	// Closing the window's subscriptions succeeds even when none is left.
	cleanup_all_connections(window, app.state()).await.unwrap();
}

#[tokio::test]
async fn window_cleanup_cancels_a_live_subscription_and_unsubscribes() {
	let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
	let address = listener.local_addr().unwrap().to_string();
	let daemon = tokio::spawn(async move {
		let (stream, _) = listener.accept().await.unwrap();
		let mut reader = BufReader::new(stream);
		let mut lines = Vec::new();
		let mut line = String::new();
		while reader.read_line(&mut line).await.unwrap() > 0 {
			lines.push(serde_json::from_str::<Value>(&line).unwrap());
			line.clear();
		}
		lines
	});
	let app = app_with_daemon(daemon_state(&address, None, None));
	app.manage(app_state());
	let window = WebviewWindowBuilder::new(&app, "main", WebviewUrl::default())
		.build()
		.unwrap();
	let closed = record(&app, "events:3:closed");
	subscribe_to_events(
		app.handle().clone(),
		window.clone(),
		app.state(),
		app.state(),
		None,
		None,
		Some("events:3".into()),
	)
	.await
	.unwrap();
	tokio::time::sleep(Duration::from_millis(100)).await;
	cleanup_all_connections(window, app.state()).await.unwrap();
	let lines = tokio::time::timeout(Duration::from_secs(5), daemon)
		.await
		.unwrap()
		.unwrap();
	assert!(lines[0].get("Subscribe").is_some());
	assert_eq!(lines.last(), Some(&json!("Unsubscribe")));
	// Cancellation by the window is not reported back to it.
	assert!(closed.lock().unwrap().is_empty());
}

#[tokio::test]
async fn the_persistent_pool_multiplexes_subscribe_and_unsubscribe() {
	let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
	let address = listener.local_addr().unwrap().to_string();
	let daemon = tokio::spawn(async move {
		let (stream, _) = listener.accept().await.unwrap();
		let (reader, mut writer) = stream.into_split();
		writer
			.write_all(b"{\"Event\":\"CoreStarted\"}\nnot json\n\n")
			.await
			.unwrap();
		let mut reader = BufReader::new(reader);
		let mut lines = Vec::new();
		let mut line = String::new();
		while lines.len() < 2 && reader.read_line(&mut line).await.unwrap() > 0 {
			lines.push(serde_json::from_str::<Value>(&line).unwrap());
			line.clear();
		}
		lines
	});
	let app = tauri::test::mock_app();
	let connected = record(&app, "daemon-connected");
	let events = record(&app, "core-event");
	let disconnected = record(&app, "daemon-disconnected");
	let pool = DaemonConnectionPool::new(address);
	pool.ensure_connected(app.handle()).await.unwrap();
	// Already connected: no second connection.
	pool.ensure_connected(app.handle()).await.unwrap();
	assert_eq!(connected.lock().unwrap().len(), 1);

	let id = pool.next_id();
	assert_eq!(pool.next_id(), id + 1);
	pool.subscribe(id, vec!["CoreStarted".into()], None)
		.await
		.unwrap();
	pool.unsubscribe(id).await.unwrap();
	assert_eq!(
		pool.unsubscribe(id).await.unwrap_err(),
		format!("Subscription {id} not found")
	);
	let lines = daemon.await.unwrap();
	assert_eq!(
		lines[0],
		json!({"Subscribe": {"event_types": ["CoreStarted"], "filter": null}})
	);
	assert_eq!(lines[1], json!("Unsubscribe"));
	wait_for(|| !disconnected.lock().unwrap().is_empty()).await;
	assert_eq!(*events.lock().unwrap(), ["\"CoreStarted\""]);

	let pool = DaemonConnectionPool::new(closed_address().await);
	assert!(pool
		.ensure_connected(app.handle())
		.await
		.unwrap_err()
		.starts_with("Failed to connect to daemon"));
}
