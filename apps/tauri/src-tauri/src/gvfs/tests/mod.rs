use super::*;

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
