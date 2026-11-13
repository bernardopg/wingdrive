//! Drag sessions and the events the windows follow them by, driven through a
//! mock Tauri app.

use std::sync::{Arc, Mutex};
use std::time::Instant;

use serde_json::{json, Value};
use tauri::{Listener, Manager};

use super::*;

/// Collects the payloads emitted on `event` anywhere in the app.
fn record<R: Runtime>(app: &tauri::App<R>, event: &str) -> Arc<Mutex<Vec<Value>>> {
	let seen = Arc::new(Mutex::new(Vec::new()));
	let sink = seen.clone();
	app.listen_any(event, move |event| {
		sink.lock()
			.unwrap()
			.push(serde_json::from_str(event.payload()).unwrap())
	});
	seen
}

fn payloads(seen: &Arc<Mutex<Vec<Value>>>) -> Vec<Value> {
	seen.lock().unwrap().clone()
}

/// The payload `begin_drag` accepts. Field names inside an item kind stay in
/// snake case, as the Swift drag code decodes them (`drag.swift`).
fn frontend_config() -> DragConfig {
	serde_json::from_value(json!({
		"items": [
			{"id": "1", "kind": {"type": "file", "path": "/home/user/a b.txt"}},
			{"id": "2", "kind": {"type": "filePromise", "name": "c.txt", "mime_type": "text/plain"}},
			{"id": "3", "kind": {"type": "text", "content": "hello"}}
		],
		"overlayUrl": "/drag-overlay",
		"overlaySize": [200, 150],
		"allowedOperations": ["copy", "move", "link"]
	}))
	.unwrap()
}

#[test]
fn frontend_drag_config_and_results_keep_their_wire_format() {
	let config = frontend_config();
	assert_eq!(config.overlay_size, (200.0, 150.0));
	assert_eq!(
		config.allowed_operations,
		[
			DragOperation::Copy,
			DragOperation::Move,
			DragOperation::Link
		]
	);
	assert!(matches!(
		&config.items[1].kind,
		DragItemKind::FilePromise { name, mime_type } if name == "c.txt" && mime_type == "text/plain"
	));
	// Unknown item kinds and operations are rejected instead of guessed.
	for bad in [
		json!({"items": [{"id": "1", "kind": {"type": "url", "href": "x"}}], "overlayUrl": "", "overlaySize": [0, 0], "allowedOperations": []}),
		json!({"items": [], "overlayUrl": "", "overlaySize": [0, 0], "allowedOperations": ["Copy"]}),
	] {
		assert!(serde_json::from_value::<DragConfig>(bad).is_err());
	}

	// `end_drag` receives these from `useDragOperation.ts` and the drop handlers.
	for (wire, expected) in [
		(json!({"type": "Cancelled"}), "Cancelled"),
		(
			json!({"type": "Dropped", "operation": "move", "target": "main"}),
			"Dropped",
		),
		(json!({"type": "Failed", "error": "boom"}), "Failed"),
	] {
		let result: DragResult = serde_json::from_value(wire.clone()).unwrap();
		assert_eq!(serde_json::to_value(&result).unwrap(), wire);
		assert!(format!("{result:?}").starts_with(expected));
	}
}

#[test]
fn a_drag_session_announces_its_items_and_blocks_a_second_drag() {
	let app = tauri::test::mock_app();
	let began = record(&app, "drag:began");
	let coordinator = DragCoordinator::default();
	assert!(coordinator.current_session().is_none());

	coordinator
		.begin_drag(app.handle(), frontend_config(), "main".into())
		.unwrap();
	let session = coordinator.current_session().unwrap();
	assert_eq!(session.source_window, "main");
	assert!(uuid::Uuid::parse_str(&session.id).is_ok());
	assert!(session.started_at > 0);
	assert_eq!(
		payloads(&began),
		[json!({
			"sessionId": session.id,
			"sourceWindow": "main",
			"items": serde_json::to_value(&frontend_config().items).unwrap(),
		})]
	);

	let error = coordinator
		.begin_drag(app.handle(), frontend_config(), "other".into())
		.unwrap_err();
	assert_eq!(error, "A drag operation is already in progress");
	assert_eq!(coordinator.current_session().unwrap().id, session.id);
	assert_eq!(payloads(&began).len(), 1);
}

/// A session whose end never arrived (a crashed drop target, a lost callback)
/// must not block drags forever.
#[test]
fn a_stale_session_is_replaced_by_the_next_drag() {
	let app = tauri::test::mock_app();
	let ended = record(&app, "drag:ended");
	let coordinator = DragCoordinator::new();
	coordinator
		.begin_drag(app.handle(), frontend_config(), "main".into())
		.unwrap();
	let stale_id = coordinator.current_session().unwrap().id;
	{
		let mut state = coordinator.state.write();
		let (_, started_at) = state.as_mut().unwrap();
		*started_at = Instant::now()
			.checked_sub(STALE_SESSION + Duration::from_secs(1))
			.unwrap();
	}

	coordinator
		.begin_drag(app.handle(), frontend_config(), "second".into())
		.unwrap();
	let session = coordinator.current_session().unwrap();
	assert_ne!(session.id, stale_id);
	assert_eq!(session.source_window, "second");
	// The stale session is dropped silently; only the live one can end.
	assert!(payloads(&ended).is_empty());
}

#[test]
fn position_and_window_events_carry_the_active_session_only() {
	let app = tauri::test::mock_app();
	let moved = record(&app, "drag:moved");
	let entered = record(&app, "drag:entered");
	let left = record(&app, "drag:left");
	let coordinator = DragCoordinator::new();

	// Without a session nothing is emitted.
	coordinator.update_position(app.handle(), 1.0, 2.0);
	coordinator.enter_window(app.handle(), "main".into());
	coordinator.leave_window(app.handle(), "main".into());
	assert!(payloads(&moved).is_empty());
	assert!(payloads(&entered).is_empty());
	assert!(payloads(&left).is_empty());

	coordinator
		.begin_drag(app.handle(), frontend_config(), "main".into())
		.unwrap();
	let id = coordinator.current_session().unwrap().id;
	coordinator.update_position(app.handle(), 10.5, -3.0);
	coordinator.enter_window(app.handle(), "inspector".into());
	coordinator.leave_window(app.handle(), "inspector".into());
	assert_eq!(
		payloads(&moved),
		[json!({"sessionId": id, "x": 10.5, "y": -3.0})]
	);
	assert_eq!(
		payloads(&entered),
		[json!({"sessionId": id, "windowLabel": "inspector"})]
	);
	assert_eq!(payloads(&left), payloads(&entered));
}

#[test]
fn ending_a_drag_reports_the_result_once_and_frees_the_coordinator() {
	let app = tauri::test::mock_app();
	let ended = record(&app, "drag:ended");
	let coordinator = DragCoordinator::new();

	// Ending with no session is a no-op, as when the Swift callback races `end_drag`.
	coordinator.end_drag(app.handle(), DragResult::Cancelled);
	assert!(payloads(&ended).is_empty());

	coordinator
		.begin_drag(app.handle(), frontend_config(), "main".into())
		.unwrap();
	let id = coordinator.current_session().unwrap().id;
	coordinator.end_drag(
		app.handle(),
		DragResult::Dropped {
			operation: DragOperation::Move,
			target: Some("main".into()),
		},
	);
	coordinator.end_drag(app.handle(), DragResult::Cancelled);
	assert!(coordinator.current_session().is_none());
	assert_eq!(
		payloads(&ended),
		[json!({
			"sessionId": id,
			"result": {"type": "Dropped", "operation": "move", "target": "main"}
		})]
	);

	// A new drag can start right away.
	coordinator
		.begin_drag(app.handle(), frontend_config(), "main".into())
		.unwrap();
}

#[test]
fn force_clearing_cancels_the_active_session() {
	let app = tauri::test::mock_app();
	let ended = record(&app, "drag:ended");
	let coordinator = DragCoordinator::new();
	coordinator.force_clear_state(app.handle());
	assert!(payloads(&ended).is_empty());

	coordinator
		.begin_drag(app.handle(), frontend_config(), "main".into())
		.unwrap();
	let id = coordinator.current_session().unwrap().id;
	coordinator.force_clear_state(app.handle());
	assert!(coordinator.current_session().is_none());
	assert_eq!(
		payloads(&ended),
		[json!({"sessionId": id, "result": {"type": "Cancelled"}})]
	);
}

#[tokio::test]
async fn commands_read_and_end_the_session_held_in_app_state() {
	let app = tauri::test::mock_app();
	app.manage(DragCoordinator::new());
	let ended = record(&app, "drag:ended");
	let handle = app.handle().clone();

	assert!(get_drag_session(handle.clone()).await.unwrap().is_none());
	app.state::<DragCoordinator>()
		.begin_drag(&handle, frontend_config(), "main".into())
		.unwrap();
	let session = get_drag_session(handle.clone()).await.unwrap().unwrap();
	assert_eq!(session.config.items.len(), 3);

	end_drag(
		handle.clone(),
		session.id.clone(),
		DragResult::Failed {
			error: "target refused".into(),
		},
	)
	.await
	.unwrap();
	assert!(get_drag_session(handle.clone()).await.unwrap().is_none());

	app.state::<DragCoordinator>()
		.begin_drag(&handle, frontend_config(), "main".into())
		.unwrap();
	force_clear_drag_state(handle.clone()).await.unwrap();
	assert!(get_drag_session(handle).await.unwrap().is_none());
	let results: Vec<Value> = payloads(&ended)
		.into_iter()
		.map(|p| p["result"].clone())
		.collect();
	assert_eq!(
		results,
		[
			json!({"type": "Failed", "error": "target refused"}),
			json!({"type": "Cancelled"})
		]
	);
}

/// Off macOS there is no native drag, so the command must not leave a session
/// behind: it would announce a drag to every window and refuse the next
/// attempt for 30 seconds.
#[cfg(not(target_os = "macos"))]
#[tokio::test]
async fn begin_drag_off_macos_fails_without_starting_a_session() {
	let app = tauri::test::mock_app();
	app.manage(DragCoordinator::new());
	let began = record(&app, "drag:began");
	let handle = app.handle().clone();

	for _ in 0..2 {
		let error = begin_drag(handle.clone(), frontend_config(), "main".into())
			.await
			.unwrap_err();
		assert_eq!(error, "Drag and drop is only supported on macOS currently");
	}
	assert!(get_drag_session(handle).await.unwrap().is_none());
	assert!(payloads(&began).is_empty());
}
