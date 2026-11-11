use super::*;

#[test]
fn mount_listing_does_not_take_attributes_from_a_neighboring_drive() {
	let listing = "Volume(0): Incomplete\n  activation_root=smb://nas/incomplete\nDrive(1): Neighbor\n  can_mount=1\nVolume(1): Ready\n  can_mount=1\n  activation_root=sftp://nas/ready\n";
	assert_eq!(
		parse_mountable(listing),
		[MountableVolume {
			name: "Ready".into(),
			uri: "sftp://nas/ready".into()
		}]
	);
}

#[test]
fn fuse_listing_sorts_labels_without_changing_mount_paths() {
	let root = tempfile::tempdir().unwrap();
	for folder in ["sftp:host=Zebra", "smb-share:host=nas,share=alpha%20Docs"] {
		std::fs::create_dir(root.path().join(folder)).unwrap();
	}
	let mounts = list_in(root.path());
	assert_eq!(
		mounts[0].path,
		root.path()
			.join("smb-share:host=nas,share=alpha%20Docs")
			.to_string_lossy()
	);
	assert_eq!(
		mounts[1].path,
		root.path().join("sftp:host=Zebra").to_string_lossy()
	);
	assert_eq!(
		mounts.iter().map(|m| m.name.as_str()).collect::<Vec<_>>(),
		["alpha Docs on nas (SMB)", "Zebra (SFTP)"]
	);
	for mount in mounts {
		assert_eq!(
			std::path::Path::new(&mount.path).parent(),
			Some(root.path())
		);
		assert!(std::path::Path::new(&mount.path).is_dir());
	}
}

#[test]
fn mount_labels_decode_complete_utf8_and_keep_invalid_escapes() {
	assert_eq!(
		describe("sftp:host=nas,user=Jos%C3%A9").name,
		"José on nas (SFTP)"
	);
	for (input, expected) in [
		("user%", "user%"),
		("user%A", "user%A"),
		("user%+A", "user%+A"),
		("user%XY", "user%XY"),
		("user%40host", "user@host"),
	] {
		assert_eq!(percent_decode(input), expected, "{input}");
	}
}

#[test]
fn mount_listing_selects_unmounted_volumes_at_multiple_drive_levels() {
	let listing = "\
Drive(0): Drive A
  Volume(0): Empty URI
    can_mount=1
  Volume(1): Already mounted
    activation_root=smb://server/mounted
    can_mount=1
    Mount(0): Already mounted
  Volume(2): Shared documents
    activation_root=smb://server/docs
    can_mount=1
Drive(1): Drive B
  Volume(0): Phone
    activation_root=mtp://phone/
    can_mount=1
  Volume(1): Disabled
    activation_root=sftp://disabled/
    can_mount=0
";
	assert_eq!(
		parse_mountable(listing),
		vec![
			MountableVolume {
				name: "Shared documents".into(),
				uri: "smb://server/docs".into(),
			},
			MountableVolume {
				name: "Phone".into(),
				uri: "mtp://phone/".into(),
			},
		]
	);
}

#[tokio::test]
async fn mount_and_unmount_reject_input_before_running_gio() {
	let error = mount_network_location("  server/share ".into(), None)
		.await
		.unwrap_err();
	assert!(error.starts_with("Enter an address"), "{error}");
	for path in ["/tmp/gvfs/sftp:host=x", "/", "relative"] {
		assert_eq!(
			unmount_network_location(path.into()).await.unwrap_err(),
			"Not a network location"
		);
	}
}

/// `sh -c script` standing in for `gio mount`; `$0` is the URI.
fn mount_script(script: &str) -> std::process::Command {
	let mut command = std::process::Command::new("sh");
	command.args(["-c", script, "sftp://server/"]);
	command
}

#[test]
fn mount_reports_gio_results_and_answers_the_password_prompt() {
	use std::time::Duration;

	use super::run_mount_command;

	let run = |script: &str, password: Option<&str>| {
		run_mount_command(
			mount_script(script),
			"sftp://server/",
			password,
			Duration::from_secs(10),
		)
	};
	run("exit 0", None).unwrap();
	// The password is the only line gio reads from stdin.
	run("read line; [ \"$line\" = secret ]", Some("secret")).unwrap();
	assert!(run("read line; [ \"$line\" = secret ]", Some("wrong")).is_err());
	// A location that is already mounted is what the caller wanted.
	run(
		"echo 'gio: sftp://server/: Location is already mounted' >&2; exit 2",
		None,
	)
	.unwrap();
	assert_eq!(
		run(
			"echo 'first' >&2; echo 'gio: sftp://server/: Permission denied' >&2; exit 2",
			None
		)
		.unwrap_err(),
		"sftp://server/: Permission denied"
	);
	assert_eq!(
		run("exit 1", None).unwrap_err(),
		"Could not mount sftp://server/"
	);
}

#[test]
fn mount_gives_up_on_a_prompt_that_never_ends() {
	use std::time::Duration;

	let error = super::run_mount_command(
		mount_script("sleep 30"),
		"sftp://server/",
		None,
		Duration::from_millis(200),
	)
	.unwrap_err();
	assert_eq!(error, "Timed out connecting to sftp://server/");
	let error = super::run_mount_command(
		std::process::Command::new("/nonexistent/gio"),
		"sftp://server/",
		None,
		Duration::from_secs(1),
	)
	.unwrap_err();
	assert!(error.starts_with("gio is not available"), "{error}");
}
