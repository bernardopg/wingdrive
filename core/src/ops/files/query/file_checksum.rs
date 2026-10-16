//! On-demand SHA-256 of one local file, for checking it against a published checksum.
//!
//! Content identity (`cas_id`) samples large files, so it cannot be compared with a
//! hash a download page publishes. This query reads the whole file instead, and
//! only when the Inspector asks for it.

use crate::{
	context::CoreContext,
	domain::addressing::WingPath,
	infra::query::{LibraryQuery, QueryError, QueryResult},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use specta::Type;
use std::{io::Read, path::Path};

/// Input for hashing one local file.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct FileChecksumInput {
	/// Physical path of the file on this device.
	pub path: WingPath,
}

/// SHA-256 digest of the file's full contents.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct FileChecksumOutput {
	/// Lowercase hexadecimal digest, as `sha256sum` prints it.
	pub sha256: String,
}

/// Hashes a local file off the async runtime, so large files do not stall the daemon.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct FileChecksumQuery {
	pub input: FileChecksumInput,
}

impl LibraryQuery for FileChecksumQuery {
	type Input = FileChecksumInput;
	type Output = FileChecksumOutput;

	fn from_input(input: Self::Input) -> QueryResult<Self> {
		Ok(Self { input })
	}

	async fn execute(
		self,
		_context: std::sync::Arc<CoreContext>,
		_session: crate::infra::api::SessionContext,
	) -> QueryResult<Self::Output> {
		let path = self.input.path.as_local_path().ok_or_else(|| {
			QueryError::InvalidInput("Checksums are only available for local files".into())
		})?;

		if !path.is_absolute() {
			return Err(QueryError::InvalidInput(format!(
				"Path is not absolute: {}",
				path.display()
			)));
		}

		let metadata = tokio::fs::metadata(path)
			.await
			.map_err(|error| QueryError::io_error(path.display().to_string(), error))?;
		if !metadata.is_file() {
			return Err(QueryError::InvalidInput(format!(
				"Path is not a file: {}",
				path.display()
			)));
		}

		let path = path.to_path_buf();
		let hash_path = path.clone();
		let sha256 = tokio::task::spawn_blocking(move || sha256_file(&hash_path))
			.await
			.map_err(|error| QueryError::Internal(format!("Checksum worker failed: {error}")))?
			.map_err(|error| QueryError::io_error(path.display().to_string(), error))?;

		Ok(FileChecksumOutput { sha256 })
	}
}

fn sha256_file(path: &Path) -> std::io::Result<String> {
	let mut file = std::fs::File::open(path)?;
	let mut hasher = Sha256::new();
	let mut buffer = vec![0u8; 1024 * 1024];
	loop {
		let read = file.read(&mut buffer)?;
		if read == 0 {
			break;
		}
		hasher.update(&buffer[..read]);
	}
	Ok(hex::encode(hasher.finalize()))
}

crate::register_library_query!(FileChecksumQuery, "files.checksum");

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn matches_known_sha256_digests() {
		let temporary = tempfile::tempdir().unwrap();
		let empty = temporary.path().join("empty");
		let abc = temporary.path().join("abc");
		std::fs::write(&empty, b"").unwrap();
		std::fs::write(&abc, b"abc").unwrap();

		assert_eq!(
			sha256_file(&empty).unwrap(),
			"e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
		);
		assert_eq!(
			sha256_file(&abc).unwrap(),
			"ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
		);
	}

	#[test]
	fn hashes_across_buffer_boundaries() {
		let temporary = tempfile::tempdir().unwrap();
		let large = temporary.path().join("large");
		// Three full buffers plus a tail, so the read loop runs several times.
		let contents = vec![b'a'; 3 * 1024 * 1024 + 17];
		std::fs::write(&large, &contents).unwrap();

		assert_eq!(
			sha256_file(&large).unwrap(),
			hex::encode(Sha256::digest(&contents))
		);
	}
}
