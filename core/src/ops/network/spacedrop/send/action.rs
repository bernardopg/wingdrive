use super::{input::SpacedropSendInput, output::SpacedropSendOutput};
use crate::infra::action::{error::ActionError, CoreAction};
use std::sync::Arc;

pub struct SpacedropSendAction {
	pub device_id: uuid::Uuid,
	pub paths: Vec<crate::domain::addressing::WingPath>,
	pub sender: Option<String>,
}

impl CoreAction for SpacedropSendAction {
	type Output = SpacedropSendOutput;
	type Input = SpacedropSendInput;

	fn from_input(input: Self::Input) -> std::result::Result<Self, String> {
		Ok(Self {
			device_id: input.device_id,
			paths: input.paths,
			sender: input.sender,
		})
	}

	async fn execute(
		self,
		context: Arc<crate::context::CoreContext>,
	) -> std::result::Result<Self::Output, ActionError> {
		let _ = context;
		// Not implemented: this used to return a random session id and report
		// success while nothing was sent. Copy to a paired device with
		// files.copy and a local://<device>/path destination instead.
		Err(ActionError::Internal(
			"Wingdrop is not implemented yet; copy to the paired device with files.copy instead"
				.to_string(),
		))
	}

	fn action_kind(&self) -> &'static str {
		"network.spacedrop.send"
	}
}

crate::register_core_action!(SpacedropSendAction, "network.spacedrop.send");
