use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct WingCreateInput {
	pub name: String,
	pub icon: String,
	pub color: String,
}
