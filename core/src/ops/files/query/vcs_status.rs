//! # Git status for Explorer emblems
//!
//! The Explorer asks for the state of the folder it shows. The query looks for
//! `.git` in the folder and its parents first, so a folder outside any working
//! tree costs a few `stat` calls and never starts `git`. Inside one, it runs
//! `git status` scoped to the folder and folds the result onto the folder's
//! direct children: a child folder carries the strongest state found anywhere
//! beneath it, as Dolphin's version-control plugin does.
//!
//! Without `git` installed, outside a working tree, or when `git` refuses the
//! repository (for example, one owned by another user), the output is empty and
//! the Explorer shows no emblems.

use std::{
	collections::HashMap,
	path::{Component, Path, PathBuf},
	process::Stdio,
	time::Duration,
};

use serde::{Deserialize, Serialize};
use specta::Type;
use tracing::debug;

use crate::{
	context::CoreContext,
	domain::addressing::WingPath,
	infra::query::{LibraryQuery, QueryError, QueryResult},
};

/// Longest `git status` may run before the folder is shown without emblems.
const GIT_STATUS_TIMEOUT: Duration = Duration::from_secs(5);

/// How an entry differs from the last commit.
///
/// Variants are ordered by how much attention they need, so a folder takes the
/// maximum of its descendants.
#[derive(
	Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Type,
)]
#[serde(rename_all = "lowercase")]
pub enum VcsFileStatus {
	/// Not tracked and not ignored.
	Untracked,
	/// Staged as a new file, including the new side of a rename.
	Added,
	/// Changed or deleted, staged or not.
	Modified,
	/// Unmerged after a merge, rebase or cherry-pick.
	Conflicted,
}

/// Input for reading the Git state of a folder's entries.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct VcsStatusInput {
	/// Physical folder shown in the Explorer.
	pub path: WingPath,
}

/// Git state of a folder's direct children.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct VcsStatusOutput {
	/// Root of the working tree, or `None` when the folder is not in one.
	pub repository_root: Option<String>,
	/// State of every entry without its own, set when the folder itself is
	/// untracked: `git` reports an untracked folder once, not per file.
	pub inherited_status: Option<VcsFileStatus>,
	/// State of each direct child that differs from the last commit, keyed by
	/// file name. Unchanged entries are absent.
	pub entries: HashMap<String, VcsFileStatus>,
}

/// Reads `git status` for one local folder.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct VcsStatusQuery {
	pub input: VcsStatusInput,
}

impl LibraryQuery for VcsStatusQuery {
	type Input = VcsStatusInput;
	type Output = VcsStatusOutput;

	fn from_input(input: Self::Input) -> QueryResult<Self> {
		Ok(Self { input })
	}

	async fn execute(
		self,
		_context: std::sync::Arc<CoreContext>,
		_session: crate::infra::api::SessionContext,
	) -> QueryResult<Self::Output> {
		let path = self.input.path.as_local_path().ok_or_else(|| {
			QueryError::InvalidInput("Git status is only available for local folders".into())
		})?;

		if !path.is_absolute() {
			return Err(QueryError::InvalidInput(format!(
				"Path is not absolute: {}",
				path.display()
			)));
		}

		Ok(folder_vcs_status(path).await)
	}
}

/// Git state of `folder`'s direct children, empty when it has none to show.
async fn folder_vcs_status(folder: &Path) -> VcsStatusOutput {
	let Some(root) = find_work_tree(folder).await else {
		return VcsStatusOutput::default();
	};

	let mut output = VcsStatusOutput {
		repository_root: Some(root.to_string_lossy().into_owned()),
		..Default::default()
	};

	if let Some(raw) = run_git_status(folder).await {
		let (inherited, entries) = fold_onto_children(folder, &root, parse_porcelain_v2(&raw));
		output.inherited_status = inherited;
		output.entries = entries;
	}

	output
}

/// The nearest folder at or above `folder` that holds a `.git` entry.
///
/// `.git` is a file in linked worktrees and submodules, so any entry counts.
async fn find_work_tree(folder: &Path) -> Option<PathBuf> {
	for candidate in folder.ancestors() {
		if tokio::fs::symlink_metadata(candidate.join(".git"))
			.await
			.is_ok()
		{
			return Some(candidate.to_path_buf());
		}
	}
	None
}

/// Raw `git status --porcelain=v2 -z` output for `folder`, or `None` when `git`
/// is missing, refuses the folder, or takes too long.
async fn run_git_status(folder: &Path) -> Option<Vec<u8>> {
	let mut command = tokio::process::Command::new("git");
	command
		.arg("-C")
		.arg(folder)
		.args([
			"status",
			"--porcelain=v2",
			"-z",
			"--untracked-files=normal",
			"--ignored=no",
			"--",
			".",
		])
		// Reading status must not take the index lock a terminal `git` may need.
		.env("GIT_OPTIONAL_LOCKS", "0")
		.stdin(Stdio::null())
		.stderr(Stdio::null())
		.kill_on_drop(true);

	match tokio::time::timeout(GIT_STATUS_TIMEOUT, command.output()).await {
		Ok(Ok(output)) if output.status.success() => Some(output.stdout),
		Ok(Ok(output)) => {
			debug!(folder = %folder.display(), status = %output.status, "git status refused the folder");
			None
		}
		Ok(Err(error)) => {
			debug!(folder = %folder.display(), %error, "git status could not start");
			None
		}
		Err(_) => {
			debug!(folder = %folder.display(), "git status timed out");
			None
		}
	}
}

/// Changed paths, relative to the working tree root, from porcelain v2 output.
fn parse_porcelain_v2(raw: &[u8]) -> Vec<(PathBuf, VcsFileStatus)> {
	let mut changes = Vec::new();
	let mut records = raw.split(|byte| *byte == 0);

	while let Some(record) = records.next() {
		let (status, fields_before_path) = match record.first() {
			Some(b'1') => (changed_status(record), 8),
			Some(b'2') => {
				// Renames and copies carry the original path as the next record.
				records.next();
				(VcsFileStatus::Added, 9)
			}
			Some(b'u') => (VcsFileStatus::Conflicted, 10),
			Some(b'?') => (VcsFileStatus::Untracked, 1),
			_ => continue,
		};

		if let Some(path) = record
			.splitn(fields_before_path + 1, |byte| *byte == b' ')
			.nth(fields_before_path)
		{
			changes.push((path_from_bytes(path), status));
		}
	}

	changes
}

/// State of an ordinary changed entry from its `XY` field.
fn changed_status(record: &[u8]) -> VcsFileStatus {
	match record.get(2) {
		Some(b'A') => VcsFileStatus::Added,
		_ => VcsFileStatus::Modified,
	}
}

#[cfg(unix)]
fn path_from_bytes(bytes: &[u8]) -> PathBuf {
	use std::os::unix::ffi::OsStrExt;
	PathBuf::from(std::ffi::OsStr::from_bytes(bytes))
}

#[cfg(not(unix))]
fn path_from_bytes(bytes: &[u8]) -> PathBuf {
	PathBuf::from(String::from_utf8_lossy(bytes).into_owned())
}

/// Maps changes under `root` onto `folder`'s direct children.
///
/// Returns the state inherited by every child (when `folder` sits inside an
/// untracked folder) and the strongest state of each child that has one.
fn fold_onto_children(
	folder: &Path,
	root: &Path,
	changes: Vec<(PathBuf, VcsFileStatus)>,
) -> (Option<VcsFileStatus>, HashMap<String, VcsFileStatus>) {
	let mut inherited: Option<VcsFileStatus> = None;
	let mut entries: HashMap<String, VcsFileStatus> = HashMap::new();

	for (relative, status) in changes {
		let changed = root.join(&relative);

		if let Ok(rest) = changed.strip_prefix(folder) {
			match rest.components().next() {
				Some(Component::Normal(name)) => {
					let entry = entries
						.entry(name.to_string_lossy().into_owned())
						.or_insert(status);
					*entry = (*entry).max(status);
				}
				// The folder itself, reported as one untracked folder.
				_ => inherited = inherited.max(Some(status)),
			}
		} else if folder.starts_with(&changed) {
			// An untracked ancestor covers everything beneath it.
			inherited = inherited.max(Some(status));
		}
	}

	(inherited, entries)
}

crate::register_library_query!(VcsStatusQuery, "files.vcs_status");

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn parses_every_porcelain_v2_record_kind() {
		let raw = b"1 .M N... 100644 100644 100644 aaa aaa src/main.rs\0\
1 A. N... 000000 100644 100644 000 bbb new file.txt\0\
2 R. N... 100644 100644 100644 ccc ccc R100 renamed.rs\0old.rs\0\
u UU N... 100644 100644 100644 100644 ddd eee fff conflict.rs\0\
? build/\0\
# branch.head main\0";

		assert_eq!(
			parse_porcelain_v2(raw),
			vec![
				(PathBuf::from("src/main.rs"), VcsFileStatus::Modified),
				(PathBuf::from("new file.txt"), VcsFileStatus::Added),
				(PathBuf::from("renamed.rs"), VcsFileStatus::Added),
				(PathBuf::from("conflict.rs"), VcsFileStatus::Conflicted),
				(PathBuf::from("build/"), VcsFileStatus::Untracked),
			]
		);
	}

	#[test]
	fn child_folders_take_the_strongest_state_beneath_them() {
		let root = Path::new("/repo");
		let (inherited, entries) = fold_onto_children(
			root,
			root,
			vec![
				(PathBuf::from("README.md"), VcsFileStatus::Modified),
				(PathBuf::from("src/a.rs"), VcsFileStatus::Untracked),
				(PathBuf::from("src/deep/b.rs"), VcsFileStatus::Conflicted),
				(PathBuf::from("docs/new/"), VcsFileStatus::Untracked),
			],
		);

		assert_eq!(inherited, None);
		assert_eq!(
			entries,
			HashMap::from([
				("README.md".to_string(), VcsFileStatus::Modified),
				("src".to_string(), VcsFileStatus::Conflicted),
				("docs".to_string(), VcsFileStatus::Untracked),
			])
		);
	}

	#[test]
	fn a_folder_inside_an_untracked_folder_inherits_its_state() {
		let root = Path::new("/repo");
		let folder = Path::new("/repo/scratch/notes");

		let (inherited, entries) = fold_onto_children(
			folder,
			root,
			vec![
				(PathBuf::from("scratch/"), VcsFileStatus::Untracked),
				(PathBuf::from("README.md"), VcsFileStatus::Modified),
			],
		);

		assert_eq!(inherited, Some(VcsFileStatus::Untracked));
		assert!(entries.is_empty());
	}

	#[tokio::test]
	async fn reports_nothing_outside_a_working_tree() {
		let temporary = tempfile::tempdir().unwrap();
		if find_work_tree(temporary.path()).await.is_some() {
			// The temporary directory sits inside a checkout on this machine.
			return;
		}

		assert_eq!(
			folder_vcs_status(temporary.path()).await,
			VcsStatusOutput::default()
		);
	}

	#[tokio::test]
	async fn reads_a_real_working_tree() {
		let temporary = tempfile::tempdir().unwrap();
		let repo = temporary.path();
		let git = |args: &[&str]| {
			std::process::Command::new("git")
				.arg("-C")
				.arg(repo)
				.args([
					"-c",
					"user.name=WingDrive",
					"-c",
					"user.email=test@wingdrive.invalid",
					"-c",
					"commit.gpgsign=false",
				])
				.args(args)
				.output()
		};
		if git(&["init", "--quiet"]).is_err() {
			// `git` is not installed, which the query treats as "no emblems".
			return;
		}

		std::fs::write(repo.join("tracked.txt"), "one").unwrap();
		std::fs::create_dir(repo.join("src")).unwrap();
		std::fs::write(repo.join("src/lib.rs"), "fn a() {}").unwrap();
		git(&["add", "."]).unwrap();
		git(&["commit", "--quiet", "-m", "initial"]).unwrap();

		std::fs::write(repo.join("tracked.txt"), "two").unwrap();
		std::fs::write(repo.join("src/new.rs"), "fn b() {}").unwrap();
		std::fs::write(repo.join("staged.txt"), "new").unwrap();
		git(&["add", "staged.txt"]).unwrap();

		let status = folder_vcs_status(repo).await;

		assert_eq!(
			status.repository_root.as_deref(),
			Some(repo.to_string_lossy().as_ref())
		);
		assert_eq!(status.inherited_status, None);
		assert_eq!(
			status.entries,
			HashMap::from([
				("tracked.txt".to_string(), VcsFileStatus::Modified),
				("staged.txt".to_string(), VcsFileStatus::Added),
				("src".to_string(), VcsFileStatus::Untracked),
			])
		);
	}
}
