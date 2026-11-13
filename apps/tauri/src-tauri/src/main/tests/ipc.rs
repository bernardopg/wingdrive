//! The commands as the webview calls them: through IPC, with the camelCase
//! argument names the TypeScript client sends.

use std::io::{BufRead, BufReader, Write};
use std::sync::Arc;

use serde_json::{json, Value};
use tauri::test::{mock_builder, mock_context, noop_assets, MockRuntime, INVOKE_KEY};
use tauri::{Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

use super::{app_state, daemon_state};
use crate::launch;

fn invoke(window: &WebviewWindow<MockRuntime>, cmd: &str, body: Value) -> Result<Value, Value> {
	tauri::test::get_ipc_response(
		window,
		tauri::webview::InvokeRequest {
			cmd: cmd.into(),
			callback: tauri::ipc::CallbackFn(0),
			error: tauri::ipc::CallbackFn(1),
			url: if cfg!(windows) {
				"http://tauri.localhost"
			} else {
				"tauri://localhost"
			}
			.parse()
			.unwrap(),
			body: tauri::ipc::InvokeBody::Json(body),
			headers: Default::default(),
			invoke_key: INVOKE_KEY.to_string(),
		},
	)
	.map(|body| body.deserialize::<Value>().unwrap())
}

/// Accepts one connection on a plain thread and answers its line with `reply`.
fn daemon_thread(reply: &'static str) -> (String, std::thread::JoinHandle<String>) {
	let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
	let address = listener.local_addr().unwrap().to_string();
	let handle = std::thread::spawn(move || {
		let (stream, _) = listener.accept().unwrap();
		let mut line = String::new();
		BufReader::new(&stream).read_line(&mut line).unwrap();
		(&stream).write_all(reply.as_bytes()).unwrap();
		line
	});
	(address, handle)
}

#[test]
fn commands_answer_the_webview_with_its_argument_names() {
	let data = tempfile::tempdir().unwrap();
	let (address, daemon) = daemon_thread("{\"JsonOk\":{\"pong\":true}}\n");
	let app = mock_builder()
		.invoke_handler(tauri::generate_handler![
			crate::app_ready,
			crate::get_daemon_socket,
			crate::get_server_url,
			crate::get_file_stream_url,
			crate::set_library_id,
			crate::get_current_library_id,
			crate::set_current_library_id,
			crate::get_selected_file_ids,
			crate::set_selected_file_ids,
			crate::daemon_request,
			crate::subscribe_to_events,
			crate::unsubscribe_from_events,
			crate::cleanup_all_connections,
			crate::get_active_subscriptions,
			crate::update_menu_items,
			crate::get_daemon_status,
			crate::start_daemon_process,
			crate::stop_daemon_process,
			crate::open_macos_settings,
		])
		.build(mock_context(noop_assets()))
		.unwrap();
	let state = daemon_state(&address, Some("http://127.0.0.1:4"), Some("t"));
	tauri::async_runtime::block_on(async { state.write().await.data_dir = data.path().into() });
	app.manage(state);
	app.manage(app_state());
	app.manage(launch::StartHidden(false.into()));
	let window = WebviewWindowBuilder::new(&app, "main", WebviewUrl::default())
		.build()
		.unwrap();

	assert_eq!(invoke(&window, "app_ready", json!({})), Ok(Value::Null));
	assert_eq!(
		invoke(&window, "get_daemon_socket", json!({})),
		Ok(json!(address))
	);
	assert_eq!(
		invoke(&window, "get_server_url", json!({})),
		Ok(json!("http://127.0.0.1:4"))
	);
	assert_eq!(
		invoke(&window, "get_file_stream_url", json!({ "path": "/a b" })),
		Ok(json!("http://127.0.0.1:4/file?path=/a%20b&token=t"))
	);

	assert_eq!(
		invoke(&window, "daemon_request", json!({ "request": "Ping" })),
		Ok(json!({ "JsonOk": { "pong": true } }))
	);
	assert_eq!(daemon.join().unwrap(), "\"Ping\"\n");

	assert_eq!(
		invoke(
			&window,
			"set_selected_file_ids",
			json!({ "fileIds": ["a", "b"] })
		),
		Ok(Value::Null)
	);
	assert_eq!(
		invoke(&window, "get_selected_file_ids", json!({})),
		Ok(json!(["a", "b"]))
	);

	assert_eq!(
		invoke(&window, "get_current_library_id", json!({})),
		Err(json!("No library selected"))
	);
	assert_eq!(
		invoke(
			&window,
			"set_current_library_id",
			json!({ "libraryId": "lib" })
		),
		Ok(Value::Null)
	);
	assert_eq!(
		invoke(&window, "get_current_library_id", json!({})),
		Ok(json!("lib"))
	);
	assert_eq!(
		std::fs::read_to_string(data.path().join("current_library_id.txt")).unwrap(),
		"lib"
	);
	assert_eq!(
		invoke(&window, "set_library_id", json!({ "libraryId": "lib" })),
		Ok(Value::Null)
	);

	assert_eq!(
		invoke(
			&window,
			"subscribe_to_events",
			json!({ "eventTypes": null, "filter": null, "channel": "no spaces" })
		),
		Err(json!("Invalid event channel: no spaces"))
	);
	assert_eq!(
		invoke(
			&window,
			"unsubscribe_from_events",
			json!({ "subscriptionId": 7 })
		),
		Err(json!("Subscription 7 not found"))
	);
	assert_eq!(
		invoke(&window, "cleanup_all_connections", json!({})),
		Ok(Value::Null)
	);
	assert_eq!(
		invoke(&window, "get_active_subscriptions", json!({})),
		Ok(json!(0))
	);
	assert_eq!(
		invoke(
			&window,
			"update_menu_items",
			json!({ "items": [{ "id": "rename", "enabled": true }] })
		),
		Err(json!("Menu state not initialized"))
	);

	// The one-shot daemon is gone, so the status reports it stopped.
	let status = invoke(&window, "get_daemon_status", json!({})).unwrap();
	assert_eq!(status["is_running"], false);
	assert_eq!(status["server_token"], "t");
	assert_eq!(
		invoke(&window, "stop_daemon_process", json!({})),
		Err(json!("Cannot stop daemon we didn't start"))
	);
	let error = invoke(&window, "start_daemon_process", json!({})).unwrap_err();
	assert!(
		error
			.as_str()
			.unwrap()
			.starts_with("Daemon binary not found"),
		"{error}"
	);
	#[cfg(not(target_os = "macos"))]
	assert_eq!(
		invoke(&window, "open_macos_settings", json!({})),
		Err(json!("Not supported on this platform"))
	);

	let state = app.state::<Arc<tokio::sync::RwLock<crate::DaemonState>>>();
	assert!(!tauri::async_runtime::block_on(state.read()).started_by_us);
}
