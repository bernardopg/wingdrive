//! Operations that manage cloud sync pairs.

use std::path::Path;
use std::sync::Arc;

use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set};
use serde::{Deserialize, Serialize};
use specta::Type;
use uuid::Uuid;

use super::{job::CloudSyncJob, CloudSyncPairInfo, SyncMode};
use crate::context::CoreContext;
use crate::domain::addressing::WingPath;
use crate::infra::action::{error::ActionError, LibraryAction};
use crate::infra::db::entities::cloud_sync_pair;
use crate::infra::query::{LibraryQuery, QueryError, QueryResult};
use crate::library::Library;

/// Shortest interval between automatic runs, to stay within API quotas.
pub const MIN_INTERVAL_MINUTES: u32 = 5;

async fn find_pair(library: &Library, id: Uuid) -> Result<cloud_sync_pair::Model, ActionError> {
	cloud_sync_pair::Entity::find()
		.filter(cloud_sync_pair::Column::Uuid.eq(id))
		.one(library.db().conn())
		.await
		.map_err(ActionError::SeaOrm)?
		.ok_or_else(|| ActionError::InvalidInput(format!("Sync pair {id} not found")))
}

fn validate_interval(interval: Option<u32>) -> Result<(), ActionError> {
	match interval {
		Some(minutes) if minutes < MIN_INTERVAL_MINUTES => Err(ActionError::Validation {
			field: "interval_minutes".to_string(),
			message: format!("Use at least {MIN_INTERVAL_MINUTES} minutes between runs"),
		}),
		_ => Ok(()),
	}
}

/// Whether one path is the other or inside it.
fn overlaps(a: &str, b: &str) -> bool {
	let a = a.trim_end_matches('/');
	let b = b.trim_end_matches('/');
	a.is_empty()
		|| b.is_empty()
		|| a == b
		|| a.starts_with(&format!("{b}/"))
		|| b.starts_with(&format!("{a}/"))
}

/// Lists the sync pairs of the library.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CloudSyncListInput {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudSyncListQuery {
	input: CloudSyncListInput,
}

impl LibraryQuery for CloudSyncListQuery {
	type Input = CloudSyncListInput;
	type Output = Vec<CloudSyncPairInfo>;

	fn from_input(input: Self::Input) -> QueryResult<Self> {
		Ok(Self { input })
	}

	async fn execute(
		self,
		context: Arc<CoreContext>,
		session: crate::infra::api::SessionContext,
	) -> QueryResult<Self::Output> {
		let library_id = session
			.current_library_id
			.ok_or_else(|| QueryError::Internal("No library in session".to_string()))?;
		let library = context
			.get_library(library_id)
			.await
			.ok_or_else(|| QueryError::Internal("Library not found".to_string()))?;
		let pairs = cloud_sync_pair::Entity::find()
			.order_by_asc(cloud_sync_pair::Column::Name)
			.all(library.db().conn())
			.await
			.map_err(|e| QueryError::Internal(e.to_string()))?;
		Ok(pairs.iter().map(CloudSyncPairInfo::from_model).collect())
	}
}

crate::register_library_query!(CloudSyncListQuery, "cloud_sync.list");

/// Creates a sync pair.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CloudSyncCreateInput {
	pub name: String,
	/// Absolute path of a folder on this device; created if missing.
	pub local_path: String,
	/// A folder on a mounted cloud volume; created if missing.
	pub remote: WingPath,
	pub mode: SyncMode,
	pub propagate_deletes: bool,
	pub interval_minutes: Option<u32>,
	/// Start the first run right away.
	pub run_now: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudSyncCreateAction {
	input: CloudSyncCreateInput,
}

impl LibraryAction for CloudSyncCreateAction {
	type Input = CloudSyncCreateInput;
	type Output = CloudSyncPairInfo;

	fn from_input(input: Self::Input) -> Result<Self, String> {
		if input.name.trim().is_empty() {
			return Err("A sync pair needs a name".to_string());
		}
		if !Path::new(&input.local_path).is_absolute() {
			return Err("The local folder must be an absolute path".to_string());
		}
		if input.remote.as_cloud().is_none() {
			return Err("The destination must be a folder on a cloud volume".to_string());
		}
		Ok(Self { input })
	}

	async fn validate(
		&self,
		library: &Arc<Library>,
		context: Arc<CoreContext>,
	) -> Result<crate::infra::action::ValidationResult, ActionError> {
		validate_interval(self.input.interval_minutes)?;
		super::resolve_remote(&context, &self.input.remote)
			.await
			.map_err(|message| ActionError::Validation {
				field: "remote".to_string(),
				message,
			})?;
		let local = Path::new(&self.input.local_path);
		if let Ok(metadata) = tokio::fs::metadata(local).await {
			if !metadata.is_dir() {
				return Err(ActionError::Validation {
					field: "local_path".to_string(),
					message: "The local path is a file, not a folder".to_string(),
				});
			}
		}
		if local.starts_with(&context.data_dir) {
			return Err(ActionError::Validation {
				field: "local_path".to_string(),
				message: "The local folder cannot be inside WingDrive's data folder".to_string(),
			});
		}
		let (service, identifier, remote_path) =
			self.input.remote.as_cloud().expect("checked in from_input");
		let existing = cloud_sync_pair::Entity::find()
			.all(library.db().conn())
			.await
			.map_err(ActionError::SeaOrm)?;
		for pair in existing {
			let same_remote_volume =
				pair.remote_service == service.scheme() && pair.remote_identifier == identifier;
			if overlaps(&pair.local_path, &self.input.local_path)
				|| (same_remote_volume && overlaps(&pair.remote_path, remote_path))
			{
				return Err(ActionError::Validation {
					field: "local_path".to_string(),
					message: format!(
						"This overlaps the sync pair '{}'; two pairs on the same files would undo each other",
						pair.name
					),
				});
			}
		}
		Ok(crate::infra::action::ValidationResult::Success { metadata: None })
	}

	async fn execute(
		self,
		library: Arc<Library>,
		_context: Arc<CoreContext>,
	) -> Result<Self::Output, ActionError> {
		let (service, identifier, remote_path) = self
			.input
			.remote
			.as_cloud()
			.ok_or_else(|| ActionError::InvalidInput("Not a cloud path".to_string()))?;
		let now = chrono::Utc::now();
		let model = cloud_sync_pair::ActiveModel {
			id: sea_orm::ActiveValue::NotSet,
			uuid: Set(Uuid::new_v4()),
			name: Set(self.input.name.trim().to_string()),
			local_path: Set(self.input.local_path.clone()),
			remote_service: Set(service.scheme().to_string()),
			remote_identifier: Set(identifier.to_string()),
			remote_path: Set(remote_path.trim_matches('/').to_string()),
			mode: Set(self.input.mode.as_str().to_string()),
			propagate_deletes: Set(self.input.propagate_deletes),
			interval_minutes: Set(self.input.interval_minutes.map(|m| m as i32)),
			enabled: Set(true),
			last_run_at: Set(None),
			last_status: Set(None),
			last_error: Set(None),
			last_report: Set(None),
			manifest: Set(None),
			created_at: Set(now),
			updated_at: Set(now),
		}
		.insert(library.db().conn())
		.await
		.map_err(ActionError::SeaOrm)?;
		if self.input.run_now {
			library
				.jobs()
				.dispatch(CloudSyncJob::new(model.uuid))
				.await
				.map_err(ActionError::Job)?;
		}
		Ok(CloudSyncPairInfo::from_model(&model))
	}

	fn action_kind(&self) -> &'static str {
		"cloud_sync.create"
	}
}

crate::register_library_action!(CloudSyncCreateAction, "cloud_sync.create");

/// Changes a sync pair. Omitted fields keep their value.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CloudSyncUpdateInput {
	pub id: Uuid,
	pub name: Option<String>,
	pub mode: Option<SyncMode>,
	pub propagate_deletes: Option<bool>,
	/// `Some(None)` clears the interval.
	pub interval_minutes: Option<Option<u32>>,
	pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudSyncUpdateAction {
	input: CloudSyncUpdateInput,
}

impl LibraryAction for CloudSyncUpdateAction {
	type Input = CloudSyncUpdateInput;
	type Output = CloudSyncPairInfo;

	fn from_input(input: Self::Input) -> Result<Self, String> {
		Ok(Self { input })
	}

	async fn execute(
		self,
		library: Arc<Library>,
		_context: Arc<CoreContext>,
	) -> Result<Self::Output, ActionError> {
		if let Some(interval) = self.input.interval_minutes {
			validate_interval(interval)?;
		}
		let pair = find_pair(&library, self.input.id).await?;
		let mut active: cloud_sync_pair::ActiveModel = pair.into();
		if let Some(name) = self.input.name.filter(|n| !n.trim().is_empty()) {
			active.name = Set(name.trim().to_string());
		}
		if let Some(mode) = self.input.mode {
			active.mode = Set(mode.as_str().to_string());
		}
		if let Some(propagate) = self.input.propagate_deletes {
			active.propagate_deletes = Set(propagate);
		}
		if let Some(interval) = self.input.interval_minutes {
			active.interval_minutes = Set(interval.map(|m| m as i32));
		}
		if let Some(enabled) = self.input.enabled {
			active.enabled = Set(enabled);
		}
		active.updated_at = Set(chrono::Utc::now());
		let model = active
			.update(library.db().conn())
			.await
			.map_err(ActionError::SeaOrm)?;
		Ok(CloudSyncPairInfo::from_model(&model))
	}

	fn action_kind(&self) -> &'static str {
		"cloud_sync.update"
	}
}

crate::register_library_action!(CloudSyncUpdateAction, "cloud_sync.update");

/// Deletes a sync pair. Files on both sides stay where they are.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CloudSyncDeleteInput {
	pub id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CloudSyncDeleteOutput {
	pub id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudSyncDeleteAction {
	input: CloudSyncDeleteInput,
}

impl LibraryAction for CloudSyncDeleteAction {
	type Input = CloudSyncDeleteInput;
	type Output = CloudSyncDeleteOutput;

	fn from_input(input: Self::Input) -> Result<Self, String> {
		Ok(Self { input })
	}

	async fn execute(
		self,
		library: Arc<Library>,
		_context: Arc<CoreContext>,
	) -> Result<Self::Output, ActionError> {
		if super::is_running(self.input.id) {
			return Err(ActionError::InvalidInput(
				"Wait for the running sync to finish before deleting the pair".to_string(),
			));
		}
		let pair = find_pair(&library, self.input.id).await?;
		cloud_sync_pair::Entity::delete_by_id(pair.id)
			.exec(library.db().conn())
			.await
			.map_err(ActionError::SeaOrm)?;
		Ok(CloudSyncDeleteOutput { id: self.input.id })
	}

	fn action_kind(&self) -> &'static str {
		"cloud_sync.delete"
	}
}

crate::register_library_action!(CloudSyncDeleteAction, "cloud_sync.delete");

/// Starts a sync run now.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CloudSyncRunInput {
	pub id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudSyncRunAction {
	input: CloudSyncRunInput,
}

impl LibraryAction for CloudSyncRunAction {
	type Input = CloudSyncRunInput;
	type Output = crate::infra::job::handle::JobReceipt;

	fn from_input(input: Self::Input) -> Result<Self, String> {
		Ok(Self { input })
	}

	async fn execute(
		self,
		library: Arc<Library>,
		_context: Arc<CoreContext>,
	) -> Result<Self::Output, ActionError> {
		find_pair(&library, self.input.id).await?;
		if super::is_running(self.input.id) {
			return Err(ActionError::InvalidInput(
				"This sync pair is already running".to_string(),
			));
		}
		let handle = library
			.jobs()
			.dispatch(CloudSyncJob::new(self.input.id))
			.await
			.map_err(ActionError::Job)?;
		Ok(handle.into())
	}

	fn action_kind(&self) -> &'static str {
		"cloud_sync.run"
	}
}

crate::register_library_action!(CloudSyncRunAction, "cloud_sync.run");

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn overlap_detects_nesting_but_not_prefix_siblings() {
		assert!(overlaps("/home/u/Docs", "/home/u/Docs"));
		assert!(overlaps("/home/u/Docs", "/home/u/Docs/Work"));
		assert!(overlaps("/home/u/Docs/Work/", "/home/u/Docs"));
		assert!(!overlaps("/home/u/Docs", "/home/u/Docs2"));
		assert!(
			overlaps("", "Backup"),
			"a pair on the whole volume covers every folder"
		);
	}

	#[test]
	fn interval_floor() {
		assert!(validate_interval(None).is_ok());
		assert!(validate_interval(Some(MIN_INTERVAL_MINUTES)).is_ok());
		assert!(validate_interval(Some(1)).is_err());
	}
}
