use crate::domain::addressing::WingPath;
use serde::{Deserialize, Serialize};
use specta::Type;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SpacedropSendInput {
	pub device_id: Uuid,
	pub paths: Vec<WingPath>,
	pub sender: Option<String>,
}
