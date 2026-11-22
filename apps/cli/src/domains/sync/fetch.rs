use std::path::PathBuf;

use anyhow::{ensure, Context as _, Result};
use uuid::Uuid;

use crate::context::Context;
use crate::util::prelude::*;
use wing_core::domain::{
	addressing::{WingPath, WingPathBatch},
	content_identity::ContentHashGenerator,
	file::EntryKind,
	File,
};
use wing_core::infra::job::handle::JobReceipt;
use wing_core::ops::devices::list::query::ListLibraryDevicesInput;
use wing_core::ops::files::{
	copy::input::FileCopyInput,
	query::{FileByIdQuery, FileChecksumInput, FileChecksumOutput},
};

pub(super) async fn run(ctx: &Context, entry: Uuid, directory: PathBuf, apply: bool) -> Result<()> {
	let library = ctx.library_id.context("Select a library first")?;
	let file: Option<File> = serde_json::from_value(
		ctx.core
			.query(&FileByIdQuery::new(entry), Some(library))
			.await?,
	)?;
	let file = file.context("Entry missing from the local catalog")?;
	ensure!(file.kind == EntryKind::File, "Select a regular file");
	let directory = tokio::fs::canonicalize(directory).await?;
	ensure!(directory.is_dir(), "Destination must be a directory");
	let source = file.wing_path;
	let source_path = source
		.path()
		.context("Select an entry with a physical path")?;
	let devices: Vec<wing_core::domain::Device> = serde_json::from_value(
		ctx.core
			.query(
				&ListLibraryDevicesInput {
					include_offline: true,
					include_details: false,
					show_paired: false,
				},
				Some(library),
			)
			.await?,
	)?;
	let owner = devices
		.iter()
		.find(|device| Some(device.slug.as_str()) == source.device_slug());
	let online = owner.map(|device| device.is_current || device.is_online);
	// Use the catalog's basename so cross-platform paths need no local parsing.
	let filename = match file.extension.as_deref() {
		Some(extension) if !extension.is_empty() => format!("{}.{}", file.name, extension),
		_ => file.name,
	};
	let destination = directory.join(safe_basename(&filename)?);
	let destination_exists = match tokio::fs::symlink_metadata(&destination).await {
		Ok(meta) => {
			ensure!(
				meta.is_file(),
				"Destination exists and is not a regular file"
			);
			Some(meta)
		}
		Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
		Err(error) => return Err(error.into()),
	};
	let present = if let Some(meta) = &destination_exists {
		if meta.len() == file.size {
			let digest = file
				.content_identity
				.as_ref()
				.and_then(|id| id.integrity_hash.clone());
			if owner.is_some_and(|device| device.is_current) {
				let expected = ContentHashGenerator::generate_integrity_hash(source_path).await?;
				ContentHashGenerator::generate_integrity_hash(&destination).await? == expected
			} else if apply && online == Some(true) {
				let source_hash = checksum(ctx, library, source.clone()).await?;
				let target_hash = checksum(ctx, library, WingPath::local(&destination)).await?;
				source_hash == target_hash
			} else if let Some(expected) = digest {
				ContentHashGenerator::generate_integrity_hash(&destination).await? == expected
			} else {
				false
			}
		} else {
			false
		}
	} else {
		false
	};
	let plan = serde_json::json!({"entry": entry, "source": source, "destination": destination, "peer_online": online, "alternate_paths": file.alternate_paths, "already_verified": present, "bytes_to_copy": if present {0} else {file.size}});
	if !apply || present {
		print_output!(ctx, &plan, |value: &serde_json::Value| {
			println!("{}", value);
		});
		return Ok(());
	}
	ensure!(online != Some(false), "Source device is offline");
	ensure!(
		destination_exists.is_none(),
		"An existing destination could not be verified"
	);
	let mut input = FileCopyInput::new(Vec::new(), directory).with_verification(true);
	input.sources = WingPathBatch::new(vec![source]);
	let receipt: JobReceipt = execute_action!(ctx, input);
	print_output!(ctx, &receipt, |receipt: &JobReceipt| {
		println!("Dispatched copy job {}", receipt.id);
	});
	Ok(())
}

async fn checksum(ctx: &Context, library: Uuid, path: WingPath) -> Result<String> {
	let output: FileChecksumOutput = serde_json::from_value(
		ctx.core
			.query(&FileChecksumInput { path }, Some(library))
			.await?,
	)?;
	Ok(output.sha256)
}

fn safe_basename(name: &str) -> Result<&std::path::Path> {
	use std::path::{Component, Path};

	ensure!(!name.contains(['/', '\\']), "Invalid file name");
	let path = Path::new(name);
	let mut components = path.components();
	ensure!(
		matches!(components.next(), Some(Component::Normal(_))) && components.next().is_none(),
		"Invalid file name"
	);
	Ok(path)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn fetch_destination_requires_a_single_basename() {
		for invalid in [
			"",
			".",
			"..",
			"../notes.txt",
			"other\\notes.txt",
			"/notes.txt",
		] {
			assert!(safe_basename(invalid).is_err(), "{invalid}");
		}
		assert!(safe_basename("notes.txt").is_ok());
	}
}
