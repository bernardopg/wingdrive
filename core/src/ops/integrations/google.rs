//! Google sign-in, OAuth client settings and Drive mounting.

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use specta::Type;
use uuid::Uuid;

use crate::context::CoreContext;
use crate::infra::action::{error::ActionError, LibraryAction};
use crate::infra::query::{LibraryQuery, QueryResult};
use crate::library::Library;
use crate::service::integrations::google::oauth::{GoogleService, OAuthClient};
use crate::service::integrations::store::IntegrationStore;
use crate::service::integrations::{
	ConnectStatus, ConnectionMode, DriveMountInfo, IntegrationError, OAuthClientsInfo,
};

/// Which OAuth clients the library can use for Google sign-in: WingDrive's
/// own ("sign in with your Google account") and the user's own from Google Cloud.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GoogleClientInfoInput {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleClientInfoQuery {
	input: GoogleClientInfoInput,
}

impl LibraryQuery for GoogleClientInfoQuery {
	type Input = GoogleClientInfoInput;
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
		Ok(context.integrations.google_clients(&store).await?)
	}
}

crate::register_library_query!(GoogleClientInfoQuery, "integrations.google.client");

/// Saves or clears the library's own Google OAuth client.
///
/// Create it in Google Cloud Console as an OAuth client of type "Desktop
/// app". It is used by sign-ins in the "custom" mode; accounts already
/// connected keep the client they signed in with.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GoogleSetClientInput {
	/// `None` clears the custom client.
	pub client_id: Option<String>,
	pub client_secret: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleSetClientAction {
	input: GoogleSetClientInput,
}

impl LibraryAction for GoogleSetClientAction {
	type Input = GoogleSetClientInput;
	type Output = OAuthClientsInfo;

	fn from_input(input: Self::Input) -> Result<Self, String> {
		match (&input.client_id, &input.client_secret) {
			(None, _) => {}
			(Some(id), Some(secret)) if !id.trim().is_empty() && !secret.trim().is_empty() => {
				if !id.trim().ends_with(".apps.googleusercontent.com") {
					return Err(
						"A Google client ID ends with .apps.googleusercontent.com".to_string()
					);
				}
			}
			_ => return Err("Enter both the client ID and the client secret".to_string()),
		}
		Ok(Self { input })
	}

	async fn execute(
		self,
		library: Arc<Library>,
		context: Arc<CoreContext>,
	) -> Result<Self::Output, ActionError> {
		let store = IntegrationStore::for_library(&library);
		let client = match (self.input.client_id, self.input.client_secret) {
			(Some(client_id), Some(client_secret)) => Some(OAuthClient {
				client_id: client_id.trim().to_string(),
				client_secret: client_secret.trim().to_string(),
			}),
			_ => None,
		};
		store
			.set_google_client(client.as_ref())
			.await
			.map_err(IntegrationError::from)?;
		Ok(context.integrations.google_clients(&store).await?)
	}

	fn action_kind(&self) -> &'static str {
		"integrations.google.set_client"
	}
}

crate::register_library_action!(GoogleSetClientAction, "integrations.google.set_client");

/// Starts a Google sign-in. Open `auth_url` in the browser, then poll
/// `integrations.google.connect_status` with `session_id`.
///
/// `integrations.google.connect_status` and `integrations.google.connect_cancel`
/// follow Microsoft sign-ins too; the session IDs are shared.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GoogleConnectInput {
	/// Whose OAuth client to sign in with.
	pub mode: ConnectionMode,
	/// Services to request; empty requests Drive and Photos.
	pub services: Vec<GoogleService>,
	/// Preselects an account on the consent page, for reconnecting.
	pub login_hint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GoogleConnectOutput {
	pub session_id: Uuid,
	pub auth_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleConnectAction {
	input: GoogleConnectInput,
}

impl LibraryAction for GoogleConnectAction {
	type Input = GoogleConnectInput;
	type Output = GoogleConnectOutput;

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
			.start_google_connect(
				library,
				self.input.mode,
				self.input.services,
				self.input.login_hint,
			)
			.await?;
		Ok(GoogleConnectOutput {
			session_id,
			auth_url,
		})
	}

	fn action_kind(&self) -> &'static str {
		"integrations.google.connect"
	}
}

crate::register_library_action!(GoogleConnectAction, "integrations.google.connect");

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GoogleConnectStatusInput {
	pub session_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleConnectStatusQuery {
	input: GoogleConnectStatusInput,
}

impl LibraryQuery for GoogleConnectStatusQuery {
	type Input = GoogleConnectStatusInput;
	type Output = ConnectStatus;

	fn from_input(input: Self::Input) -> QueryResult<Self> {
		Ok(Self { input })
	}

	async fn execute(
		self,
		context: Arc<CoreContext>,
		session: crate::infra::api::SessionContext,
	) -> QueryResult<Self::Output> {
		let library = super::session_library(&context, &session).await?;
		Ok(context
			.integrations
			.connect_status(library.id(), self.input.session_id)
			.await
			.unwrap_or(ConnectStatus::Failed {
				message: "The sign-in session expired; start again".to_string(),
			}))
	}
}

crate::register_library_query!(
	GoogleConnectStatusQuery,
	"integrations.google.connect_status"
);

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GoogleConnectCancelInput {
	pub session_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GoogleConnectCancelOutput {
	pub cancelled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleConnectCancelAction {
	input: GoogleConnectCancelInput,
}

impl LibraryAction for GoogleConnectCancelAction {
	type Input = GoogleConnectCancelInput;
	type Output = GoogleConnectCancelOutput;

	fn from_input(input: Self::Input) -> Result<Self, String> {
		Ok(Self { input })
	}

	async fn execute(
		self,
		library: Arc<Library>,
		context: Arc<CoreContext>,
	) -> Result<Self::Output, ActionError> {
		Ok(GoogleConnectCancelOutput {
			cancelled: context
				.integrations
				.cancel_connect(library.id(), self.input.session_id)
				.await,
		})
	}

	fn action_kind(&self) -> &'static str {
		"integrations.google.connect_cancel"
	}
}

crate::register_library_action!(
	GoogleConnectCancelAction,
	"integrations.google.connect_cancel"
);

/// Mounts or unmounts an account's Google Drive.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GoogleDriveMountInput {
	pub account_id: String,
	pub mounted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GoogleDriveMountOutput {
	pub drive: Option<DriveMountInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleDriveMountAction {
	input: GoogleDriveMountInput,
}

impl LibraryAction for GoogleDriveMountAction {
	type Input = GoogleDriveMountInput;
	type Output = GoogleDriveMountOutput;

	fn from_input(input: Self::Input) -> Result<Self, String> {
		Ok(Self { input })
	}

	async fn execute(
		self,
		library: Arc<Library>,
		context: Arc<CoreContext>,
	) -> Result<Self::Output, ActionError> {
		let store = IntegrationStore::for_library(&library);
		let mut account = store
			.google_account(&self.input.account_id)
			.await
			.map_err(IntegrationError::from)?
			.ok_or_else(|| IntegrationError::AccountNotFound(self.input.account_id.clone()))?;
		if self.input.mounted {
			if !GoogleService::Drive.is_granted(&account.scopes) {
				return Err(ActionError::InvalidInput(
					"This account did not grant Google Drive access; reconnect it with Drive enabled"
						.to_string(),
				));
			}
			let drive = context
				.integrations
				.mount_drive(&library, &mut account)
				.await?;
			Ok(GoogleDriveMountOutput { drive: Some(drive) })
		} else {
			context
				.integrations
				.unmount_drive(&library, &mut account)
				.await?;
			store
				.save_google_account(&account)
				.await
				.map_err(IntegrationError::from)?;
			Ok(GoogleDriveMountOutput { drive: None })
		}
	}

	fn action_kind(&self) -> &'static str {
		"integrations.google.drive.mount"
	}
}

crate::register_library_action!(GoogleDriveMountAction, "integrations.google.drive.mount");
