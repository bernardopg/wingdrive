use super::*;

#[tokio::test]
async fn small_files_match_the_in_memory_identity_at_the_sampling_boundary() {
	let root = tempfile::tempdir().unwrap();
	let path = root.path().join("content.bin");
	for size in [1, 8192, MINIMUM_FILE_SIZE as usize] {
		let bytes: Vec<u8> = (0..size).map(|i| (i % 251) as u8).collect();
		tokio::fs::write(&path, &bytes).await.unwrap();
		assert_eq!(
			ContentHashGenerator::generate_content_hash(&path)
				.await
				.unwrap(),
			ContentHashGenerator::generate_from_content(&bytes),
			"size {size}"
		);
	}
}

#[tokio::test]
async fn empty_files_have_no_content_identity_but_have_an_integrity_digest() {
	let root = tempfile::tempdir().unwrap();
	let path = root.path().join("empty");
	tokio::fs::write(&path, []).await.unwrap();
	assert!(matches!(
		ContentHashGenerator::generate_content_hash(&path).await,
		Err(ContentHashError::EmptyFile)
	));
	assert_eq!(
		ContentHashGenerator::generate_integrity_hash(&path)
			.await
			.unwrap(),
		blake3::hash(&[]).to_hex().to_string()
	);
}

#[tokio::test]
async fn missing_files_propagate_io_errors_for_both_hash_tiers_and_verification() {
	let root = tempfile::tempdir().unwrap();
	let path = root.path().join("missing");
	for result in [
		ContentHashGenerator::generate_content_hash(&path).await,
		ContentHashGenerator::generate_integrity_hash(&path).await,
	] {
		assert!(matches!(result, Err(ContentHashError::Io(_))));
	}
	assert!(matches!(
		ContentHashGenerator::verify_content_hash(&path, "missing").await,
		Err(ContentHashError::Io(_))
	));
}

#[tokio::test]
async fn verification_detects_changed_bytes_and_renaming_preserves_identity() {
	let root = tempfile::tempdir().unwrap();
	let path = root.path().join("original");
	let renamed = root.path().join("renamed");
	tokio::fs::write(&path, b"original bytes").await.unwrap();
	let hash = ContentHashGenerator::generate_content_hash(&path)
		.await
		.unwrap();
	assert!(ContentHashGenerator::verify_content_hash(&path, &hash)
		.await
		.unwrap());
	tokio::fs::rename(&path, &renamed).await.unwrap();
	assert!(ContentHashGenerator::verify_content_hash(&renamed, &hash)
		.await
		.unwrap());
	tokio::fs::write(&renamed, b"modified bytes").await.unwrap();
	assert!(!ContentHashGenerator::verify_content_hash(&renamed, &hash)
		.await
		.unwrap());
}

#[tokio::test]
async fn sampled_identity_is_stable_above_the_boundary_and_sensitive_to_size() {
	let root = tempfile::tempdir().unwrap();
	let path = root.path().join("large");
	let mut bytes = vec![42; MINIMUM_FILE_SIZE as usize + 1];
	tokio::fs::write(&path, &bytes).await.unwrap();
	let hash = ContentHashGenerator::generate_content_hash(&path)
		.await
		.unwrap();
	assert_eq!(hash.len(), 16);
	assert_eq!(
		ContentHashGenerator::generate_content_hash(&path)
			.await
			.unwrap(),
		hash
	);
	bytes.push(42);
	tokio::fs::write(&path, &bytes).await.unwrap();
	assert_ne!(
		ContentHashGenerator::generate_content_hash(&path)
			.await
			.unwrap(),
		hash
	);
}

#[tokio::test]
async fn sampled_identity_detects_changes_in_headers_samples_and_footers() {
	let root = tempfile::tempdir().unwrap();
	let path = root.path().join("sampled");
	let bytes = vec![42; 1024 * 1024];
	tokio::fs::write(&path, &bytes).await.unwrap();
	let original = ContentHashGenerator::generate_content_hash(&path)
		.await
		.unwrap();
	for offset in [0, HEADER_OR_FOOTER_SIZE as usize, bytes.len() - 1] {
		let mut changed = bytes.clone();
		changed[offset] = 43;
		tokio::fs::write(&path, changed).await.unwrap();
		assert_ne!(
			ContentHashGenerator::generate_content_hash(&path)
				.await
				.unwrap(),
			original,
			"offset {offset}"
		);
	}
}

#[tokio::test]
async fn integrity_hash_distinguishes_changes_outside_the_sampled_regions() {
	let root = tempfile::tempdir().unwrap();
	let path = root.path().join("unsampled");
	let mut bytes = vec![42; 1024 * 1024];
	tokio::fs::write(&path, &bytes).await.unwrap();
	let sampled = ContentHashGenerator::generate_content_hash(&path)
		.await
		.unwrap();
	let integrity = ContentHashGenerator::generate_integrity_hash(&path)
		.await
		.unwrap();
	assert_eq!(integrity, blake3::hash(&bytes).to_hex().to_string());
	bytes[40_000] = 43;
	tokio::fs::write(&path, &bytes).await.unwrap();
	assert_eq!(
		ContentHashGenerator::generate_content_hash(&path)
			.await
			.unwrap(),
		sampled
	);
	assert_ne!(
		ContentHashGenerator::generate_integrity_hash(&path)
			.await
			.unwrap(),
		integrity
	);
}

#[tokio::test]
async fn integrity_hash_reads_past_its_streaming_buffer() {
	let root = tempfile::tempdir().unwrap();
	let path = root.path().join("streamed");
	let bytes: Vec<u8> = (0..2 * 1024 * 1024 + 17).map(|i| (i % 251) as u8).collect();
	tokio::fs::write(&path, &bytes).await.unwrap();
	assert_eq!(
		ContentHashGenerator::generate_integrity_hash(&path)
			.await
			.unwrap(),
		blake3::hash(&bytes).to_hex().to_string()
	);
}
