//! # Copy Layout Safety
//!
//! Directory copies must never place their output inside the source tree. A recursive
//! walker can otherwise discover its own output and copy forever until the volume is
//! full. Paths are resolved through existing ancestors so the check also catches a
//! missing destination below a symlink into the source.

use crate::domain::addressing::{SdPath, SdPathBatch};
use std::{
	ffi::OsString,
	io,
	path::{Path, PathBuf},
};

/// Returns validation errors for directory copies that would write into themselves.
pub(super) fn recursive_copy_errors(sources: &SdPathBatch, destination: &SdPath) -> Vec<String> {
	let Some(destination) = destination.as_local_path() else {
		return Vec::new();
	};

	let mut errors = Vec::new();

	for source in &sources.paths {
		let Some(source) = source.as_local_path() else {
			continue;
		};
		if !source.is_dir() {
			continue;
		}

		let final_destination =
			final_destination_for_source(source, destination, sources.paths.len());

		let (Ok(source), Ok(final_destination)) = (
			resolve_with_missing_leaf(source),
			resolve_with_missing_leaf(&final_destination),
		) else {
			continue;
		};

		if final_destination.starts_with(&source) {
			errors.push(format!(
				"Destination {} is inside source directory {}; copying it would recurse until the volume is full",
				final_destination.display(),
				source.display()
			));
		}
	}

	errors
}

fn final_destination_for_source(source: &Path, destination: &Path, source_count: usize) -> PathBuf {
	let file_name = source.file_name().unwrap_or_default();

	if destination.is_file() {
		destination.parent().map_or_else(
			|| destination.to_path_buf(),
			|parent| parent.join(file_name),
		)
	} else if destination.is_dir() || source_count > 1 {
		destination.join(file_name)
	} else {
		destination.to_path_buf()
	}
}

/// Canonicalizes the nearest existing ancestor and restores any missing suffix.
fn resolve_with_missing_leaf(path: &Path) -> io::Result<PathBuf> {
	let absolute = if path.is_absolute() {
		path.to_path_buf()
	} else {
		std::env::current_dir()?.join(path)
	};

	if let Ok(canonical) = absolute.canonicalize() {
		return Ok(canonical);
	}

	let mut ancestor = absolute.as_path();
	let mut missing = Vec::<OsString>::new();

	while !ancestor.exists() {
		let name = ancestor.file_name().ok_or_else(|| {
			io::Error::new(io::ErrorKind::NotFound, "path has no existing ancestor")
		})?;
		missing.push(name.to_os_string());
		ancestor = ancestor.parent().ok_or_else(|| {
			io::Error::new(io::ErrorKind::NotFound, "path has no existing ancestor")
		})?;
	}

	let mut resolved = ancestor.canonicalize()?;
	for component in missing.iter().rev() {
		resolved.push(component);
	}
	Ok(resolved)
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::fs;

	fn batch(path: &Path) -> SdPathBatch {
		SdPathBatch::new(vec![SdPath::local(path.to_path_buf())])
	}

	#[test]
	fn rejects_missing_destination_inside_source() {
		let temp = tempfile::tempdir().unwrap();
		let source = temp.path().join("source");
		fs::create_dir(&source).unwrap();

		let errors = recursive_copy_errors(&batch(&source), &SdPath::local(source.join("backup")));

		assert_eq!(errors.len(), 1);
		assert!(errors[0].contains("would recurse"));
	}

	#[test]
	fn rejects_copying_directory_onto_itself() {
		let temp = tempfile::tempdir().unwrap();
		let source = temp.path().join("source");
		fs::create_dir(&source).unwrap();

		let errors = recursive_copy_errors(&batch(&source), &SdPath::local(source.clone()));

		assert_eq!(errors.len(), 1);
	}

	#[test]
	fn allows_sibling_destination() {
		let temp = tempfile::tempdir().unwrap();
		let source = temp.path().join("source");
		fs::create_dir(&source).unwrap();

		let errors =
			recursive_copy_errors(&batch(&source), &SdPath::local(temp.path().join("copy")));

		assert!(errors.is_empty());
	}

	#[cfg(unix)]
	#[test]
	fn rejects_destination_reentering_source_through_symlink() {
		use std::os::unix::fs::symlink;

		let temp = tempfile::tempdir().unwrap();
		let source = temp.path().join("source");
		let alias = temp.path().join("alias");
		fs::create_dir(&source).unwrap();
		symlink(&source, &alias).unwrap();

		let errors = recursive_copy_errors(&batch(&source), &SdPath::local(alias.join("backup")));

		assert_eq!(errors.len(), 1);
	}
}
