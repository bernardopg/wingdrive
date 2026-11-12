//! # Plans for copy, move and delete
//!
//! The rules a copy or move job applies to each source, extracted so the
//! read-only preflight (`files.preflight`) and the job compute the same answer:
//!
//! - where a source lands ([`final_destination`]);
//! - what the conflict policy does when that place is taken ([`decide`]);
//! - which free name a rename picks ([`next_free_name`]).
//!
//! Nothing here writes to the disk. The functions read metadata only, so a plan
//! describes the job without running it.
//!
//! ## Example
//! ```rust,ignore
//! let target = plan::final_destination(&source, &destination, sources.len()).await;
//! let decision = plan::decide(&target, Some(FileConflictResolution::Skip)).await?;
//! ```

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use specta::Type;

use super::copy::action::FileConflictResolution;
use crate::domain::addressing::WingPath;

/// Most names [`next_free_name`] tries before giving up.
const MAX_RENAME_ATTEMPTS: u32 = 1000;

/// Where `source` lands when copied or moved to `destination`.
///
/// A destination that is a file means the parent folder (a drop on a file); a
/// folder, or a batch of several sources, gets the source's name appended; a
/// single source onto a path that does not exist yet is a copy under that name.
/// Remote and cloud destinations always append the name, since they cannot be
/// inspected from here.
pub async fn final_destination(
	source: &WingPath,
	destination: &WingPath,
	source_count: usize,
) -> WingPath {
	let name = source.file_name().unwrap_or_default().to_string();
	let Some(dest_path) = destination.as_local_path() else {
		return destination.join(&name);
	};
	match tokio::fs::metadata(dest_path).await {
		Ok(metadata) if metadata.is_file() => match dest_path.parent() {
			Some(parent) => WingPath::local(parent.join(&name)),
			None => destination.clone(),
		},
		Ok(metadata) if metadata.is_dir() => destination.join(&name),
		_ if source_count > 1 => destination.join(&name),
		_ => destination.clone(),
	}
}

/// What a copy or move does with one source, given the conflict policy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(tag = "kind", content = "path")]
pub enum PlanDecision {
	/// Nothing is in the way.
	Write,
	/// The existing item is replaced.
	Overwrite,
	/// The existing item stays and this source is left out.
	Skip,
	/// The source is written under this free name instead.
	Rename(PathBuf),
	/// The whole operation stops here; later sources are not touched.
	Abort,
	/// Something is in the way and no policy was chosen yet; the user will be asked.
	Ask,
}

/// Applies the conflict policy to a final destination.
///
/// Only local destinations can be checked; anything else plans as a plain write.
pub async fn decide(
	target: &WingPath,
	policy: Option<FileConflictResolution>,
) -> std::io::Result<PlanDecision> {
	let Some(path) = target.as_local_path() else {
		return Ok(PlanDecision::Write);
	};
	match tokio::fs::symlink_metadata(path).await {
		Ok(_) => {}
		Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
			return Ok(PlanDecision::Write);
		}
		Err(error) => return Err(error),
	}
	Ok(match policy {
		None => PlanDecision::Ask,
		Some(FileConflictResolution::Overwrite) => PlanDecision::Overwrite,
		Some(FileConflictResolution::Skip) => PlanDecision::Skip,
		Some(FileConflictResolution::Abort) => PlanDecision::Abort,
		Some(FileConflictResolution::AutoModifyName) => {
			PlanDecision::Rename(next_free_name(path).await?)
		}
	})
}

/// The first of `name (1).ext`, `name (2).ext`, ... that does not exist yet.
pub async fn next_free_name(path: &Path) -> std::io::Result<PathBuf> {
	let parent = path
		.parent()
		.ok_or_else(|| std::io::Error::other("Could not get parent directory"))?;
	let file_name = path
		.file_name()
		.ok_or_else(|| std::io::Error::other("Could not get filename"))?
		.as_encoded_bytes();
	// Split the raw name so bytes that are not UTF-8 survive in the new name.
	let split = match file_name.iter().rposition(|&byte| byte == b'.') {
		Some(dot) if dot > 0 => dot,
		_ => file_name.len(),
	};
	// SAFETY: both halves come from one OsStr split next to an ASCII `.`, or at its end.
	let (stem, extension) = unsafe {
		(
			std::ffi::OsStr::from_encoded_bytes_unchecked(&file_name[..split]),
			std::ffi::OsStr::from_encoded_bytes_unchecked(&file_name[split..]),
		)
	};

	let mut candidate = path.to_path_buf();
	let mut counter = 1;
	loop {
		match tokio::fs::symlink_metadata(&candidate).await {
			Ok(_) => {}
			Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(candidate),
			Err(error) => return Err(error),
		}
		if counter > MAX_RENAME_ATTEMPTS {
			return Err(std::io::Error::other(format!(
				"Could not generate unique filename after {MAX_RENAME_ATTEMPTS} attempts"
			)));
		}
		let mut name = stem.to_os_string();
		name.push(format!(" ({counter})"));
		name.push(extension);
		candidate = parent.join(name);
		counter += 1;
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[cfg(unix)]
	#[tokio::test]
	async fn inaccessible_target_is_not_treated_as_available() {
		use std::os::unix::fs::PermissionsExt;

		let dir = tempfile::tempdir().unwrap();
		let locked = dir.path().join("locked");
		std::fs::create_dir(&locked).unwrap();
		std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o000)).unwrap();
		let target = locked.join("file.txt");
		let metadata_error = match std::fs::symlink_metadata(&target) {
			Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => error,
			_ => {
				std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o700)).unwrap();
				return;
			}
		};
		let decision = decide(&WingPath::local(&target), None).await;
		let free_name = next_free_name(&target).await;
		std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o700)).unwrap();
		assert_eq!(metadata_error.kind(), std::io::ErrorKind::PermissionDenied);
		assert_eq!(
			decision.unwrap_err().kind(),
			std::io::ErrorKind::PermissionDenied
		);
		assert_eq!(
			free_name.unwrap_err().kind(),
			std::io::ErrorKind::PermissionDenied
		);
	}

	#[tokio::test]
	async fn conflict_policies_preserve_the_requested_action() {
		let dir = tempfile::tempdir().unwrap();
		let target = dir.path().join("existing");
		std::fs::write(&target, b"original").unwrap();
		for (policy, expected) in [
			(FileConflictResolution::Overwrite, PlanDecision::Overwrite),
			(FileConflictResolution::Skip, PlanDecision::Skip),
			(FileConflictResolution::Abort, PlanDecision::Abort),
		] {
			assert_eq!(
				decide(&WingPath::local(&target), Some(policy))
					.await
					.unwrap(),
				expected
			);
		}
		assert_eq!(std::fs::read(target).unwrap(), b"original");
	}

	#[tokio::test]
	async fn generated_names_keep_extensions_and_find_gaps() {
		let dir = tempfile::tempdir().unwrap();
		for (name, expected) in [
			("archive.tar.gz", "archive.tar (1).gz"),
			(".profile", ".profile (1)"),
			("no-extension", "no-extension (1)"),
			("café 文件.txt", "café 文件 (1).txt"),
		] {
			let target = dir.path().join(name);
			std::fs::write(&target, b"x").unwrap();
			assert_eq!(
				next_free_name(&target).await.unwrap(),
				dir.path().join(expected)
			);
		}
		let target = dir.path().join("gap.txt");
		std::fs::write(&target, b"x").unwrap();
		std::fs::write(dir.path().join("gap (2).txt"), b"x").unwrap();
		assert_eq!(
			next_free_name(&target).await.unwrap(),
			dir.path().join("gap (1).txt")
		);
	}

	#[tokio::test]
	async fn generated_name_search_stops_after_its_attempt_limit() {
		let dir = tempfile::tempdir().unwrap();
		let target = dir.path().join("taken.txt");
		std::fs::write(&target, b"").unwrap();
		for counter in 1..=MAX_RENAME_ATTEMPTS {
			std::fs::write(dir.path().join(format!("taken ({counter}).txt")), b"").unwrap();
		}
		let error = next_free_name(&target).await.unwrap_err();
		assert!(error.to_string().contains("1000 attempts"));
	}

	#[cfg(unix)]
	#[tokio::test]
	async fn dangling_symlinks_are_conflicts_and_reserve_names() {
		use std::os::unix::fs::symlink;

		let dir = tempfile::tempdir().unwrap();
		let target = dir.path().join("file.txt");
		symlink(dir.path().join("missing"), &target).unwrap();
		symlink(dir.path().join("missing"), dir.path().join("file (1).txt")).unwrap();
		assert_eq!(
			decide(&WingPath::local(&target), None).await.unwrap(),
			PlanDecision::Ask
		);
		assert_eq!(
			next_free_name(&target).await.unwrap(),
			dir.path().join("file (2).txt")
		);
	}

	#[cfg(unix)]
	#[tokio::test]
	async fn free_name_keeps_bytes_that_are_not_utf8() {
		use std::os::unix::ffi::OsStrExt;
		let dir = tempfile::tempdir().unwrap();
		let taken = dir.path().join(std::ffi::OsStr::from_bytes(b"caf\xff.txt"));
		std::fs::write(&taken, "x").unwrap();
		let free = next_free_name(&taken).await.unwrap();
		assert_eq!(free.file_name().unwrap().as_bytes(), b"caf\xff (1).txt");

		let dotfile = dir.path().join(".hidden");
		std::fs::write(&dotfile, "x").unwrap();
		assert_eq!(
			next_free_name(&dotfile).await.unwrap(),
			dir.path().join(".hidden (1)")
		);
	}

	#[tokio::test]
	async fn destination_rules() {
		let dir = tempfile::tempdir().unwrap();
		let root = dir.path();
		std::fs::create_dir(root.join("folder")).unwrap();
		std::fs::write(root.join("file.txt"), "x").unwrap();
		let source = WingPath::local(root.join("src/a.txt"));

		let into_folder =
			final_destination(&source, &WingPath::local(root.join("folder")), 1).await;
		assert_eq!(
			into_folder.as_local_path().unwrap(),
			root.join("folder/a.txt")
		);

		let onto_file =
			final_destination(&source, &WingPath::local(root.join("file.txt")), 1).await;
		assert_eq!(onto_file.as_local_path().unwrap(), root.join("a.txt"));

		let new_name = final_destination(&source, &WingPath::local(root.join("b.txt")), 1).await;
		assert_eq!(new_name.as_local_path().unwrap(), root.join("b.txt"));

		let batch = final_destination(&source, &WingPath::local(root.join("missing")), 2).await;
		assert_eq!(batch.as_local_path().unwrap(), root.join("missing/a.txt"));
	}

	#[tokio::test]
	async fn policies_and_free_names() {
		let dir = tempfile::tempdir().unwrap();
		let taken = dir.path().join("report.pdf");
		std::fs::write(&taken, "x").unwrap();
		std::fs::write(dir.path().join("report (1).pdf"), "x").unwrap();
		let target = WingPath::local(&taken);
		let free = WingPath::local(dir.path().join("free.pdf"));

		assert_eq!(decide(&free, None).await.unwrap(), PlanDecision::Write);
		assert_eq!(decide(&target, None).await.unwrap(), PlanDecision::Ask);
		assert_eq!(
			decide(&target, Some(FileConflictResolution::Skip))
				.await
				.unwrap(),
			PlanDecision::Skip
		);
		assert_eq!(
			decide(&target, Some(FileConflictResolution::AutoModifyName))
				.await
				.unwrap(),
			PlanDecision::Rename(dir.path().join("report (2).pdf"))
		);

		std::fs::write(dir.path().join(".hidden"), "x").unwrap();
		assert_eq!(
			next_free_name(&dir.path().join(".hidden")).await.unwrap(),
			dir.path().join(".hidden (1)")
		);
	}
}
