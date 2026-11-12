//! Google Photos operations and jobs: albums, upload and picker import.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use specta::Type;

use crate::context::CoreContext;
use crate::domain::addressing::WingPath;
use crate::infra::action::{error::ActionError, LibraryAction};
use crate::infra::job::prelude::*;
use crate::infra::query::{LibraryQuery, QueryResult};
use crate::library::Library;
use crate::service::integrations::google::photos::{PhotosAlbum, PickedMediaItem};

/// Lists the albums WingDrive created in an account.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PhotosAlbumsInput {
	pub account_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhotosAlbumsQuery {
	input: PhotosAlbumsInput,
}

impl LibraryQuery for PhotosAlbumsQuery {
	type Input = PhotosAlbumsInput;
	type Output = Vec<PhotosAlbum>;

	fn from_input(input: Self::Input) -> QueryResult<Self> {
		Ok(Self { input })
	}

	async fn execute(
		self,
		context: Arc<CoreContext>,
		session: crate::infra::api::SessionContext,
	) -> QueryResult<Self::Output> {
		let library = super::session_library(&context, &session).await?;
		let client = context
			.integrations
			.photos_client(&library, &self.input.account_id)
			.await?;
		Ok(client
			.list_albums()
			.await
			.map_err(crate::service::integrations::IntegrationError::from)?)
	}
}

crate::register_library_query!(PhotosAlbumsQuery, "integrations.google.photos.albums");

/// Media types Google Photos accepts, by extension.
const PHOTO_EXTENSIONS: &[&str] = &[
	"jpg", "jpeg", "png", "gif", "webp", "heic", "heif", "avif", "bmp", "ico", "tif", "tiff",
	"raw", "dng", "cr2", "cr3", "nef", "arw", "orf", "rw2", "raf", "srw",
];
const VIDEO_EXTENSIONS: &[&str] = &[
	"mp4", "mov", "m4v", "3gp", "3g2", "avi", "mkv", "mpg", "mpeg", "mts", "m2ts", "wmv", "asf",
	"divx", "mmv", "mod", "tod", "webm",
];

/// Whether Google Photos accepts a file with this name.
pub fn is_photos_media(path: &Path) -> bool {
	path.extension()
		.map(|e| e.to_string_lossy().to_lowercase())
		.is_some_and(|ext| {
			PHOTO_EXTENSIONS.contains(&ext.as_str()) || VIDEO_EXTENSIONS.contains(&ext.as_str())
		})
}

/// Uploads local photos and videos to Google Photos.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PhotosUploadInput {
	pub account_id: String,
	/// Local files or folders; folders are walked recursively.
	pub paths: Vec<WingPath>,
	/// Add the uploads to this existing WingDrive album.
	pub album_id: Option<String>,
	/// Or create a new album with this title.
	pub new_album_title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhotosUploadAction {
	input: PhotosUploadInput,
}

impl LibraryAction for PhotosUploadAction {
	type Input = PhotosUploadInput;
	type Output = crate::infra::job::handle::JobReceipt;

	fn from_input(input: Self::Input) -> Result<Self, String> {
		if input.paths.is_empty() {
			return Err("Choose at least one file to upload".to_string());
		}
		if input.paths.iter().any(|p| p.as_local_path().is_none()) {
			return Err("Only files on this device can be uploaded to Google Photos".to_string());
		}
		Ok(Self { input })
	}

	async fn execute(
		self,
		library: Arc<Library>,
		context: Arc<CoreContext>,
	) -> Result<Self::Output, ActionError> {
		let client = context
			.integrations
			.photos_client(&library, &self.input.account_id)
			.await?;
		let album_id = match (&self.input.album_id, &self.input.new_album_title) {
			(Some(id), _) => Some(id.clone()),
			(None, Some(title)) if !title.trim().is_empty() => Some(
				client
					.create_album(title.trim())
					.await
					.map_err(crate::service::integrations::IntegrationError::from)?
					.id,
			),
			_ => None,
		};
		let paths = self
			.input
			.paths
			.iter()
			.filter_map(|p| p.as_local_path().map(Path::to_path_buf))
			.collect();
		let job = PhotosUploadJob {
			account_id: self.input.account_id,
			paths,
			album_id,
			done: Vec::new(),
		};
		let handle = library
			.jobs()
			.dispatch(job)
			.await
			.map_err(ActionError::Job)?;
		Ok(handle.into())
	}

	fn action_kind(&self) -> &'static str {
		"integrations.google.photos.upload"
	}
}

crate::register_library_action!(PhotosUploadAction, "integrations.google.photos.upload");

/// Result of a Photos upload or import.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Type)]
pub struct PhotosTransferReport {
	pub transferred: u32,
	pub skipped: u32,
	pub failed: u32,
	pub bytes: u64,
	pub errors: Vec<String>,
}

impl From<PhotosTransferReport> for JobOutput {
	fn from(report: PhotosTransferReport) -> Self {
		JobOutput::Custom(serde_json::to_value(report).unwrap_or_default())
	}
}

/// Uploads files to Google Photos in batches of 50.
///
/// Resumable: `done` records every file whose media item exists, so a resumed
/// job does not create duplicates.
#[derive(Debug, Serialize, Deserialize, Job)]
pub struct PhotosUploadJob {
	pub account_id: String,
	pub paths: Vec<PathBuf>,
	pub album_id: Option<String>,
	#[serde(default)]
	pub done: Vec<PathBuf>,
}

impl Job for PhotosUploadJob {
	const NAME: &'static str = "google_photos_upload";
	const RESUMABLE: bool = true;
	const DESCRIPTION: Option<&'static str> = Some("Upload photos and videos to Google Photos");
}

impl crate::infra::job::traits::DynJob for PhotosUploadJob {
	fn job_name(&self) -> &'static str {
		Self::NAME
	}
}

async fn collect_media(paths: &[PathBuf]) -> (Vec<PathBuf>, u32) {
	let mut files = Vec::new();
	let mut skipped = 0;
	let mut stack: Vec<PathBuf> = paths.to_vec();
	while let Some(path) = stack.pop() {
		let Ok(metadata) = tokio::fs::symlink_metadata(&path).await else {
			skipped += 1;
			continue;
		};
		if metadata.is_dir() {
			if let Ok(mut entries) = tokio::fs::read_dir(&path).await {
				while let Ok(Some(entry)) = entries.next_entry().await {
					stack.push(entry.path());
				}
			}
		} else if metadata.is_file() && is_photos_media(&path) {
			files.push(path);
		} else {
			skipped += 1;
		}
	}
	files.sort();
	(files, skipped)
}

#[async_trait::async_trait]
impl JobHandler for PhotosUploadJob {
	type Output = PhotosTransferReport;

	async fn run(&mut self, ctx: JobContext<'_>) -> JobResult<Self::Output> {
		let library = ctx.library_arc();
		let client = library
			.core_context()
			.integrations
			.photos_client(&library, &self.account_id)
			.await
			.map_err(|e| JobError::execution(e.to_string()))?;
		let (files, skipped) = collect_media(&self.paths).await;
		let pending: Vec<PathBuf> = files
			.iter()
			.filter(|f| !self.done.contains(f))
			.cloned()
			.collect();
		let mut report = PhotosTransferReport {
			skipped,
			transferred: (files.len() - pending.len()) as u32,
			..Default::default()
		};
		ctx.log(format!(
			"Uploading {} files to Google Photos ({} not photos or videos)",
			pending.len(),
			skipped
		));
		let total = files.len();
		for batch in
			pending.chunks(crate::service::integrations::google::photos::BATCH_CREATE_LIMIT)
		{
			ctx.check_interrupt().await?;
			let mut tokens = Vec::new();
			let mut token_files = Vec::new();
			for file in batch {
				ctx.check_interrupt().await?;
				match client.upload_file(file).await {
					Ok(token) => {
						let name = file
							.file_name()
							.map(|n| n.to_string_lossy().into_owned())
							.unwrap_or_default();
						report.bytes += tokio::fs::metadata(file)
							.await
							.map(|m| m.len())
							.unwrap_or(0);
						tokens.push((token, name));
						token_files.push(file.clone());
					}
					Err(error) => {
						report.failed += 1;
						report.errors.push(format!("{}: {error}", file.display()));
						ctx.add_warning(format!("{}: {error}", file.display()));
					}
				}
			}
			if tokens.is_empty() {
				continue;
			}
			let created = client
				.create_media_items(&tokens, self.album_id.as_deref())
				.await
				.map_err(|e| JobError::execution(e.to_string()))?;
			for (result, file) in created.iter().zip(token_files.iter()) {
				match &result.error {
					None => {
						report.transferred += 1;
						self.done.push(file.clone());
					}
					Some(error) => {
						report.failed += 1;
						report.errors.push(format!("{}: {error}", file.display()));
						ctx.add_warning(format!("{}: {error}", file.display()));
					}
				}
			}
			ctx.progress(Progress::Count {
				current: report.transferred as usize + report.failed as usize,
				total,
			});
			ctx.checkpoint().await?;
		}
		ctx.log(format!(
			"Google Photos upload done: {} uploaded, {} failed",
			report.transferred, report.failed
		));
		report.errors.truncate(20);
		Ok(report)
	}
}

/// Starts a Photos picker session. Open `picker_uri` in the browser.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PhotosPickerStartInput {
	pub account_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PhotosPickerStartOutput {
	pub session_id: String,
	pub picker_uri: String,
	/// Seconds the UI should wait between status checks.
	pub poll_interval_seconds: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhotosPickerStartAction {
	input: PhotosPickerStartInput,
}

impl LibraryAction for PhotosPickerStartAction {
	type Input = PhotosPickerStartInput;
	type Output = PhotosPickerStartOutput;

	fn from_input(input: Self::Input) -> Result<Self, String> {
		Ok(Self { input })
	}

	async fn execute(
		self,
		library: Arc<Library>,
		context: Arc<CoreContext>,
	) -> Result<Self::Output, ActionError> {
		let client = context
			.integrations
			.photos_client(&library, &self.input.account_id)
			.await?;
		let session = client
			.create_picker_session()
			.await
			.map_err(crate::service::integrations::IntegrationError::from)?;
		Ok(PhotosPickerStartOutput {
			poll_interval_seconds: session.poll_interval().as_secs().max(1) as u32,
			session_id: session.id,
			picker_uri: session.picker_uri,
		})
	}

	fn action_kind(&self) -> &'static str {
		"integrations.google.photos.picker.start"
	}
}

crate::register_library_action!(
	PhotosPickerStartAction,
	"integrations.google.photos.picker.start"
);

/// Whether the user finished selecting in the picker.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PhotosPickerStatusInput {
	pub account_id: String,
	pub session_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PhotosPickerStatus {
	pub ready: bool,
	pub expire_time: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhotosPickerStatusQuery {
	input: PhotosPickerStatusInput,
}

impl LibraryQuery for PhotosPickerStatusQuery {
	type Input = PhotosPickerStatusInput;
	type Output = PhotosPickerStatus;

	fn from_input(input: Self::Input) -> QueryResult<Self> {
		Ok(Self { input })
	}

	async fn execute(
		self,
		context: Arc<CoreContext>,
		session: crate::infra::api::SessionContext,
	) -> QueryResult<Self::Output> {
		let library = super::session_library(&context, &session).await?;
		let client = context
			.integrations
			.photos_client(&library, &self.input.account_id)
			.await?;
		let picker = client
			.get_picker_session(&self.input.session_id)
			.await
			.map_err(crate::service::integrations::IntegrationError::from)?;
		Ok(PhotosPickerStatus {
			ready: picker.media_items_set,
			expire_time: picker.expire_time,
		})
	}
}

crate::register_library_query!(
	PhotosPickerStatusQuery,
	"integrations.google.photos.picker.status"
);

/// Downloads what the user picked into a local folder.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PhotosPickerImportInput {
	pub account_id: String,
	pub session_id: String,
	/// A folder on this device; created if missing.
	pub destination: WingPath,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhotosPickerImportAction {
	input: PhotosPickerImportInput,
}

impl LibraryAction for PhotosPickerImportAction {
	type Input = PhotosPickerImportInput;
	type Output = crate::infra::job::handle::JobReceipt;

	fn from_input(input: Self::Input) -> Result<Self, String> {
		if input.destination.as_local_path().is_none() {
			return Err("Choose a folder on this device".to_string());
		}
		Ok(Self { input })
	}

	async fn execute(
		self,
		library: Arc<Library>,
		_context: Arc<CoreContext>,
	) -> Result<Self::Output, ActionError> {
		let destination = self
			.input
			.destination
			.as_local_path()
			.map(Path::to_path_buf)
			.ok_or_else(|| ActionError::InvalidInput("Not a local folder".to_string()))?;
		let job = PhotosImportJob {
			account_id: self.input.account_id,
			session_id: self.input.session_id,
			destination,
			done: Vec::new(),
		};
		let handle = library
			.jobs()
			.dispatch(job)
			.await
			.map_err(ActionError::Job)?;
		Ok(handle.into())
	}

	fn action_kind(&self) -> &'static str {
		"integrations.google.photos.picker.import"
	}
}

crate::register_library_action!(
	PhotosPickerImportAction,
	"integrations.google.photos.picker.import"
);

/// Downloads the items of a finished picker session.
#[derive(Debug, Serialize, Deserialize, Job)]
pub struct PhotosImportJob {
	pub account_id: String,
	pub session_id: String,
	pub destination: PathBuf,
	/// IDs of items already downloaded, for resuming.
	#[serde(default)]
	pub done: Vec<String>,
}

impl Job for PhotosImportJob {
	const NAME: &'static str = "google_photos_import";
	const RESUMABLE: bool = true;
	const DESCRIPTION: Option<&'static str> = Some("Download photos picked from Google Photos");
}

impl crate::infra::job::traits::DynJob for PhotosImportJob {
	fn job_name(&self) -> &'static str {
		Self::NAME
	}
}

/// A path in `dir` for `name` that does not exist yet: "IMG.jpg", "IMG (2).jpg", ...
pub async fn unique_destination(dir: &Path, name: &str) -> PathBuf {
	let candidate = dir.join(name);
	if tokio::fs::symlink_metadata(&candidate).await.is_err() {
		return candidate;
	}
	let path = Path::new(name);
	let stem = path
		.file_stem()
		.map(|s| s.to_string_lossy().into_owned())
		.unwrap_or_default();
	let extension = path
		.extension()
		.map(|e| format!(".{}", e.to_string_lossy()))
		.unwrap_or_default();
	let mut counter = 2;
	loop {
		let candidate = dir.join(format!("{stem} ({counter}){extension}"));
		if tokio::fs::symlink_metadata(&candidate).await.is_err() {
			return candidate;
		}
		counter += 1;
	}
}

#[async_trait::async_trait]
impl JobHandler for PhotosImportJob {
	type Output = PhotosTransferReport;

	async fn run(&mut self, ctx: JobContext<'_>) -> JobResult<Self::Output> {
		let library = ctx.library_arc();
		let client = library
			.core_context()
			.integrations
			.photos_client(&library, &self.account_id)
			.await
			.map_err(|e| JobError::execution(e.to_string()))?;
		let items: Vec<PickedMediaItem> = client
			.list_picked_items(&self.session_id)
			.await
			.map_err(|e| JobError::execution(e.to_string()))?;
		tokio::fs::create_dir_all(&self.destination)
			.await
			.map_err(|e| JobError::execution(e.to_string()))?;
		let mut report = PhotosTransferReport::default();
		let total = items.len();
		ctx.log(format!(
			"Downloading {total} items from Google Photos to {}",
			self.destination.display()
		));
		for (index, item) in items.iter().enumerate() {
			ctx.check_interrupt().await?;
			if self.done.contains(&item.id) {
				report.transferred += 1;
				continue;
			}
			let target = unique_destination(&self.destination, &item.file_name()).await;
			match client.download_picked_item(item, &target).await {
				Ok(bytes) => {
					report.transferred += 1;
					report.bytes += bytes;
					self.done.push(item.id.clone());
				}
				Err(error) => {
					report.failed += 1;
					report.errors.push(format!("{}: {error}", item.file_name()));
					ctx.add_warning(format!("{}: {error}", item.file_name()));
				}
			}
			ctx.progress(Progress::Count {
				current: index + 1,
				total,
			});
			ctx.checkpoint().await?;
		}
		if report.failed == 0 {
			// Best effort: Google expires sessions on its own.
			let _ = client.delete_picker_session(&self.session_id).await;
		}
		report.errors.truncate(20);
		Ok(report)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn media_detection_by_extension() {
		assert!(is_photos_media(Path::new("/a/IMG_1.JPG")));
		assert!(is_photos_media(Path::new("clip.mov")));
		assert!(!is_photos_media(Path::new("notes.txt")));
		assert!(!is_photos_media(Path::new("noext")));
	}

	#[tokio::test]
	async fn unique_destination_counts_up() {
		let dir = tempfile::tempdir().unwrap();
		assert_eq!(
			unique_destination(dir.path(), "a.jpg").await,
			dir.path().join("a.jpg")
		);
		std::fs::write(dir.path().join("a.jpg"), "x").unwrap();
		std::fs::write(dir.path().join("a (2).jpg"), "x").unwrap();
		assert_eq!(
			unique_destination(dir.path(), "a.jpg").await,
			dir.path().join("a (3).jpg")
		);
	}

	#[tokio::test]
	async fn collect_media_walks_folders_and_skips_other_files() {
		let dir = tempfile::tempdir().unwrap();
		std::fs::create_dir_all(dir.path().join("trip/day1")).unwrap();
		std::fs::write(dir.path().join("trip/day1/a.jpg"), "x").unwrap();
		std::fs::write(dir.path().join("trip/b.mp4"), "x").unwrap();
		std::fs::write(dir.path().join("trip/readme.txt"), "x").unwrap();
		let (files, skipped) = collect_media(&[dir.path().join("trip")]).await;
		assert_eq!(files.len(), 2);
		assert_eq!(skipped, 1);
	}
}
