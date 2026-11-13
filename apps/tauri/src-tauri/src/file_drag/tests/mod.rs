//! The hand-off from the drag command to GTK, with a recorder in place of GTK.

use std::sync::{Arc, Mutex};

use tauri::{WebviewUrl, WebviewWindowBuilder};

use super::{drag_on_main_thread, start_file_drag};

type Calls = Arc<Mutex<Vec<(String, Vec<String>, &'static str)>>>;

fn window() -> (
	tauri::App<tauri::test::MockRuntime>,
	tauri::WebviewWindow<tauri::test::MockRuntime>,
) {
	let app = tauri::test::mock_app();
	let window = WebviewWindowBuilder::new(&app, "main", WebviewUrl::default())
		.build()
		.unwrap();
	(app, window)
}

async fn drag(
	window: &tauri::WebviewWindow<tauri::test::MockRuntime>,
	paths: &[String],
	result: Result<(), String>,
) -> (Result<(), String>, Calls) {
	let calls: Calls = Arc::default();
	let sink = calls.clone();
	let outcome = drag_on_main_thread(window.clone(), paths.to_vec(), move |target, uris, icon| {
		sink.lock()
			.unwrap()
			.push((target.label().to_string(), uris, icon));
		result
	})
	.await;
	(outcome, calls)
}

#[tokio::test]
async fn an_empty_selection_is_refused_before_reaching_gtk() {
	let (_app, window) = window();
	assert_eq!(
		start_file_drag(window, vec![]).await.unwrap_err(),
		"Nothing to drag"
	);
}

#[tokio::test]
async fn gtk_receives_encoded_uris_from_the_source_window() {
	let dir = tempfile::tempdir().unwrap();
	let file = dir.path().join("a b#1.txt");
	std::fs::write(&file, "x").unwrap();
	let file = file.to_string_lossy().into_owned();
	let (_app, window) = window();

	let (outcome, calls) = drag(&window, std::slice::from_ref(&file), Ok(())).await;
	outcome.unwrap();
	let calls = calls.lock().unwrap();
	let (label, uris, icon) = &calls[0];
	assert_eq!(label, "main");
	assert_eq!(*icon, "text-x-generic");
	assert_eq!(uris.len(), 1);
	assert!(uris[0].starts_with("file:///"), "{}", uris[0]);
	assert!(uris[0].ends_with("/a%20b%231.txt"), "{}", uris[0]);
	assert_eq!(
		tauri::Url::parse(&uris[0]).unwrap().to_file_path().unwrap(),
		std::path::PathBuf::from(file)
	);
}

/// Only a single folder shows the folder icon; mixed or multiple items, and
/// paths that vanished, show a generic file.
#[tokio::test]
async fn the_drag_icon_follows_the_selection() {
	let dir = tempfile::tempdir().unwrap();
	let folder = dir.path().to_string_lossy().into_owned();
	let file = dir.path().join("f.txt");
	std::fs::write(&file, "x").unwrap();
	let file = file.to_string_lossy().into_owned();
	let missing = dir.path().join("gone").to_string_lossy().into_owned();
	let (_app, window) = window();

	for (paths, expected) in [
		(vec![folder.clone()], "folder"),
		(vec![folder.clone(), folder.clone()], "text-x-generic"),
		(vec![folder, file], "text-x-generic"),
		(vec![missing], "text-x-generic"),
	] {
		let (outcome, calls) = drag(&window, &paths, Ok(())).await;
		outcome.unwrap();
		assert_eq!(calls.lock().unwrap()[0].2, expected, "{paths:?}");
		assert_eq!(calls.lock().unwrap()[0].1.len(), paths.len());
	}
}

#[tokio::test]
async fn relative_paths_and_gtk_failures_return_errors() {
	let (_app, window) = window();
	let (outcome, calls) = drag(&window, &["/tmp/ok".into(), "relative.txt".into()], Ok(())).await;
	assert_eq!(
		outcome.unwrap_err(),
		"Clipboard path must be absolute: relative.txt"
	);
	assert!(calls.lock().unwrap().is_empty());

	let (outcome, calls) = drag(
		&window,
		&["/tmp/ok".into()],
		Err("GTK refused to start the drag".into()),
	)
	.await;
	assert_eq!(outcome.unwrap_err(), "GTK refused to start the drag");
	assert_eq!(calls.lock().unwrap().len(), 1);
}
