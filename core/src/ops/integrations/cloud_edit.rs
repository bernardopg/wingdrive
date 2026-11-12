//! Opening cloud files in local apps (`files.cloud_edit.*`).

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use specta::Type;
use uuid::Uuid;

use crate::context::CoreContext;
use crate::domain::addressing::WingPath;
use crate::infra::action::{error::ActionError, LibraryAction};
use crate::infra::query::{LibraryQuery, QueryResult};
use crate::library::Library;
use crate::service::integrations::cloud_edit::{CloudEditOpen, CloudEditSessionInfo};

/// Opens a file on a cloud volume for editing.
///
/// Returns a local copy to hand to the default app, or a browser URL for
/// files that only edit online, such as Google Docs.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CloudEditOpenInput {
	pub path: WingPath,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudEditOpenAction {
	input: CloudEditOpenInput,
}

impl LibraryAction for CloudEditOpenAction {
	type Input = CloudEditOpenInput;
	type Output = CloudEditOpen;

	fn from_input(input: Self::Input) -> Result<Self, String> {
		if input.path.as_cloud().is_none() {
			return Err("Only files on cloud volumes need a cloud editing session".to_string());
		}
		Ok(Self { input })
	}

	async fn execute(
		self,
		_library: Arc<Library>,
		context: Arc<CoreContext>,
	) -> Result<Self::Output, ActionError> {
		let (backend, remote_path) =
			crate::ops::cloud_sync::resolve_remote(&context, &self.input.path)
				.await
				.map_err(ActionError::InvalidInput)?;
		context
			.integrations
			.cloud_edits
			.open(self.input.path, backend, remote_path)
			.await
			.map_err(|e| ActionError::Internal(e.to_string()))
	}

	fn action_kind(&self) -> &'static str {
		"files.cloud_edit.open"
	}
}

crate::register_library_action!(CloudEditOpenAction, "files.cloud_edit.open");

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CloudEditListInput {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudEditListQuery {
	input: CloudEditListInput,
}

impl LibraryQuery for CloudEditListQuery {
	type Input = CloudEditListInput;
	type Output = Vec<CloudEditSessionInfo>;

	fn from_input(input: Self::Input) -> QueryResult<Self> {
		Ok(Self { input })
	}

	async fn execute(
		self,
		context: Arc<CoreContext>,
		_session: crate::infra::api::SessionContext,
	) -> QueryResult<Self::Output> {
		Ok(context.integrations.cloud_edits.list().await)
	}
}

crate::register_library_query!(CloudEditListQuery, "files.cloud_edit.list");

/// Uploads pending changes and removes the local copy.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CloudEditCloseInput {
	pub id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CloudEditCloseOutput {
	pub id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudEditCloseAction {
	input: CloudEditCloseInput,
}

impl LibraryAction for CloudEditCloseAction {
	type Input = CloudEditCloseInput;
	type Output = CloudEditCloseOutput;

	fn from_input(input: Self::Input) -> Result<Self, String> {
		Ok(Self { input })
	}

	async fn execute(
		self,
		_library: Arc<Library>,
		context: Arc<CoreContext>,
	) -> Result<Self::Output, ActionError> {
		context
			.integrations
			.cloud_edits
			.close(self.input.id)
			.await
			.map_err(|e| ActionError::Internal(e.to_string()))?;
		Ok(CloudEditCloseOutput { id: self.input.id })
	}

	fn action_kind(&self) -> &'static str {
		"files.cloud_edit.close"
	}
}

crate::register_library_action!(CloudEditCloseAction, "files.cloud_edit.close");
