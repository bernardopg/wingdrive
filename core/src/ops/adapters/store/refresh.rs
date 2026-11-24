//! `adapters.store.refresh`: downloads and verifies the newest signed index.

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use specta::Type;

use super::{fetch_index, listing, open, StoreListing};
use crate::{
	context::CoreContext,
	infra::action::{error::ActionError, CoreAction},
};

#[derive(Debug, Clone, Default, Serialize, Deserialize, Type)]
pub struct RefreshStoreInput {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshStoreAction;

impl CoreAction for RefreshStoreAction {
	type Input = RefreshStoreInput;
	type Output = StoreListing;

	fn from_input(_input: Self::Input) -> Result<Self, String> {
		Ok(Self)
	}

	async fn execute(self, context: Arc<CoreContext>) -> Result<Self::Output, ActionError> {
		let store = open(&context).map_err(ActionError::Internal)?;
		let index = fetch_index(&store).await.map_err(ActionError::Internal)?;
		listing(&store, Some(&index)).map_err(ActionError::Internal)
	}

	fn action_kind(&self) -> &'static str {
		"adapters.store.refresh"
	}
}

crate::register_core_action!(RefreshStoreAction, "adapters.store.refresh");
