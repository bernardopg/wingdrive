//! # OneDrive volume backend
//!
//! `OneDriveBackend` exposes one account's OneDrive as a `VolumeBackend`.
//! File operations go through OpenDAL's OneDrive service, which can refresh
//! tokens itself but always sends a client secret (public clients have none)
//! and never hands back the refresh token Microsoft rotates. So the backend
//! gives OpenDAL only an access token from `MicrosoftTokenSource` and builds a
//! new operator whenever that token changes, which is cheap.
//!
//! Deleting a OneDrive item moves it to the OneDrive recycle bin, so `trash`
//! and `delete` are the same request.

use std::ops::Range;
use std::path::Path;
use std::sync::Arc;

use async_trait::async_trait;
use bytes::Bytes;
use serde::Deserialize;
use tokio::sync::Mutex;

use super::tokens::MicrosoftTokenSource;
use super::MicrosoftEndpoints;
use crate::volume::backend::{
	BackendType, CloudBackend, CloudServiceType, RawDirEntry, RawMetadata, VolumeBackend,
};
use crate::volume::error::VolumeError;

/// One account's OneDrive.
#[derive(Debug)]
pub struct OneDriveBackend {
	http: reqwest::Client,
	endpoints: MicrosoftEndpoints,
	tokens: Arc<MicrosoftTokenSource>,
	/// Operator built for the access token it holds.
	current: Mutex<Option<(String, CloudBackend)>>,
}

#[derive(Debug, Deserialize)]
struct DriveInfo {
	#[serde(default)]
	quota: Option<Quota>,
}

#[derive(Debug, Deserialize)]
struct Quota {
	#[serde(default)]
	total: Option<u64>,
	#[serde(default)]
	remaining: Option<u64>,
}

impl OneDriveBackend {
	pub fn new(
		http: reqwest::Client,
		endpoints: MicrosoftEndpoints,
		tokens: Arc<MicrosoftTokenSource>,
	) -> Self {
		Self {
			http,
			endpoints,
			tokens,
			current: Mutex::new(None),
		}
	}

	/// The OpenDAL backend for the current access token.
	async fn backend(&self) -> Result<CloudBackend, VolumeError> {
		let token = self.tokens.access_token().await?;
		let mut current = self.current.lock().await;
		if let Some((held, backend)) = current.as_ref() {
			if *held == token {
				return Ok(backend.clone());
			}
		}
		let backend = CloudBackend::new_onedrive_with_access_token(&token)?;
		*current = Some((token, backend.clone()));
		Ok(backend)
	}
}

#[async_trait]
impl VolumeBackend for OneDriveBackend {
	async fn read(&self, path: &Path) -> Result<Bytes, VolumeError> {
		self.backend().await?.read(path).await
	}

	async fn read_range(&self, path: &Path, range: Range<u64>) -> Result<Bytes, VolumeError> {
		self.backend().await?.read_range(path, range).await
	}

	async fn write(&self, path: &Path, data: Bytes) -> Result<(), VolumeError> {
		self.backend().await?.write(path, data).await
	}

	async fn read_dir(&self, path: &Path) -> Result<Vec<RawDirEntry>, VolumeError> {
		self.backend().await?.read_dir(path).await
	}

	async fn metadata(&self, path: &Path) -> Result<RawMetadata, VolumeError> {
		self.backend().await?.metadata(path).await
	}

	async fn exists(&self, path: &Path) -> Result<bool, VolumeError> {
		self.backend().await?.exists(path).await
	}

	async fn delete(&self, path: &Path) -> Result<(), VolumeError> {
		self.backend().await?.delete(path).await
	}

	async fn create_directory(&self, path: &Path, recursive: bool) -> Result<(), VolumeError> {
		self.backend()
			.await?
			.create_directory(path, recursive)
			.await
	}

	fn is_local(&self) -> bool {
		false
	}

	fn backend_type(&self) -> BackendType {
		BackendType::Cloud(CloudServiceType::OneDrive)
	}

	async fn rename(&self, from: &Path, to: &Path) -> Result<(), VolumeError> {
		self.backend().await?.rename(from, to).await
	}

	async fn copy(&self, from: &Path, to: &Path) -> Result<(), VolumeError> {
		self.backend().await?.copy(from, to).await
	}

	async fn trash(&self, path: &Path) -> Result<(), VolumeError> {
		self.delete(path).await
	}

	async fn capacity(&self) -> Result<Option<(u64, u64)>, VolumeError> {
		let token = self.tokens.access_token().await?;
		let response = self
			.http
			.get(format!("{}/me/drive?$select=quota", self.endpoints.graph))
			.bearer_auth(token)
			.send()
			.await
			.map_err(|e| VolumeError::Service(e.to_string()))?;
		if !response.status().is_success() {
			return Ok(None);
		}
		let body = response
			.bytes()
			.await
			.map_err(|e| VolumeError::Service(e.to_string()))?;
		let info: DriveInfo =
			serde_json::from_slice(&body).map_err(|e| VolumeError::Service(e.to_string()))?;
		Ok(info.quota.and_then(|q| Some((q.total?, q.remaining?))))
	}
}
