use std::path::Path;

use super::super::unit_matches_daemon;

#[test]
fn installed_unit_matches_the_exact_data_path_and_instance() {
	let path = Path::new("/data/backup room");
	let quoted =
		"ExecStart=\"/usr/local/bin/wing-daemon\" --data-dir \"/data/backup room\" --instance test";
	assert!(unit_matches_daemon(quoted, path, Some("test")));
	assert!(!unit_matches_daemon(quoted, path, Some("other")));
	assert!(!unit_matches_daemon(
		quoted,
		Path::new("/data/backup"),
		Some("test")
	));
	assert!(!unit_matches_daemon(quoted, path, None));

	let legacy = "ExecStart=/usr/bin/wing-daemon --data-dir /data/backup-longer";
	assert!(unit_matches_daemon(
		legacy,
		Path::new("/data/backup-longer"),
		None
	));
	assert!(!unit_matches_daemon(
		legacy,
		Path::new("/data/backup"),
		None
	));
	assert!(!unit_matches_daemon(
		"Description=Daemon --data-dir /data/backup",
		Path::new("/data/backup"),
		None,
	));
}
