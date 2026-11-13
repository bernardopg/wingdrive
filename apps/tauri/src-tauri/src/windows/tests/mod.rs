use std::collections::HashSet;

use serde_json::json;

use tauri::Manager;

use super::{
	clamp_menu_position, close_window, explorer_url, hash_string, list_windows, overlay_position,
	percent_encode, place_overlay, position_context_menu, resize_overlay_window, show_menu_at,
	show_window, toggle_voice_overlay_internal, WingDriveWindow,
};

fn every_window() -> Vec<WingDriveWindow> {
	vec![
		WingDriveWindow::Main,
		WingDriveWindow::Explorer {
			library_id: "lib".into(),
			path: "/home".into(),
		},
		WingDriveWindow::MediaViewer {
			file_id: "f1".into(),
		},
		WingDriveWindow::Settings { page: None },
		WingDriveWindow::JobManager,
		WingDriveWindow::DeviceDiscovery,
		WingDriveWindow::Wingbot,
		WingDriveWindow::Inspector { item_id: None },
		WingDriveWindow::QuickPreview {
			file_id: "f1".into(),
		},
		WingDriveWindow::TagAssignment,
		WingDriveWindow::SearchOverlay,
		WingDriveWindow::FloatingControls,
		WingDriveWindow::VoiceOverlay,
		WingDriveWindow::DragDemo,
		WingDriveWindow::Wingdrop,
		WingDriveWindow::DragOverlay {
			session_id: "s1".into(),
		},
		WingDriveWindow::ContextMenu {
			context_id: "c1".into(),
		},
	]
}

#[test]
fn every_window_kind_gets_a_distinct_valid_label() {
	let labels: Vec<String> = every_window().iter().map(WingDriveWindow::label).collect();
	assert_eq!(labels.iter().collect::<HashSet<_>>().len(), labels.len());
	for label in &labels {
		assert!(
			label
				.chars()
				.all(|c| c.is_alphanumeric() || matches!(c, '-' | '/' | ':' | '_')),
			"{label} is not a valid Tauri window label"
		);
	}
	assert_eq!(labels[0], "main");
	assert_eq!(labels[3], "settings-general");
	assert_eq!(labels[7], "inspector-floating");
}

#[test]
fn explorer_windows_are_keyed_by_library_and_path() {
	let window = |library: &str, path: &str| WingDriveWindow::Explorer {
		library_id: library.into(),
		path: path.into(),
	};
	assert_eq!(window("a", "/x").label(), window("a", "/x").label());
	assert_ne!(window("a", "/x").label(), window("a", "/y").label());
	assert_ne!(window("a", "/x").label(), window("b", "/x").label());
	assert_eq!(hash_string("/x"), hash_string("/x"));
	assert_eq!(
		WingDriveWindow::Settings {
			page: Some("privacy".into())
		}
		.label(),
		"settings-privacy"
	);
}

#[test]
fn windows_deserialize_from_the_tagged_shape_the_frontend_sends() {
	let window: WingDriveWindow =
		serde_json::from_value(json!({"type": "Explorer", "library_id": "l", "path": "/p"}))
			.unwrap();
	assert!(
		matches!(window, WingDriveWindow::Explorer { ref library_id, ref path }
		if library_id == "l" && path == "/p")
	);
	let window: WingDriveWindow = serde_json::from_value(json!({"type": "Settings"})).unwrap();
	assert!(matches!(window, WingDriveWindow::Settings { page: None }));
	assert!(serde_json::from_value::<WingDriveWindow>(json!({"type": "Unknown"})).is_err());
	assert!(serde_json::from_value::<WingDriveWindow>(json!({"type": "MediaViewer"})).is_err());
	for window in every_window() {
		let value = serde_json::to_value(&window).unwrap();
		let back: WingDriveWindow = serde_json::from_value(value).unwrap();
		assert_eq!(back.label(), window.label());
	}
}

#[test]
fn percent_encoding_matches_encode_uri_component() {
	assert_eq!(percent_encode("AZaz09-_.!~*'()"), "AZaz09-_.!~*'()");
	assert_eq!(percent_encode("/ ?&=#%+"), "%2F%20%3F%26%3D%23%25%2B");
	assert_eq!(percent_encode("é"), "%C3%A9");
	assert_eq!(
		explorer_url("/"),
		"/explorer?path=%7B%22Physical%22%3A%7B%22device_slug%22%3A%22local%22%2C%22path%22%3A%22%2F%22%7D%7D"
	);
	// Quotes in the path stay inside the JSON string.
	assert!(explorer_url("/a\"b").contains("%5C%22"));
}

#[tokio::test]
async fn every_window_kind_opens_once_and_closes_by_label() {
	let app = tauri::test::mock_app();
	let handle = app.handle().clone();
	for window in every_window() {
		let label = window.label();
		assert_eq!(
			show_window(handle.clone(), window.clone()).await.unwrap(),
			label
		);
		assert!(handle.get_webview_window(&label).is_some(), "{label}");
		// A second request surfaces the existing window instead of failing on a
		// duplicate label.
		assert_eq!(show_window(handle.clone(), window).await.unwrap(), label);
	}
	let mut open = list_windows(handle.clone()).await.unwrap();
	open.sort();
	let mut expected: Vec<String> = every_window().iter().map(WingDriveWindow::label).collect();
	expected.sort();
	assert_eq!(open, expected);

	close_window(handle.clone(), "main".into()).await.unwrap();
	// Closing a window that is not open is not an error.
	close_window(handle.clone(), "no-such-window".into())
		.await
		.unwrap();
}

#[tokio::test]
async fn window_routes_encode_identifiers_from_the_frontend() {
	let app = tauri::test::mock_app();
	let handle = app.handle().clone();
	// `/` and `:` are valid in a label but must not split the route.
	let window = WingDriveWindow::ContextMenu {
		context_id: "a/b:c".into(),
	};
	let label = show_window(handle.clone(), window).await.unwrap();
	let url = handle.get_webview_window(&label).unwrap().url().unwrap();
	assert_eq!(url.query(), Some("context=a%2Fb%3Ac"));
	let label = show_window(
		handle.clone(),
		WingDriveWindow::MediaViewer {
			file_id: "x/y".into(),
		},
	)
	.await
	.unwrap();
	let url = handle.get_webview_window(&label).unwrap().url().unwrap();
	assert!(url.path().ends_with("/media-viewer/x%2Fy"), "{url}");
	// Characters a label cannot hold are refused with the reason.
	let error = show_window(
		handle,
		WingDriveWindow::ContextMenu {
			context_id: "a&b".into(),
		},
	)
	.await
	.unwrap_err();
	assert!(error.contains("Window labels must only include"), "{error}");
}

#[tokio::test]
async fn overlay_and_menu_commands_need_an_open_window() {
	let app = tauri::test::mock_app();
	let handle = app.handle().clone();
	assert_eq!(
		resize_overlay_window(handle.clone(), "missing".into(), 10.0, 10.0)
			.await
			.unwrap_err(),
		"Overlay window not found"
	);
	assert_eq!(
		position_context_menu(handle.clone(), "missing".into(), 0.0, 0.0, 1.0, 1.0)
			.await
			.unwrap_err(),
		"Context menu window not found"
	);
}

#[test]
fn voice_overlay_toggle_closes_an_open_overlay() {
	let app = tauri::test::mock_app();
	let handle = app.handle().clone();
	tauri::async_runtime::block_on(show_window(handle.clone(), WingDriveWindow::VoiceOverlay))
		.unwrap();
	let label = WingDriveWindow::VoiceOverlay.label();
	assert!(handle.get_webview_window(&label).is_some());
	toggle_voice_overlay_internal(handle.clone()).unwrap();
	// Without an open overlay the toggle opens one in the background.
	let app = tauri::test::mock_app();
	toggle_voice_overlay_internal(app.handle().clone()).unwrap();
}

#[test]
fn context_menu_stays_inside_the_monitor() {
	let monitor = ((100, 50), (1920, 1080));
	// Fits: the corner lands on the cursor, offset by the monitor origin.
	assert_eq!(
		clamp_menu_position((10.0, 20.0), (250.0, 300.0), monitor.0, monitor.1),
		(110, 70)
	);
	// Overflows the right and bottom edges: pulled back inside.
	assert_eq!(
		clamp_menu_position((1900.0, 1000.0), (250.0, 300.0), monitor.0, monitor.1),
		(100 + 1920 - 250, 50 + 1080 - 300)
	);
	// Taller than the monitor: the top edge wins.
	assert_eq!(
		clamp_menu_position((0.0, 500.0), (250.0, 2000.0), monitor.0, monitor.1),
		(100, 50)
	);
	assert_eq!(
		clamp_menu_position((-40.0, -40.0), (10.0, 10.0), (0, 0), (800, 600)),
		(0, 0)
	);
}

#[test]
fn overlay_is_centered_above_the_bottom_edge_in_physical_pixels() {
	assert_eq!(
		overlay_position((0, 0), (1920, 1080), (520.0, 112.0), 1.0),
		((1920 - 520) / 2, 1080 - 112 - 24)
	);
	assert_eq!(
		overlay_position((1920, 0), (2880, 1800), (520.0, 112.0), 1.5),
		(1920 + (2880 - 780) / 2, 1800 - 168 - 36)
	);
}

#[cfg(target_os = "linux")]
#[test]
fn file_chooser_opens_the_explorer_at_the_folder() {
	let app = tauri::test::mock_app();
	let folder = std::path::Path::new("/home/me/Documentos");
	let window =
		super::create_file_chooser_window(app.handle(), "file-chooser-1", folder, "  ").unwrap();
	assert_eq!(window.label(), "file-chooser-1");
	let url = window.url().unwrap();
	let route = format!("{}?{}", url.path(), url.query().unwrap_or_default());
	assert!(
		route.ends_with(&explorer_url("/home/me/Documentos")),
		"{route}"
	);
	// The picker label is chosen by the portal request; a duplicate is refused.
	let error = super::create_file_chooser_window(app.handle(), "file-chooser-1", folder, "Open")
		.unwrap_err();
	assert!(error.starts_with("Failed to create window"), "{error}");
}

#[tokio::test]
async fn windows_with_invalid_identifiers_are_refused() {
	let app = tauri::test::mock_app();
	for window in [
		WingDriveWindow::MediaViewer {
			file_id: "a&b".into(),
		},
		WingDriveWindow::Inspector {
			item_id: Some("a&b".into()),
		},
		WingDriveWindow::DragOverlay {
			session_id: "a&b".into(),
		},
	] {
		let error = show_window(app.handle().clone(), window).await.unwrap_err();
		assert!(error.contains("Window labels must only include"), "{error}");
	}
	assert!(list_windows(app.handle().clone()).await.unwrap().is_empty());
}

#[tokio::test]
async fn placing_open_overlays_needs_monitor_information() {
	let app = tauri::test::mock_app();
	let handle = app.handle().clone();
	let overlay = show_window(handle.clone(), WingDriveWindow::VoiceOverlay)
		.await
		.unwrap();
	let menu = show_window(
		handle.clone(),
		WingDriveWindow::ContextMenu {
			context_id: "c1".into(),
		},
	)
	.await
	.unwrap();
	// The mock runtime reports no monitor, as a headless session does.
	assert_eq!(
		resize_overlay_window(handle.clone(), overlay, 300.0, 90.0)
			.await
			.unwrap_err(),
		"No monitor found"
	);
	assert_eq!(
		position_context_menu(handle.clone(), menu, 10.0, 10.0, 200.0, 100.0)
			.await
			.unwrap_err(),
		"No monitor found"
	);
}

/// With monitor information (the mock runtime has none) placing an overlay
/// or a menu succeeds; the target positions are covered by the pure helpers.
#[test]
fn overlays_and_menus_are_placed_once_the_monitor_is_known() {
	let app = tauri::test::mock_app();
	let window = tauri::WebviewWindowBuilder::new(&app, "overlay", tauri::WebviewUrl::default())
		.build()
		.unwrap();
	let area = ((1920, 0), (2560, 1440));
	place_overlay(&window, area, (520.0, 112.0)).unwrap();
	show_menu_at(&window, area, (2500.0, 1400.0), (250.0, 300.0)).unwrap();
}

fn invoke(
	webview: &tauri::WebviewWindow<tauri::test::MockRuntime>,
	cmd: &str,
	body: serde_json::Value,
) -> Result<serde_json::Value, serde_json::Value> {
	tauri::test::get_ipc_response(
		webview,
		tauri::webview::InvokeRequest {
			cmd: cmd.into(),
			callback: tauri::ipc::CallbackFn(0),
			error: tauri::ipc::CallbackFn(1),
			url: "tauri://localhost".parse().unwrap(),
			body: tauri::ipc::InvokeBody::Json(body),
			headers: Default::default(),
			invoke_key: tauri::test::INVOKE_KEY.to_string(),
		},
	)
	.map(|body| body.deserialize().unwrap())
}

/// The frontend calls these commands by name with camelCase arguments.
#[test]
fn window_commands_answer_the_frontend_over_ipc() {
	let app = tauri::test::mock_builder()
		.invoke_handler(tauri::generate_handler![
			super::show_window,
			super::close_window,
			super::list_windows,
			super::toggle_voice_overlay,
			super::resize_overlay_window,
			super::position_context_menu,
			super::apply_macos_styling,
		])
		.build(tauri::test::mock_context(tauri::test::noop_assets()))
		.unwrap();
	let webview = tauri::WebviewWindowBuilder::new(&app, "main", tauri::WebviewUrl::default())
		.build()
		.unwrap();

	assert_eq!(
		invoke(
			&webview,
			"show_window",
			json!({ "window": { "type": "Settings", "page": "privacy" } })
		),
		Ok(json!("settings-privacy"))
	);
	let mut open = invoke(&webview, "list_windows", json!({}))
		.unwrap()
		.as_array()
		.unwrap()
		.clone();
	open.sort_by_key(|label| label.to_string());
	assert_eq!(open, [json!("main"), json!("settings-privacy")]);

	assert_eq!(
		invoke(
			&webview,
			"resize_overlay_window",
			json!({ "label": "missing", "width": 1.0, "height": 1.0 })
		),
		Err(json!("Overlay window not found"))
	);
	assert_eq!(
		invoke(
			&webview,
			"position_context_menu",
			json!({ "label": "missing", "x": 0.0, "y": 0.0, "menuWidth": 1.0, "menuHeight": 1.0 })
		),
		Err(json!("Context menu window not found"))
	);
	assert_eq!(
		invoke(
			&webview,
			"close_window",
			json!({ "label": "settings-privacy" })
		),
		Ok(json!(null))
	);
	assert_eq!(
		invoke(&webview, "toggle_voice_overlay", json!({})),
		Ok(json!(null))
	);
	// Every window calls this once ready; only macOS restyles the titlebar.
	assert_eq!(
		invoke(&webview, "apply_macos_styling", json!({})),
		Ok(json!(null))
	);
}
