//! Settings changed while the core runs must survive its shutdown.

use tempfile::TempDir;
use wing_core::{config::AppConfig, Core};

#[tokio::test]
async fn settings_written_while_running_survive_shutdown() {
	let temp_dir = TempDir::new().unwrap();
	let data_dir = temp_dir.path().to_path_buf();
	let core = Core::new(data_dir.clone()).await.unwrap();

	// What the config.app.update action does: load the file, change, save.
	let mut config = AppConfig::load_from(&data_dir).unwrap();
	config.preferences.theme = "t3-chat".to_string();
	config.wingdrop.auto_accept = true;
	config.save().unwrap();

	core.shutdown().await.unwrap();

	let reloaded = AppConfig::load_from(&data_dir).unwrap();
	assert_eq!(reloaded.preferences.theme, "t3-chat");
	assert!(reloaded.wingdrop.auto_accept);
}
