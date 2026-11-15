use crate::domain::Wing;
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct WingUpdateOutput {
	pub wing: Wing,
}
