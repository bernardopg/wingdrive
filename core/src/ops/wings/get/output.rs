use crate::domain::Wing;
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct WingGetOutput {
	pub wing: Wing,
}
