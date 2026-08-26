//! HTTP server for serving files and sidecars
//!
//! Tauri's custom URI protocols can't be async, so we use an Axum HTTP server
//! similar to the V1 implementation. The server is bound to localhost on a random
//! port and requires an auth token injected into the webview for security.

use axum::{
	body::Body,
	extract::{Path, Query, State},
	http::{header, HeaderMap, HeaderValue, Request, Response, StatusCode},
	middleware::{self, Next},
	routing::get,
	Router,
};
use serde::Deserialize;
use std::{net::Ipv4Addr, path::PathBuf, sync::Arc};
use tokio::{
	fs::File,
	io::{self, AsyncReadExt, AsyncSeekExt},
	net::TcpListener,
};
use tracing::{error, info};

#[derive(Clone)]
pub struct ServerState {
	/// Path to the Spacedrive data directory
	data_dir: PathBuf,
	/// Random per-session token required by the file endpoint
	token: Arc<String>,
}

/// Find library folder by UUID (reads library.json files to match ID)
async fn find_library_folder(
	data_dir: &std::path::Path,
	library_id: &str,
) -> Result<PathBuf, StatusCode> {
	let libraries_dir = data_dir.join("libraries");

	// Read all .sdlibrary folders
	let mut entries = tokio::fs::read_dir(&libraries_dir)
		.await
		.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

	while let Some(entry) = entries
		.next_entry()
		.await
		.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
	{
		let path = entry.path();
		if path.extension().and_then(|s| s.to_str()) == Some("sdlibrary") {
			// Try to read library.json
			let library_json_path = path.join("library.json");
			if let Ok(contents) = tokio::fs::read_to_string(&library_json_path).await {
				if let Ok(json) = serde_json::from_str::<serde_json::Value>(&contents) {
					if let Some(id) = json.get("id").and_then(|v| v.as_str()) {
						if id == library_id {
							return Ok(path);
						}
					}
				}
			}
		}
	}

	Err(StatusCode::NOT_FOUND)
}

/// Serve a sidecar file (e.g., thumbnail)
async fn serve_sidecar(
	State(state): State<ServerState>,
	Path((library_id, content_uuid, kind, variant_and_ext)): Path<(String, String, String, String)>,
) -> Result<Response<Body>, StatusCode> {
	// Find the actual library folder (might be named differently than the ID)
	let library_folder = find_library_folder(&state.data_dir, &library_id).await?;

	// Actual path structure: sidecars/content/{first2}/{next2}/{uuid}/{kind}s/{variant}.{ext}
	// Example: sidecars/content/0c/c0/0cc0b48f-a475-53ec-a580-bc7d47b486a9/thumbs/detail@1x.webp
	let first_two = &content_uuid[0..2];
	let next_two = &content_uuid[2..4];

	// Special case: "transcript" stays singular (not "transcripts")
	let kind_dir = if kind == "transcript" {
		kind.to_string()
	} else {
		format!("{}s", kind) // "thumb" -> "thumbs"
	};

	let sidecar_path = library_folder
		.join("sidecars")
		.join("content")
		.join(first_two)
		.join(next_two)
		.join(&content_uuid)
		.join(&kind_dir)
		.join(&variant_and_ext);

	// Security: prevent directory traversal
	let sidecars_root = state.data_dir.join("libraries");
	if !sidecar_path.starts_with(&sidecars_root) {
		error!(
			"Directory traversal attempt: {:?} not under {:?}",
			sidecar_path, sidecars_root
		);
		return Err(StatusCode::FORBIDDEN);
	}

	// Open the file
	let file = File::open(&sidecar_path).await.map_err(|e| {
		if e.kind() == io::ErrorKind::NotFound {
			error!("Sidecar file not found: {:?}", sidecar_path);
			StatusCode::NOT_FOUND
		} else {
			error!("Error opening sidecar {:?}: {}", sidecar_path, e);
			StatusCode::INTERNAL_SERVER_ERROR
		}
	})?;

	let metadata = file.metadata().await.map_err(|e| {
		error!("Error reading metadata for {:?}: {}", sidecar_path, e);
		StatusCode::INTERNAL_SERVER_ERROR
	})?;

	// Determine content type from extension
	let content_type = variant_and_ext
		.rsplit('.')
		.next()
		.and_then(|ext| match ext {
			"webp" => Some("image/webp"),
			"jpg" | "jpeg" => Some("image/jpeg"),
			"png" => Some("image/png"),
			_ => None,
		})
		.unwrap_or("application/octet-stream");

	// Build response with proper headers
	let content_length = metadata.len();
	let body = Body::from_stream(tokio_util::io::ReaderStream::new(file));

	Response::builder()
		.status(StatusCode::OK)
		.header(header::CONTENT_TYPE, HeaderValue::from_static(content_type))
		.header(header::CONTENT_LENGTH, content_length)
		.header(
			header::CACHE_CONTROL,
			HeaderValue::from_static("public, max-age=31536000, immutable"),
		)
		.header(
			header::ACCESS_CONTROL_ALLOW_ORIGIN,
			HeaderValue::from_static("*"),
		)
		.body(body)
		.map_err(|e| {
			error!("Error building response: {}", e);
			StatusCode::INTERNAL_SERVER_ERROR
		})
}

#[derive(Deserialize)]
pub struct FileQuery {
	path: String,
	token: String,
}

/// Streams an arbitrary local file with HTTP range support.
///
/// Media elements on Linux are decoded by GStreamer, which fetches the URI
/// itself instead of going through the webview's custom scheme handler, so
/// `asset:`/`http://asset.localhost` URLs never load audio or video there.
/// Serving media over the loopback HTTP server sidesteps that and gives
/// seeking for free, since range requests are handled here.
///
/// The endpoint would otherwise let any local process read the user's files,
/// so it requires the per-session token that only the webview receives.
async fn serve_file(
	State(state): State<ServerState>,
	Query(query): Query<FileQuery>,
	headers: HeaderMap,
) -> Result<Response<Body>, StatusCode> {
	if query.token != *state.token {
		return Err(StatusCode::FORBIDDEN);
	}

	let path = PathBuf::from(&query.path);
	if !path.is_absolute() {
		return Err(StatusCode::BAD_REQUEST);
	}

	let mut file = File::open(&path).await.map_err(|e| {
		if e.kind() == io::ErrorKind::NotFound {
			StatusCode::NOT_FOUND
		} else {
			error!("Error opening {:?}: {}", path, e);
			StatusCode::FORBIDDEN
		}
	})?;

	let metadata = file
		.metadata()
		.await
		.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

	if metadata.is_dir() {
		return Err(StatusCode::BAD_REQUEST);
	}

	let total = metadata.len();
	let content_type = content_type_for(&path);
	let range = headers
		.get(header::RANGE)
		.and_then(|v| v.to_str().ok())
		.and_then(|v| parse_range(v, total));

	let (status, start, end) = match range {
		Some((start, end)) => (StatusCode::PARTIAL_CONTENT, start, end),
		None => (StatusCode::OK, 0, total.saturating_sub(1)),
	};

	if total > 0 && start >= total {
		return Response::builder()
			.status(StatusCode::RANGE_NOT_SATISFIABLE)
			.header(header::CONTENT_RANGE, format!("bytes */{}", total))
			.body(Body::empty())
			.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR);
	}

	file.seek(io::SeekFrom::Start(start))
		.await
		.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

	let length = end.saturating_sub(start) + 1;
	let body = Body::from_stream(tokio_util::io::ReaderStream::new(file.take(length)));

	let mut builder = Response::builder()
		.status(status)
		.header(header::CONTENT_TYPE, content_type)
		.header(header::CONTENT_LENGTH, length)
		.header(header::ACCEPT_RANGES, HeaderValue::from_static("bytes"))
		.header(
			header::ACCESS_CONTROL_ALLOW_ORIGIN,
			HeaderValue::from_static("*"),
		);

	if status == StatusCode::PARTIAL_CONTENT {
		builder = builder.header(
			header::CONTENT_RANGE,
			format!("bytes {}-{}/{}", start, end, total),
		);
	}

	builder.body(body).map_err(|e| {
		error!("Error building file response: {}", e);
		StatusCode::INTERNAL_SERVER_ERROR
	})
}

/// Parses a single-range `bytes=` header. Multi-range requests are ignored so
/// the caller falls back to a full response, which players handle fine.
fn parse_range(value: &str, total: u64) -> Option<(u64, u64)> {
	let spec = value.strip_prefix("bytes=")?;
	if spec.contains(',') {
		return None;
	}

	let (start, end) = spec.split_once('-')?;
	let last = total.saturating_sub(1);

	match (start.trim(), end.trim()) {
		("", "") => None,
		// Suffix range: last N bytes.
		("", n) => {
			let n: u64 = n.parse().ok()?;
			Some((total.saturating_sub(n), last))
		}
		(s, "") => Some((s.parse().ok()?, last)),
		(s, e) => {
			let start: u64 = s.parse().ok()?;
			let end: u64 = e.parse().ok()?;
			Some((start, end.min(last)))
		}
	}
}

fn content_type_for(path: &std::path::Path) -> HeaderValue {
	let ext = path
		.extension()
		.and_then(|e| e.to_str())
		.unwrap_or_default()
		.to_ascii_lowercase();

	let mime = match ext.as_str() {
		"mp4" | "m4v" => "video/mp4",
		"mov" => "video/quicktime",
		"webm" => "video/webm",
		"mkv" => "video/x-matroska",
		"avi" => "video/x-msvideo",
		"ogv" => "video/ogg",
		"mp3" => "audio/mpeg",
		"m4a" => "audio/mp4",
		"aac" => "audio/aac",
		"flac" => "audio/flac",
		"wav" => "audio/wav",
		"ogg" | "oga" => "audio/ogg",
		"opus" => "audio/opus",
		_ => "application/octet-stream",
	};

	HeaderValue::from_static(mime)
}

/// CORS middleware to add headers to all responses (including errors)
async fn add_cors_headers(request: Request<Body>, next: Next) -> Response<Body> {
	let mut response = next.run(request).await;
	response.headers_mut().insert(
		header::ACCESS_CONTROL_ALLOW_ORIGIN,
		HeaderValue::from_static("*"),
	);
	response
}

/// Create the HTTP router
fn create_router(data_dir: PathBuf, token: Arc<String>) -> Router {
	let state = ServerState { data_dir, token };

	Router::new()
		.route(
			"/sidecar/:library_id/:content_uuid/:kind/*variant",
			get(serve_sidecar),
		)
		.route("/file", get(serve_file))
		.layer(middleware::from_fn(add_cors_headers))
		.with_state(state)
}

/// Start the HTTP server on a random port
///
/// Returns the server address and a channel to trigger shutdown
pub async fn start_server(
	data_dir: PathBuf,
) -> Result<(String, String, tokio::sync::mpsc::Sender<()>), String> {
	// Bind to localhost on random port
	let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0))
		.await
		.map_err(|e| e.to_string())?;
	let addr = listener.local_addr().map_err(|e| e.to_string())?;
	let listen_url = format!("http://{}", addr);
	let token = uuid::Uuid::new_v4().to_string();

	info!("Starting sidecar HTTP server on {}", listen_url);

	let app = create_router(data_dir, Arc::new(token.clone()));
	let (shutdown_tx, mut shutdown_rx) = tokio::sync::mpsc::channel::<()>(1);

	// Spawn server task
	tokio::spawn(async move {
		axum::serve(listener, app)
			.with_graceful_shutdown(async move {
				shutdown_rx.recv().await;
				info!("Shutting down sidecar HTTP server");
			})
			.await
			.expect("HTTP server error");
	});

	Ok((listen_url, token, shutdown_tx))
}
