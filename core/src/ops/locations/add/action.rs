//! Location add action handler

use super::output::LocationAddOutput;
use crate::{
	context::CoreContext,
	infra::action::{
		context::ActionContextProvider,
		error::{ActionError, ActionResult},
		LibraryAction,
	},
	infra::db::entities,
	location::manager::LocationManager,
	ops::indexing::IndexMode,
};
use async_trait::async_trait;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use serde_json::json;
use specta::Type;
use std::{
	path::{Path, PathBuf},
	sync::Arc,
};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct LocationAddInput {
	pub path: crate::domain::addressing::SdPath,
	pub name: Option<String>,
	pub mode: IndexMode,
	pub job_policies: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationAddAction {
	input: LocationAddInput,
}

impl LocationAddAction {
	pub fn new(input: LocationAddInput) -> Self {
		Self { input }
	}
}

fn protected_location_reason(path: &Path, data_dir: &Path) -> Option<&'static str> {
	if path.parent().is_none() {
		return Some("Filesystem roots cannot be indexed as locations");
	}

	for system_tree in ["/proc", "/sys", "/dev", "/boot"] {
		if path.starts_with(system_tree) {
			return Some("Operating-system trees cannot be indexed as locations");
		}
	}
	if path == Path::new("/run") {
		return Some("The operating-system runtime directory cannot be indexed");
	}
	if path.starts_with(data_dir) {
		return Some("WingDrive's internal data directory cannot be indexed");
	}

	None
}

// Implement the new modular ActionType trait
impl LibraryAction for LocationAddAction {
	type Input = LocationAddInput;
	type Output = LocationAddOutput;

	fn from_input(input: LocationAddInput) -> Result<Self, String> {
		Ok(LocationAddAction::new(input))
	}

	async fn execute(
		self,
		library: std::sync::Arc<crate::library::Library>,
		context: std::sync::Arc<CoreContext>,
	) -> Result<Self::Output, ActionError> {
		// Get the device UUID from the device manager
		let device_uuid = context
			.device_manager
			.device_id()
			.map_err(ActionError::device_manager_error)?;

		// Get device record from database to get the integer ID
		let db = library.db().conn();
		let device_record = entities::device::Entity::find()
			.filter(entities::device::Column::Uuid.eq(device_uuid))
			.one(db)
			.await
			.map_err(ActionError::SeaOrm)?
			.ok_or_else(|| ActionError::DeviceNotFound(device_uuid))?;

		// Add the location using LocationManager
		let location_manager = LocationManager::new(context.events.as_ref().clone());

		let location_mode = match self.input.mode {
			IndexMode::None => crate::location::IndexMode::None,
			IndexMode::Shallow => crate::location::IndexMode::Shallow,
			IndexMode::Content => crate::location::IndexMode::Content,
			IndexMode::Deep => crate::location::IndexMode::Deep,
		};

		// Create action context for job tracking
		let action_context = self.create_action_context();

		// Serialize job_policies to JSON string if provided
		let job_policies_json = self
			.input
			.job_policies
			.as_ref()
			.and_then(|jp| serde_json::to_string(jp).ok());

		let (location_id, job_id_string) = location_manager
			.add_location(
				library.clone(),
				self.input.path.clone(),
				self.input.name.clone(),
				device_record.id,
				location_mode,
				Some(action_context),
				job_policies_json,
				&context.volume_manager,
			)
			.await
			.map_err(|e| ActionError::Internal(e.to_string()))?;

		// Register the new location with the filesystem watcher so changes
		// (creates, deletes, renames) are detected in real-time.
		// Without this, the watcher only learns about locations at startup.
		if let Some(local_path) = self.input.path.as_local_path() {
			if let Some(fs_watcher) = context.get_fs_watcher().await {
				use crate::ops::indexing::handlers::LocationMeta;
				use crate::ops::indexing::RuleToggles;

				let root_path = tokio::fs::canonicalize(local_path)
					.await
					.unwrap_or_else(|_| local_path.to_path_buf());
				let root_path = crate::common::utils::strip_windows_extended_prefix(root_path);

				let meta = LocationMeta {
					id: location_id,
					library_id: library.id(),
					root_path,
					rule_toggles: RuleToggles::default(),
				};
				if let Err(e) = fs_watcher.watch_location(meta).await {
					tracing::warn!("Failed to register location with watcher: {}", e);
				}
			}
		}

		// Parse the job ID from the string returned by add_location
		let job_id = if !job_id_string.is_empty() {
			Some(
				Uuid::parse_str(&job_id_string)
					.map_err(|e| ActionError::Internal(format!("Failed to parse job ID: {}", e)))?,
			)
		} else {
			None
		};

		let mut output = LocationAddOutput::new(location_id, self.input.path, self.input.name);

		if let Some(job_id) = job_id {
			output = output.with_job_id(job_id);
		}

		Ok(output)
	}

	fn action_kind(&self) -> &'static str {
		"locations.add"
	}

	async fn validate(
		&self,
		library: &std::sync::Arc<crate::library::Library>,
		context: std::sync::Arc<crate::context::CoreContext>,
	) -> Result<crate::infra::action::ValidationResult, ActionError> {
		use crate::domain::addressing::SdPath;

		match &self.input.path {
			SdPath::Physical { path, .. } => {
				if !self.input.path.is_local() {
					return Err(ActionError::Validation {
						field: "path".to_string(),
						message: "Remote physical paths cannot be added from this device"
							.to_string(),
					});
				}

				let metadata =
					tokio::fs::metadata(path)
						.await
						.map_err(|_| ActionError::Validation {
							field: "path".to_string(),
							message: "Path does not exist or cannot be read".to_string(),
						})?;
				if !metadata.is_dir() {
					return Err(ActionError::Validation {
						field: "path".to_string(),
						message: "Path must be a directory".to_string(),
					});
				}

				let canonical_path = tokio::fs::canonicalize(path).await.map_err(|error| {
					ActionError::Validation {
						field: "path".to_string(),
						message: format!("Path cannot be resolved: {error}"),
					}
				})?;
				let canonical_data_dir = tokio::fs::canonicalize(&context.data_dir)
					.await
					.unwrap_or_else(|_| context.data_dir.clone());
				if let Some(message) =
					protected_location_reason(&canonical_path, &canonical_data_dir)
				{
					return Err(ActionError::Validation {
						field: "path".to_string(),
						message: message.to_string(),
					});
				}
			}
			SdPath::Cloud {
				service,
				identifier,
				path: cloud_path,
			} => {
				// Validate cloud path by looking up the volume using VolumeManager
				let _volume = context
					.volume_manager
					.find_cloud_volume(*service, identifier)
					.await
					.ok_or_else(|| ActionError::Validation {
						field: "cloud_volume".to_string(),
						message: format!(
							"Cloud volume not found: {}://{}",
							service.scheme(),
							identifier
						),
					})?;

				// TODO: Validate that the path exists on the cloud volume
				// This would require accessing the VolumeBackend, which isn't available in validation
				// For now, we trust the user's input
			}
			SdPath::Content { .. } => {
				return Err(ActionError::Validation {
					field: "path".to_string(),
					message: "Content paths cannot be used as locations".to_string(),
				});
			}
			SdPath::Sidecar { .. } => {
				return Err(ActionError::Validation {
					field: "path".to_string(),
					message: "Sidecar paths cannot be used as locations".to_string(),
				});
			}
		}

		// Check for duplicate locations
		// TODO: Implement proper duplicate detection for both Physical and Cloud paths

		Ok(crate::infra::action::ValidationResult::Success { metadata: None })
	}
}

impl ActionContextProvider for LocationAddAction {
	fn create_action_context(&self) -> crate::infra::action::context::ActionContext {
		use crate::infra::action::context::{sanitize_action_input, ActionContext};

		ActionContext::new(
			Self::action_type_name(),
			sanitize_action_input(&self.input),
			json!({
				"operation": "add_location",
				"trigger": "user_action",
				"path": self.input.path.to_string(),
				"name": self.input.name,
				"mode": self.input.mode
			}),
		)
	}

	fn action_type_name() -> &'static str
	where
		Self: Sized,
	{
		"locations.add"
	}
}

// Register action
crate::register_library_action!(LocationAddAction, "locations.add");

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn rejects_system_and_internal_location_trees() {
		let data_dir = Path::new("/home/user/.local/share/wingdrive");

		for path in [
			Path::new("/"),
			Path::new("/proc/1"),
			Path::new("/sys/kernel"),
			Path::new("/dev/disk"),
			Path::new("/run"),
			Path::new("/boot/efi"),
			data_dir,
			Path::new("/home/user/.local/share/wingdrive/libraries/1"),
		] {
			assert!(protected_location_reason(path, data_dir).is_some());
		}
	}

	#[test]
	fn allows_user_data_and_removable_media() {
		let data_dir = Path::new("/home/user/.local/share/wingdrive");

		for path in [
			Path::new("/home/user"),
			Path::new("/home/user/Pictures"),
			Path::new("/run/media/user/drive"),
			Path::new("/mnt/archive"),
		] {
			assert!(protected_location_reason(path, data_dir).is_none());
		}
	}
}
