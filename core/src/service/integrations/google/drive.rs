//! # Google Drive volume backend
//!
//! `GoogleDriveBackend` exposes one account's "My Drive" as a `VolumeBackend`,
//! so the Explorer, the copy job and cloud sync treat it like any other volume.
//!
//! Drive addresses files by ID and lets a folder hold several items with the
//! same name, while WingDrive addresses by path. The backend lists each folder
//! and gives every child a unique display name: duplicates get " (2)", " (3)"
//! in creation order, and a "/" inside a name becomes "_". Google Docs, Sheets,
//! Slides and Drawings have no bytes of their own, so they appear with the
//! extension of the format they export to (".docx", ".xlsx", ".pptx", ".pdf")
//! and reading them exports. Forms, Sites, Maps and shortcuts cannot be
//! exported and are hidden. Listings are cached for `LISTING_TTL` and every
//! write invalidates the folders it touches.
//!
//! Uploads use Drive's resumable protocol in `TRANSFER_CHUNK_SIZE` chunks, so a
//! large file never sits in memory whole and a dropped connection resumes from
//! the last byte Google confirmed.

use std::collections::{HashMap, HashSet};
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};

use async_trait::async_trait;
use bytes::Bytes;
use serde::Deserialize;
use tokio::io::{AsyncReadExt, AsyncSeekExt};
use tokio::sync::Mutex;
use tracing::debug;

use super::error::{GoogleError, GoogleResult};
use super::tokens::{GoogleApi, QueryPairs};
use crate::ops::indexing::state::EntryKind;
use crate::volume::backend::{
	BackendType, CloudServiceType, RawDirEntry, RawMetadata, VolumeBackend, TRANSFER_CHUNK_SIZE,
};
use crate::volume::error::VolumeError;

/// Fields requested for every file so listings and lookups share one shape.
const FILE_FIELDS: &str =
	"id,name,mimeType,size,modifiedTime,createdTime,md5Checksum,webViewLink,version";

const FOLDER_MIME: &str = "application/vnd.google-apps.folder";

/// How long a folder listing is reused before Drive is asked again.
const LISTING_TTL: Duration = Duration::from_secs(30);

/// Page size for folder listings; Drive caps it at 1000.
const PAGE_SIZE: u32 = 1000;

/// A file or folder as Drive v3 describes it.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DriveFile {
	pub id: String,
	pub name: String,
	pub mime_type: String,
	#[serde(default)]
	pub size: Option<String>,
	#[serde(default)]
	pub modified_time: Option<String>,
	#[serde(default)]
	pub created_time: Option<String>,
	#[serde(default)]
	pub md5_checksum: Option<String>,
	#[serde(default)]
	pub web_view_link: Option<String>,
	#[serde(default)]
	pub version: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FileList {
	#[serde(default)]
	files: Vec<DriveFile>,
	#[serde(default)]
	next_page_token: Option<String>,
}

/// How WingDrive treats a Drive MIME type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriveKind {
	Folder,
	/// A regular file with downloadable bytes.
	Binary,
	/// A Google Workspace file read through export.
	Workspace {
		export_mime: &'static str,
		extension: &'static str,
	},
	/// Forms, Sites, shortcuts and other types with nothing to download.
	Hidden,
}

/// Classifies a Drive MIME type.
pub fn classify(mime_type: &str) -> DriveKind {
	match mime_type {
		FOLDER_MIME => DriveKind::Folder,
		"application/vnd.google-apps.document" => DriveKind::Workspace {
			export_mime: "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
			extension: "docx",
		},
		"application/vnd.google-apps.spreadsheet" => DriveKind::Workspace {
			export_mime: "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
			extension: "xlsx",
		},
		"application/vnd.google-apps.presentation" => DriveKind::Workspace {
			export_mime:
				"application/vnd.openxmlformats-officedocument.presentationml.presentation",
			extension: "pptx",
		},
		"application/vnd.google-apps.drawing" => DriveKind::Workspace {
			export_mime: "application/pdf",
			extension: "pdf",
		},
		other if other.starts_with("application/vnd.google-apps.") => DriveKind::Hidden,
		_ => DriveKind::Binary,
	}
}

/// A child of a folder with the unique name WingDrive shows for it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DriveNode {
	pub display_name: String,
	pub file: DriveFile,
}

impl DriveNode {
	pub fn kind(&self) -> DriveKind {
		classify(&self.file.mime_type)
	}

	pub fn is_folder(&self) -> bool {
		self.kind() == DriveKind::Folder
	}

	fn size(&self) -> u64 {
		self.file
			.size
			.as_deref()
			.and_then(|s| s.parse().ok())
			.unwrap_or(0)
	}

	fn revision(&self) -> Option<String> {
		self.file
			.md5_checksum
			.clone()
			.or_else(|| self.file.version.as_ref().map(|v| format!("v{v}")))
	}
}

fn parse_time(value: Option<&str>) -> Option<SystemTime> {
	value
		.and_then(|v| chrono::DateTime::parse_from_rfc3339(v).ok())
		.map(|t| SystemTime::from(t.with_timezone(&chrono::Utc)))
}

fn split_extension(name: &str) -> (&str, &str) {
	match name.rfind('.') {
		Some(index) if index > 0 => (&name[..index], &name[index..]),
		_ => (name, ""),
	}
}

/// Gives every listed file a unique, path-safe name.
///
/// Sorting by creation time and ID before deduplicating keeps the names
/// stable between listings, so a path keeps pointing at the same file.
pub fn display_names(files: Vec<DriveFile>) -> Vec<DriveNode> {
	let mut files: Vec<DriveFile> = files
		.into_iter()
		.filter(|f| classify(&f.mime_type) != DriveKind::Hidden)
		.collect();
	files.sort_by(|a, b| (&a.created_time, &a.id).cmp(&(&b.created_time, &b.id)));

	let mut used = HashSet::new();
	let mut nodes = Vec::with_capacity(files.len());
	for file in files {
		let mut base = file.name.replace('/', "_");
		if base.is_empty() || base == "." || base == ".." {
			base = "Untitled".to_string();
		}
		if let DriveKind::Workspace { extension, .. } = classify(&file.mime_type) {
			let suffix = format!(".{extension}");
			if !base.to_lowercase().ends_with(&suffix) {
				base.push_str(&suffix);
			}
		}
		let mut name = base.clone();
		let mut counter = 2;
		while used.contains(&name) {
			let (stem, ext) = split_extension(&base);
			name = format!("{stem} ({counter}){ext}");
			counter += 1;
		}
		used.insert(name.clone());
		nodes.push(DriveNode {
			display_name: name,
			file,
		});
	}
	nodes.sort_by(|a, b| a.display_name.cmp(&b.display_name));
	nodes
}

/// Splits a volume path into its components, ignoring empty parts and ".".
fn segments(path: &Path) -> Vec<String> {
	path.to_string_lossy()
		.split(['/', '\\'])
		.filter(|s| !s.is_empty() && *s != ".")
		.map(str::to_string)
		.collect()
}

/// Escapes a value for a single-quoted Drive query string.
fn query_literal(value: &str) -> String {
	value.replace('\\', "\\\\").replace('\'', "\\'")
}

/// What a path resolved to.
#[derive(Debug, Clone)]
enum Resolved {
	Root,
	Node(DriveNode),
}

impl Resolved {
	fn id<'a>(&'a self, root_id: &'a str) -> &'a str {
		match self {
			Self::Root => root_id,
			Self::Node(node) => &node.file.id,
		}
	}

	fn is_folder(&self) -> bool {
		match self {
			Self::Root => true,
			Self::Node(node) => node.is_folder(),
		}
	}
}

struct CachedListing {
	children: Arc<Vec<DriveNode>>,
	listed_at: Instant,
}

enum UploadSource<'a> {
	Bytes(&'a Bytes),
	File(&'a Path),
}

/// Google Drive as a WingDrive volume backend.
pub struct GoogleDriveBackend {
	api: GoogleApi,
	root_id: String,
	listings: Mutex<HashMap<String, CachedListing>>,
}

impl std::fmt::Debug for GoogleDriveBackend {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("GoogleDriveBackend")
			.field("root_id", &self.root_id)
			.finish_non_exhaustive()
	}
}

impl GoogleDriveBackend {
	/// Creates a backend rooted at "My Drive", or at the folder `root_id`.
	pub fn new(api: GoogleApi, root_id: Option<String>) -> Self {
		Self {
			api,
			root_id: root_id.unwrap_or_else(|| "root".to_string()),
			listings: Mutex::new(HashMap::new()),
		}
	}

	fn url(&self, suffix: &str) -> String {
		format!("{}/{suffix}", self.api.endpoints.drive)
	}

	fn upload_url(&self, suffix: &str) -> String {
		format!("{}/{suffix}", self.api.endpoints.drive_upload)
	}

	async fn list_folder(&self, folder_id: &str) -> GoogleResult<Vec<DriveFile>> {
		let query = format!(
			"'{}' in parents and trashed=false",
			query_literal(folder_id)
		);
		let fields = format!("nextPageToken,files({FILE_FIELDS})");
		let mut files = Vec::new();
		let mut page_token: Option<String> = None;
		loop {
			let url = self.url("files");
			let page: FileList = self
				.api
				.send_json(|http| {
					let mut request = http.get(&url).query_pairs(&[
						("q", query.as_str()),
						("fields", fields.as_str()),
						("pageSize", &PAGE_SIZE.to_string()),
						("spaces", "drive"),
						("supportsAllDrives", "true"),
						("includeItemsFromAllDrives", "true"),
					]);
					if let Some(token) = &page_token {
						request = request.query_pairs(&[("pageToken", token.as_str())]);
					}
					request
				})
				.await?;
			files.extend(page.files);
			match page.next_page_token {
				Some(token) if !token.is_empty() => page_token = Some(token),
				_ => break,
			}
		}
		Ok(files)
	}

	/// Children of a folder, from cache when fresh.
	async fn children(&self, key: &str, folder_id: &str) -> GoogleResult<Arc<Vec<DriveNode>>> {
		{
			let listings = self.listings.lock().await;
			if let Some(cached) = listings.get(key) {
				if cached.listed_at.elapsed() < LISTING_TTL {
					return Ok(cached.children.clone());
				}
			}
		}
		let children = Arc::new(display_names(self.list_folder(folder_id).await?));
		self.listings.lock().await.insert(
			key.to_string(),
			CachedListing {
				children: children.clone(),
				listed_at: Instant::now(),
			},
		);
		Ok(children)
	}

	/// Forgets the listing of `key` and of every folder below it.
	async fn invalidate(&self, key: &str) {
		let mut listings = self.listings.lock().await;
		if key.is_empty() {
			listings.clear();
			return;
		}
		let prefix = format!("{key}/");
		listings.retain(|k, _| k != key && !k.starts_with(&prefix));
	}

	async fn resolve(&self, parts: &[String]) -> GoogleResult<Option<Resolved>> {
		let mut current = Resolved::Root;
		let mut key = String::new();
		for part in parts {
			if !current.is_folder() {
				return Ok(None);
			}
			let children = self.children(&key, current.id(&self.root_id)).await?;
			let Some(node) = children.iter().find(|n| &n.display_name == part) else {
				return Ok(None);
			};
			current = Resolved::Node(node.clone());
			if !key.is_empty() {
				key.push('/');
			}
			key.push_str(part);
		}
		Ok(Some(current))
	}

	async fn resolve_existing(&self, path: &Path) -> GoogleResult<Resolved> {
		self.resolve(&segments(path))
			.await?
			.ok_or_else(|| GoogleError::NotFound(path.display().to_string()))
	}

	/// The folder that holds `path`, the cache key of that folder, and the last name.
	async fn parent_of(&self, path: &Path) -> GoogleResult<(String, String, String)> {
		let mut parts = segments(path);
		let name = parts
			.pop()
			.ok_or_else(|| GoogleError::Unsupported("the Drive root has no parent".to_string()))?;
		let parent = self
			.resolve(&parts)
			.await?
			.ok_or_else(|| GoogleError::NotFound(parts.join("/")))?;
		if !parent.is_folder() {
			return Err(GoogleError::Unsupported(format!(
				"{} is not a folder",
				parts.join("/")
			)));
		}
		Ok((parent.id(&self.root_id).to_string(), parts.join("/"), name))
	}

	async fn get_file(&self, id: &str) -> GoogleResult<DriveFile> {
		let url = self.url(&format!("files/{id}"));
		self.api
			.send_json(|http| {
				http.get(&url)
					.query_pairs(&[("fields", FILE_FIELDS), ("supportsAllDrives", "true")])
			})
			.await
	}

	async fn create_folder(&self, parent_id: &str, name: &str) -> GoogleResult<DriveFile> {
		let url = self.url("files");
		let body = serde_json::to_vec(&serde_json::json!({
			"name": name,
			"mimeType": FOLDER_MIME,
			"parents": [parent_id],
		}))?;
		self.api
			.send_json(|http| {
				http.post(&url)
					.query_pairs(&[("fields", FILE_FIELDS), ("supportsAllDrives", "true")])
					.header(reqwest::header::CONTENT_TYPE, "application/json")
					.body(body.clone())
			})
			.await
	}

	async fn download(&self, node: &DriveNode, range: Option<Range<u64>>) -> GoogleResult<Bytes> {
		match node.kind() {
			DriveKind::Binary => {
				let url = self.url(&format!("files/{}", node.file.id));
				let range_header = range
					.as_ref()
					.map(|r| format!("bytes={}-{}", r.start, r.end.saturating_sub(1)));
				let response = self
					.api
					.send(|http| {
						let mut request = http
							.get(&url)
							.query_pairs(&[("alt", "media"), ("supportsAllDrives", "true")]);
						if let Some(header) = &range_header {
							request = request.header(reqwest::header::RANGE, header);
						}
						request
					})
					.await?;
				let partial = response.status() == reqwest::StatusCode::PARTIAL_CONTENT;
				let bytes = response.bytes().await?;
				match range {
					// A server that ignores Range sends the whole file; slice it here.
					Some(r) if !partial => Ok(slice(bytes, r)),
					_ => Ok(bytes),
				}
			}
			DriveKind::Workspace { export_mime, .. } => {
				let url = self.url(&format!("files/{}/export", node.file.id));
				let response = self
					.api
					.send(|http| http.get(&url).query_pairs(&[("mimeType", export_mime)]))
					.await?;
				let bytes = response.bytes().await?;
				Ok(match range {
					Some(r) => slice(bytes, r),
					None => bytes,
				})
			}
			DriveKind::Folder => Err(GoogleError::Unsupported(format!(
				"{} is a folder",
				node.display_name
			))),
			DriveKind::Hidden => Err(GoogleError::Unsupported(format!(
				"{} cannot be downloaded",
				node.display_name
			))),
		}
	}

	/// Uploads new content, creating the file or replacing `existing_id`.
	async fn upload(
		&self,
		parent_id: &str,
		name: &str,
		existing_id: Option<&str>,
		total: u64,
		source: UploadSource<'_>,
	) -> GoogleResult<DriveFile> {
		let mime = mime_guess::from_path(name)
			.first_or_octet_stream()
			.essence_str()
			.to_string();
		let fields = FILE_FIELDS;
		let (url, method, metadata) = match existing_id {
			Some(id) => (
				self.upload_url(&format!("files/{id}")),
				reqwest::Method::PATCH,
				serde_json::json!({}),
			),
			None => (
				self.upload_url("files"),
				reqwest::Method::POST,
				serde_json::json!({ "name": name, "parents": [parent_id] }),
			),
		};
		let metadata = serde_json::to_vec(&metadata)?;
		let response = self
			.api
			.send(|http| {
				http.request(method.clone(), &url)
					.query_pairs(&[
						("uploadType", "resumable"),
						("fields", fields),
						("supportsAllDrives", "true"),
					])
					.header(
						reqwest::header::CONTENT_TYPE,
						"application/json; charset=UTF-8",
					)
					.header("X-Upload-Content-Type", mime.as_str())
					.header("X-Upload-Content-Length", total.to_string())
					.body(metadata.clone())
			})
			.await?;
		let session = response
			.headers()
			.get(reqwest::header::LOCATION)
			.and_then(|v| v.to_str().ok())
			.map(str::to_string)
			.ok_or_else(|| {
				GoogleError::InvalidResponse("resumable upload returned no session URL".to_string())
			})?;

		let mut file = match source {
			UploadSource::File(path) => Some(tokio::fs::File::open(path).await?),
			UploadSource::Bytes(_) => None,
		};
		let mut offset = 0u64;
		loop {
			let end = (offset + TRANSFER_CHUNK_SIZE).min(total);
			let chunk: Bytes = match (&source, file.as_mut()) {
				(UploadSource::Bytes(data), _) => data.slice(offset as usize..end as usize),
				(UploadSource::File(_), Some(file)) => {
					file.seek(std::io::SeekFrom::Start(offset)).await?;
					let mut buffer = vec![0u8; (end - offset) as usize];
					file.read_exact(&mut buffer).await?;
					Bytes::from(buffer)
				}
				(UploadSource::File(_), None) => unreachable!("file opened above"),
			};
			let content_range = if total == 0 {
				"bytes */0".to_string()
			} else {
				format!("bytes {}-{}/{}", offset, end - 1, total)
			};
			let response = self
				.api
				.send(|http| {
					http.put(&session)
						.header(reqwest::header::CONTENT_RANGE, content_range.as_str())
						.body(chunk.clone())
				})
				.await?;
			if response.status() == reqwest::StatusCode::PERMANENT_REDIRECT {
				// Google reports what it stored; resume from there, not from our offset.
				offset = response
					.headers()
					.get(reqwest::header::RANGE)
					.and_then(|v| v.to_str().ok())
					.and_then(|v| v.rsplit('-').next())
					.and_then(|v| v.parse::<u64>().ok())
					.map(|last| last + 1)
					.unwrap_or(0);
				continue;
			}
			let bytes = response.bytes().await?;
			return Ok(serde_json::from_slice(&bytes)?);
		}
	}

	async fn write_source(
		&self,
		path: &Path,
		total: u64,
		source: UploadSource<'_>,
	) -> GoogleResult<()> {
		let (parent_id, parent_key, name) = self.parent_of(path).await?;
		let existing = self
			.children(&parent_key, &parent_id)
			.await?
			.iter()
			.find(|n| n.display_name == name)
			.cloned();
		let existing_id = match &existing {
			Some(node) if node.kind() == DriveKind::Binary => Some(node.file.id.clone()),
			Some(node) if node.is_folder() => {
				return Err(GoogleError::Unsupported(format!("{name} is a folder")));
			}
			Some(_) => {
				return Err(GoogleError::Unsupported(format!(
					"{name} is a Google Docs, Sheets or Slides file and can only be edited in the browser"
				)));
			}
			None => None,
		};
		debug!(path = %path.display(), total, "Uploading to Google Drive");
		self.upload(&parent_id, &name, existing_id.as_deref(), total, source)
			.await?;
		self.invalidate(&parent_key).await;
		Ok(())
	}

	/// The Drive name to store when the user names a Workspace file with its export extension.
	fn drive_name(node: &DriveNode, display_name: &str) -> String {
		if let DriveKind::Workspace { extension, .. } = node.kind() {
			let suffix = format!(".{extension}");
			if let Some(stem) = display_name.strip_suffix(&suffix) {
				return stem.to_string();
			}
		}
		display_name.to_string()
	}

	/// Storage quota as `(limit, used)`; `limit` is absent for unlimited plans.
	pub async fn quota(&self) -> GoogleResult<(Option<u64>, u64)> {
		#[derive(Deserialize)]
		struct About {
			#[serde(rename = "storageQuota")]
			storage_quota: Quota,
		}
		#[derive(Deserialize)]
		struct Quota {
			#[serde(default)]
			limit: Option<String>,
			#[serde(default)]
			usage: Option<String>,
		}
		let url = self.url("about");
		let about: About = self
			.api
			.send_json(|http| http.get(&url).query_pairs(&[("fields", "storageQuota")]))
			.await?;
		let parse = |v: Option<String>| v.and_then(|s| s.parse::<u64>().ok());
		Ok((
			parse(about.storage_quota.limit),
			parse(about.storage_quota.usage).unwrap_or(0),
		))
	}
}

fn slice(bytes: Bytes, range: Range<u64>) -> Bytes {
	let len = bytes.len() as u64;
	let start = range.start.min(len) as usize;
	let end = range.end.min(len) as usize;
	bytes.slice(start..end.max(start))
}

fn to_entry_kind(node: &DriveNode) -> EntryKind {
	if node.is_folder() {
		EntryKind::Directory
	} else {
		EntryKind::File
	}
}

#[async_trait]
impl VolumeBackend for GoogleDriveBackend {
	async fn read(&self, path: &Path) -> Result<Bytes, VolumeError> {
		match self.resolve_existing(path).await? {
			Resolved::Root => Err(VolumeError::Unsupported(
				"the Drive root is a folder".into(),
			)),
			Resolved::Node(node) => Ok(self.download(&node, None).await?),
		}
	}

	async fn read_range(&self, path: &Path, range: Range<u64>) -> Result<Bytes, VolumeError> {
		match self.resolve_existing(path).await? {
			Resolved::Root => Err(VolumeError::Unsupported(
				"the Drive root is a folder".into(),
			)),
			Resolved::Node(node) => Ok(self.download(&node, Some(range)).await?),
		}
	}

	async fn write(&self, path: &Path, data: Bytes) -> Result<(), VolumeError> {
		let total = data.len() as u64;
		Ok(self
			.write_source(path, total, UploadSource::Bytes(&data))
			.await?)
	}

	async fn read_dir(&self, path: &Path) -> Result<Vec<RawDirEntry>, VolumeError> {
		let parts = segments(path);
		let folder = self.resolve_existing(path).await?;
		if !folder.is_folder() {
			return Err(VolumeError::Io(std::io::Error::new(
				std::io::ErrorKind::NotADirectory,
				format!("{} is not a folder", path.display()),
			)));
		}
		let children = self
			.children(&parts.join("/"), folder.id(&self.root_id))
			.await?;
		Ok(children
			.iter()
			.map(|node| RawDirEntry {
				name: node.display_name.clone(),
				kind: to_entry_kind(node),
				size: node.size(),
				modified: parse_time(node.file.modified_time.as_deref()),
				created: parse_time(node.file.created_time.as_deref()),
				inode: None,
				revision: node.revision(),
			})
			.collect())
	}

	async fn metadata(&self, path: &Path) -> Result<RawMetadata, VolumeError> {
		Ok(match self.resolve_existing(path).await? {
			Resolved::Root => RawMetadata {
				kind: EntryKind::Directory,
				size: 0,
				modified: None,
				created: None,
				accessed: None,
				inode: None,
				permissions: None,
				revision: None,
			},
			Resolved::Node(node) => RawMetadata {
				kind: to_entry_kind(&node),
				size: node.size(),
				modified: parse_time(node.file.modified_time.as_deref()),
				created: parse_time(node.file.created_time.as_deref()),
				accessed: None,
				inode: None,
				permissions: None,
				revision: node.revision(),
			},
		})
	}

	async fn exists(&self, path: &Path) -> Result<bool, VolumeError> {
		Ok(self.resolve(&segments(path)).await?.is_some())
	}

	async fn delete(&self, path: &Path) -> Result<(), VolumeError> {
		let Resolved::Node(node) = self.resolve_existing(path).await? else {
			return Err(VolumeError::Unsupported(
				"the Drive root cannot be deleted".into(),
			));
		};
		let url = self.url(&format!("files/{}", node.file.id));
		self.api
			.send(|http| {
				http.delete(&url)
					.query_pairs(&[("supportsAllDrives", "true")])
			})
			.await
			.map_err(GoogleError::from)?;
		let mut parts = segments(path);
		let own_key = parts.join("/");
		parts.pop();
		self.invalidate(&parts.join("/")).await;
		self.invalidate(&own_key).await;
		Ok(())
	}

	async fn create_directory(&self, path: &Path, recursive: bool) -> Result<(), VolumeError> {
		let parts = segments(path);
		let mut parent_id = self.root_id.clone();
		let mut key = String::new();
		for (index, part) in parts.iter().enumerate() {
			let is_last = index + 1 == parts.len();
			let children = self.children(&key, &parent_id).await?;
			let next_id = match children.iter().find(|n| &n.display_name == part) {
				Some(node) if node.is_folder() => node.file.id.clone(),
				Some(_) => {
					return Err(VolumeError::Io(std::io::Error::new(
						std::io::ErrorKind::AlreadyExists,
						format!("{part} exists and is not a folder"),
					)));
				}
				None if is_last || recursive => {
					let created = self.create_folder(&parent_id, part).await?;
					self.invalidate(&key).await;
					created.id
				}
				None => {
					return Err(VolumeError::Io(std::io::Error::new(
						std::io::ErrorKind::NotFound,
						format!("parent folder of {} does not exist", path.display()),
					)));
				}
			};
			parent_id = next_id;
			if !key.is_empty() {
				key.push('/');
			}
			key.push_str(part);
		}
		Ok(())
	}

	fn is_local(&self) -> bool {
		false
	}

	fn backend_type(&self) -> BackendType {
		BackendType::Cloud(CloudServiceType::GoogleDrive)
	}

	async fn rename(&self, from: &Path, to: &Path) -> Result<(), VolumeError> {
		let Resolved::Node(node) = self.resolve_existing(from).await? else {
			return Err(VolumeError::Unsupported(
				"the Drive root cannot be moved".into(),
			));
		};
		let (new_parent, new_parent_key, new_name) = self.parent_of(to).await?;
		if self.resolve(&segments(to)).await?.is_some() {
			return Err(VolumeError::Io(std::io::Error::new(
				std::io::ErrorKind::AlreadyExists,
				format!("{} already exists", to.display()),
			)));
		}
		let (old_parent, old_parent_key, _) = self.parent_of(from).await?;
		let url = self.url(&format!("files/{}", node.file.id));
		let body = serde_json::to_vec(&serde_json::json!({
			"name": Self::drive_name(&node, &new_name),
		}))
		.map_err(GoogleError::from)?;
		self.api
			.send(|http| {
				let mut request = http
					.patch(&url)
					.query_pairs(&[("fields", FILE_FIELDS), ("supportsAllDrives", "true")]);
				if new_parent != old_parent {
					request = request.query_pairs(&[
						("addParents", new_parent.as_str()),
						("removeParents", old_parent.as_str()),
					]);
				}
				request
					.header(reqwest::header::CONTENT_TYPE, "application/json")
					.body(body.clone())
			})
			.await
			.map_err(GoogleError::from)?;
		self.invalidate(&old_parent_key).await;
		self.invalidate(&new_parent_key).await;
		self.invalidate(&segments(from).join("/")).await;
		Ok(())
	}

	async fn copy(&self, from: &Path, to: &Path) -> Result<(), VolumeError> {
		let Resolved::Node(node) = self.resolve_existing(from).await? else {
			return Err(VolumeError::Unsupported(
				"the Drive root cannot be copied".into(),
			));
		};
		if node.is_folder() {
			// Drive has no folder copy; the caller recreates the tree file by file.
			return Err(VolumeError::Unsupported(
				"Drive cannot copy folders server side".into(),
			));
		}
		let (parent_id, parent_key, name) = self.parent_of(to).await?;
		let url = self.url(&format!("files/{}/copy", node.file.id));
		let body = serde_json::to_vec(&serde_json::json!({
			"name": Self::drive_name(&node, &name),
			"parents": [parent_id],
		}))
		.map_err(GoogleError::from)?;
		self.api
			.send(|http| {
				http.post(&url)
					.query_pairs(&[("fields", FILE_FIELDS), ("supportsAllDrives", "true")])
					.header(reqwest::header::CONTENT_TYPE, "application/json")
					.body(body.clone())
			})
			.await
			.map_err(GoogleError::from)?;
		self.invalidate(&parent_key).await;
		Ok(())
	}

	async fn trash(&self, path: &Path) -> Result<(), VolumeError> {
		let Resolved::Node(node) = self.resolve_existing(path).await? else {
			return Err(VolumeError::Unsupported(
				"the Drive root cannot be trashed".into(),
			));
		};
		let url = self.url(&format!("files/{}", node.file.id));
		let body = serde_json::to_vec(&serde_json::json!({ "trashed": true }))
			.map_err(GoogleError::from)?;
		self.api
			.send(|http| {
				http.patch(&url)
					.query_pairs(&[("supportsAllDrives", "true")])
					.header(reqwest::header::CONTENT_TYPE, "application/json")
					.body(body.clone())
			})
			.await
			.map_err(GoogleError::from)?;
		let mut parts = segments(path);
		let own_key = parts.join("/");
		parts.pop();
		self.invalidate(&parts.join("/")).await;
		self.invalidate(&own_key).await;
		Ok(())
	}

	async fn write_from_file(&self, path: &Path, local: &Path) -> Result<u64, VolumeError> {
		let total = tokio::fs::metadata(local).await?.len();
		self.write_source(path, total, UploadSource::File(local))
			.await?;
		Ok(total)
	}

	async fn web_link(&self, path: &Path) -> Result<Option<String>, VolumeError> {
		Ok(match self.resolve_existing(path).await? {
			Resolved::Root => Some("https://drive.google.com/drive/my-drive".to_string()),
			Resolved::Node(node) if node.is_folder() => Some(format!(
				"https://drive.google.com/drive/folders/{}",
				node.file.id
			)),
			Resolved::Node(node) => match node.file.web_view_link.clone() {
				Some(link) => Some(link),
				None => self.get_file(&node.file.id).await?.web_view_link,
			},
		})
	}

	async fn edits_in_browser(&self, path: &Path) -> Result<bool, VolumeError> {
		Ok(matches!(
			self.resolve_existing(path).await?,
			Resolved::Node(node) if matches!(node.kind(), DriveKind::Workspace { .. })
		))
	}

	async fn capacity(&self) -> Result<Option<(u64, u64)>, VolumeError> {
		let (limit, used) = self.quota().await?;
		Ok(limit.map(|limit| (limit, limit.saturating_sub(used))))
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn file(id: &str, name: &str, mime: &str, created: &str) -> DriveFile {
		DriveFile {
			id: id.into(),
			name: name.into(),
			mime_type: mime.into(),
			size: None,
			modified_time: None,
			created_time: Some(created.into()),
			md5_checksum: None,
			web_view_link: None,
			version: None,
		}
	}

	#[test]
	fn classify_maps_workspace_types_to_export_formats() {
		assert_eq!(classify(FOLDER_MIME), DriveKind::Folder);
		assert_eq!(classify("image/jpeg"), DriveKind::Binary);
		assert!(matches!(
			classify("application/vnd.google-apps.spreadsheet"),
			DriveKind::Workspace {
				extension: "xlsx",
				..
			}
		));
		assert_eq!(
			classify("application/vnd.google-apps.form"),
			DriveKind::Hidden
		);
		assert_eq!(
			classify("application/vnd.google-apps.shortcut"),
			DriveKind::Hidden
		);
	}

	#[test]
	fn display_names_are_unique_stable_and_path_safe() {
		let nodes = display_names(vec![
			file("b", "report.pdf", "application/pdf", "2026-01-02T00:00:00Z"),
			file("a", "report.pdf", "application/pdf", "2026-01-01T00:00:00Z"),
			file("c", "a/b", "text/plain", "2026-01-01T00:00:00Z"),
			file(
				"d",
				"Plan",
				"application/vnd.google-apps.document",
				"2026-01-01T00:00:00Z",
			),
			file(
				"e",
				"Form",
				"application/vnd.google-apps.form",
				"2026-01-01T00:00:00Z",
			),
			file("f", "", "text/plain", "2026-01-01T00:00:00Z"),
		]);
		let names: Vec<(&str, &str)> = nodes
			.iter()
			.map(|n| (n.display_name.as_str(), n.file.id.as_str()))
			.collect();
		assert!(
			names.contains(&("report.pdf", "a")),
			"oldest keeps the name"
		);
		assert!(names.contains(&("report (2).pdf", "b")));
		assert!(names.contains(&("a_b", "c")));
		assert!(names.contains(&("Plan.docx", "d")));
		assert!(names.contains(&("Untitled", "f")));
		assert!(!names.iter().any(|(_, id)| *id == "e"), "forms are hidden");
	}

	#[test]
	fn segments_ignore_empty_parts() {
		assert_eq!(segments(Path::new("/a//b/./c/")), vec!["a", "b", "c"]);
		assert!(segments(Path::new("")).is_empty());
		assert!(segments(Path::new("/")).is_empty());
	}

	#[test]
	fn query_literals_escape_quotes() {
		assert_eq!(query_literal("it's"), "it\\'s");
		assert_eq!(query_literal("a\\b"), "a\\\\b");
	}

	#[test]
	fn workspace_rename_strips_the_export_extension() {
		let node = DriveNode {
			display_name: "Plan.docx".into(),
			file: file("d", "Plan", "application/vnd.google-apps.document", "x"),
		};
		assert_eq!(
			GoogleDriveBackend::drive_name(&node, "Budget.docx"),
			"Budget"
		);
		let binary = DriveNode {
			display_name: "a.docx".into(),
			file: file("x", "a.docx", "application/octet-stream", "x"),
		};
		assert_eq!(GoogleDriveBackend::drive_name(&binary, "b.docx"), "b.docx");
	}
}
