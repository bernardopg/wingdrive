//! Set favorite action

use super::{SetFavoriteInput, SetFavoriteOutput};
use crate::{
	context::CoreContext,
	infra::action::{error::ActionError, LibraryAction},
	infra::db::entities::entry,
	library::Library,
	ops::metadata::manager::UserMetadataManager,
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct SetFavoriteAction {
	input: SetFavoriteInput,
}

impl SetFavoriteAction {
	pub fn new(input: SetFavoriteInput) -> Self {
		Self { input }
	}
}

impl LibraryAction for SetFavoriteAction {
	type Input = SetFavoriteInput;
	type Output = SetFavoriteOutput;

	fn from_input(input: SetFavoriteInput) -> Result<Self, String> {
		Ok(SetFavoriteAction::new(input))
	}

	async fn execute(
		self,
		library: Arc<Library>,
		context: Arc<CoreContext>,
	) -> Result<Self::Output, ActionError> {
		let db = library.db();
		let exists = entry::Entity::find()
			.filter(entry::Column::Uuid.eq(self.input.entry_uuid))
			.one(db.conn())
			.await?
			.is_some();
		if !exists {
			return Err(ActionError::InvalidInput(format!(
				"Entry {} does not exist",
				self.input.entry_uuid
			)));
		}
		let metadata_manager = UserMetadataManager::new(Arc::new(db.conn().clone()));

		let (metadata, was_created) = metadata_manager
			.set_favorite(self.input.entry_uuid, self.input.is_favorite)
			.await
			.map_err(|e| ActionError::Internal(format!("Failed to set favorite: {}", e)))?;

		library
			.sync_model(
				&metadata,
				if was_created {
					crate::infra::sync::ChangeType::Insert
				} else {
					crate::infra::sync::ChangeType::Update
				},
			)
			.await
			.map_err(|e| ActionError::Internal(format!("Failed to sync favorite: {}", e)))?;

		// Emit resource event so the frontend updates the favorite state reactively
		let resource_manager = crate::domain::ResourceManager::new(
			Arc::new(db.conn().clone()),
			context.events.clone(),
		);
		if let Err(e) = resource_manager
			.emit_resource_events("file", vec![self.input.entry_uuid])
			.await
		{
			tracing::warn!(
				"Failed to emit resource events after setting favorite: {}",
				e
			);
		}

		Ok(SetFavoriteOutput {
			entry_uuid: self.input.entry_uuid,
			favorite: self.input.is_favorite,
		})
	}

	fn action_kind(&self) -> &'static str {
		"metadata.set_favorite"
	}
}

crate::register_library_action!(SetFavoriteAction, "metadata.set_favorite");
