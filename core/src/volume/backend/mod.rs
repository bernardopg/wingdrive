//! Volume backend implementations
//!
//! This module provides the backend abstraction layer for heterogeneous storage I/O.

use async_trait::async_trait;
use bytes::Bytes;
use std::fmt::Debug;
use std::ops::Range;
use std::path::Path;
use std::time::SystemTime;

use crate::ops::indexing::state::EntryKind;
use crate::volume::error::VolumeError;

pub mod cloud;
pub mod local;

pub use cloud::CloudBackend;
pub use local::LocalBackend;

/// Minimal I/O backend trait for volume operations
///
/// This trait provides only low-level filesystem operations. All domain logic
/// (Entry creation, content identification, etc.) is handled by existing
/// WingDrive infrastructure that consumes these raw operations.
#[async_trait]
pub trait VolumeBackend: Send + Sync + Debug {
	/// Read entire file content
	async fn read(&self, path: &Path) -> Result<Bytes, VolumeError>;

	/// Read specific byte range from file (critical for cloud efficiency)
	async fn read_range(&self, path: &Path, range: Range<u64>) -> Result<Bytes, VolumeError>;

	/// Write file content
	async fn write(&self, path: &Path, data: Bytes) -> Result<(), VolumeError>;

	/// List directory entries (returns minimal metadata)
	async fn read_dir(&self, path: &Path) -> Result<Vec<RawDirEntry>, VolumeError>;

	/// Get file/directory metadata
	async fn metadata(&self, path: &Path) -> Result<RawMetadata, VolumeError>;

	/// Check if path exists (optimized when possible)
	async fn exists(&self, path: &Path) -> Result<bool, VolumeError>;

	/// Delete file or directory
	async fn delete(&self, path: &Path) -> Result<(), VolumeError>;

	/// Create a directory at the specified path
	async fn create_directory(&self, path: &Path, recursive: bool) -> Result<(), VolumeError>;

	/// Backend identification (used to optimize operations)
	fn is_local(&self) -> bool;

	/// Get backend type identifier
	fn backend_type(&self) -> BackendType;

	/// Renames or moves a file or directory inside this backend.
	///
	/// Backends that cannot move server side return `VolumeError::Unsupported`,
	/// and callers fall back to copy plus delete.
	async fn rename(&self, from: &Path, to: &Path) -> Result<(), VolumeError> {
		let _ = (from, to);
		Err(VolumeError::Unsupported(
			"rename is not supported by this backend".to_string(),
		))
	}

	/// Copies a file inside this backend without moving its bytes through the client.
	async fn copy(&self, from: &Path, to: &Path) -> Result<(), VolumeError> {
		let _ = (from, to);
		Err(VolumeError::Unsupported(
			"server-side copy is not supported by this backend".to_string(),
		))
	}

	/// Moves a file or directory to the service's recoverable trash.
	///
	/// Services without a trash return `VolumeError::Unsupported` so the caller
	/// can ask before deleting permanently.
	async fn trash(&self, path: &Path) -> Result<(), VolumeError> {
		let _ = path;
		Err(VolumeError::Unsupported(
			"trash is not supported by this backend".to_string(),
		))
	}

	/// Uploads a local file to `path`, replacing any file already there.
	///
	/// The default reads the whole file into memory. Backends with resumable or
	/// chunked uploads override it so large files do not need that much RAM.
	async fn write_from_file(&self, path: &Path, local: &Path) -> Result<u64, VolumeError> {
		let data = tokio::fs::read(local).await?;
		let size = data.len() as u64;
		self.write(path, Bytes::from(data)).await?;
		Ok(size)
	}

	/// Downloads `path` into the local file `local`, replacing it.
	///
	/// The default reads in ranges of `TRANSFER_CHUNK_SIZE` so a large file never
	/// sits in memory whole. The data goes to a sibling temporary file first, so a
	/// failed download never leaves a truncated file under the final name.
	async fn read_to_file(&self, path: &Path, local: &Path) -> Result<u64, VolumeError> {
		use tokio::io::AsyncWriteExt;

		let size = self.metadata(path).await?.size;
		let partial = partial_download_path(local);
		let mut file = tokio::fs::File::create(&partial).await?;
		let mut written = 0u64;
		let result: Result<(), VolumeError> = async {
			if size == 0 {
				// Some services report no size for exported documents, so read whole.
				let data = self.read(path).await?;
				file.write_all(&data).await?;
				written = data.len() as u64;
			} else {
				while written < size {
					let end = (written + TRANSFER_CHUNK_SIZE).min(size);
					let chunk = self.read_range(path, written..end).await?;
					if chunk.is_empty() {
						break;
					}
					file.write_all(&chunk).await?;
					written += chunk.len() as u64;
				}
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
		tokio::fs::rename(&partial, local).await?;
		Ok(written)
	}

	/// Whether `path` can only be edited in the service's web app.
	///
	/// Google Docs, Sheets and Slides export to Office formats for reading,
	/// but an edited export cannot be written back as the same document.
	async fn edits_in_browser(&self, path: &Path) -> Result<bool, VolumeError> {
		let _ = path;
		Ok(false)
	}

	/// Returns a URL that opens `path` in the service's web interface, if it has one.
	async fn web_link(&self, path: &Path) -> Result<Option<String>, VolumeError> {
		let _ = path;
		Ok(None)
	}

	/// Reports storage totals as `(total, available)` bytes when the service exposes them.
	async fn capacity(&self) -> Result<Option<(u64, u64)>, VolumeError> {
		Ok(None)
	}
}

/// Chunk size for ranged downloads and chunked uploads.
///
/// Google Drive requires resumable upload chunks to be multiples of 256 KiB, and
/// 8 MiB keeps request overhead low without holding much memory per transfer.
pub const TRANSFER_CHUNK_SIZE: u64 = 8 * 1024 * 1024;

/// Temporary sibling path a download writes to before it is renamed into place.
pub fn partial_download_path(local: &Path) -> std::path::PathBuf {
	let mut name = local
		.file_name()
		.map(|name| name.to_os_string())
		.unwrap_or_default();
	name.push(".wingpart");
	local.with_file_name(name)
}

/// Backend type identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendType {
	Local,
	Cloud(CloudServiceType),
}

/// Cloud service type identifier
#[derive(
	Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, specta::Type,
)]
pub enum CloudServiceType {
	#[serde(rename = "s3")]
	S3,
	#[serde(rename = "gdrive")]
	GoogleDrive,
	#[serde(rename = "dropbox")]
	Dropbox,
	#[serde(rename = "onedrive")]
	OneDrive,
	#[serde(rename = "gcs")]
	GoogleCloudStorage,
	#[serde(rename = "azblob")]
	AzureBlob,
	#[serde(rename = "b2")]
	BackblazeB2,
	#[serde(rename = "wasabi")]
	Wasabi,
	#[serde(rename = "spaces")]
	DigitalOceanSpaces,
	#[serde(rename = "cloud")]
	Other,
}

impl CloudServiceType {
	/// Get the URI scheme for this cloud service
	/// Used for service-native addressing (e.g., "s3://bucket/path")
	pub fn scheme(&self) -> &'static str {
		match self {
			Self::S3 => "s3",
			Self::GoogleDrive => "gdrive",
			Self::OneDrive => "onedrive",
			Self::Dropbox => "dropbox",
			Self::AzureBlob => "azblob",
			Self::GoogleCloudStorage => "gcs",
			Self::BackblazeB2 => "b2",
			Self::Wasabi => "wasabi",
			Self::DigitalOceanSpaces => "spaces",
			Self::Other => "cloud",
		}
	}

	/// Parse cloud service type from URI scheme
	/// Returns None if the scheme doesn't match any known service
	pub fn from_scheme(scheme: &str) -> Option<Self> {
		match scheme {
			"s3" => Some(Self::S3),
			"gdrive" => Some(Self::GoogleDrive),
			"onedrive" => Some(Self::OneDrive),
			"dropbox" => Some(Self::Dropbox),
			"azblob" => Some(Self::AzureBlob),
			"gcs" => Some(Self::GoogleCloudStorage),
			"b2" => Some(Self::BackblazeB2),
			"wasabi" => Some(Self::Wasabi),
			"spaces" => Some(Self::DigitalOceanSpaces),
			"cloud" => Some(Self::Other),
			_ => None,
		}
	}
}

/// Raw directory entry returned by volume backends
#[derive(Debug, Clone)]
pub struct RawDirEntry {
	pub name: String,
	pub kind: EntryKind,
	pub size: u64,
	pub modified: Option<SystemTime>,
	/// Birth time, when the filesystem records one
	pub created: Option<SystemTime>,
	pub inode: Option<u64>,
	/// Content revision reported by the service (an MD5, ETag or version id).
	///
	/// Sync compares it with the last value it saw to detect remote edits that
	/// keep the size and timestamp.
	pub revision: Option<String>,
}

/// Raw metadata returned by volume backends
#[derive(Debug, Clone)]
pub struct RawMetadata {
	pub kind: EntryKind,
	pub size: u64,
	pub modified: Option<SystemTime>,
	pub created: Option<SystemTime>,
	pub accessed: Option<SystemTime>,
	pub inode: Option<u64>,
	/// Unix permission bits (mode), None for cloud backends or Windows
	pub permissions: Option<u32>,
	/// Content revision reported by the service, see `RawDirEntry::revision`.
	pub revision: Option<String>,
}
