use clap::{Args, ValueEnum};
use std::path::PathBuf;
use uuid::Uuid;

use wing_core::{
	domain::addressing::WingPath,
	ops::core::ephemeral_status::EphemeralCacheStatusInput,
	ops::indexing::{
		input::IndexInput,
		job::{IndexMode, IndexPersistence, IndexScope},
		verify::input::IndexVerifyInput,
	},
};

fn local_index_path(path: &str) -> anyhow::Result<PathBuf> {
	let address = WingPath::from_uri(path)
		.map_err(|error| anyhow::anyhow!("Invalid address {path:?}: {error}"))?;
	address
		.as_local_path()
		.map(|path| path.to_path_buf())
		.ok_or_else(|| anyhow::anyhow!("Non-local address not supported for indexing: {path}"))
}

#[derive(Debug, Clone, ValueEnum)]
pub enum IndexModeArg {
	Shallow,
	Content,
	Deep,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum IndexScopeArg {
	Current,
	Recursive,
}

impl From<IndexModeArg> for IndexMode {
	fn from(m: IndexModeArg) -> Self {
		match m {
			IndexModeArg::Shallow => Self::Shallow,
			IndexModeArg::Content => Self::Content,
			IndexModeArg::Deep => Self::Deep,
		}
	}
}

impl From<IndexScopeArg> for IndexScope {
	fn from(s: IndexScopeArg) -> Self {
		match s {
			IndexScopeArg::Current => Self::Current,
			IndexScopeArg::Recursive => Self::Recursive,
		}
	}
}

#[derive(Args, Debug, Clone)]
pub struct IndexStartArgs {
	/// Addresses to index (WingPath URIs or local paths)
	pub paths: Vec<String>,

	/// Library ID to run indexing in (defaults to the only library if just one exists)
	#[arg(long)]
	pub library: Option<Uuid>,

	/// Indexing mode
	#[arg(long, value_enum, default_value = "deep")]
	pub mode: IndexModeArg,

	/// Indexing scope
	#[arg(long, value_enum, default_value = "recursive")]
	pub scope: IndexScopeArg,

	/// Include hidden files
	#[arg(long, default_value_t = false)]
	pub include_hidden: bool,

	/// Persist results to the database instead of in-memory
	#[arg(long, default_value_t = false)]
	pub persistent: bool,
}

impl IndexStartArgs {
	pub fn to_input(&self, library_id: Uuid) -> anyhow::Result<IndexInput> {
		let local_paths = self
			.paths
			.iter()
			.map(|path| local_index_path(path))
			.collect::<anyhow::Result<Vec<_>>>()?;

		let persistence = if self.persistent {
			IndexPersistence::Persistent
		} else {
			IndexPersistence::Ephemeral
		};

		Ok(IndexInput::new(library_id, local_paths)
			.with_mode(IndexMode::from(self.mode.clone()))
			.with_scope(IndexScope::from(self.scope.clone()))
			.with_include_hidden(self.include_hidden)
			.with_persistence(persistence))
	}
}

#[derive(Args, Debug, Clone)]
pub struct QuickScanArgs {
	pub path: String,
	#[arg(long, value_enum, default_value = "current")]
	pub scope: IndexScopeArg,
}

impl QuickScanArgs {
	pub fn to_input(&self, library_id: Uuid) -> anyhow::Result<IndexInput> {
		let path = local_index_path(&self.path)?;
		Ok(IndexInput::new(library_id, vec![path])
			.with_mode(IndexMode::Shallow)
			.with_scope(IndexScope::from(self.scope.clone()))
			.with_persistence(IndexPersistence::Ephemeral))
	}
}

#[derive(Args, Debug, Clone)]
pub struct BrowseArgs {
	pub path: String,
	#[arg(long, value_enum, default_value = "current")]
	pub scope: IndexScopeArg,
	#[arg(long, default_value_t = false)]
	pub content: bool,
}

impl BrowseArgs {
	pub fn to_input(&self, library_id: Uuid) -> anyhow::Result<IndexInput> {
		let path = local_index_path(&self.path)?;
		Ok(IndexInput::new(library_id, vec![path])
			.with_mode(if self.content {
				IndexMode::Content
			} else {
				IndexMode::Shallow
			})
			.with_scope(IndexScope::from(self.scope.clone()))
			.with_persistence(IndexPersistence::Ephemeral))
	}
}

#[derive(Args, Debug, Clone)]
pub struct IndexVerifyArgs {
	/// Path to verify (can be location root or subdirectory)
	pub path: PathBuf,

	/// Verify content hashes (slower but more thorough)
	#[arg(long, default_value_t = false)]
	pub verify_content: bool,

	/// Show detailed file-by-file comparison
	#[arg(long, default_value_t = true)]
	pub detailed: bool,

	/// Automatically fix issues (not yet implemented)
	#[arg(long, default_value_t = false)]
	pub auto_fix: bool,
}

impl IndexVerifyArgs {
	pub fn to_input(&self) -> IndexVerifyInput {
		IndexVerifyInput {
			path: self.path.clone(),
			verify_content: self.verify_content,
			detailed_report: self.detailed,
			auto_fix: self.auto_fix,
		}
	}
}

/// Arguments for ephemeral cache status
#[derive(Args, Debug, Clone)]
pub struct EphemeralCacheArgs {
	/// Filter by path substring
	#[arg(long)]
	pub filter: Option<String>,

	/// Show detailed memory breakdown
	#[arg(long, default_value_t = false)]
	pub detailed: bool,
}

impl EphemeralCacheArgs {
	pub fn to_input(&self) -> EphemeralCacheStatusInput {
		EphemeralCacheStatusInput {
			path_filter: self.filter.clone(),
			detailed: self.detailed,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use clap::Parser;

	#[derive(Parser)]
	struct StartCli {
		#[command(flatten)]
		args: IndexStartArgs,
	}

	#[derive(Parser)]
	struct QuickCli {
		#[command(flatten)]
		args: QuickScanArgs,
	}

	#[derive(Parser)]
	struct BrowseCli {
		#[command(flatten)]
		args: BrowseArgs,
	}

	#[test]
	fn start_preserves_multiple_paths_and_explicit_index_options() {
		let library = Uuid::new_v4();
		let args = StartCli::try_parse_from([
			"wing",
			"one",
			"two",
			"--mode",
			"content",
			"--scope",
			"current",
			"--persistent",
			"--include-hidden",
		])
		.unwrap()
		.args;
		let input = args.to_input(library).unwrap();
		assert_eq!(input.library_id, library);
		assert_eq!(input.paths, [PathBuf::from("one"), PathBuf::from("two")]);
		assert!(matches!(input.mode, IndexMode::Content));
		assert!(matches!(input.scope, IndexScope::Current));
		assert!(matches!(input.persistence, IndexPersistence::Persistent));
		assert!(input.include_hidden);
	}

	#[test]
	fn index_commands_apply_their_default_modes_and_scope() {
		let library = Uuid::new_v4();
		let start = StartCli::try_parse_from(["wing", "one"])
			.unwrap()
			.args
			.to_input(library)
			.unwrap();
		assert!(matches!(start.mode, IndexMode::Deep));
		assert!(matches!(start.scope, IndexScope::Recursive));
		assert!(matches!(start.persistence, IndexPersistence::Ephemeral));
		assert!(!start.include_hidden);
		let quick = QuickCli::try_parse_from(["wing", "one", "--scope", "recursive"])
			.unwrap()
			.args
			.to_input(library)
			.unwrap();
		assert!(matches!(quick.mode, IndexMode::Shallow));
		assert!(matches!(quick.scope, IndexScope::Recursive));
		assert!(matches!(quick.persistence, IndexPersistence::Ephemeral));
		let browse = BrowseCli::try_parse_from(["wing", "one", "--content"])
			.unwrap()
			.args
			.to_input(library)
			.unwrap();
		assert!(matches!(browse.mode, IndexMode::Content));
		assert!(matches!(browse.scope, IndexScope::Current));
		assert!(matches!(browse.persistence, IndexPersistence::Ephemeral));
	}

	#[test]
	fn malformed_and_nonlocal_addresses_are_rejected_by_all_index_commands() {
		let library = Uuid::new_v4();
		for path in [
			"content://invalid",
			"unknown://bucket/file",
			"s3://bucket/file",
			"local://other-device/file",
		] {
			let start = StartCli::try_parse_from(["wing", path])
				.unwrap()
				.args
				.to_input(library);
			let quick = QuickCli::try_parse_from(["wing", path])
				.unwrap()
				.args
				.to_input(library);
			let browse = BrowseCli::try_parse_from(["wing", path])
				.unwrap()
				.args
				.to_input(library);
			assert!(start.is_err(), "start: {path}");
			assert!(quick.is_err(), "quick: {path}");
			assert!(browse.is_err(), "browse: {path}");
		}
		assert!(local_index_path("content://invalid")
			.unwrap_err()
			.to_string()
			.contains("Invalid address"));
		assert_eq!(
			local_index_path("relative/path").unwrap(),
			PathBuf::from("relative/path")
		);
		assert!(local_index_path("local://local/folder").is_ok());
	}

	#[test]
	fn index_arguments_reject_invalid_values() {
		for args in [
			vec!["wing", "one", "--library", "invalid"],
			vec!["wing", "one", "--mode", "unknown"],
			vec!["wing", "one", "--scope", "unknown"],
		] {
			assert!(StartCli::try_parse_from(&args).is_err(), "{args:?}");
		}
		let empty = StartCli::try_parse_from(["wing"])
			.unwrap()
			.args
			.to_input(Uuid::new_v4())
			.unwrap();
		assert!(empty.validate().is_err());
	}
}

#[cfg(test)]
#[path = "args/matrix.rs"]
mod matrix;
