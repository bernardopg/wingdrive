use crate::domain::addressing::WingPath;
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum DiffStrategy {
	/// Compare by relative path + size + mtime. Fast, no hashing.
	#[default]
	Heuristic,
	/// Compare by BLAKE3 content hash. Catches renames/moves.
	Content,
	/// Heuristic first, content-verify ambiguous matches.
	Hybrid,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PathDiffInput {
	pub source: WingPath,
	pub target: WingPath,
	#[serde(default)]
	pub strategy: DiffStrategy,
	/// When false, use a complete scan for full coverage.
	#[serde(default = "default_use_index_rules")]
	pub use_index_rules: bool,
}

fn default_use_index_rules() -> bool {
	true
}
