//! `adapters.store.remove`: uninstalls a store adapter from every library.

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use specta::Type;

use super::{open, reload_libraries};
use crate::{
	context::CoreContext,
	infra::action::{error::ActionError, CoreAction},
};

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct RemoveAdapterInput {
	pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct RemoveAdapterOutput {
	pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveAdapterAction {
	input: RemoveAdapterInput,
}

impl CoreAction for RemoveAdapterAction {
	type Input = RemoveAdapterInput;
	type Output = RemoveAdapterOutput;

	fn from_input(input: Self::Input) -> Result<Self, String> {
		if input.id.trim().is_empty() {
			return Err("Adapter id cannot be empty".to_string());
		}
		Ok(Self { input })
	}

	async fn execute(self, context: Arc<CoreContext>) -> Result<Self::Output, ActionError> {
		let id = self.input.id.trim().to_string();
		let store = open(&context).map_err(ActionError::Internal)?;
		store
			.remove(&id)
			.map_err(|e| ActionError::Internal(e.to_string()))?;
		reload_libraries(&context, Some(&id)).await;
		Ok(RemoveAdapterOutput { id })
	}

	fn action_kind(&self) -> &'static str {
		"adapters.store.remove"
	}
}

crate::register_core_action!(RemoveAdapterAction, "adapters.store.remove");
