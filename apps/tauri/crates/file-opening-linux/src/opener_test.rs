use std::path::Path;
use std::process::Command;

use file_opening::{FileOpener, OpenResult};

use super::LinuxFileOpener;

#[test]
fn opener_uses_isolated_desktop_associations_and_handles_missing_inputs() {
	const CHILD_ENV: &str = "WINGDRIVE_TEST_FILE_OPENER_CHILD";
	if let Some(root) = std::env::var_os(CHILD_ENV) {
		let root = Path::new(&root);
		let file = root.join("sample.txt");
		let missing = root.join("missing.txt");
		let opener = LinuxFileOpener;
		let apps = opener.get_apps_for_file(&file).unwrap();
		assert_eq!(apps.len(), 1);
		assert_eq!(apps[0].id, "test-editor.desktop");
		assert_eq!(apps[0].name, "Test Editor");
		assert!(apps[0].is_default);
		assert!(matches!(
			opener.open_with_default(&missing).unwrap(),
			OpenResult::FileNotFound { .. }
		));
		assert!(matches!(
			opener.open_with_app(&file, "missing.desktop").unwrap(),
			OpenResult::AppNotFound { .. }
		));
		assert!(matches!(
			opener.open_with_app(&file, "test-editor.desktop").unwrap(),
			OpenResult::Success
		));
		opener
			.set_default_app(&file, "test-editor.desktop")
			.unwrap();
		assert_eq!(
			std::fs::read_to_string(root.join("xdg-args")).unwrap(),
			"default\ntest-editor.desktop\ntext/plain\n"
		);
		assert_eq!(
			opener.set_default_app(&file, "fail.desktop").unwrap_err(),
			"fixture failure"
		);
		return;
	}

	let dir = tempfile::tempdir().unwrap();
	let root = dir.path();
	let config = root.join("config");
	let data = root.join("data");
	let system = root.join("system");
	std::fs::create_dir_all(data.join("mime")).unwrap();
	std::fs::create_dir_all(data.join("applications")).unwrap();
	std::fs::create_dir_all(&config).unwrap();
	std::fs::create_dir_all(&system).unwrap();
	std::fs::create_dir_all(root.join("bin")).unwrap();
	let xdg_mime = root.join("bin/xdg-mime");
	std::fs::write(&xdg_mime, "#!/bin/sh\nif [ \"$2\" = fail.desktop ]; then echo 'fixture failure' >&2; exit 1; fi\nprintf '%s\\n' \"$@\" > \"$WINGDRIVE_TEST_FILE_OPENER_CHILD/xdg-args\"\n").unwrap();
	use std::os::unix::fs::PermissionsExt;
	std::fs::set_permissions(&xdg_mime, std::fs::Permissions::from_mode(0o755)).unwrap();
	std::fs::write(root.join("sample.txt"), "fixture").unwrap();
	std::fs::write(data.join("mime/globs2"), "50:text/plain:*.txt\n").unwrap();
	std::fs::write(data.join("applications/test-editor.desktop"), "[Desktop Entry]\nType=Application\nName=Test Editor\nExec=/bin/true %f\nMimeType=text/plain;\n").unwrap();
	std::fs::write(
		config.join("mimeapps.list"),
		"[Default Applications]\ntext/plain=test-editor.desktop;\n",
	)
	.unwrap();
	let output = Command::new(std::env::current_exe().unwrap())
		.arg("--exact")
		.arg("opener_test::opener_uses_isolated_desktop_associations_and_handles_missing_inputs")
		.arg("--nocapture")
		.env(CHILD_ENV, root)
		.env("HOME", root)
		.env("XDG_CONFIG_HOME", config)
		.env("XDG_DATA_HOME", data)
		.env("XDG_CONFIG_DIRS", &system)
		.env("XDG_DATA_DIRS", &system)
		.env("XDG_CURRENT_DESKTOP", "")
		.env("PATH", root.join("bin"))
		.env_remove("APPDIR")
		.output()
		.unwrap();
	assert!(
		output.status.success(),
		"child failed: {}",
		String::from_utf8_lossy(&output.stderr)
	);
}
