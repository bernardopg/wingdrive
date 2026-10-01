use crate::domain::WingPath;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SuggestedLocation {
	pub name: String,
	pub path: PathBuf,
	pub wing_path: WingPath,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SuggestedLocationsOutput {
	pub locations: Vec<SuggestedLocation>,
}
