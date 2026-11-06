use super::{canonical_sidecar_uuid, find_library_folder, get_sidecar_path};

#[tokio::test]
async fn invalid_sidecar_uuid_returns_error_before_library_lookup() {
	for value in ["", "a", "éé", "not-a-uuid"] {
		let result = get_sidecar_path(
			"unused".into(),
			value.into(),
			"thumb".into(),
			"grid".into(),
			"webp".into(),
		)
		.await;
		assert_eq!(result.unwrap_err(), "Invalid content UUID", "{value}");
	}
	assert_eq!(
		canonical_sidecar_uuid("550E8400E29B41D4A716446655440000").unwrap(),
		"550e8400-e29b-41d4-a716-446655440000"
	);
}

#[tokio::test]
async fn sidecar_segments_that_could_leave_the_content_folder_return_an_error() {
	let uuid = "550e8400-e29b-41d4-a716-446655440000";
	for (kind, variant, format) in [
		("thumb", "..", "webp"),
		("thumb", "../../etc/passwd", "webp"),
		("thumb", "grid", "../webp"),
		("../thumb", "grid", "webp"),
		("thumb/x", "grid", "webp"),
		("", "grid", "webp"),
	] {
		let result = get_sidecar_path(
			"unused".into(),
			uuid.into(),
			kind.into(),
			variant.into(),
			format.into(),
		)
		.await;
		assert_eq!(
			result.unwrap_err(),
			"Invalid sidecar path segment",
			"{kind}/{variant}.{format}"
		);
	}
}

#[tokio::test]
async fn finds_library_by_json_id_in_current_and_legacy_folders() {
	let dir = tempfile::tempdir().unwrap();
	let libraries = dir.path().join("libraries");
	tokio::fs::create_dir(&libraries).await.unwrap();
	for (folder, contents) in [
		("broken.winglibrary", "{invalid"),
		("current.winglibrary", r#"{"id":"current-id"}"#),
		("legacy.sdlibrary", r#"{"id":"legacy-id"}"#),
		("ignored.txt", r#"{"id":"ignored-id"}"#),
	] {
		let path = libraries.join(folder);
		tokio::fs::create_dir(&path).await.unwrap();
		tokio::fs::write(path.join("library.json"), contents)
			.await
			.unwrap();
	}
	for (id, folder) in [
		("current-id", "current.winglibrary"),
		("legacy-id", "legacy.sdlibrary"),
	] {
		assert_eq!(
			find_library_folder(dir.path(), id).await.unwrap(),
			libraries.join(folder)
		);
	}
	assert!(find_library_folder(dir.path(), "ignored-id").await.is_err());
	assert!(find_library_folder(dir.path(), "missing-id").await.is_err());
}

#[tokio::test]
async fn missing_libraries_directory_returns_an_error() {
	let dir = tempfile::tempdir().unwrap();
	let error = find_library_folder(dir.path(), "missing")
		.await
		.unwrap_err();
	assert!(error.contains("libraries directory"), "{error}");
}
