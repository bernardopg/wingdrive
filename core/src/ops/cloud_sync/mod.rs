//! # Cloud folder sync
//!
//! A sync pair keeps a local folder and a folder on a cloud volume (Google
//! Drive, S3 and the other services behind cloud volumes) in step. Pairs live
//! in the library database with the manifest of their last run. `engine` holds
//! the comparison rules, `job` runs a pair as a job, `scheduler` starts pairs
//! that have an interval, and the operations below manage pairs:
//!
//! - `cloud_sync.list` lists pairs with their last result.
//! - `cloud_sync.create`, `cloud_sync.update` and `cloud_sync.delete` manage them.
//! - `cloud_sync.run` starts a run now.

pub mod engine;
pub mod job;
pub mod ops;
pub mod scheduler;

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::{Arc, LazyLock, Mutex};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use specta::Type;
use uuid::Uuid;

pub use engine::{SyncMode, SyncReport};

use crate::domain::addressing::WingPath;
use crate::infra::db::entities::cloud_sync_pair;
use crate::volume::backend::{CloudServiceType, VolumeBackend};

/// Pairs with a run in progress in this daemon, so a pair never runs twice at once.
static RUNNING: LazyLock<Mutex<HashSet<Uuid>>> = LazyLock::new(|| Mutex::new(HashSet::new()));

/// Marks a pair as running; returns `false` when it already is.
pub fn try_mark_running(id: Uuid) -> bool {
	RUNNING.lock().expect("running set lock").insert(id)
}

pub fn mark_finished(id: Uuid) {
	RUNNING.lock().expect("running set lock").remove(&id);
}

pub fn is_running(id: Uuid) -> bool {
	RUNNING.lock().expect("running set lock").contains(&id)
}

/// A sync pair as clients see it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct CloudSyncPairInfo {
	pub id: Uuid,
	pub name: String,
	pub local_path: String,
	pub remote: WingPath,
	pub mode: SyncMode,
	pub propagate_deletes: bool,
	/// Minutes between automatic runs; `None` runs only on demand.
	pub interval_minutes: Option<u32>,
	pub enabled: bool,
	pub last_run_at: Option<DateTime<Utc>>,
	/// "success", "partial" or "failed".
	pub last_status: Option<String>,
	pub last_error: Option<String>,
	pub last_report: Option<SyncReport>,
	pub running: bool,
}

impl CloudSyncPairInfo {
	pub fn from_model(model: &cloud_sync_pair::Model) -> Self {
		Self {
			id: model.uuid,
			name: model.name.clone(),
			local_path: model.local_path.clone(),
			remote: remote_path_of(model),
			mode: SyncMode::parse(&model.mode).unwrap_or(SyncMode::TwoWay),
			propagate_deletes: model.propagate_deletes,
			interval_minutes: model.interval_minutes.map(|m| m.max(0) as u32),
			enabled: model.enabled,
			last_run_at: model.last_run_at,
			last_status: model.last_status.clone(),
			last_error: model.last_error.clone(),
			last_report: model
				.last_report
				.as_deref()
				.and_then(|r| serde_json::from_str(r).ok()),
			running: is_running(model.uuid),
		}
	}
}

/// The cloud folder of a pair as a `WingPath`.
pub fn remote_path_of(model: &cloud_sync_pair::Model) -> WingPath {
	WingPath::Cloud {
		service: CloudServiceType::from_scheme(&model.remote_service)
			.unwrap_or(CloudServiceType::Other),
		identifier: model.remote_identifier.clone(),
		path: model.remote_path.clone(),
	}
}

/// Finds the backend of a cloud path among the volumes this daemon has mounted.
pub async fn resolve_remote(
	context: &crate::context::CoreContext,
	remote: &WingPath,
) -> Result<(Arc<dyn VolumeBackend>, PathBuf), String> {
	let (service, identifier, path) = remote
		.as_cloud()
		.ok_or_else(|| "The sync destination must be a cloud folder".to_string())?;
	let volume = context
		.volume_manager
		.find_cloud_volume(service, identifier)
		.await
		.ok_or_else(|| {
			format!(
				"The cloud volume {}://{} is not connected",
				service.scheme(),
				identifier
			)
		})?;
	let backend = volume
		.backend
		.clone()
		.ok_or_else(|| "The cloud volume has no backend".to_string())?;
	Ok((backend, PathBuf::from(path)))
}
