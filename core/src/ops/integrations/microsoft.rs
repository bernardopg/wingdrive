//! Microsoft sign-in, app registration settings and OneDrive mounting.

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use specta::Type;
use uuid::Uuid;

use crate::context::CoreContext;
use crate::infra::action::{error::ActionError, LibraryAction};
use crate::infra::query::{LibraryQuery, QueryResult};
use crate::library::Library;
use crate::service::integrations::microsoft::oauth::is_client_id;
use crate::service::integrations::microsoft::MicrosoftClient;
use crate::service::integrations::store::IntegrationStore;
use crate::service::integrations::{
	ConnectionMode, DriveMountInfo, IntegrationError, OAuthClientsInfo,
};

/// Which app registrations the library can use for Microsoft sign-in:
/// WingDrive's own and the user's own from Microsoft Entra.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct MicrosoftClientInfoInput {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicrosoftClientInfoQuery {
	input: MicrosoftClientInfoInput,
}

impl LibraryQuery for MicrosoftClientInfoQuery {
	type Input = MicrosoftClientInfoInput;
	type Output = OAuthClientsInfo;

	fn from_input(input: Self::Input) -> QueryResult<Self> {
		Ok(Self { input })
	}

	async fn execute(
		self,
		context: Arc<CoreContext>,
		session: crate::infra::api::SessionContext,
	) -> QueryResult<Self::Output> {
		let library = super::session_library(&context, &session).await?;
		let store = IntegrationStore::for_library(&library);
		Ok(context.integrations.microsoft_clients(&store).await?)
	}
}

crate::register_library_query!(MicrosoftClientInfoQuery, "integrations.microsoft.client");

/// Saves or clears the library's own Microsoft Entra app registration.
///
/// Register it as a public client ("Mobile and desktop applications") with
/// the redirect URI `http://localhost/oauth/callback`. Public clients have no
/// secret, so only the application (client) ID is needed.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct MicrosoftSetClientInput {
	/// `None` clears the custom app registration.
	pub client_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicrosoftSetClientAction {
	input: MicrosoftSetClientInput,
}

impl LibraryAction for MicrosoftSetClientAction {
	type Input = MicrosoftSetClientInput;
	type Output = OAuthClientsInfo;

	fn from_input(input: Self::Input) -> Result<Self, String> {
		if let Some(id) = &input.client_id {
			if !is_client_id(id) {
				return Err(
					"A Microsoft application (client) ID looks like 00000000-0000-0000-0000-000000000000"
						.to_string(),
				);
			}
		}
		Ok(Self { input })
	}

	async fn execute(
		self,
		library: Arc<Library>,
		context: Arc<CoreContext>,
	) -> Result<Self::Output, ActionError> {
		let store = IntegrationStore::for_library(&library);
		let client = self.input.client_id.map(|id| MicrosoftClient {
			client_id: id.trim().to_lowercase(),
		});
		store
			.set_microsoft_client(client.as_ref())
			.await
			.map_err(IntegrationError::from)?;
		Ok(context.integrations.microsoft_clients(&store).await?)
	}

	fn action_kind(&self) -> &'static str {
		"integrations.microsoft.set_client"
	}
}

crate::register_library_action!(
	MicrosoftSetClientAction,
	"integrations.microsoft.set_client"
);

/// Starts a Microsoft sign-in. Open `auth_url` in the browser, then poll
/// `integrations.google.connect_status` with `session_id`.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct MicrosoftConnectInput {
	/// Whose app registration to sign in with.
	pub mode: ConnectionMode,
	/// Preselects an account on the sign-in page, for reconnecting.
	pub login_hint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct MicrosoftConnectOutput {
	pub session_id: Uuid,
	pub auth_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicrosoftConnectAction {
	input: MicrosoftConnectInput,
}

impl LibraryAction for MicrosoftConnectAction {
	type Input = MicrosoftConnectInput;
	type Output = MicrosoftConnectOutput;

	fn from_input(input: Self::Input) -> Result<Self, String> {
		Ok(Self { input })
	}

	async fn execute(
		self,
		library: Arc<Library>,
		context: Arc<CoreContext>,
	) -> Result<Self::Output, ActionError> {
		let (session_id, auth_url) = context
			.integrations
			.start_microsoft_connect(library, self.input.mode, self.input.login_hint)
			.await?;
		Ok(MicrosoftConnectOutput {
			session_id,
			auth_url,
		})
	}

	fn action_kind(&self) -> &'static str {
		"integrations.microsoft.connect"
	}
}

crate::register_library_action!(MicrosoftConnectAction, "integrations.microsoft.connect");

/// Mounts or unmounts a Microsoft account's OneDrive.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct OneDriveMountInput {
	pub account_id: String,
	pub mounted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct OneDriveMountOutput {
	pub drive: Option<DriveMountInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OneDriveMountAction {
	input: OneDriveMountInput,
}

impl LibraryAction for OneDriveMountAction {
	type Input = OneDriveMountInput;
	type Output = OneDriveMountOutput;

	fn from_input(input: Self::Input) -> Result<Self, String> {
		Ok(Self { input })
	}

	async fn execute(
		self,
		library: Arc<Library>,
		context: Arc<CoreContext>,
	) -> Result<Self::Output, ActionError> {
		let mut account = IntegrationStore::for_library(&library)
			.microsoft_account(&self.input.account_id)
			.await
			.map_err(IntegrationError::from)?
			.ok_or_else(|| IntegrationError::AccountNotFound(self.input.account_id.clone()))?;
		if self.input.mounted {
			let drive = context
				.integrations
				.mount_onedrive(&library, &mut account)
				.await?;
			Ok(OneDriveMountOutput { drive: Some(drive) })
		} else {
			context
				.integrations
				.unmount_onedrive(&library, &mut account)
				.await?;
			Ok(OneDriveMountOutput { drive: None })
		}
	}

	fn action_kind(&self) -> &'static str {
		"integrations.onedrive.mount"
	}
}

crate::register_library_action!(OneDriveMountAction, "integrations.onedrive.mount");
