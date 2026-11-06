use wing_core::domain::addressing::{WingPath, WingPathBatch, WingPathParseError};

#[test]
fn cloud_paths_preserve_identity_across_join_and_parent() {
	let base = WingPath::from_uri("s3://photos/2026").unwrap();
	let file = base.join("trip/image.jpg");
	assert_eq!(file.to_uri(), "s3://photos/2026/trip/image.jpg");
	assert_eq!(file.file_name().as_deref(), Some("image.jpg"));
	assert_eq!(file.parent().unwrap().to_uri(), "s3://photos/2026/trip");
	assert_eq!(file.cloud_path(), Some("2026/trip/image.jpg"));
	assert_eq!(file.cloud_identity().unwrap().1, "photos");
	assert!(file.is_cloud());
	assert!(file.as_physical().is_none());
}

#[test]
fn cloud_join_does_not_duplicate_separators() {
	let base = WingPath::from_uri("s3://photos/2026/").unwrap();
	assert_eq!(
		base.join("/image.jpg").to_uri(),
		"s3://photos/2026/image.jpg"
	);
	let root = WingPath::from_uri("s3://photos").unwrap();
	assert_eq!(root.join("image.jpg").to_uri(), "s3://photos/image.jpg");
	assert_eq!(root.join("/image.jpg").to_uri(), "s3://photos/image.jpg");
	assert_eq!(
		base.join("//image.jpg").to_uri(),
		"s3://photos/2026//image.jpg"
	);
}

#[test]
fn physical_and_cloud_file_names_borrow_their_source() {
	let physical = WingPath::new("device".into(), "/home/user/file.txt");
	assert!(matches!(
		physical.file_name(),
		Some(std::borrow::Cow::Borrowed("file.txt"))
	));
	let cloud = WingPath::from_uri("s3://photos/folder/image.jpg").unwrap();
	assert!(matches!(
		cloud.file_name(),
		Some(std::borrow::Cow::Borrowed("image.jpg"))
	));
}

#[test]
fn sidecar_uri_round_trips_and_exposes_its_parts() {
	let uri = "sidecar://550e8400-e29b-41d4-a716-446655440000/thumbs/grid@2x.webp";
	let path = WingPath::from_uri(uri).unwrap();
	assert_eq!(path.to_uri(), uri);
	assert!(path.is_sidecar());
	assert!(!path.is_content());
	assert!(
		matches!(path.file_name(), Some(std::borrow::Cow::Owned(ref name)) if name == "grid@2x.webp")
	);
	assert!(path.parent().is_none());
	let (_, _, variant, _) = path.as_sidecar().unwrap();
	assert_eq!(variant.as_str(), "grid@2x");
}

#[test]
fn parser_rejects_invalid_content_and_sidecar_uris() {
	let cases = [
		("content://invalid", WingPathParseError::InvalidContentId),
		("sidecar://invalid", WingPathParseError::InvalidSidecarPath),
		(
			"sidecar://bad/thumbs/grid.webp",
			WingPathParseError::InvalidContentId,
		),
		(
			"sidecar://550e8400-e29b-41d4-a716-446655440000/unknown/grid.webp",
			WingPathParseError::InvalidSidecarKind,
		),
		(
			"sidecar://550e8400-e29b-41d4-a716-446655440000/thumbs/grid",
			WingPathParseError::MissingExtension,
		),
		(
			"sidecar://550e8400-e29b-41d4-a716-446655440000/thumbs/grid.invalid",
			WingPathParseError::InvalidSidecarFormat,
		),
		("unknown://bucket/key", WingPathParseError::UnknownScheme),
	];
	for (uri, expected) in cases {
		assert_eq!(WingPath::from_uri(uri).unwrap_err(), expected, "{uri}");
	}
}

#[test]
fn batch_groups_physical_paths_without_including_cloud_or_content() {
	let mut batch = WingPathBatch::new(vec![
		WingPath::new("peer-a".into(), "/one"),
		WingPath::new("peer-a".into(), "/two"),
		WingPath::new("peer-b".into(), "/three"),
	]);
	batch.extend(vec![WingPath::from_uri("s3://bucket/key").unwrap()]);
	let groups = batch.by_device();
	assert_eq!(groups.len(), 2);
	assert_eq!(groups["peer-a"].len(), 2);
	assert_eq!(groups["peer-b"].len(), 1);
}

#[test]
fn deserialization_checks_cloud_service_and_content_id() {
	let cloud: WingPath =
		serde_json::from_str(r#"{"Cloud":{"service":"s3","identifier":"bucket","path":"a/b"}}"#)
			.unwrap();
	assert_eq!(cloud.to_uri(), "s3://bucket/a/b");
	let physical: WingPath =
		serde_json::from_str(r#"{"Physical":{"device_slug":"peer","path":"/a"}}"#).unwrap();
	assert_eq!(physical.device_slug(), Some("peer"));
	assert!(serde_json::from_str::<WingPath>(
		r#"{"Cloud":{"service":"unknown","identifier":"bucket","path":"a"}}"#
	)
	.is_err());
	assert!(serde_json::from_str::<WingPath>(r#"{"Content":{"content_id":"bad"}}"#).is_err());
}
