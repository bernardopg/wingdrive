use clap::Args;
use std::path::PathBuf;

use wing_core::{
	domain::addressing::{WingPath, WingPathBatch},
	ops::files::{
		copy::input::{CopyMethod, FileCopyInput},
		create_folder::input::CreateFolderInput,
		delete::input::FileDeleteInput,
		rename::input::FileRenameInput,
	},
};

/// Builds a local WingPath resolved against the CLI's working directory.
///
/// The daemon runs with its own working directory, so a relative path sent
/// unchanged would point somewhere else. Symlinks are not followed, so
/// deleting or renaming a link acts on the link itself.
pub fn local_path(path: PathBuf) -> WingPath {
	// `local://<device>/path` addresses a paired device; anything else is a
	// path on this machine.
	if let Some(uri) = path.to_str().filter(|p| p.contains("://")) {
		if let Ok(parsed) = WingPath::from_uri(uri) {
			return parsed;
		}
	}
	WingPath::local(std::path::absolute(&path).unwrap_or(path))
}

#[derive(Args, Debug, Clone)]
pub struct FileCopyArgs {
	/// Source files or directories to copy (one or more)
	pub sources: Vec<PathBuf>,

	/// Destination path
	#[arg(long)]
	pub destination: PathBuf,

	/// Overwrite existing files
	#[arg(long, default_value_t = false)]
	pub overwrite: bool,

	/// Verify checksums during copy
	#[arg(long, default_value_t = false)]
	pub verify_checksum: bool,

	/// Preserve file timestamps
	#[arg(long, default_value_t = true)]
	pub preserve_timestamps: bool,

	/// Delete source files after copy (move)
	#[arg(long, default_value_t = false)]
	pub move_files: bool,

	/// Copy method to use
	#[arg(long, default_value_t = CopyMethod::Auto)]
	pub method: CopyMethod,

	/// Print the plan (files, bytes, conflicts) and stop without copying
	#[arg(long, default_value_t = false)]
	pub dry_run: bool,
}

impl From<FileCopyArgs> for FileCopyInput {
	fn from(args: FileCopyArgs) -> Self {
		let sources = args
			.sources
			.iter()
			.map(|p| local_path(p.clone()))
			.collect::<Vec<_>>();
		let destination = local_path(args.destination);
		Self {
			sources: WingPathBatch { paths: sources },
			destination,
			overwrite: args.overwrite,
			verify_checksum: args.verify_checksum,
			preserve_timestamps: args.preserve_timestamps,
			move_files: args.move_files,
			copy_method: args.method,
			on_conflict: None,
		}
	}
}

#[derive(Args, Debug, Clone)]
pub struct FileInfoArgs {
	/// File path to get information about
	pub path: PathBuf,
}

#[derive(Args, Debug, Clone)]
pub struct FileListArgs {
	/// Directory path to list contents of
	pub path: PathBuf,

	/// Maximum number of items to return
	#[arg(long)]
	pub limit: Option<u32>,

	/// Include hidden files and directories
	#[arg(long, default_value_t = false)]
	pub include_hidden: bool,

	/// Sort order for the results (name, modified, size, type)
	#[arg(long, default_value = "name")]
	pub sort_by: String,
}

#[derive(Args, Debug, Clone)]
pub struct FileRenameArgs {
	/// File or directory to rename
	pub path: PathBuf,

	/// New name (file name only, not a path)
	pub new_name: String,
}

impl From<FileRenameArgs> for FileRenameInput {
	fn from(args: FileRenameArgs) -> Self {
		Self::new(local_path(args.path), args.new_name)
	}
}

#[derive(Args, Debug, Clone)]
pub struct FileDeleteArgs {
	/// Files or directories to delete (moved to trash unless --permanent)
	#[arg(required = true)]
	pub paths: Vec<PathBuf>,

	/// Delete permanently instead of moving to trash
	#[arg(long, default_value_t = false)]
	pub permanent: bool,

	/// Skip the confirmation prompt for permanent deletion
	#[arg(long, short = 'y', default_value_t = false)]
	pub yes: bool,

	/// Print the plan (files, bytes, last copies) and stop without deleting
	#[arg(long, default_value_t = false)]
	pub dry_run: bool,
}

impl From<FileDeleteArgs> for FileDeleteInput {
	fn from(args: FileDeleteArgs) -> Self {
		let paths = args.paths.into_iter().map(local_path).collect();
		Self::new(WingPathBatch { paths }).with_permanent(args.permanent)
	}
}

#[derive(Args, Debug, Clone)]
pub struct FileMkdirArgs {
	/// Directory to create the folder in
	pub parent: PathBuf,

	/// Name of the new folder
	pub name: String,
}

impl From<FileMkdirArgs> for CreateFolderInput {
	fn from(args: FileMkdirArgs) -> Self {
		Self::new(local_path(args.parent), args.name)
	}
}

#[derive(Args, Debug, Clone)]
pub struct FileUndoArgs {
	/// Job to undo, as `wing file history` lists it (default: the most recent
	/// copy, move, rename or trash that still has something to undo)
	pub job_id: Option<uuid::Uuid>,
}

#[derive(Args, Debug, Clone)]
pub struct FileHistoryArgs {
	/// Most operations to show
	#[arg(long, default_value_t = 20)]
	pub limit: u32,
}

#[cfg(test)]
mod tests {
	use super::*;
	use clap::Parser;

	#[derive(Parser)]
	struct CopyCli {
		#[command(flatten)]
		args: FileCopyArgs,
	}

	#[derive(Parser)]
	struct DeleteCli {
		#[command(flatten)]
		args: FileDeleteArgs,
	}

	#[test]
	fn copy_defaults_and_explicit_flags_reach_the_core() {
		let args = CopyCli::try_parse_from(["wing", "source", "--destination", "destination"])
			.unwrap()
			.args;
		let input: FileCopyInput = args.into();
		assert_eq!(
			input.sources.paths[0].as_local_path().unwrap(),
			std::path::absolute("source").unwrap()
		);
		assert_eq!(
			input.destination.as_local_path().unwrap(),
			std::path::absolute("destination").unwrap()
		);
		assert!(!input.overwrite && !input.verify_checksum && !input.move_files);
		assert!(input.preserve_timestamps);
		assert_eq!(input.copy_method, CopyMethod::Auto);
		assert!(input.on_conflict.is_none());

		let args = CopyCli::try_parse_from([
			"wing",
			"first",
			"second",
			"--destination",
			"target",
			"--overwrite",
			"--verify-checksum",
			"--move-files",
			"--method",
			"streaming",
			"--dry-run",
		])
		.unwrap()
		.args;
		assert!(args.dry_run);
		let input: FileCopyInput = args.into();
		assert_eq!(input.sources.paths.len(), 2);
		assert!(input.overwrite && input.verify_checksum && input.move_files);
		assert_eq!(input.copy_method, CopyMethod::Streaming);
		assert!(input.validate().is_ok());
	}

	#[test]
	fn copy_requires_a_destination_and_valid_method() {
		assert!(CopyCli::try_parse_from(["wing", "source"]).is_err());
		assert!(CopyCli::try_parse_from([
			"wing",
			"source",
			"--destination",
			"target",
			"--method",
			"bad"
		])
		.is_err());
		let input: FileCopyInput = CopyCli::try_parse_from(["wing", "--destination", "target"])
			.unwrap()
			.args
			.into();
		assert!(input.validate().is_err());
	}

	#[test]
	fn local_path_preserves_remote_address_and_resolves_relative_paths() {
		let remote = local_path(PathBuf::from("local://other-device/folder/file"));
		assert!(
			matches!(remote, WingPath::Physical { device_slug, .. } if device_slug == "other-device")
		);
		let relative = local_path(PathBuf::from("folder/file"));
		assert_eq!(
			relative.as_local_path().unwrap(),
			std::path::absolute("folder/file").unwrap()
		);
	}

	#[test]
	fn delete_conversion_keeps_targets_and_requires_explicit_permanent_deletion() {
		for (flags, permanent) in [(vec![], false), (vec!["--permanent", "--yes"], true)] {
			let mut argv = vec!["wing", "first", "second"];
			argv.extend(flags);
			let input: FileDeleteInput = DeleteCli::try_parse_from(argv).unwrap().args.into();
			assert_eq!(input.permanent, permanent);
			assert!(input.recursive);
			assert_eq!(input.targets.paths.len(), 2);
			assert_eq!(
				input.targets.paths[0].as_local_path().unwrap(),
				std::path::absolute("first").unwrap()
			);
			assert_eq!(
				input.targets.paths[1].as_local_path().unwrap(),
				std::path::absolute("second").unwrap()
			);
			assert!(input.validate().is_ok());
		}
		assert!(DeleteCli::try_parse_from(["wing", "--permanent"]).is_err());
	}

	#[test]
	fn rename_and_mkdir_preserve_names_and_absolute_paths() {
		let rename: FileRenameInput = FileRenameArgs {
			path: "old".into(),
			new_name: "new.txt".into(),
		}
		.into();
		assert_eq!(
			rename.target.as_local_path().unwrap(),
			std::path::absolute("old").unwrap()
		);
		assert_eq!(rename.new_name, "new.txt");
		let mkdir: CreateFolderInput = FileMkdirArgs {
			parent: "parent".into(),
			name: "folder".into(),
		}
		.into();
		assert_eq!(
			mkdir.parent.as_local_path().unwrap(),
			std::path::absolute("parent").unwrap()
		);
		assert_eq!(mkdir.name, "folder");
		assert!(mkdir.items.is_empty());
	}

	#[cfg(unix)]
	#[test]
	fn local_path_keeps_the_symlink_itself() {
		let dir = tempfile::tempdir().unwrap();
		let target = dir.path().join("target");
		let link = dir.path().join("link");
		std::fs::write(&target, b"data").unwrap();
		std::os::unix::fs::symlink(&target, &link).unwrap();
		assert_eq!(local_path(link.clone()).as_local_path().unwrap(), link);
	}
}

#[cfg(test)]
#[path = "args/matrix.rs"]
mod matrix;
