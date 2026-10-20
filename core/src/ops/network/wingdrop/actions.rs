//! Wingdrop actions and queries.

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use specta::Type;
use uuid::Uuid;

use super::job::WingdropSendJob;
use crate::context::CoreContext;
use crate::domain::addressing::WingPath;
use crate::infra::action::{error::ActionError, CoreAction, LibraryAction, ValidationResult};
use crate::infra::job::handle::JobReceipt;
use crate::infra::query::{CoreQuery, QueryResult};
use crate::service::wingdrop::{self, WingdropMessage, WingdropOffer, WingdropStatus};

/// Input for sending files to a paired device
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct WingdropSendInput {
	/// The paired device that receives the files
	pub device_id: Uuid,
	/// Files and folders on this device
	pub paths: Vec<WingPath>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WingdropSendAction {
	input: WingdropSendInput,
}

fn local_sources(paths: &[WingPath]) -> Result<Vec<std::path::PathBuf>, ActionError> {
	paths
		.iter()
		.map(|path| {
			path.as_local_path()
				.map(std::path::Path::to_path_buf)
				.ok_or_else(|| ActionError::Validation {
					field: "paths".to_string(),
					message: "Wingdrop sends files from this device only".to_string(),
				})
		})
		.collect()
}

impl LibraryAction for WingdropSendAction {
	type Input = WingdropSendInput;
	type Output = JobReceipt;

	fn from_input(input: Self::Input) -> Result<Self, String> {
		Ok(Self { input })
	}

	async fn validate(
		&self,
		_library: &Arc<crate::library::Library>,
		context: Arc<CoreContext>,
	) -> Result<ValidationResult, ActionError> {
		if self.input.paths.is_empty() {
			return Err(ActionError::Validation {
				field: "paths".to_string(),
				message: "Nothing to send".to_string(),
			});
		}
		for source in local_sources(&self.input.paths)? {
			if std::fs::symlink_metadata(&source).is_err() {
				return Err(ActionError::Validation {
					field: "paths".to_string(),
					message: format!("{} does not exist", source.display()),
				});
			}
		}
		if self.input.device_id == context.device_manager.device_id().unwrap_or_default() {
			return Err(ActionError::Validation {
				field: "device_id".to_string(),
				message: "Choose another device".to_string(),
			});
		}
		Ok(ValidationResult::Success { metadata: None })
	}

	async fn execute(
		self,
		library: Arc<crate::library::Library>,
		_context: Arc<CoreContext>,
	) -> Result<Self::Output, ActionError> {
		let job = WingdropSendJob {
			drop_id: Uuid::new_v4(),
			device_id: self.input.device_id,
			sources: local_sources(&self.input.paths)?,
		};
		Ok(library
			.jobs()
			.dispatch(job)
			.await
			.map_err(ActionError::Job)?
			.into())
	}

	fn action_kind(&self) -> &'static str {
		"network.wingdrop.send"
	}
}

/// Input for answering an offer
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct WingdropRespondInput {
	pub drop_id: Uuid,
	pub accept: bool,
}

/// Where accepted items will land, in offer order
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct WingdropRespondOutput {
	pub destinations: Vec<String>,
}

pub struct WingdropRespondAction {
	input: WingdropRespondInput,
}

impl CoreAction for WingdropRespondAction {
	type Input = WingdropRespondInput;
	type Output = WingdropRespondOutput;

	fn from_input(input: Self::Input) -> Result<Self, String> {
		Ok(Self { input })
	}

	async fn execute(self, context: Arc<CoreContext>) -> Result<Self::Output, ActionError> {
		let drop_id = self.input.drop_id;
		let state = &context.wingdrop;
		let (sender, destinations, status) = if self.input.accept {
			let root = crate::config::AppConfig::load_from(&context.data_dir)
				.ok()
				.and_then(|config| config.wingdrop.destination);
			let root = wingdrop::destination_root(root.as_deref());
			let (sender, paths) = state
				.accept(drop_id, &root)
				.map_err(|e| ActionError::Internal(e.to_string()))?;
			let paths: Vec<String> = paths
				.iter()
				.map(|p| p.to_string_lossy().into_owned())
				.collect();
			(sender, paths, WingdropStatus::Accepted)
		} else {
			let sender = state
				.decline(drop_id)
				.map_err(|e| ActionError::Internal(e.to_string()))?;
			(sender, Vec::new(), WingdropStatus::Declined)
		};

		if let Err(error) = wingdrop::send_decision(
			&context,
			sender,
			drop_id,
			self.input.accept,
			destinations.clone(),
		)
		.await
		{
			// The sender cannot be told, so nothing will arrive; free what was reserved.
			state.cancel_incoming(drop_id, None);
			wingdrop::emit_status(
				&context.events,
				drop_id,
				WingdropStatus::Cancelled {
					reason: Some(error.clone()),
				},
			);
			return Err(ActionError::Internal(format!(
				"Could not reach the sending device: {error}"
			)));
		}
		wingdrop::emit_status(&context.events, drop_id, status);
		Ok(WingdropRespondOutput { destinations })
	}

	fn action_kind(&self) -> &'static str {
		"network.wingdrop.respond"
	}
}

/// Input for stopping a drop this device receives
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct WingdropCancelInput {
	pub drop_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct WingdropCancelOutput {
	pub cancelled: bool,
}

pub struct WingdropCancelAction {
	input: WingdropCancelInput,
}

impl CoreAction for WingdropCancelAction {
	type Input = WingdropCancelInput;
	type Output = WingdropCancelOutput;

	fn from_input(input: Self::Input) -> Result<Self, String> {
		Ok(Self { input })
	}

	async fn execute(self, context: Arc<CoreContext>) -> Result<Self::Output, ActionError> {
		let drop_id = self.input.drop_id;
		let Some(sender) = context.wingdrop.incoming_sender(drop_id) else {
			return Ok(WingdropCancelOutput { cancelled: false });
		};
		context.wingdrop.cancel_incoming(drop_id, None);
		let reason = Some("Cancelled by the receiving device".to_string());
		// Best effort: the sender also stops when its next file is refused.
		if let Err(error) = wingdrop::send_message(
			&context,
			sender,
			WingdropMessage::Cancelled {
				drop_id,
				reason: reason.clone(),
			},
		)
		.await
		{
			tracing::warn!(%drop_id, %error, "Could not tell the sender a Wingdrop was cancelled");
		}
		wingdrop::emit_status(
			&context.events,
			drop_id,
			WingdropStatus::Cancelled { reason },
		);
		Ok(WingdropCancelOutput { cancelled: true })
	}

	fn action_kind(&self) -> &'static str {
		"network.wingdrop.cancel"
	}
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct WingdropPendingInput;

/// Offers waiting for an answer, oldest first
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct WingdropPendingQuery;

impl CoreQuery for WingdropPendingQuery {
	type Input = WingdropPendingInput;
	type Output = Vec<WingdropOffer>;

	fn from_input(_input: Self::Input) -> QueryResult<Self> {
		Ok(Self)
	}

	async fn execute(
		self,
		context: Arc<CoreContext>,
		_session: crate::infra::api::SessionContext,
	) -> QueryResult<Self::Output> {
		Ok(context.wingdrop.pending_offers())
	}
}

crate::register_library_action!(WingdropSendAction, "network.wingdrop.send");
crate::register_core_action!(WingdropRespondAction, "network.wingdrop.respond");
crate::register_core_action!(WingdropCancelAction, "network.wingdrop.cancel");
crate::register_core_query!(WingdropPendingQuery, "network.wingdrop.pending");
