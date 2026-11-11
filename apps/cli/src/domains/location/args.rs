use clap::Args;
use std::path::PathBuf;
use uuid::Uuid;

use wing_core::{
	domain::addressing::WingPath,
	ops::{
		indexing::job::IndexMode,
		locations::{
			add::action::LocationAddInput, export::LocationExportInput,
			import::LocationImportInput, remove::action::LocationRemoveInput,
			rescan::action::LocationRescanInput,
		},
	},
};

use crate::domains::index::args::IndexModeArg;

#[derive(Args, Debug)]
pub struct LocationAddArgs {
	/// Path to add (local filesystem path or service-based cloud URI)
	/// Examples:
	///   - /Users/james/Documents (local path)
	///   - s3://my-bucket/photos (S3 cloud path)
	///   - gdrive://My Drive/photos (Google Drive path)
	/// If not provided, enters interactive mode
	pub path: Option<String>,

	/// Display name for the location
	#[arg(long)]
	pub name: Option<String>,

	/// Indexing mode
	#[arg(long, value_enum)]
	pub mode: Option<IndexModeArg>,
}

impl LocationAddArgs {
	/// Build an WingPath from the args (non-interactive mode)
	pub fn build_wing_path(&self) -> anyhow::Result<WingPath> {
		let path_str = self
			.path
			.as_ref()
			.ok_or_else(|| anyhow::anyhow!("Path is required in non-interactive mode"))?;

		// Use WingPath::from_uri() to parse service-based paths or local paths
		WingPath::from_uri(path_str).map_err(|e| anyhow::anyhow!("Invalid path: {}", e))
	}

	/// Check if interactive mode should be triggered
	pub fn is_interactive(&self) -> bool {
		self.path.is_none()
	}
}

#[derive(Args, Debug)]
pub struct LocationRemoveArgs {
	pub location_id: Uuid,
	#[arg(long, short = 'y', default_value_t = false)]
	pub yes: bool,
}

impl From<LocationRemoveArgs> for LocationRemoveInput {
	fn from(args: LocationRemoveArgs) -> Self {
		Self {
			location_id: args.location_id,
		}
	}
}

#[derive(Args, Debug)]
pub struct LocationRescanArgs {
	pub location_id: Uuid,
	#[arg(long, default_value_t = false)]
	pub force: bool,
}

impl From<LocationRescanArgs> for LocationRescanInput {
	fn from(args: LocationRescanArgs) -> Self {
		Self {
			location_id: args.location_id,
			full_rescan: args.force,
		}
	}
}

#[derive(Args, Debug)]
pub struct LocationExportArgs {
	/// UUID of the location to export
	pub location_id: Uuid,

	/// Path to save the export file (SQL dump)
	#[arg(long, short = 'o')]
	pub output: PathBuf,

	/// Include content identities (file hashes)
	#[arg(long, default_value_t = true)]
	pub content: bool,

	/// Include media metadata (image/video/audio data)
	#[arg(long, default_value_t = true)]
	pub media: bool,

	/// Include user metadata (notes, favorites, etc.)
	#[arg(long, default_value_t = true)]
	pub metadata: bool,

	/// Include tags
	#[arg(long, default_value_t = true)]
	pub tags: bool,
}

impl From<LocationExportArgs> for LocationExportInput {
	fn from(args: LocationExportArgs) -> Self {
		Self {
			location_uuid: args.location_id,
			export_path: args.output,
			include_content_identities: args.content,
			include_media_data: args.media,
			include_user_metadata: args.metadata,
			include_tags: args.tags,
		}
	}
}

#[derive(Args, Debug)]
pub struct LocationImportArgs {
	/// Path to the export file to import
	pub import_path: PathBuf,

	/// New name for the imported location (optional)
	#[arg(long)]
	pub name: Option<String>,

	/// Skip entries that already exist
	#[arg(long, default_value_t = false)]
	pub skip_existing: bool,
}

impl From<LocationImportArgs> for LocationImportInput {
	fn from(args: LocationImportArgs) -> Self {
		Self {
			import_path: args.import_path,
			new_name: args.name,
			skip_existing: args.skip_existing,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use clap::Parser;
	#[derive(Parser)]
	struct Add {
		#[command(flatten)]
		args: LocationAddArgs,
	}
	#[derive(Parser)]
	struct Remove {
		#[command(flatten)]
		args: LocationRemoveArgs,
	}
	#[derive(Parser)]
	struct Rescan {
		#[command(flatten)]
		args: LocationRescanArgs,
	}
	#[derive(Parser)]
	struct Export {
		#[command(flatten)]
		args: LocationExportArgs,
	}
	#[derive(Parser)]
	struct Import {
		#[command(flatten)]
		args: LocationImportArgs,
	}
	const ID: &str = "550e8400-e29b-41d4-a716-446655440000";

	#[test]
	fn add_without_path_is_interactive() {
		let a = Add::try_parse_from(["wing"]).unwrap().args;
		assert!(a.is_interactive());
		assert!(a.build_wing_path().is_err());
	}
	#[test]
	fn add_local_path_is_non_interactive() {
		let a = Add::try_parse_from(["wing", "folder"]).unwrap().args;
		assert!(!a.is_interactive());
		assert_eq!(
			a.build_wing_path().unwrap().as_local_path().unwrap(),
			std::path::Path::new("folder")
		);
	}
	#[test]
	fn add_cloud_uri_is_preserved() {
		let a = Add::try_parse_from(["wing", "s3://bucket/folder"])
			.unwrap()
			.args;
		assert!(matches!(
			a.build_wing_path().unwrap(),
			WingPath::Cloud { .. }
		));
	}
	#[test]
	fn add_rejects_invalid_uri() {
		let a = Add::try_parse_from(["wing", "content://invalid"])
			.unwrap()
			.args;
		assert!(a.build_wing_path().is_err());
	}
	#[test]
	fn add_rejects_invalid_mode() {
		assert!(Add::try_parse_from(["wing", "folder", "--mode", "invalid"]).is_err());
	}
	#[test]
	fn remove_preserves_uuid_and_confirmation() {
		let a = Remove::try_parse_from(["wing", ID, "-y"]).unwrap().args;
		assert!(a.yes);
		let input: LocationRemoveInput = a.into();
		assert_eq!(input.location_id, Uuid::parse_str(ID).unwrap());
	}
	#[test]
	fn rescan_force_reaches_core() {
		let a = Rescan::try_parse_from(["wing", ID, "--force"])
			.unwrap()
			.args;
		let input: LocationRescanInput = a.into();
		assert_eq!(input.location_id, Uuid::parse_str(ID).unwrap());
		assert!(input.full_rescan);
	}
	#[test]
	fn export_defaults_include_all_metadata() {
		let a = Export::try_parse_from(["wing", ID, "--output", "dump.sql"])
			.unwrap()
			.args;
		let input: LocationExportInput = a.into();
		assert_eq!(input.location_uuid, Uuid::parse_str(ID).unwrap());
		assert_eq!(input.export_path, PathBuf::from("dump.sql"));
		assert!(
			input.include_content_identities
				&& input.include_media_data
				&& input.include_user_metadata
				&& input.include_tags
		);
	}
	#[test]
	fn export_requires_output_path() {
		assert!(Export::try_parse_from(["wing", ID]).is_err());
	}
	#[test]
	fn import_preserves_name_and_skip_policy() {
		let a = Import::try_parse_from(["wing", "dump.sql", "--name", "new", "--skip-existing"])
			.unwrap()
			.args;
		let input: LocationImportInput = a.into();
		assert_eq!(input.import_path, PathBuf::from("dump.sql"));
		assert_eq!(input.new_name.as_deref(), Some("new"));
		assert!(input.skip_existing);
	}
}

#[cfg(test)]
#[path = "args/matrix.rs"]
mod matrix;
