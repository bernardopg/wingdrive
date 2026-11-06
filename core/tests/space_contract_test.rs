use wing_core::domain::Space;

#[test]
fn space_defaults_and_color_shape() {
	let space = Space::create_default();
	assert_eq!(space.name, "All Devices");
	assert_eq!(space.order, 0);
	assert!(Space::validate_color(&space.color));
	assert!(Space::validate_color("#012aEF"));
	for invalid in ["", "012aEF", "#123", "#1234567"] {
		assert!(!Space::validate_color(invalid), "{invalid}");
	}
}

#[test]
fn space_color_rejects_non_hex_characters() {
	assert!(!Space::validate_color("#zzzzzz"));
	assert!(!Space::validate_color("#ééé"));
}
