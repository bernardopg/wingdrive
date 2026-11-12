//! Listing and removing connected accounts.

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use specta::Type;

use crate::context::CoreContext;
use crate::infra::action::{error::ActionError, LibraryAction};
use crate::infra::query::{LibraryQuery, QueryResult};
use crate::library::Library;
use crate::service::integrations::IntegrationAccountInfo;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct IntegrationAccountsListInput {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationAccountsListQuery {
	input: IntegrationAccountsListInput,
}

impl LibraryQuery for IntegrationAccountsListQuery {
	type Input = IntegrationAccountsListInput;
	type Output = Vec<IntegrationAccountInfo>;

	fn from_input(input: Self::Input) -> QueryResult<Self> {
		Ok(Self { input })
	}

	async fn execute(
		self,
		context: Arc<CoreContext>,
		session: crate::infra::api::SessionContext,
	) -> QueryResult<Self::Output> {
		let library = super::session_library(&context, &session).await?;
		Ok(context.integrations.list_accounts(&library).await?)
	}
}

crate::register_library_query!(IntegrationAccountsListQuery, "integrations.accounts.list");

/// Disconnects an account from the library.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct IntegrationAccountRemoveInput {
	pub account_id: String,
	/// Also revoke WingDrive's access at the provider. Leave on unless the
	/// same account stays connected in another library.
	pub revoke: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct IntegrationAccountRemoveOutput {
	pub account_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationAccountRemoveAction {
	input: IntegrationAccountRemoveInput,
}

impl LibraryAction for IntegrationAccountRemoveAction {
	type Input = IntegrationAccountRemoveInput;
	type Output = IntegrationAccountRemoveOutput;

	fn from_input(input: Self::Input) -> Result<Self, String> {
		if input.account_id.trim().is_empty() {
			return Err("account_id is required".to_string());
		}
		Ok(Self { input })
	}

	async fn execute(
		self,
		library: Arc<Library>,
		context: Arc<CoreContext>,
	) -> Result<Self::Output, ActionError> {
		context
			.integrations
			.remove_google_account(&library, &self.input.account_id, self.input.revoke)
			.await?;
		Ok(IntegrationAccountRemoveOutput {
			account_id: self.input.account_id,
		})
	}

	fn action_kind(&self) -> &'static str {
		"integrations.accounts.remove"
	}
}

crate::register_library_action!(
	IntegrationAccountRemoveAction,
	"integrations.accounts.remove"
);
