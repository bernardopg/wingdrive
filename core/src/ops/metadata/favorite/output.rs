//! Output for set favorite action

use serde::{Deserialize, Serialize};
use specta::Type;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SetFavoriteOutput {
	/// Entry UUID that was updated
	pub entry_uuid: Uuid,

	/// The favorite state after the operation
	pub favorite: bool,
}
