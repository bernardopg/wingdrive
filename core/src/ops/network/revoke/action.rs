use super::{input::DeviceRevokeInput, output::DeviceRevokeOutput};
use crate::infra::action::{error::ActionError, CoreAction};
use std::sync::Arc;

/// How long a revocation waits for the peer to acknowledge it.
const PEER_NOTICE_WAIT: std::time::Duration = std::time::Duration::from_secs(5);

pub struct DeviceRevokeAction {
	pub device_id: uuid::Uuid,
	pub remove_from_library: bool,
}

impl CoreAction for DeviceRevokeAction {
	type Output = DeviceRevokeOutput;
	type Input = DeviceRevokeInput;

	fn from_input(input: Self::Input) -> std::result::Result<Self, String> {
		Ok(Self {
			device_id: input.device_id,
			remove_from_library: input.remove_from_library,
		})
	}

	async fn execute(
		self,
		context: Arc<crate::context::CoreContext>,
	) -> std::result::Result<Self::Output, ActionError> {
		tracing::info!("Revoking device: {}", self.device_id);

		let net = context
			.get_networking()
			.await
			.ok_or_else(|| ActionError::Internal("Networking not initialized".to_string()))?;

		// Tell the peer while the pairing still exists here: once it is
		// removed, this device no longer knows which node to talk to.
		let peer_node = net
			.device_registry()
			.read()
			.await
			.get_node_by_device(self.device_id);
		let peer_notified = match peer_node {
			Some(_) => match net.notify_unpaired(self.device_id, PEER_NOTICE_WAIT).await {
				Ok(()) => {
					tracing::info!("Device {} dropped the pairing on its side", self.device_id);
					true
				}
				Err(error) => {
					tracing::warn!(
						"Device {} was not told about the revocation and keeps the pairing on its side: {}",
						self.device_id,
						error
					);
					false
				}
			},
			None => false,
		};

		// Remove from network registry state, slug cache and persistence.
		// This also emits the ResourceDeleted event.
		{
			let reg = net.device_registry();
			let mut guard = reg.write().await;
			match guard.forget_paired_device(self.device_id).await {
				Ok(true) => {
					tracing::info!("Device {} removed from network storage", self.device_id)
				}
				Ok(false) => tracing::warn!(
					"Device {} not found in network persistent storage (already removed?)",
					self.device_id
				),
				Err(e) => {
					tracing::error!("Failed to remove device from network persistence: {}", e);
					return Err(ActionError::Internal(format!(
						"Failed to remove device from network persistence: {}",
						e
					)));
				}
			}
		}

		// An unpaired node has no business keeping open connections.
		if let Some(node_id) = peer_node {
			net.close_node_connections(node_id, "unpaired").await;
		}

		// Remove from all library databases (if requested)
		if self.remove_from_library {
			tracing::info!(
				"Removing device {} from library databases (remove_from_library=true)",
				self.device_id
			);
			let libraries = context.libraries().await;
			let mut removed_from_libraries = 0;

			for library in libraries.get_open_libraries().await {
				let db = library.db().conn();

				// Delete device from library database
				use crate::infra::db::entities::device;
				use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

				match device::Entity::delete_many()
					.filter(device::Column::Uuid.eq(self.device_id))
					.exec(db)
					.await
				{
					Ok(result) => {
						if result.rows_affected > 0 {
							tracing::info!(
								"Device {} removed from library {} database",
								self.device_id,
								library.id()
							);
							removed_from_libraries += 1;
						}
					}
					Err(e) => {
						tracing::warn!(
							"Failed to remove device from library {} database: {}",
							library.id(),
							e
						);
					}
				}
			}

			if removed_from_libraries > 0 {
				tracing::info!(
					"Device {} removed from {} library database(s)",
					self.device_id,
					removed_from_libraries
				);
			} else {
				tracing::warn!(
					"Device {} not found in any library databases (may have been removed already)",
					self.device_id
				);
			}
		} else {
			tracing::info!(
				"Skipping library database removal for device {} (remove_from_library=false)",
				self.device_id
			);
		}

		tracing::info!("Device {} successfully revoked", self.device_id);
		Ok(DeviceRevokeOutput {
			revoked: true,
			peer_notified,
		})
	}

	fn action_kind(&self) -> &'static str {
		"network.device.revoke"
	}
}

crate::register_core_action!(DeviceRevokeAction, "network.device.revoke");
