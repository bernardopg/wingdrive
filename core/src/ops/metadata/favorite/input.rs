//! Input for set favorite action

use serde::{Deserialize, Serialize};
use specta::Type;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SetFavoriteInput {
	/// Entry UUID of the file to favorite (File.id from the frontend)
	pub entry_uuid: Uuid,

	/// New favorite state
	pub is_favorite: bool,
}