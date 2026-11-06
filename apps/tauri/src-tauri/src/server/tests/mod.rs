use super::*;
use axum::body::to_bytes;

fn state(data_dir: PathBuf) -> ServerState {
	ServerState {
		data_dir,
		token: Arc::new("test-token".into()),
		thumb_permits: Arc::new(tokio::sync::Semaphore::new(1)),
	}
}

async fn file_response(
	path: &std::path::Path,
	token: &str,
	range: Option<&str>,
) -> Result<Response<Body>, StatusCode> {
	let mut headers = HeaderMap::new();
	if let Some(range) = range {
		headers.insert(header::RANGE, HeaderValue::from_str(range).unwrap());
	}
	serve_file(
		State(state(path.parent().unwrap().to_path_buf())),
		Query(FileQuery {
			path: path.to_string_lossy().into_owned(),
			token: token.into(),
		}),
		headers,
	)
	.await
}

async fn request(address: &str, target: &str, range: Option<&str>) -> Vec<u8> {
	use tokio::io::{AsyncReadExt, AsyncWriteExt};
	let mut stream = tokio::net::TcpStream::connect(address.trim_start_matches("http://"))
		.await
		.unwrap();
	let range = range
		.map(|value| format!("Range: {value}\r\n"))
		.unwrap_or_default();
	stream
		.write_all(
			format!("GET {target} HTTP/1.1\r\nHost: localhost\r\n{range}Connection: close\r\n\r\n")
				.as_bytes(),
		)
		.await
		.unwrap();
	let mut response = Vec::new();
	stream.read_to_end(&mut response).await.unwrap();
	response
}

#[tokio::test]
async fn loopback_router_streams_a_range_only_with_the_session_token() {
	let dir = tempfile::tempdir().unwrap();
	let path = dir.path().join("movie.mp4");
	tokio::fs::write(&path, b"0123456789").await.unwrap();
	let (address, token, shutdown) = start_server(dir.path().to_path_buf()).await.unwrap();
	let target = format!("/file?path={}&token=", path.display());
	let denied = request(&address, &format!("{target}invalid"), None).await;
	assert!(denied.starts_with(b"HTTP/1.1 403"));
	let response = request(&address, &format!("{target}{token}"), Some("bytes=2-5")).await;
	assert!(response.starts_with(b"HTTP/1.1 206"));
	assert!(response.ends_with(b"\r\n\r\n2345"));
	shutdown.send(()).await.unwrap();
}

#[tokio::test]
async fn file_stream_enforces_token_and_returns_exact_requested_bytes() {
	let dir = tempfile::tempdir().unwrap();
	let path = dir.path().join("sample.MP4");
	tokio::fs::write(&path, b"0123456789").await.unwrap();
	assert_eq!(
		file_response(&path, "bad-token", None).await.unwrap_err(),
		StatusCode::FORBIDDEN
	);

	for (range, expected, content_range) in [
		("bytes=2-5", &b"2345"[..], "bytes 2-5/10"),
		("bytes=7-", &b"789"[..], "bytes 7-9/10"),
		("bytes=-3", &b"789"[..], "bytes 7-9/10"),
		("bytes=8-99", &b"89"[..], "bytes 8-9/10"),
	] {
		let response = file_response(&path, "test-token", Some(range))
			.await
			.unwrap();
		assert_eq!(response.status(), StatusCode::PARTIAL_CONTENT);
		assert_eq!(response.headers()[header::CONTENT_TYPE], "video/mp4");
		assert_eq!(response.headers()[header::CONTENT_RANGE], content_range);
		assert_eq!(
			response.headers()[header::CONTENT_LENGTH].to_str().unwrap(),
			expected.len().to_string()
		);
		assert_eq!(
			&to_bytes(response.into_body(), usize::MAX).await.unwrap()[..],
			expected
		);
	}
	let response = file_response(&path, "test-token", Some("bytes=10-20"))
		.await
		.unwrap();
	assert_eq!(response.status(), StatusCode::RANGE_NOT_SATISFIABLE);
	assert_eq!(response.headers()[header::CONTENT_RANGE], "bytes */10");
}

#[tokio::test]
async fn empty_files_and_invalid_ranges_do_not_advertise_nonexistent_bytes() {
	let dir = tempfile::tempdir().unwrap();
	let path = dir.path().join("empty.mp4");
	tokio::fs::write(&path, []).await.unwrap();
	let response = file_response(&path, "test-token", None).await.unwrap();
	assert_eq!(response.status(), StatusCode::OK);
	assert_eq!(response.headers()[header::CONTENT_LENGTH], "0");
	assert!(to_bytes(response.into_body(), usize::MAX)
		.await
		.unwrap()
		.is_empty());
}

#[tokio::test]
async fn empty_file_rejects_byte_ranges() {
	let dir = tempfile::tempdir().unwrap();
	let path = dir.path().join("empty.mp4");
	tokio::fs::write(&path, []).await.unwrap();
	let response = file_response(&path, "test-token", Some("bytes=0-"))
		.await
		.unwrap();
	assert_eq!(response.status(), StatusCode::RANGE_NOT_SATISFIABLE);
	assert_eq!(response.headers()[header::CONTENT_RANGE], "bytes */0");
}

#[tokio::test]
async fn inverted_range_falls_back_to_complete_response() {
	let dir = tempfile::tempdir().unwrap();
	let path = dir.path().join("file.mp4");
	tokio::fs::write(&path, b"0123456789").await.unwrap();
	let response = file_response(&path, "test-token", Some("bytes=6-2"))
		.await
		.unwrap();
	assert_eq!(response.status(), StatusCode::OK);
	assert_eq!(
		&to_bytes(response.into_body(), usize::MAX).await.unwrap()[..],
		b"0123456789"
	);
}

#[tokio::test]
async fn sidecar_serving_checks_token_segments_and_library_identity() {
	let dir = tempfile::tempdir().unwrap();
	let library = dir.path().join("libraries/test.winglibrary");
	let uuid = "0cc0b48f-a475-53ec-a580-bc7d47b486a9";
	let thumbs = library
		.join("sidecars/content/0c/c0")
		.join(uuid)
		.join("thumbs");
	tokio::fs::create_dir_all(&thumbs).await.unwrap();
	tokio::fs::write(library.join("library.json"), r#"{"id":"library-id"}"#)
		.await
		.unwrap();
	tokio::fs::write(thumbs.join("grid.webp"), b"fixture")
		.await
		.unwrap();
	let call = |token: &str, id: &str, variant: &str| {
		serve_sidecar(
			State(state(dir.path().to_path_buf())),
			Path((id.into(), uuid.into(), "thumb".into(), variant.into())),
			Query(SidecarQuery {
				token: token.into(),
			}),
		)
	};
	assert_eq!(
		call("bad-token", "library-id", "grid.webp")
			.await
			.unwrap_err(),
		StatusCode::FORBIDDEN
	);
	assert_eq!(
		call("test-token", "library-id", "../../escape")
			.await
			.unwrap_err(),
		StatusCode::BAD_REQUEST
	);
	assert_eq!(
		call("test-token", "missing", "grid.webp")
			.await
			.unwrap_err(),
		StatusCode::NOT_FOUND
	);
	let response = call("test-token", "library-id", "grid.webp").await.unwrap();
	assert_eq!(response.status(), StatusCode::OK);
	assert_eq!(response.headers()[header::CONTENT_TYPE], "image/webp");
	assert_eq!(
		&to_bytes(response.into_body(), usize::MAX).await.unwrap()[..],
		b"fixture"
	);
}
