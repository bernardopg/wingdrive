use tauri::Manager;

use super::{get_registered_keybinds, register_keybind, unregister_keybind, KeybindState};

#[tokio::test]
async fn keybinds_register_replace_and_unregister_by_id() {
	let app = tauri::test::mock_app();
	app.manage(KeybindState::default());

	register_keybind(app.state(), "copy".into(), "Ctrl+C".into())
		.await
		.unwrap();
	register_keybind(app.state(), "paste".into(), "Ctrl+V".into())
		.await
		.unwrap();
	// A second registration for the same id replaces its accelerator.
	register_keybind(app.state(), "copy".into(), "Ctrl+Shift+C".into())
		.await
		.unwrap();

	let state = app.state::<KeybindState>();
	assert_eq!(
		state.get_accelerator("copy").as_deref(),
		Some("Ctrl+Shift+C")
	);
	let all = get_registered_keybinds(app.state()).await.unwrap();
	assert_eq!(all.len(), 2);
	assert_eq!(all["paste"], "Ctrl+V");

	unregister_keybind(app.state(), "copy".into())
		.await
		.unwrap();
	// Unregistering an unknown id is not an error.
	unregister_keybind(app.state(), "missing".into())
		.await
		.unwrap();
	assert_eq!(state.get_accelerator("copy"), None);
	assert_eq!(state.list_all().len(), 1);
}
