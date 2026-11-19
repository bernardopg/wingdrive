//! Remove cloud volume action
//!
//! This action removes a cloud storage volume from a library, deleting encrypted
//! credentials and untracking the volume.

use super::output::VolumeRemoveCloudOutput;
use crate::{
	context::CoreContext,
	infra::action::{error::ActionError, LibraryAction},
	volume::VolumeFingerprint,
};
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct VolumeRemoveCloudInput {
	pub fingerprint: VolumeFingerprint,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct VolumeRemoveCloudAction {
	input: VolumeRemoveCloudInput,
}

impl VolumeRemoveCloudAction {
	pub fn new(input: VolumeRemoveCloudInput) -> Self {
		Self { input }
	}
}

impl LibraryAction for VolumeRemoveCloudAction {
	type Input = VolumeRemoveCloudInput;
	type Output = VolumeRemoveCloudOutput;

	fn from_input(input: VolumeRemoveCloudInput) -> Result<Self, String> {
		Ok(VolumeRemoveCloudAction::new(input))
	}

	async fn execute(
		self,
		library: std::sync::Arc<crate::library::Library>,
		context: std::sync::Arc<CoreContext>,
	) -> Result<Self::Output, ActionError> {
		context
			.volume_manager
			.untrack_cloud_volume(&library, &self.input.fingerprint)
			.await
			.map_err(|e| ActionError::InvalidInput(format!("Volume untracking failed: {}", e)))?;

		Ok(VolumeRemoveCloudOutput::new(self.input.fingerprint))
	}

	fn action_kind(&self) -> &'static str {
		"volumes.remove_cloud"
	}
}

crate::register_library_action!(VolumeRemoveCloudAction, "volumes.remove_cloud");
