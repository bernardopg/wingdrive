use super::*;
use axum::body::to_bytes;

#[tokio::test]
async fn malformed_and_overflowing_ranges_return_the_complete_file() {
	let root = tempfile::tempdir().unwrap();
	let path = root.path().join("file.bin");
	tokio::fs::write(&path, b"0123456789").await.unwrap();
	for range in [
		"items=1-2",
		"bytes=0-1,3-4",
		"bytes=-",
		"bytes=abc-5",
		"bytes=0-abc",
		"bytes=18446744073709551616-",
		"bytes=-18446744073709551616",
	] {
		let response = file_response(&path, "test-token", Some(range))
			.await
			.unwrap();
		assert_eq!(response.status(), StatusCode::OK, "{range}");
		assert_eq!(response.headers()[header::CONTENT_LENGTH], "10", "{range}");
		assert!(
			!response.headers().contains_key(header::CONTENT_RANGE),
			"{range}"
		);
		assert_eq!(
			&to_bytes(response.into_body(), 100).await.unwrap()[..],
			b"0123456789",
			"{range}"
		);
	}
}

#[tokio::test]
async fn suffix_and_extreme_ranges_return_only_existing_bytes() {
	let root = tempfile::tempdir().unwrap();
	let path = root.path().join("file.bin");
	tokio::fs::write(&path, b"abc").await.unwrap();
	for (range, expected, content_range) in [
		("bytes=-18446744073709551615", &b"abc"[..], "bytes 0-2/3"),
		("bytes=2-18446744073709551615", &b"c"[..], "bytes 2-2/3"),
	] {
		let response = file_response(&path, "test-token", Some(range))
			.await
			.unwrap();
		assert_eq!(response.status(), StatusCode::PARTIAL_CONTENT);
		assert_eq!(response.headers()[header::CONTENT_RANGE], content_range);
		assert_eq!(
			&to_bytes(response.into_body(), 100).await.unwrap()[..],
			expected
		);
	}
	for range in ["bytes=-0", "bytes=18446744073709551615-"] {
		let response = file_response(&path, "test-token", Some(range))
			.await
			.unwrap();
		assert_eq!(response.status(), StatusCode::RANGE_NOT_SATISFIABLE);
		assert_eq!(response.headers()[header::CONTENT_RANGE], "bytes */3");
		assert!(to_bytes(response.into_body(), 100)
			.await
			.unwrap()
			.is_empty());
	}
}

#[tokio::test]
async fn file_stream_rejects_missing_files_directories_and_relative_paths() {
	let root = tempfile::tempdir().unwrap();
	for (path, expected) in [
		(root.path().join("missing"), StatusCode::NOT_FOUND),
		(root.path().to_path_buf(), StatusCode::BAD_REQUEST),
		(PathBuf::from("relative/file"), StatusCode::BAD_REQUEST),
	] {
		assert_eq!(
			file_response(&path, "test-token", None).await.unwrap_err(),
			expected
		);
		assert_eq!(
			file_response(&path, "bad-token", None).await.unwrap_err(),
			StatusCode::FORBIDDEN
		);
	}
}

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

fn sidecar_call(
	data_dir: &std::path::Path,
	library_id: &str,
	kind: &str,
	variant: &str,
) -> impl std::future::Future<Output = Result<Response<Body>, StatusCode>> {
	serve_sidecar(
		State(state(data_dir.to_path_buf())),
		Path((
			library_id.into(),
			SIDECAR_UUID.into(),
			kind.into(),
			variant.into(),
		)),
		Query(SidecarQuery {
			token: "test-token".into(),
		}),
	)
}

const SIDECAR_UUID: &str = "0cc0b48f-a475-53ec-a580-bc7d47b486a9";

#[tokio::test]
async fn sidecars_are_found_in_renamed_and_legacy_library_folders() {
	let dir = tempfile::tempdir().unwrap();
	let libraries = dir.path().join("libraries");
	// Folders that are not libraries, or whose metadata is unreadable or
	// belongs to another library, are skipped.
	for (folder, json) in [
		("notes.txt", Some(r#"{"id":"library-id"}"#)),
		("broken.winglibrary", Some("not json")),
		("anonymous.winglibrary", Some(r#"{"name":"x"}"#)),
		("other.winglibrary", Some(r#"{"id":"other"}"#)),
		("empty.winglibrary", None),
	] {
		let folder = libraries.join(folder);
		tokio::fs::create_dir_all(&folder).await.unwrap();
		if let Some(json) = json {
			tokio::fs::write(folder.join("library.json"), json)
				.await
				.unwrap();
		}
	}
	let legacy = libraries.join("Old Name.sdlibrary");
	let content = legacy.join("sidecars/content/0c/c0").join(SIDECAR_UUID);
	tokio::fs::create_dir_all(content.join("transcript"))
		.await
		.unwrap();
	tokio::fs::create_dir_all(content.join("thumbs"))
		.await
		.unwrap();
	tokio::fs::write(legacy.join("library.json"), r#"{"id":"library-id"}"#)
		.await
		.unwrap();
	tokio::fs::write(content.join("transcript/text.vtt"), b"WEBVTT")
		.await
		.unwrap();
	for name in ["a.jpg", "b.jpeg", "c.png", "d.bin"] {
		tokio::fs::write(content.join("thumbs").join(name), name)
			.await
			.unwrap();
	}

	// "transcript" keeps its singular folder name; other kinds are plural.
	let response = sidecar_call(dir.path(), "library-id", "transcript", "text.vtt")
		.await
		.unwrap();
	assert_eq!(
		response.headers()[header::CONTENT_TYPE],
		"application/octet-stream"
	);
	assert_eq!(response.headers()[header::CONTENT_LENGTH], "6");
	for (name, mime) in [
		("a.jpg", "image/jpeg"),
		("b.jpeg", "image/jpeg"),
		("c.png", "image/png"),
		("d.bin", "application/octet-stream"),
	] {
		let response = sidecar_call(dir.path(), "library-id", "thumb", name)
			.await
			.unwrap();
		assert_eq!(response.headers()[header::CONTENT_TYPE], mime, "{name}");
		assert_eq!(
			response.headers()[header::CACHE_CONTROL],
			"public, max-age=31536000, immutable"
		);
	}
	assert_eq!(
		sidecar_call(dir.path(), "library-id", "thumb", "missing.webp")
			.await
			.unwrap_err(),
		StatusCode::NOT_FOUND
	);
	assert_eq!(
		sidecar_call(dir.path(), "other", "thumb", "a.jpg")
			.await
			.unwrap_err(),
		StatusCode::NOT_FOUND
	);
}

#[tokio::test]
async fn sidecar_errors_distinguish_missing_libraries_and_unreadable_files() {
	let dir = tempfile::tempdir().unwrap();
	// A data folder without libraries is a server error, not a missing file.
	assert_eq!(
		sidecar_call(dir.path(), "library-id", "thumb", "a.webp")
			.await
			.unwrap_err(),
		StatusCode::INTERNAL_SERVER_ERROR
	);

	let library = dir.path().join("libraries/one.winglibrary");
	let thumbs = library
		.join("sidecars/content/0c/c0")
		.join(SIDECAR_UUID)
		.join("thumbs");
	tokio::fs::create_dir_all(&thumbs).await.unwrap();
	tokio::fs::write(library.join("library.json"), r#"{"id":"library-id"}"#)
		.await
		.unwrap();
	// A loop of symbolic links cannot be opened at all.
	std::os::unix::fs::symlink(thumbs.join("loop.webp"), thumbs.join("loop.webp")).unwrap();
	assert_eq!(
		sidecar_call(dir.path(), "library-id", "thumb", "loop.webp")
			.await
			.unwrap_err(),
		StatusCode::INTERNAL_SERVER_ERROR
	);
}

#[tokio::test]
async fn unreadable_files_are_forbidden() {
	use std::os::unix::fs::PermissionsExt;

	let dir = tempfile::tempdir().unwrap();
	let path = dir.path().join("secret.mp3");
	tokio::fs::write(&path, b"x").await.unwrap();
	std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o000)).unwrap();
	// Root reads any file, so the check only means something for other users.
	if std::fs::read(&path).is_err() {
		assert_eq!(
			file_response(&path, "test-token", None).await.unwrap_err(),
			StatusCode::FORBIDDEN
		);
	}
	std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
	let response = file_response(&path, "test-token", None).await.unwrap();
	assert_eq!(response.headers()[header::CONTENT_TYPE], "audio/mpeg");
	assert_eq!(response.headers()[header::ACCEPT_RANGES], "bytes");
}

#[test]
fn media_types_cover_the_formats_players_stream() {
	for (name, mime) in [
		("a.MP4", "video/mp4"),
		("a.m4v", "video/mp4"),
		("a.mov", "video/quicktime"),
		("a.webm", "video/webm"),
		("a.mkv", "video/x-matroska"),
		("a.avi", "video/x-msvideo"),
		("a.ogv", "video/ogg"),
		("a.mp3", "audio/mpeg"),
		("a.m4a", "audio/mp4"),
		("a.aac", "audio/aac"),
		("a.flac", "audio/flac"),
		("a.wav", "audio/wav"),
		("a.ogg", "audio/ogg"),
		("a.oga", "audio/ogg"),
		("a.opus", "audio/opus"),
		("a.txt", "application/octet-stream"),
		("noext", "application/octet-stream"),
	] {
		assert_eq!(content_type_for(std::path::Path::new(name)), mime, "{name}");
	}
}

#[test]
fn thumbnails_share_size_buckets_and_know_their_decoder() {
	assert_eq!(
		[0, 128, 129, 256, 257, 4096].map(thumb_bucket),
		[128, 128, 256, 256, 512, 512]
	);
	let kind = |name: &str| thumb_kind(std::path::Path::new(name));
	assert!(matches!(kind("a.JPG"), Some(ThumbKind::Image)));
	assert!(matches!(kind("a.tiff"), Some(ThumbKind::Image)));
	assert!(matches!(kind("a.MKV"), Some(ThumbKind::Video)));
	assert!(matches!(kind("a.3gp"), Some(ThumbKind::Video)));
	assert!(matches!(kind("a.pdf"), Some(ThumbKind::Pdf)));
	assert!(kind("a.txt").is_none());
	assert!(kind("noext").is_none());
}

async fn thumb_response(
	data_dir: &std::path::Path,
	path: &str,
	token: &str,
) -> Result<Response<Body>, StatusCode> {
	serve_thumb(
		State(state(data_dir.to_path_buf())),
		Query(ThumbQuery {
			path: path.into(),
			size: 64,
			token: token.into(),
		}),
	)
	.await
}

#[tokio::test]
async fn thumbnails_reject_what_they_cannot_decode() {
	let dir = tempfile::tempdir().unwrap();
	let root = dir.path();
	let text = |name: &str| root.join(name).to_string_lossy().into_owned();
	tokio::fs::write(root.join("notes.txt"), b"x")
		.await
		.unwrap();
	tokio::fs::write(root.join("broken.png"), b"not a png")
		.await
		.unwrap();
	tokio::fs::create_dir(root.join("folder.png"))
		.await
		.unwrap();

	for (path, token, expected) in [
		(text("notes.txt"), "bad-token", StatusCode::FORBIDDEN),
		(
			"relative.png".to_string(),
			"test-token",
			StatusCode::BAD_REQUEST,
		),
		(
			text("notes.txt"),
			"test-token",
			StatusCode::UNSUPPORTED_MEDIA_TYPE,
		),
		(text("missing.png"), "test-token", StatusCode::NOT_FOUND),
		(text("folder.png"), "test-token", StatusCode::BAD_REQUEST),
		(
			text("broken.png"),
			"test-token",
			StatusCode::UNPROCESSABLE_ENTITY,
		),
	] {
		assert_eq!(
			thumb_response(root, &path, token).await.unwrap_err(),
			expected,
			"{path}"
		);
	}
	// A failed decode is not cached, so a fixed file gets a thumbnail.
	assert!(!root.join("cache/thumbs").exists());
}

#[tokio::test]
async fn cached_thumbnails_are_served_without_decoding_again() {
	let dir = tempfile::tempdir().unwrap();
	let source = dir.path().join("photo.png");
	image::RgbImage::from_pixel(300, 150, image::Rgb([1, 2, 3]))
		.save(&source)
		.unwrap();
	let path = source.to_string_lossy().into_owned();
	let first = thumb_response(dir.path(), &path, "test-token")
		.await
		.unwrap();
	let first = to_bytes(first.into_body(), usize::MAX).await.unwrap();

	// Swap the cached bytes; a second request must return them as they are.
	let cache = dir.path().join("cache/thumbs");
	let entry = std::fs::read_dir(&cache).unwrap().next().unwrap().unwrap();
	std::fs::write(entry.path(), b"cached").unwrap();
	let second = thumb_response(dir.path(), &path, "test-token")
		.await
		.unwrap();
	assert_eq!(second.headers()[header::CONTENT_TYPE], "image/webp");
	assert_eq!(
		second.headers()[header::CACHE_CONTROL],
		"private, max-age=3600"
	);
	assert_eq!(
		&to_bytes(second.into_body(), 100).await.unwrap()[..],
		b"cached"
	);
	assert!(!first.is_empty());
}

#[tokio::test]
async fn host_tools_that_fail_hang_or_print_nothing_are_errors() {
	let sh = |script: &'static str| -> [&'static std::ffi::OsStr; 2] {
		["-c".as_ref(), script.as_ref()]
	};
	let short = std::time::Duration::from_millis(200);
	assert_eq!(
		tool_png_within("sh", &sh("sleep 5"), short)
			.await
			.unwrap_err(),
		"sh timed out"
	);
	assert_eq!(
		tool_png_within("sh", &sh("exit 1"), short)
			.await
			.unwrap_err(),
		"sh produced no image"
	);
	assert_eq!(
		tool_png_within("sh", &sh("true"), short).await.unwrap_err(),
		"sh produced no image"
	);
	assert_eq!(
		tool_png_within("sh", &sh("printf png"), short)
			.await
			.unwrap(),
		b"png"
	);
	let error = tool_png("/nonexistent/ffmpeg", &[]).await.unwrap_err();
	assert!(error.starts_with("/nonexistent/ffmpeg: "), "{error}");
}

#[tokio::test]
async fn cors_header_is_added_to_error_responses_too() {
	let dir = tempfile::tempdir().unwrap();
	let (address, _, shutdown) = start_server(dir.path().to_path_buf()).await.unwrap();
	let response = request(&address, "/file?path=/x&token=wrong", None).await;
	let response = String::from_utf8_lossy(&response).to_ascii_lowercase();
	assert!(response.starts_with("http/1.1 403"));
	assert!(response.contains("access-control-allow-origin: *"));
	shutdown.send(()).await.unwrap();
}
