//! `adapters.store.install`: installs or updates one adapter from the store.

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use specta::Type;

use super::{download_archive, fetch_index, open, reload_libraries, StorePermissions};
use crate::{
	context::CoreContext,
	infra::action::{error::ActionError, CoreAction},
};

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct InstallAdapterInput {
	pub id: String,
	/// The user saw the permissions the adapter asks for and accepted them. Required when the
	/// adapter declares any, and again when an update asks for more.
	#[serde(default)]
	pub accept_permissions: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct InstallAdapterOutput {
	pub id: String,
	pub version: String,
	/// The version this install replaced, when it was an update.
	pub previous_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallAdapterAction {
	input: InstallAdapterInput,
}

impl CoreAction for InstallAdapterAction {
	type Input = InstallAdapterInput;
	type Output = InstallAdapterOutput;

	fn from_input(input: Self::Input) -> Result<Self, String> {
		if input.id.trim().is_empty() {
			return Err("Adapter id cannot be empty".to_string());
		}
		Ok(Self { input })
	}

	async fn execute(self, context: Arc<CoreContext>) -> Result<Self::Output, ActionError> {
		let store = open(&context).map_err(ActionError::Internal)?;
		let index = match store
			.cached_index()
			.map_err(|e| ActionError::Internal(e.to_string()))?
		{
			Some(index) => index,
			None => fetch_index(&store).await.map_err(ActionError::Internal)?,
		};
		let id = self.input.id.trim();
		let adapter = index.adapter(id).ok_or_else(|| ActionError::Validation {
			field: "id".to_string(),
			message: format!("{id} is not in the adapter store"),
		})?;
		if let Some(revocation) = index.revocation(id, &adapter.version) {
			return Err(ActionError::Internal(format!(
				"{id} {} is revoked: {}",
				adapter.version, revocation.reason
			)));
		}
		if !adapter.supports_this_platform() {
			return Err(ActionError::Internal(format!(
				"{id} does not run on {}",
				wing_archive::store::current_os()
			)));
		}

		let installed = store
			.installed()
			.map_err(|e| ActionError::Internal(e.to_string()))?
			.remove(id);
		let asked = StorePermissions::from(&adapter.permissions);
		let needs_consent = match &installed {
			None => !asked.is_empty(),
			Some(_) => store
				.updates(&index)
				.map_err(|e| ActionError::Internal(e.to_string()))?
				.iter()
				.any(|u| u.id == id && u.new_permissions),
		};
		if needs_consent && !self.input.accept_permissions {
			return Err(ActionError::Validation {
				field: "accept_permissions".to_string(),
				message: format!(
					"{id} asks for permissions (read: {:?}, network: {:?}, secrets: {:?}, tools: {:?}); accept them to install",
					asked.read, asked.network, asked.secrets, asked.tools
				),
			});
		}

		let archive = download_archive(adapter)
			.await
			.map_err(ActionError::Internal)?;
		let entry = store
			.install(adapter, &archive)
			.map_err(|e| ActionError::Internal(e.to_string()))?;
		reload_libraries(&context, None).await;
		Ok(InstallAdapterOutput {
			id: id.to_string(),
			version: entry.version,
			previous_version: installed.map(|e| e.version),
		})
	}

	fn action_kind(&self) -> &'static str {
		"adapters.store.install"
	}
}

crate::register_core_action!(InstallAdapterAction, "adapters.store.install");
