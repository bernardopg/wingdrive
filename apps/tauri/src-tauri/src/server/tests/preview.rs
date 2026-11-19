use super::*;

#[tokio::test]
async fn file_preview_serves_full_and_partial_media() {
	let directory = tempfile::tempdir().unwrap();
	let source = directory.path().join("voice sample.MP3");
	tokio::fs::write(&source, b"0123456789").await.unwrap();
	let (url, token, shutdown) = start_server(directory.path().to_path_buf()).await.unwrap();
	let target = file_target(&source, &token);

	for (range, status, content_range, body) in [
		("", "200 OK", None, &b"0123456789"[..]),
		(
			"bytes=2-4",
			"206 Partial Content",
			Some("bytes 2-4/10"),
			&b"234"[..],
		),
		(
			"bytes=7-",
			"206 Partial Content",
			Some("bytes 7-9/10"),
			&b"789"[..],
		),
		(
			"bytes=-3",
			"206 Partial Content",
			Some("bytes 7-9/10"),
			&b"789"[..],
		),
		(
			"bytes=8-99",
			"206 Partial Content",
			Some("bytes 8-9/10"),
			&b"89"[..],
		),
		(
			"bytes=10-",
			"416 Range Not Satisfiable",
			Some("bytes */10"),
			&b""[..],
		),
		("bytes=9-1", "200 OK", None, &b"0123456789"[..]),
		("bytes=0-1,3-4", "200 OK", None, &b"0123456789"[..]),
	] {
		let headers = format!("Range: {range}\r\n");
		let response = get_with_headers(&url, &target, &headers).await;
		let (head, result) = split_response(&response);
		assert!(head.starts_with(&format!("HTTP/1.1 {status}")), "{range}");
		assert!(head.contains("access-control-allow-origin: *"));
		if status != "416 Range Not Satisfiable" {
			assert!(head.contains("content-type: audio/mpeg"));
			assert!(head.contains("accept-ranges: bytes"));
		}
		match content_range {
			Some(value) => assert!(head.contains(&format!("content-range: {value}"))),
			None => assert!(!head.contains("content-range:")),
		}
		assert_eq!(result, body, "{range}");
	}
	shutdown.send(()).await.unwrap();
}

#[tokio::test]
async fn file_preview_rejects_non_files_and_handles_empty_files() {
	let directory = tempfile::tempdir().unwrap();
	let empty = directory.path().join("empty.wav");
	tokio::fs::write(&empty, []).await.unwrap();
	let (url, token, shutdown) = start_server(directory.path().to_path_buf()).await.unwrap();

	for (path, status) in [
		(std::path::Path::new("relative.wav"), "400"),
		(directory.path(), "400"),
		(directory.path().join("missing.wav").as_path(), "404"),
	] {
		let response = get(&url, &file_target(path, &token)).await;
		assert!(response.starts_with(format!("HTTP/1.1 {status}").as_bytes()));
	}

	let target = file_target(&empty, &token);
	let response = get(&url, &target).await;
	let (head, body) = split_response(&response);
	assert!(head.starts_with("HTTP/1.1 200"));
	assert!(head.contains("content-length: 0"));
	assert!(body.is_empty());

	let response = get_with_headers(&url, &target, "Range: bytes=0-\r\n").await;
	let (head, body) = split_response(&response);
	assert!(head.starts_with("HTTP/1.1 416"));
	assert!(head.contains("content-range: bytes */0"));
	assert!(body.is_empty());
	shutdown.send(()).await.unwrap();
}

#[tokio::test]
async fn sidecar_preview_resolves_library_and_serves_variant_types() {
	let directory = tempfile::tempdir().unwrap();
	let folder = directory.path().join("libraries/Test.winglibrary");
	let content_uuid = "0cc0b48f-a475-53ec-a580-bc7d47b486a9";
	let content = folder.join("sidecars/content/0c/c0").join(content_uuid);
	let library_id = "6dd13bc8-ba21-4c13-9bc0-a9c1b2f98df1";
	let (url, token, shutdown) = start_server(directory.path().to_path_buf()).await.unwrap();

	let route = format!("/sidecar/{library_id}/{content_uuid}");
	let missing_libraries = get(&url, &format!("{route}/thumb/detail.webp?token={token}")).await;
	assert!(missing_libraries.starts_with(b"HTTP/1.1 500"));

	for (kind, variant, content_type) in [
		("thumb", "detail.webp", "image/webp"),
		("thumb", "detail.jpeg", "image/jpeg"),
		("thumb", "detail.png", "image/png"),
		("transcript", "original.txt", "application/octet-stream"),
	] {
		let category = if kind == "transcript" {
			"transcript"
		} else {
			"thumbs"
		};
		let output = content.join(category);
		tokio::fs::create_dir_all(&output).await.unwrap();
		tokio::fs::write(output.join(variant), variant.as_bytes())
			.await
			.unwrap();
		tokio::fs::write(
			folder.join("library.json"),
			serde_json::json!({"id": library_id}).to_string(),
		)
		.await
		.unwrap();
		let response = get(&url, &format!("{route}/{kind}/{variant}?token={token}")).await;
		let (head, body) = split_response(&response);
		assert!(head.starts_with("HTTP/1.1 200"));
		assert!(head.contains(&format!("content-type: {content_type}")));
		assert!(head.contains("cache-control: public, max-age=31536000, immutable"));
		assert_eq!(body, variant.as_bytes());
	}

	let missing_variant = get(&url, &format!("{route}/thumb/missing.webp?token={token}")).await;
	assert!(missing_variant.starts_with(b"HTTP/1.1 404"));
	let missing_library = get(
		&url,
		&format!("/sidecar/missing/{content_uuid}/thumb/detail.webp?token={token}"),
	)
	.await;
	assert!(missing_library.starts_with(b"HTTP/1.1 404"));
	let denied = get(&url, &format!("{route}/thumb/detail.webp?token=wrong")).await;
	assert!(denied.starts_with(b"HTTP/1.1 403"));
	shutdown.send(()).await.unwrap();
}

fn file_target(path: &std::path::Path, token: &str) -> String {
	let mut url = tauri::Url::parse("http://localhost/file").unwrap();
	url.query_pairs_mut()
		.append_pair("path", &path.to_string_lossy())
		.append_pair("token", token);
	format!("/file?{}", url.query().unwrap())
}

fn split_response(response: &[u8]) -> (&str, &[u8]) {
	let split = response
		.windows(4)
		.position(|part| part == b"\r\n\r\n")
		.unwrap();
	(
		std::str::from_utf8(&response[..split]).unwrap(),
		&response[split + 4..],
	)
}
