//! In-memory fake of the Google APIs the integration calls: OAuth token,
//! revoke and userinfo, Drive v3 (folders, listing with pagination, ranged
//! downloads, export, resumable uploads, rename, copy, trash, delete, quota),
//! the Photos Library upload endpoints and the Photos Picker. It keeps state
//! between calls so tests exercise real flows, and it checks bearer tokens so
//! refresh and 401 retries are covered.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use axum::body::Bytes;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, post, put};
use axum::{Json, Router};
use serde_json::{json, Value};

pub const FOLDER: &str = "application/vnd.google-apps.folder";

/// A Google user the fake knows. The authorization code picks the user.
#[derive(Clone, Debug)]
pub struct FakeUser {
	pub code: String,
	pub sub: String,
	pub email: String,
	pub name: String,
}

#[derive(Clone, Debug)]
pub struct FakeFile {
	pub id: String,
	pub name: String,
	pub mime: String,
	pub parents: Vec<String>,
	pub content: Vec<u8>,
	pub trashed: bool,
	pub created: String,
	pub version: u64,
}

impl FakeFile {
	fn json(&self) -> Value {
		let mut value = json!({
			"id": self.id,
			"name": self.name,
			"mimeType": self.mime,
			"modifiedTime": "2026-11-12T10:00:00.000Z",
			"createdTime": self.created,
			"webViewLink": format!("https://drive.google.com/file/d/{}/view", self.id),
			"version": self.version.to_string(),
		});
		if !self.mime.starts_with("application/vnd.google-apps.") {
			value["size"] = json!(self.content.len().to_string());
			value["md5Checksum"] = json!(format!("{:x}", md5_of(&self.content)));
		}
		value
	}
}

fn md5_of(data: &[u8]) -> u128 {
	// A stable stand-in for MD5: tests only compare it for equality.
	let mut hash: u128 = 0xcbf29ce484222325;
	for byte in data {
		hash ^= *byte as u128;
		hash = hash.wrapping_mul(0x100000001b3);
	}
	hash
}

struct UploadSession {
	target: Option<String>,
	name: String,
	parents: Vec<String>,
	data: Vec<u8>,
}

#[derive(Default)]
pub struct FakeState {
	pub users: Vec<FakeUser>,
	/// access token -> user sub
	pub access_tokens: HashMap<String, String>,
	/// refresh token -> user sub
	pub refresh_tokens: HashMap<String, String>,
	pub revoked: Vec<String>,
	pub token_requests: u32,
	pub refresh_requests: u32,
	pub fail_next_with_401: bool,
	pub files: HashMap<String, FakeFile>,
	next_id: u64,
	sessions: HashMap<String, UploadSession>,
	pub upload_chunks: u32,
	/// Forces small listing pages to exercise pagination.
	pub page_size: usize,
	pub photos_uploads: HashMap<String, (String, Vec<u8>)>,
	pub media_items: Vec<(String, String, Option<String>)>,
	pub albums: Vec<(String, String)>,
	pub picker_ready: bool,
	pub picked: Vec<(String, String, String, Vec<u8>)>,
	pub picker_deleted: bool,
	pub base_url: String,
}

impl FakeState {
	fn id(&mut self, prefix: &str) -> String {
		self.next_id += 1;
		format!("{prefix}{}", self.next_id)
	}

	/// Adds a file or folder directly, as if created in the Drive web UI.
	pub fn insert(&mut self, name: &str, mime: &str, parent: &str, content: &[u8]) -> String {
		let id = self.id("f");
		let created = format!("2026-01-01T00:00:{:02}.000Z", self.next_id % 60);
		self.files.insert(
			id.clone(),
			FakeFile {
				id: id.clone(),
				name: name.to_string(),
				mime: mime.to_string(),
				parents: vec![parent.to_string()],
				content: content.to_vec(),
				trashed: false,
				created,
				version: 1,
			},
		);
		id
	}
}

pub type Shared = Arc<Mutex<FakeState>>;

pub struct FakeGoogle {
	pub state: Shared,
	pub base_url: String,
}

fn unauthorized() -> Response {
	(
		StatusCode::UNAUTHORIZED,
		Json(json!({"error": {"code": 401, "message": "Invalid Credentials"}})),
	)
		.into_response()
}

fn not_found(id: &str) -> Response {
	(
		StatusCode::NOT_FOUND,
		Json(json!({"error": {"code": 404, "message": format!("File not found: {id}.")}})),
	)
		.into_response()
}

/// Checks the bearer token and returns the user's sub.
fn auth(state: &Shared, headers: &HeaderMap) -> Result<String, Response> {
	let mut state = state.lock().unwrap();
	if state.fail_next_with_401 {
		state.fail_next_with_401 = false;
		return Err(unauthorized());
	}
	let token = headers
		.get("authorization")
		.and_then(|v| v.to_str().ok())
		.and_then(|v| v.strip_prefix("Bearer "))
		.ok_or_else(unauthorized)?;
	state
		.access_tokens
		.get(token)
		.cloned()
		.ok_or_else(unauthorized)
}

fn parse_form(body: &[u8]) -> HashMap<String, String> {
	url::form_urlencoded::parse(body).into_owned().collect()
}

async fn token(State(state): State<Shared>, body: Bytes) -> Response {
	let form = parse_form(&body);
	let mut s = state.lock().unwrap();
	s.token_requests += 1;
	let scope = "openid https://www.googleapis.com/auth/userinfo.email https://www.googleapis.com/auth/userinfo.profile https://www.googleapis.com/auth/drive https://www.googleapis.com/auth/photoslibrary.appendonly https://www.googleapis.com/auth/photoslibrary.readonly.appcreateddata https://www.googleapis.com/auth/photoslibrary.edit.appcreateddata https://www.googleapis.com/auth/photospicker.mediaitems.readonly";
	match form.get("grant_type").map(String::as_str) {
		Some("authorization_code") => {
			let Some(user) = s
				.users
				.iter()
				.find(|u| Some(&u.code) == form.get("code"))
				.cloned()
			else {
				return (
					StatusCode::BAD_REQUEST,
					Json(json!({"error": "invalid_grant"})),
				)
					.into_response();
			};
			if form.get("code_verifier").map_or(true, |v| v.len() < 43) {
				return (
					StatusCode::BAD_REQUEST,
					Json(json!({"error": "invalid_request", "error_description": "code_verifier"})),
				)
					.into_response();
			}
			let access = s.id(&format!("at-{}-", user.sub));
			let refresh = format!("rt-{}", user.sub);
			s.access_tokens.insert(access.clone(), user.sub.clone());
			s.refresh_tokens.insert(refresh.clone(), user.sub.clone());
			Json(json!({
				"access_token": access,
				"expires_in": 3599,
				"refresh_token": refresh,
				"scope": scope,
				"token_type": "Bearer",
			}))
			.into_response()
		}
		Some("refresh_token") => {
			s.refresh_requests += 1;
			let Some(sub) = form
				.get("refresh_token")
				.and_then(|r| s.refresh_tokens.get(r))
				.cloned()
			else {
				return (
					StatusCode::BAD_REQUEST,
					Json(
						json!({"error": "invalid_grant", "error_description": "Token has been expired or revoked."}),
					),
				)
					.into_response();
			};
			let access = s.id(&format!("at-{sub}-"));
			s.access_tokens.insert(access.clone(), sub);
			Json(json!({"access_token": access, "expires_in": 3599, "scope": scope}))
				.into_response()
		}
		_ => (
			StatusCode::BAD_REQUEST,
			Json(json!({"error": "unsupported_grant_type"})),
		)
			.into_response(),
	}
}

async fn revoke(State(state): State<Shared>, body: Bytes) -> Response {
	let form = parse_form(&body);
	let mut s = state.lock().unwrap();
	if let Some(token) = form.get("token") {
		s.revoked.push(token.clone());
		s.refresh_tokens.remove(token);
	}
	StatusCode::OK.into_response()
}

async fn userinfo(State(state): State<Shared>, headers: HeaderMap) -> Response {
	let sub = match auth(&state, &headers) {
		Ok(sub) => sub,
		Err(r) => return r,
	};
	let s = state.lock().unwrap();
	let user = s.users.iter().find(|u| u.sub == sub).unwrap();
	Json(json!({"sub": user.sub, "email": user.email, "name": user.name, "picture": "https://example.com/a.png"})).into_response()
}

async fn list_files(
	State(state): State<Shared>,
	headers: HeaderMap,
	Query(query): Query<HashMap<String, String>>,
) -> Response {
	if let Err(r) = auth(&state, &headers) {
		return r;
	}
	let q = query.get("q").cloned().unwrap_or_default();
	let parent = q
		.strip_prefix('\'')
		.and_then(|rest| rest.split("' in parents").next())
		.unwrap_or("root")
		.replace("\\'", "'");
	let s = state.lock().unwrap();
	let mut files: Vec<&FakeFile> = s
		.files
		.values()
		.filter(|f| !f.trashed && f.parents.contains(&parent))
		.collect();
	files.sort_by(|a, b| a.id.cmp(&b.id));
	let page_size = if s.page_size > 0 { s.page_size } else { 1000 };
	let offset: usize = query
		.get("pageToken")
		.and_then(|t| t.parse().ok())
		.unwrap_or(0);
	let page: Vec<Value> = files
		.iter()
		.skip(offset)
		.take(page_size)
		.map(|f| f.json())
		.collect();
	let mut body = json!({ "files": page });
	if offset + page_size < files.len() {
		body["nextPageToken"] = json!((offset + page_size).to_string());
	}
	Json(body).into_response()
}

async fn create_file(State(state): State<Shared>, headers: HeaderMap, body: Bytes) -> Response {
	if let Err(r) = auth(&state, &headers) {
		return r;
	}
	let meta: Value = serde_json::from_slice(&body).unwrap();
	let mut s = state.lock().unwrap();
	let parent = meta["parents"][0].as_str().unwrap_or("root").to_string();
	let id = s.insert(
		meta["name"].as_str().unwrap(),
		meta["mimeType"]
			.as_str()
			.unwrap_or("application/octet-stream"),
		&parent,
		b"",
	);
	Json(s.files[&id].json()).into_response()
}

async fn get_file(
	State(state): State<Shared>,
	headers: HeaderMap,
	Path(id): Path<String>,
	Query(query): Query<HashMap<String, String>>,
) -> Response {
	if let Err(r) = auth(&state, &headers) {
		return r;
	}
	let s = state.lock().unwrap();
	let Some(file) = s.files.get(&id) else {
		return not_found(&id);
	};
	if query.get("alt").map(String::as_str) != Some("media") {
		return Json(file.json()).into_response();
	}
	if let Some(range) = headers.get("range").and_then(|v| v.to_str().ok()) {
		let spec = range.trim_start_matches("bytes=");
		let (start, end) = spec.split_once('-').unwrap();
		let start: usize = start.parse().unwrap();
		let end: usize = end
			.parse::<usize>()
			.unwrap()
			.min(file.content.len().saturating_sub(1));
		let slice = file.content.get(start..=end).unwrap_or_default().to_vec();
		return (StatusCode::PARTIAL_CONTENT, slice).into_response();
	}
	file.content.clone().into_response()
}

async fn export_file(
	State(state): State<Shared>,
	headers: HeaderMap,
	Path(id): Path<String>,
	Query(query): Query<HashMap<String, String>>,
) -> Response {
	if let Err(r) = auth(&state, &headers) {
		return r;
	}
	let s = state.lock().unwrap();
	let Some(file) = s.files.get(&id) else {
		return not_found(&id);
	};
	let mut data = format!(
		"EXPORT({}):",
		query.get("mimeType").cloned().unwrap_or_default()
	)
	.into_bytes();
	data.extend_from_slice(&file.content);
	data.into_response()
}

async fn patch_file(
	State(state): State<Shared>,
	headers: HeaderMap,
	Path(id): Path<String>,
	Query(query): Query<HashMap<String, String>>,
	body: Bytes,
) -> Response {
	if let Err(r) = auth(&state, &headers) {
		return r;
	}
	let meta: Value = serde_json::from_slice(&body).unwrap_or(json!({}));
	let mut s = state.lock().unwrap();
	let Some(file) = s.files.get_mut(&id) else {
		return not_found(&id);
	};
	if let Some(name) = meta["name"].as_str() {
		file.name = name.to_string();
	}
	if meta["trashed"].as_bool() == Some(true) {
		file.trashed = true;
	}
	if let (Some(add), Some(remove)) = (query.get("addParents"), query.get("removeParents")) {
		file.parents.retain(|p| p != remove);
		file.parents.push(add.clone());
	}
	Json(file.json()).into_response()
}

async fn delete_file(
	State(state): State<Shared>,
	headers: HeaderMap,
	Path(id): Path<String>,
) -> Response {
	if let Err(r) = auth(&state, &headers) {
		return r;
	}
	let mut s = state.lock().unwrap();
	if s.files.remove(&id).is_none() {
		return not_found(&id);
	}
	// Deleting a folder deletes what it holds.
	let mut orphans: Vec<String> = s
		.files
		.values()
		.filter(|f| f.parents.contains(&id))
		.map(|f| f.id.clone())
		.collect();
	while let Some(orphan) = orphans.pop() {
		s.files.remove(&orphan);
		orphans.extend(
			s.files
				.values()
				.filter(|f| f.parents.contains(&orphan))
				.map(|f| f.id.clone()),
		);
	}
	StatusCode::NO_CONTENT.into_response()
}

async fn copy_file(
	State(state): State<Shared>,
	headers: HeaderMap,
	Path(id): Path<String>,
	body: Bytes,
) -> Response {
	if let Err(r) = auth(&state, &headers) {
		return r;
	}
	let meta: Value = serde_json::from_slice(&body).unwrap();
	let mut s = state.lock().unwrap();
	let Some(source) = s.files.get(&id).cloned() else {
		return not_found(&id);
	};
	let new_id = s.insert(
		meta["name"].as_str().unwrap_or(&source.name),
		&source.mime,
		meta["parents"][0].as_str().unwrap_or("root"),
		&source.content,
	);
	Json(s.files[&new_id].json()).into_response()
}

async fn about(State(state): State<Shared>, headers: HeaderMap) -> Response {
	if let Err(r) = auth(&state, &headers) {
		return r;
	}
	Json(json!({"storageQuota": {"limit": "16106127360", "usage": "1073741824"}})).into_response()
}

fn start_upload(state: &Shared, target: Option<String>, body: &[u8]) -> Response {
	let meta: Value = serde_json::from_slice(body).unwrap_or(json!({}));
	let mut s = state.lock().unwrap();
	let sid = s.id("upload-");
	s.sessions.insert(
		sid.clone(),
		UploadSession {
			target,
			name: meta["name"].as_str().unwrap_or_default().to_string(),
			parents: meta["parents"]
				.as_array()
				.map(|a| {
					a.iter()
						.filter_map(|p| p.as_str().map(str::to_string))
						.collect()
				})
				.unwrap_or_default(),
			data: Vec::new(),
		},
	);
	let location = format!("{}/upload-session/{sid}", s.base_url);
	(StatusCode::OK, [("location", location)]).into_response()
}

async fn upload_create(State(state): State<Shared>, headers: HeaderMap, body: Bytes) -> Response {
	if let Err(r) = auth(&state, &headers) {
		return r;
	}
	start_upload(&state, None, &body)
}

async fn upload_update(
	State(state): State<Shared>,
	headers: HeaderMap,
	Path(id): Path<String>,
	body: Bytes,
) -> Response {
	if let Err(r) = auth(&state, &headers) {
		return r;
	}
	if !state.lock().unwrap().files.contains_key(&id) {
		return not_found(&id);
	}
	start_upload(&state, Some(id), &body)
}

async fn upload_chunk(
	State(state): State<Shared>,
	headers: HeaderMap,
	Path(sid): Path<String>,
	body: Bytes,
) -> Response {
	let range = headers
		.get("content-range")
		.and_then(|v| v.to_str().ok())
		.unwrap_or_default()
		.to_string();
	let mut s = state.lock().unwrap();
	s.upload_chunks += 1;
	let Some(session) = s.sessions.get_mut(&sid) else {
		return StatusCode::NOT_FOUND.into_response();
	};
	session.data.extend_from_slice(&body);
	let total: usize = range
		.rsplit('/')
		.next()
		.and_then(|t| t.parse().ok())
		.unwrap_or(0);
	if session.data.len() < total {
		let last = session.data.len() - 1;
		return (
			StatusCode::PERMANENT_REDIRECT,
			[("range", format!("bytes=0-{last}"))],
		)
			.into_response();
	}
	let session = s.sessions.remove(&sid).unwrap();
	let id = match session.target {
		Some(id) => {
			let file = s.files.get_mut(&id).unwrap();
			file.content = session.data;
			file.version += 1;
			id
		}
		None => {
			let parent = session
				.parents
				.first()
				.cloned()
				.unwrap_or_else(|| "root".into());
			let mime = mime_guess::from_path(&session.name)
				.first_or_octet_stream()
				.to_string();
			s.insert(&session.name, &mime, &parent, &session.data)
		}
	};
	Json(s.files[&id].json()).into_response()
}

async fn photos_upload(State(state): State<Shared>, headers: HeaderMap, body: Bytes) -> Response {
	if let Err(r) = auth(&state, &headers) {
		return r;
	}
	if headers
		.get("x-goog-upload-protocol")
		.and_then(|v| v.to_str().ok())
		!= Some("raw")
	{
		return StatusCode::BAD_REQUEST.into_response();
	}
	let name = headers
		.get("x-goog-upload-file-name")
		.and_then(|v| v.to_str().ok())
		.unwrap_or_default()
		.to_string();
	let mut s = state.lock().unwrap();
	let token = s.id("utoken-");
	s.photos_uploads
		.insert(token.clone(), (name, body.to_vec()));
	token.into_response()
}

async fn photos_batch_create(
	State(state): State<Shared>,
	headers: HeaderMap,
	body: Bytes,
) -> Response {
	if let Err(r) = auth(&state, &headers) {
		return r;
	}
	let request: Value = serde_json::from_slice(&body).unwrap();
	let album = request["albumId"].as_str().map(str::to_string);
	let mut s = state.lock().unwrap();
	let mut results = Vec::new();
	for item in request["newMediaItems"].as_array().unwrap() {
		let token = item["simpleMediaItem"]["uploadToken"]
			.as_str()
			.unwrap()
			.to_string();
		if s.photos_uploads.remove(&token).is_some() {
			let id = s.id("media-");
			let name = item["simpleMediaItem"]["fileName"]
				.as_str()
				.unwrap()
				.to_string();
			s.media_items.push((id.clone(), name, album.clone()));
			results.push(
				json!({"uploadToken": token, "status": {"message": "Success"}, "mediaItem": {"id": id}}),
			);
		} else {
			results.push(
				json!({"uploadToken": token, "status": {"code": 3, "message": "Invalid upload token"}}),
			);
		}
	}
	Json(json!({"newMediaItemResults": results})).into_response()
}

async fn photos_create_album(
	State(state): State<Shared>,
	headers: HeaderMap,
	body: Bytes,
) -> Response {
	if let Err(r) = auth(&state, &headers) {
		return r;
	}
	let request: Value = serde_json::from_slice(&body).unwrap();
	let mut s = state.lock().unwrap();
	let id = s.id("album-");
	let title = request["album"]["title"].as_str().unwrap().to_string();
	s.albums.push((id.clone(), title.clone()));
	Json(
		json!({"id": id, "title": title, "productUrl": format!("https://photos.google.com/album/{id}")}),
	)
	.into_response()
}

async fn photos_list_albums(State(state): State<Shared>, headers: HeaderMap) -> Response {
	if let Err(r) = auth(&state, &headers) {
		return r;
	}
	let s = state.lock().unwrap();
	let albums: Vec<Value> = s
		.albums
		.iter()
		.map(|(id, title)| json!({"id": id, "title": title}))
		.collect();
	Json(json!({"albums": albums})).into_response()
}

async fn picker_create(State(state): State<Shared>, headers: HeaderMap) -> Response {
	if let Err(r) = auth(&state, &headers) {
		return r;
	}
	Json(json!({
		"id": "picker-1",
		"pickerUri": "https://photos.google.com/picker/picker-1",
		"pollingConfig": {"pollInterval": "1s", "timeoutIn": "600s"},
		"mediaItemsSet": false,
	}))
	.into_response()
}

async fn picker_get(
	State(state): State<Shared>,
	headers: HeaderMap,
	Path(id): Path<String>,
) -> Response {
	if let Err(r) = auth(&state, &headers) {
		return r;
	}
	let ready = state.lock().unwrap().picker_ready;
	Json(json!({"id": id, "pickerUri": "", "mediaItemsSet": ready})).into_response()
}

async fn picker_delete(State(state): State<Shared>, headers: HeaderMap) -> Response {
	if let Err(r) = auth(&state, &headers) {
		return r;
	}
	state.lock().unwrap().picker_deleted = true;
	Json(json!({})).into_response()
}

async fn picker_items(State(state): State<Shared>, headers: HeaderMap) -> Response {
	if let Err(r) = auth(&state, &headers) {
		return r;
	}
	let s = state.lock().unwrap();
	let items: Vec<Value> = s
		.picked
		.iter()
		.map(|(id, name, mime, _)| {
			json!({
				"id": id,
				"type": if mime.starts_with("video/") { "VIDEO" } else { "PHOTO" },
				"mediaFile": {"baseUrl": format!("{}/media/{id}", s.base_url), "mimeType": mime, "filename": name},
			})
		})
		.collect();
	Json(json!({"mediaItems": items})).into_response()
}

async fn media_download(
	State(state): State<Shared>,
	headers: HeaderMap,
	Path(tail): Path<String>,
) -> Response {
	if let Err(r) = auth(&state, &headers) {
		return r;
	}
	let (id, suffix) = tail.split_once('=').unwrap_or((&tail, ""));
	let s = state.lock().unwrap();
	let Some((_, _, mime, bytes)) = s.picked.iter().find(|(i, ..)| i == id) else {
		return StatusCode::NOT_FOUND.into_response();
	};
	let expected = if mime.starts_with("video/") {
		"dv"
	} else {
		"d"
	};
	if suffix != expected {
		return StatusCode::BAD_REQUEST.into_response();
	}
	bytes.clone().into_response()
}

impl FakeGoogle {
	/// Starts the fake on a random loopback port.
	pub async fn start(users: Vec<FakeUser>) -> Self {
		let state: Shared = Arc::new(Mutex::new(FakeState {
			users,
			..Default::default()
		}));
		let app = Router::new()
			.route("/token", post(token))
			.route("/revoke", post(revoke))
			.route("/v1/userinfo", get(userinfo))
			.route("/drive/v3/files", get(list_files).post(create_file))
			.route(
				"/drive/v3/files/{id}",
				get(get_file).patch(patch_file).delete(delete_file),
			)
			.route("/drive/v3/files/{id}/export", get(export_file))
			.route("/drive/v3/files/{id}/copy", post(copy_file))
			.route("/drive/v3/about", get(about))
			.route("/upload/drive/v3/files", post(upload_create))
			.route(
				"/upload/drive/v3/files/{id}",
				axum::routing::patch(upload_update),
			)
			.route("/upload-session/{sid}", put(upload_chunk))
			.route("/photos/v1/uploads", post(photos_upload))
			.route(
				"/photos/v1/mediaItems:batchCreate",
				post(photos_batch_create),
			)
			.route(
				"/photos/v1/albums",
				post(photos_create_album).get(photos_list_albums),
			)
			.route("/picker/v1/sessions", post(picker_create))
			.route(
				"/picker/v1/sessions/{id}",
				get(picker_get).merge(delete(picker_delete)),
			)
			.route("/picker/v1/mediaItems", get(picker_items))
			.route("/media/{tail}", get(media_download))
			.layer(axum::extract::DefaultBodyLimit::max(64 * 1024 * 1024))
			.with_state(state.clone());
		let listener = tokio::net::TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], 0)))
			.await
			.unwrap();
		let base_url = format!("http://{}", listener.local_addr().unwrap());
		state.lock().unwrap().base_url = base_url.clone();
		tokio::spawn(async move {
			axum::serve(listener, app).await.unwrap();
		});
		Self { state, base_url }
	}
}
