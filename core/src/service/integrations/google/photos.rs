//! # Google Photos client
//!
//! Google narrowed the Photos Library API on 2025-03-31: an app can upload,
//! create albums and read back only the media and albums it created itself.
//! Reading the rest of a library goes through the Photos Picker API, where the
//! user selects items on a Google page and the app downloads that selection.
//!
//! WingDrive therefore offers three operations. Upload sends local photos and
//! videos to the library, optionally into an album WingDrive created. Albums
//! lists those albums. Import opens a picker session, waits until the user
//! finishes selecting, and downloads the chosen items at full quality.

use std::path::Path;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use specta::Type;
use tokio::io::AsyncWriteExt;
use tracing::debug;

use super::error::{GoogleError, GoogleResult};
use super::tokens::{GoogleApi, QueryPairs};

/// Photos accepts at most 50 items per `mediaItems:batchCreate` call.
pub const BATCH_CREATE_LIMIT: usize = 50;

/// An album WingDrive created in Google Photos.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PhotosAlbum {
	pub id: String,
	#[serde(default)]
	pub title: String,
	#[serde(default)]
	pub product_url: Option<String>,
	#[serde(default)]
	pub media_items_count: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AlbumList {
	#[serde(default)]
	albums: Vec<PhotosAlbum>,
	#[serde(default)]
	next_page_token: Option<String>,
}

/// Outcome of creating one media item from an upload token.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatedMediaItem {
	pub file_name: String,
	/// The new item's ID, or `None` when Google rejected it.
	pub media_item_id: Option<String>,
	pub error: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BatchCreateResponse {
	#[serde(default)]
	new_media_item_results: Vec<NewMediaItemResult>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct NewMediaItemResult {
	#[serde(default)]
	upload_token: Option<String>,
	#[serde(default)]
	status: Option<ResultStatus>,
	#[serde(default)]
	media_item: Option<MediaItemRef>,
}

#[derive(Debug, Deserialize)]
struct ResultStatus {
	#[serde(default)]
	code: Option<i32>,
	#[serde(default)]
	message: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MediaItemRef {
	id: String,
}

/// A picker session the user completes in the browser.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PickerSession {
	pub id: String,
	/// Page where the user selects photos.
	#[serde(default)]
	pub picker_uri: String,
	/// Set once the user pressed "Done" in the picker.
	#[serde(default)]
	pub media_items_set: bool,
	#[serde(default)]
	pub expire_time: Option<String>,
	#[serde(default)]
	pub polling_config: Option<PollingConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PollingConfig {
	/// Suggested wait between status checks, such as "5s".
	#[serde(default)]
	pub poll_interval: Option<String>,
}

impl PickerSession {
	/// The poll interval Google suggests, clamped to 1 to 30 seconds.
	pub fn poll_interval(&self) -> Duration {
		let seconds = self
			.polling_config
			.as_ref()
			.and_then(|c| c.poll_interval.as_deref())
			.and_then(|v| v.trim_end_matches('s').parse::<f64>().ok())
			.unwrap_or(5.0);
		Duration::from_secs_f64(seconds.clamp(1.0, 30.0))
	}
}

/// An item the user selected in the picker.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PickedMediaItem {
	pub id: String,
	#[serde(default)]
	pub create_time: Option<String>,
	/// "PHOTO" or "VIDEO".
	#[serde(default, rename = "type")]
	pub kind: Option<String>,
	pub media_file: PickedMediaFile,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PickedMediaFile {
	pub base_url: String,
	#[serde(default)]
	pub mime_type: Option<String>,
	#[serde(default)]
	pub filename: Option<String>,
}

impl PickedMediaItem {
	/// URL for the original bytes: "=d" for photos, "=dv" for videos.
	pub fn download_url(&self) -> String {
		let is_video = self.kind.as_deref() == Some("VIDEO")
			|| self
				.media_file
				.mime_type
				.as_deref()
				.is_some_and(|m| m.starts_with("video/"));
		format!(
			"{}={}",
			self.media_file.base_url,
			if is_video { "dv" } else { "d" }
		)
	}

	/// A file name safe to create locally.
	pub fn file_name(&self) -> String {
		let name = self
			.media_file
			.filename
			.clone()
			.filter(|n| !n.trim().is_empty())
			.unwrap_or_else(|| format!("{}.jpg", self.id));
		name.replace(['/', '\\', '\0'], "_")
	}
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PickedList {
	#[serde(default)]
	media_items: Vec<PickedMediaItem>,
	#[serde(default)]
	next_page_token: Option<String>,
}

/// Google Photos for one account.
#[derive(Debug, Clone)]
pub struct PhotosClient {
	api: GoogleApi,
}

impl PhotosClient {
	pub fn new(api: GoogleApi) -> Self {
		Self { api }
	}

	fn library(&self, suffix: &str) -> String {
		format!("{}/v1/{suffix}", self.api.endpoints.photos)
	}

	fn picker(&self, suffix: &str) -> String {
		format!("{}/v1/{suffix}", self.api.endpoints.picker)
	}

	/// Uploads the bytes of a local file and returns its upload token.
	///
	/// The file is streamed from disk, so videos up to Google's 20 GB limit do
	/// not need to fit in memory. A token is valid for one day.
	pub async fn upload_file(&self, path: &Path) -> GoogleResult<String> {
		let file_name = path
			.file_name()
			.map(|n| n.to_string_lossy().into_owned())
			.unwrap_or_else(|| "upload".to_string());
		let mime = mime_guess::from_path(path)
			.first_or_octet_stream()
			.essence_str()
			.to_string();
		let url = self.library("uploads");
		let size = tokio::fs::metadata(path).await?.len();
		debug!(file = %file_name, size, "Uploading to Google Photos");
		// The body is a stream, so each retry reopens the file.
		let path = path.to_path_buf();
		let response = {
			let mut attempt = 0;
			loop {
				attempt += 1;
				let file = tokio::fs::File::open(&path).await?;
				let body = reqwest::Body::wrap_stream(tokio_util::io::ReaderStream::new(file));
				let token = self.api.tokens.access_token().await?;
				let result = self
					.api
					.http
					.post(&url)
					.bearer_auth(token)
					.header(reqwest::header::CONTENT_TYPE, "application/octet-stream")
					.header(reqwest::header::CONTENT_LENGTH, size)
					.header("X-Goog-Upload-Content-Type", mime.as_str())
					.header("X-Goog-Upload-Protocol", "raw")
					.header("X-Goog-Upload-File-Name", file_name.as_str())
					.body(body)
					.send()
					.await;
				match result {
					Ok(response) if response.status().is_success() => break response,
					Ok(response)
						if response.status() == reqwest::StatusCode::UNAUTHORIZED
							&& attempt == 1 =>
					{
						self.api.tokens.invalidate().await;
					}
					Ok(response)
						if (response.status().is_server_error()
							|| response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS)
							&& attempt < 3 =>
					{
						tokio::time::sleep(Duration::from_secs(attempt)).await;
					}
					Ok(response) => {
						let status = response.status();
						let body = response.text().await.unwrap_or_default();
						return Err(super::tokens::api_error(status, &body));
					}
					Err(error) if attempt < 3 => {
						debug!(%error, "Photos upload failed, retrying");
						tokio::time::sleep(Duration::from_secs(attempt)).await;
					}
					Err(error) => return Err(error.into()),
				}
			}
		};
		let token = response.text().await?;
		if token.trim().is_empty() {
			return Err(GoogleError::InvalidResponse(
				"Photos upload returned an empty token".to_string(),
			));
		}
		Ok(token.trim().to_string())
	}

	/// Turns upload tokens into media items, at most `BATCH_CREATE_LIMIT` per call.
	///
	/// `items` pairs each token with the file name shown in Photos. Items that
	/// Google rejects come back with an error instead of failing the batch.
	pub async fn create_media_items(
		&self,
		items: &[(String, String)],
		album_id: Option<&str>,
	) -> GoogleResult<Vec<CreatedMediaItem>> {
		let mut results = Vec::with_capacity(items.len());
		for batch in items.chunks(BATCH_CREATE_LIMIT) {
			let mut body = serde_json::json!({
				"newMediaItems": batch.iter().map(|(token, name)| serde_json::json!({
					"simpleMediaItem": { "uploadToken": token, "fileName": name },
				})).collect::<Vec<_>>(),
			});
			if let Some(album) = album_id {
				body["albumId"] = serde_json::Value::String(album.to_string());
			}
			let body = serde_json::to_vec(&body)?;
			let url = self.library("mediaItems:batchCreate");
			let response: BatchCreateResponse = self
				.api
				.send_json(|http| {
					http.post(&url)
						.header(reqwest::header::CONTENT_TYPE, "application/json")
						.body(body.clone())
				})
				.await?;
			for (token, name) in batch {
				let result = response
					.new_media_item_results
					.iter()
					.find(|r| r.upload_token.as_deref() == Some(token.as_str()));
				let created = match result {
					Some(NewMediaItemResult {
						media_item: Some(item),
						..
					}) => CreatedMediaItem {
						file_name: name.clone(),
						media_item_id: Some(item.id.clone()),
						error: None,
					},
					Some(NewMediaItemResult {
						status: Some(status),
						..
					}) if status.code.unwrap_or(0) == 0 => CreatedMediaItem {
						file_name: name.clone(),
						media_item_id: None,
						error: None,
					},
					Some(NewMediaItemResult { status, .. }) => CreatedMediaItem {
						file_name: name.clone(),
						media_item_id: None,
						error: Some(
							status
								.as_ref()
								.and_then(|s| s.message.clone())
								.unwrap_or_else(|| "rejected by Google Photos".to_string()),
						),
					},
					None => CreatedMediaItem {
						file_name: name.clone(),
						media_item_id: None,
						error: Some("missing from the Google Photos response".to_string()),
					},
				};
				results.push(created);
			}
		}
		Ok(results)
	}

	/// Creates an album owned by this app.
	pub async fn create_album(&self, title: &str) -> GoogleResult<PhotosAlbum> {
		let url = self.library("albums");
		let body = serde_json::to_vec(&serde_json::json!({ "album": { "title": title } }))?;
		self.api
			.send_json(|http| {
				http.post(&url)
					.header(reqwest::header::CONTENT_TYPE, "application/json")
					.body(body.clone())
			})
			.await
	}

	/// Lists the albums this app created.
	pub async fn list_albums(&self) -> GoogleResult<Vec<PhotosAlbum>> {
		let url = self.library("albums");
		let mut albums = Vec::new();
		let mut page_token: Option<String> = None;
		loop {
			let page: AlbumList = self
				.api
				.send_json(|http| {
					let mut request = http
						.get(&url)
						.query_pairs(&[("pageSize", "50"), ("excludeNonAppCreatedData", "true")]);
					if let Some(token) = &page_token {
						request = request.query_pairs(&[("pageToken", token.as_str())]);
					}
					request
				})
				.await?;
			albums.extend(page.albums);
			match page.next_page_token {
				Some(token) if !token.is_empty() => page_token = Some(token),
				_ => break,
			}
		}
		Ok(albums)
	}

	/// Starts a picker session; send the user to `picker_uri`.
	pub async fn create_picker_session(&self) -> GoogleResult<PickerSession> {
		let url = self.picker("sessions");
		self.api
			.send_json(|http| {
				http.post(&url)
					.header(reqwest::header::CONTENT_TYPE, "application/json")
					.body("{}")
			})
			.await
	}

	/// Reads a picker session to see whether the user finished selecting.
	pub async fn get_picker_session(&self, session_id: &str) -> GoogleResult<PickerSession> {
		let url = self.picker(&format!("sessions/{session_id}"));
		self.api.send_json(|http| http.get(&url)).await
	}

	/// Lists every item the user selected in a finished session.
	pub async fn list_picked_items(&self, session_id: &str) -> GoogleResult<Vec<PickedMediaItem>> {
		let url = self.picker("mediaItems");
		let mut items = Vec::new();
		let mut page_token: Option<String> = None;
		loop {
			let page: PickedList = self
				.api
				.send_json(|http| {
					let mut request = http
						.get(&url)
						.query_pairs(&[("sessionId", session_id), ("pageSize", "100")]);
					if let Some(token) = &page_token {
						request = request.query_pairs(&[("pageToken", token.as_str())]);
					}
					request
				})
				.await?;
			items.extend(page.media_items);
			match page.next_page_token {
				Some(token) if !token.is_empty() => page_token = Some(token),
				_ => break,
			}
		}
		Ok(items)
	}

	/// Deletes a session so Google can release the selection.
	pub async fn delete_picker_session(&self, session_id: &str) -> GoogleResult<()> {
		let url = self.picker(&format!("sessions/{session_id}"));
		self.api.send(|http| http.delete(&url)).await?;
		Ok(())
	}

	/// Downloads a picked item to `destination`, writing through a temporary
	/// file so a failed download leaves nothing under the final name.
	pub async fn download_picked_item(
		&self,
		item: &PickedMediaItem,
		destination: &Path,
	) -> GoogleResult<u64> {
		use futures::StreamExt;

		let url = item.download_url();
		let response = self.api.send(|http| http.get(&url)).await?;
		let partial = crate::volume::backend::partial_download_path(destination);
		let mut file = tokio::fs::File::create(&partial).await?;
		let mut written = 0u64;
		let mut stream = response.bytes_stream();
		let result: GoogleResult<()> = async {
			while let Some(chunk) = stream.next().await {
				let chunk = chunk?;
				file.write_all(&chunk).await?;
				written += chunk.len() as u64;
			}
			file.flush().await?;
			file.sync_all().await?;
			Ok(())
		}
		.await;
		drop(file);
		if let Err(error) = result {
			let _ = tokio::fs::remove_file(&partial).await;
			return Err(error);
		}
		tokio::fs::rename(&partial, destination).await?;
		Ok(written)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn item(kind: &str, mime: &str, filename: Option<&str>) -> PickedMediaItem {
		PickedMediaItem {
			id: "abc".into(),
			create_time: None,
			kind: Some(kind.into()),
			media_file: PickedMediaFile {
				base_url: "https://lh3.example/x".into(),
				mime_type: Some(mime.into()),
				filename: filename.map(str::to_string),
			},
		}
	}

	#[test]
	fn download_url_uses_original_quality_suffix() {
		assert_eq!(
			item("PHOTO", "image/jpeg", None).download_url(),
			"https://lh3.example/x=d"
		);
		assert_eq!(
			item("VIDEO", "video/mp4", None).download_url(),
			"https://lh3.example/x=dv"
		);
	}

	#[test]
	fn picked_file_names_are_safe() {
		assert_eq!(
			item("PHOTO", "image/jpeg", Some("a/b.jpg")).file_name(),
			"a_b.jpg"
		);
		assert_eq!(
			item("PHOTO", "image/jpeg", Some(" ")).file_name(),
			"abc.jpg"
		);
	}

	#[test]
	fn poll_interval_is_parsed_and_clamped() {
		let mut session = PickerSession {
			id: "s".into(),
			picker_uri: String::new(),
			media_items_set: false,
			expire_time: None,
			polling_config: Some(PollingConfig {
				poll_interval: Some("2.5s".into()),
			}),
		};
		assert_eq!(session.poll_interval(), Duration::from_millis(2500));
		session.polling_config = Some(PollingConfig {
			poll_interval: Some("0s".into()),
		});
		assert_eq!(session.poll_interval(), Duration::from_secs(1));
		session.polling_config = None;
		assert_eq!(session.poll_interval(), Duration::from_secs(5));
	}
}
