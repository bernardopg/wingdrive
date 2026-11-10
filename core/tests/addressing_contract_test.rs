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

fn sidecar_round_trip_cases() -> Vec<WingPath> {
	use wing_core::ops::sidecar::types::{SidecarFormat, SidecarKind};

	let id = uuid::Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
	let mut paths = Vec::new();
	for kind in [
		SidecarKind::Thumb,
		SidecarKind::Thumbstrip,
		SidecarKind::Proxy,
		SidecarKind::Embeddings,
		SidecarKind::Ocr,
		SidecarKind::Transcript,
		SidecarKind::GaussianSplat,
	] {
		for format in [
			SidecarFormat::Webp,
			SidecarFormat::Mp4,
			SidecarFormat::Json,
			SidecarFormat::MessagePack,
			SidecarFormat::Text,
			SidecarFormat::Ply,
		] {
			paths.push(WingPath::sidecar(id, kind.clone(), "prévia.v2@2x", format));
		}
	}
	paths
}

#[test]
fn every_sidecar_kind_and_format_round_trips_through_uri() {
	for path in sidecar_round_trip_cases() {
		let uri = path.to_uri();
		assert_eq!(WingPath::from_uri(&uri).unwrap(), path, "{uri}");
	}
}

#[test]
fn every_sidecar_kind_and_format_round_trips_through_json() {
	for path in sidecar_round_trip_cases() {
		let json = serde_json::to_value(&path).unwrap();
		assert_eq!(serde_json::from_value::<WingPath>(json).unwrap(), path);
	}
}

#[test]
fn sidecar_parser_rejects_extra_segments_and_invalid_extensions() {
	let prefix = "sidecar://550e8400-e29b-41d4-a716-446655440000";
	for (suffix, error) in [
		("", WingPathParseError::InvalidSidecarPath),
		("/thumbs", WingPathParseError::InvalidSidecarPath),
		("/thumbs/a/b.webp", WingPathParseError::InvalidSidecarPath),
		("/thumbs/", WingPathParseError::MissingExtension),
		("/thumbs/grid.", WingPathParseError::InvalidSidecarFormat),
		(
			"/thumbs/grid.WEBP",
			WingPathParseError::InvalidSidecarFormat,
		),
	] {
		assert_eq!(WingPath::from_uri(&format!("{prefix}{suffix}")), Err(error));
	}
}

#[test]
fn malformed_json_variants_are_errors_instead_of_panics() {
	use serde_json::json;

	for value in [
		json!(null),
		json!("local://peer/file"),
		json!({"Physical": {"path": "/file"}}),
		json!({"Physical": {"device_slug": 1, "path": "/file"}}),
		json!({"Cloud": {"service": "s3", "identifier": "bucket"}}),
		json!({"Sidecar": {"content_id": "bad", "kind": "thumb", "variant": "grid", "format": "webp"}}),
		json!({"Sidecar": {"content_id": uuid::Uuid::nil(), "kind": "unknown", "variant": "grid", "format": "webp"}}),
		json!({"Sidecar": {"content_id": uuid::Uuid::nil(), "kind": "thumb", "variant": "grid", "format": "unknown"}}),
		json!({"Unknown": {}}),
	] {
		assert!(
			serde_json::from_value::<WingPath>(value.clone()).is_err(),
			"{value}"
		);
	}
}

#[test]
fn mixed_batch_json_keeps_order_duplicates_and_local_filtering() {
	let local = WingPath::physical("local".into(), "café/文件.txt");
	let remote = WingPath::physical("batch-contract-peer".into(), "café/文件.txt");
	let content = WingPath::content(uuid::Uuid::nil());
	let cloud = WingPath::from_uri("s3://bucket/a%20b#c?d").unwrap();
	let batch = WingPathBatch::new(vec![local.clone(), remote, content, cloud, local.clone()]);
	let restored: WingPathBatch =
		serde_json::from_value(serde_json::to_value(&batch).unwrap()).unwrap();
	assert_eq!(restored, batch);
	assert_eq!(
		restored.local_only(),
		vec![local.path().unwrap().as_path(); 2]
	);
	assert_eq!(restored.by_device()["local"].len(), 2);
	assert_eq!(restored.by_device()["batch-contract-peer"].len(), 1);
}

#[test]
fn retargeting_physical_addresses_preserves_path_and_nonphysical_identity() {
	let device = uuid::Uuid::from_u128(42);
	let original = WingPath::physical("old-peer".into(), "folder/../café.txt");
	let retargeted = original.addressed_to(device);
	assert_eq!(retargeted.device_slug(), Some(device.to_string().as_str()));
	assert_eq!(retargeted.path(), original.path());
	for path in [
		WingPath::content(uuid::Uuid::nil()),
		WingPath::from_uri("s3://bucket/key").unwrap(),
		WingPath::from_uri("sidecar://00000000-0000-0000-0000-000000000000/ocr/default.json")
			.unwrap(),
	] {
		assert_eq!(path.addressed_to(device), path);
		assert!(!path.is_cross_device(&original));
		assert!(!original.is_cross_device(&path));
	}
}

#[cfg(unix)]
#[test]
fn physical_join_and_parent_preserve_non_utf8_bytes_without_following_links() {
	use std::os::unix::{ffi::OsStrExt, fs::symlink};

	let dir = tempfile::tempdir().unwrap();
	let link = dir.path().join("link");
	symlink("missing", &link).unwrap();
	let name = std::ffi::OsStr::from_bytes(b"caf\xff.txt");
	let base = WingPath::physical("local".into(), &link);
	let child = base.join(name);
	assert_eq!(child.path().unwrap(), &link.join(name));
	assert_eq!(child.parent(), Some(base));
	assert!(child.file_name().is_none());
	assert_eq!(
		std::fs::read_link(link).unwrap(),
		std::path::Path::new("missing")
	);
}

#[test]
fn every_cloud_service_preserves_keys_through_uri_and_json() {
	use wing_core::volume::backend::CloudServiceType;

	for service in [
		CloudServiceType::S3,
		CloudServiceType::GoogleDrive,
		CloudServiceType::Dropbox,
		CloudServiceType::OneDrive,
		CloudServiceType::GoogleCloudStorage,
		CloudServiceType::AzureBlob,
		CloudServiceType::BackblazeB2,
		CloudServiceType::Wasabi,
		CloudServiceType::DigitalOceanSpaces,
		CloudServiceType::Other,
	] {
		let path = WingPath::cloud(service, "photos".into(), "café/文件%20#?.jpg");
		assert_eq!(WingPath::from_uri(&path.to_uri()).unwrap(), path);
		assert_eq!(
			serde_json::from_value::<WingPath>(serde_json::to_value(&path).unwrap()).unwrap(),
			path
		);
		let root = WingPath::cloud(service, "photos".into(), "");
		assert_eq!(WingPath::from_uri(&root.to_uri()).unwrap(), root);
		assert!(root.parent().is_none());
		assert_eq!(path.parent().unwrap().cloud_path(), Some("café"));
	}
}
