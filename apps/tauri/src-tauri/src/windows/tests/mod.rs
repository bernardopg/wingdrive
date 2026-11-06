use std::collections::HashSet;

use serde_json::json;

use super::{explorer_url, hash_string, percent_encode, WingDriveWindow};

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
