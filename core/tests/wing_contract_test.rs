use wing_core::domain::Wing;

#[test]
fn wing_defaults_and_color_shape() {
	let wing = Wing::create_default();
	assert_eq!(wing.name, "All Devices");
	assert_eq!(wing.order, 0);
	assert!(Wing::validate_color(&wing.color));
	assert!(Wing::validate_color("#012aEF"));
	for invalid in ["", "012aEF", "#123", "#1234567"] {
		assert!(!Wing::validate_color(invalid), "{invalid}");
	}
}

#[test]
fn wing_color_rejects_non_hex_characters() {
	assert!(!Wing::validate_color("#zzzzzz"));
	assert!(!Wing::validate_color("#ééé"));
}
