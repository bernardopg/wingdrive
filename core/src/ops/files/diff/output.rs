use crate::domain::{addressing::WingPath, file::EntryKind};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum CompareStatus {
	OnlyInSource,
	OnlyInTarget,
	Modified,
	Matched,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct DiffEntry {
	pub relative_path: PathBuf,
	pub wing_path: WingPath,
	pub uuid: Option<Uuid>,
	pub size: u64,
	pub modified_at: Option<i64>,
	pub content_id: Option<String>,
	pub kind: EntryKind,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Type)]
pub struct PathDiffResult {
	pub only_in_source: Vec<DiffEntry>,
	pub only_in_target: Vec<DiffEntry>,
	pub modified: Vec<DiffEntry>,
	pub matched_count: usize,
	pub copy_size: u64,
	pub total_scanned: usize,
}
