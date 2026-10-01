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
