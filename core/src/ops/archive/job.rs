//! Compress and extract jobs and the actions that start them.

use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use specta::Type;
use uuid::Uuid;

use super::engine::{self, ArchiveError, Control};
use super::format::ArchiveFormat;
use crate::context::CoreContext;
use crate::domain::addressing::WingPath;
use crate::infra::action::{error::ActionError, LibraryAction, ValidationResult};
use crate::infra::job::handle::JobReceipt;
use crate::infra::job::{generic_progress::GenericProgress, prelude::*};
use crate::ops::files::rename::validation::validate_filename;

/// Input for compressing files into a new archive
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ArchiveCompressInput {
	/// Files and folders to include, all on this device
	pub sources: Vec<WingPath>,
	/// Folder that receives the archive
	pub destination: WingPath,
	/// Archive name without extension; the format's extension is added
	pub name: String,
	pub format: ArchiveFormat,
}

/// Input for extracting an archive
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ArchiveExtractInput {
	pub archive: WingPath,
	/// Folder that receives the extracted content
	pub destination: WingPath,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveCompressAction {
	input: ArchiveCompressInput,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveExtractAction {
	input: ArchiveExtractInput,
}

fn local(path: &WingPath, field: &str) -> Result<PathBuf, ActionError> {
	path.is_local()
		.then(|| path.as_local_path().map(Path::to_path_buf))
		.flatten()
		.ok_or_else(|| ActionError::Validation {
			field: field.to_string(),
			message: "Archives can only be used with files on this device".to_string(),
		})
}

impl LibraryAction for ArchiveCompressAction {
	type Input = ArchiveCompressInput;
	type Output = JobReceipt;

	fn from_input(input: Self::Input) -> Result<Self, String> {
		Ok(Self { input })
	}

	async fn validate(
		&self,
		_library: &Arc<crate::library::Library>,
		_context: Arc<CoreContext>,
	) -> Result<ValidationResult, ActionError> {
		if self.input.sources.is_empty() {
			return Err(ActionError::Validation {
				field: "sources".to_string(),
				message: "Nothing to compress".to_string(),
			});
		}
		for source in &self.input.sources {
			local(source, "sources")?;
		}
		local(&self.input.destination, "destination")?;
		validate_filename(&self.input.name).map_err(|e| ActionError::Validation {
			field: "name".to_string(),
			message: e.to_string(),
		})?;
		if !self.input.format.can_write() {
			return Err(ActionError::Validation {
				field: "format".to_string(),
				message: "This format can only be extracted".to_string(),
			});
		}
		Ok(ValidationResult::Success { metadata: None })
	}

	async fn execute(
		self,
		library: Arc<crate::library::Library>,
		_context: Arc<CoreContext>,
	) -> Result<Self::Output, ActionError> {
		let sources = self
			.input
			.sources
			.iter()
			.map(|s| local(s, "sources"))
			.collect::<Result<Vec<_>, _>>()?;
		let destination = local(&self.input.destination, "destination")?;
		let output = destination.join(format!(
			"{}.{}",
			self.input.name,
			self.input.format.extension()
		));
		let job = ArchiveCompressJob {
			sources,
			output,
			format: self.input.format,
		};
		Ok(library
			.jobs()
			.dispatch(job)
			.await
			.map_err(ActionError::Job)?
			.into())
	}

	fn action_kind(&self) -> &'static str {
		"archive.compress"
	}
}

impl LibraryAction for ArchiveExtractAction {
	type Input = ArchiveExtractInput;
	type Output = JobReceipt;

	fn from_input(input: Self::Input) -> Result<Self, String> {
		Ok(Self { input })
	}

	async fn validate(
		&self,
		_library: &Arc<crate::library::Library>,
		_context: Arc<CoreContext>,
	) -> Result<ValidationResult, ActionError> {
		let archive = local(&self.input.archive, "archive")?;
		if ArchiveFormat::detect(&archive).is_none() {
			return Err(ActionError::Validation {
				field: "archive".to_string(),
				message: "Not a supported archive".to_string(),
			});
		}
		local(&self.input.destination, "destination")?;
		Ok(ValidationResult::Success { metadata: None })
	}

	async fn execute(
		self,
		library: Arc<crate::library::Library>,
		_context: Arc<CoreContext>,
	) -> Result<Self::Output, ActionError> {
		let job = ArchiveExtractJob {
			archive: local(&self.input.archive, "archive")?,
			destination: local(&self.input.destination, "destination")?,
		};
		Ok(library
			.jobs()
			.dispatch(job)
			.await
			.map_err(ActionError::Job)?
			.into())
	}

	fn action_kind(&self) -> &'static str {
		"archive.extract"
	}
}

crate::register_library_action!(ArchiveCompressAction, "archive.compress");
crate::register_library_action!(ArchiveExtractAction, "archive.extract");

/// Writes an archive of `sources` to `output`, or a free variant of that name.
#[derive(Debug, Serialize, Deserialize, Job)]
pub struct ArchiveCompressJob {
	pub sources: Vec<PathBuf>,
	pub output: PathBuf,
	pub format: ArchiveFormat,
}

/// Extracts `archive` into `destination`. A single top-level entry is placed
/// directly; several are grouped in a folder named after the archive.
#[derive(Debug, Serialize, Deserialize, Job)]
pub struct ArchiveExtractJob {
	pub archive: PathBuf,
	pub destination: PathBuf,
}

impl Job for ArchiveCompressJob {
	const NAME: &'static str = "archive_compress";
	// The temporary archive is discarded on interruption, so a rerun starts over.
	const RESUMABLE: bool = false;
	const DESCRIPTION: Option<&'static str> = Some("Compress files into an archive");
}

impl Job for ArchiveExtractJob {
	const NAME: &'static str = "archive_extract";
	const RESUMABLE: bool = false;
	const DESCRIPTION: Option<&'static str> = Some("Extract an archive");
}

impl crate::infra::job::traits::DynJob for ArchiveCompressJob {
	fn job_name(&self) -> &'static str {
		Self::NAME
	}
}

impl crate::infra::job::traits::DynJob for ArchiveExtractJob {
	fn job_name(&self) -> &'static str {
		Self::NAME
	}
}

/// Sets the cancel flag when the job future is dropped, which is how the job
/// system stops a cancelled job, so the blocking worker stops writing too.
struct CancelOnDrop(Arc<Control>);

impl Drop for CancelOnDrop {
	fn drop(&mut self) {
		self.0.cancel.store(true, Ordering::Relaxed);
	}
}

/// Removes a temporary file or folder; failures leave a hidden leftover only.
fn remove_temp(path: &Path) {
	let result = if path.is_dir() {
		std::fs::remove_dir_all(path)
	} else {
		std::fs::remove_file(path)
	};
	if let Err(error) = result {
		if error.kind() != std::io::ErrorKind::NotFound {
			tracing::warn!(?path, %error, "Could not remove temporary archive data");
		}
	}
}

/// Runs `work` on a blocking thread while reporting progress and relaying
/// cancellation from the job system.
///
/// The worker removes `temp` itself when it fails or is cancelled: a
/// cancelled job's future is dropped, so cleanup after the await never runs.
async fn supervise<F>(
	ctx: &JobContext<'_>,
	label: &str,
	total: u64,
	temp: PathBuf,
	work: F,
) -> JobResult<()>
where
	F: FnOnce(&Control) -> Result<(), ArchiveError> + Send + 'static,
{
	let control = Arc::new(Control::default());
	let _cancel_on_drop = CancelOnDrop(control.clone());
	let worker = {
		let control = control.clone();
		tokio::task::spawn_blocking(move || {
			let result = work(&control);
			if result.is_err() {
				remove_temp(&temp);
			}
			result
		})
	};
	tokio::pin!(worker);
	let started = Instant::now();
	loop {
		tokio::select! {
			result = &mut worker => {
				return match result.map_err(|e| JobError::execution(e.to_string()))? {
					Ok(()) => Ok(()),
					Err(ArchiveError::Cancelled) => Err(JobError::Interrupted),
					Err(error) => Err(JobError::execution(error.to_string())),
				};
			}
			_ = tokio::time::sleep(Duration::from_millis(250)) => {
				if ctx.check_interrupt().await.is_err() {
					control.cancel.store(true, Ordering::Relaxed);
				}
				let done = control.done.load(Ordering::Relaxed);
				let fraction = if total > 0 { (done as f64 / total as f64).min(1.0) } else { 0.0 };
				let message = format!("{} of {}", human_bytes(done.min(total)), human_bytes(total));
				ctx.progress(Progress::Generic(
					GenericProgress::new(fraction as f32, label, message)
						.with_bytes(done.min(total), total)
						.with_performance(0.0, None, Some(started.elapsed())),
				));
			}
		}
	}
}

fn human_bytes(bytes: u64) -> String {
	const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
	let mut value = bytes as f64;
	let mut unit = 0;
	while value >= 1024.0 && unit < UNITS.len() - 1 {
		value /= 1024.0;
		unit += 1;
	}
	if unit == 0 {
		format!("{bytes} B")
	} else {
		format!("{value:.1} {}", UNITS[unit])
	}
}

/// `parent/name`, or `name 2`, `name 3`... before any archive suffix.
pub fn free_path(parent: &Path, name: &str) -> PathBuf {
	let candidate = parent.join(name);
	if std::fs::symlink_metadata(&candidate).is_err() {
		return candidate;
	}
	let (stem, suffix) = match ArchiveFormat::stem(Path::new(name)) {
		Some(stem) => {
			let suffix = name[stem.len()..].to_string();
			(stem, suffix)
		}
		None => match name.rfind('.').filter(|&i| i > 0) {
			Some(i) => (name[..i].to_string(), name[i..].to_string()),
			None => (name.to_string(), String::new()),
		},
	};
	(2..)
		.map(|n| parent.join(format!("{stem} {n}{suffix}")))
		.find(|path| std::fs::symlink_metadata(path).is_err())
		.expect("an unused name exists")
}

/// Moves `from` to `to` without replacing anything already at `to`.
fn rename_no_replace(from: &Path, to: &Path) -> std::io::Result<()> {
	#[cfg(target_os = "linux")]
	{
		use std::os::unix::ffi::OsStrExt;
		let from = std::ffi::CString::new(from.as_os_str().as_bytes())?;
		let to = std::ffi::CString::new(to.as_os_str().as_bytes())?;
		let rc = unsafe {
			libc::renameat2(
				libc::AT_FDCWD,
				from.as_ptr(),
				libc::AT_FDCWD,
				to.as_ptr(),
				libc::RENAME_NOREPLACE,
			)
		};
		if rc == 0 {
			Ok(())
		} else {
			Err(std::io::Error::last_os_error())
		}
	}
	#[cfg(not(target_os = "linux"))]
	{
		if std::fs::symlink_metadata(to).is_ok() {
			return Err(std::io::ErrorKind::AlreadyExists.into());
		}
		std::fs::rename(from, to)
	}
}

/// Places `temp` under a free variant of `parent/name`, retrying if another
/// process takes the name in between.
fn place(temp: &Path, parent: &Path, name: &str) -> std::io::Result<PathBuf> {
	for _ in 0..16 {
		let target = free_path(parent, name);
		match rename_no_replace(temp, &target) {
			Ok(()) => return Ok(target),
			Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
			Err(e) => return Err(e),
		}
	}
	Err(std::io::ErrorKind::AlreadyExists.into())
}

fn output_json(path: &Path) -> JobOutput {
	JobOutput::Custom(serde_json::json!({ "path": path }))
}

#[async_trait::async_trait]
impl JobHandler for ArchiveCompressJob {
	type Output = JobOutput;

	async fn run(&mut self, ctx: JobContext<'_>) -> JobResult<Self::Output> {
		let parent = self
			.output
			.parent()
			.ok_or_else(|| JobError::execution("Archive has no parent folder"))?
			.to_path_buf();
		let name = self
			.output
			.file_name()
			.ok_or_else(|| JobError::execution("Archive has no name"))?
			.to_string_lossy()
			.into_owned();
		ctx.log(format!(
			"Compressing {} items into {name}",
			self.sources.len()
		));
		let temp = parent.join(format!(".{name}.wingdrive-partial-{}", Uuid::new_v4()));

		let sources = self.sources.clone();
		let total = tokio::task::spawn_blocking(move || engine::source_size(&sources))
			.await
			.unwrap_or(0);
		let (sources, format, work_temp) = (self.sources.clone(), self.format, temp.clone());
		let compress_temp = work_temp.clone();
		supervise(&ctx, "Compressing", total, work_temp, move |control| {
			engine::compress(&sources, format, &compress_temp, control)
		})
		.await?;
		let placed = place(&temp, &parent, &name).map_err(|e| {
			remove_temp(&temp);
			JobError::execution(format!("Failed to save archive: {e}"))
		})?;
		ctx.log(format!("Created {}", placed.display()));
		Ok(output_json(&placed))
	}
}

#[async_trait::async_trait]
impl JobHandler for ArchiveExtractJob {
	type Output = JobOutput;

	async fn run(&mut self, ctx: JobContext<'_>) -> JobResult<Self::Output> {
		let format = ArchiveFormat::detect(&self.archive)
			.ok_or_else(|| JobError::execution("Not a supported archive"))?;
		let stem = ArchiveFormat::stem(&self.archive).unwrap_or_else(|| "Extracted".to_string());
		let temp = self
			.destination
			.join(format!(".{stem}.wingdrive-extract-{}", Uuid::new_v4()));
		tokio::fs::create_dir(&temp)
			.await
			.map_err(|e| JobError::execution(format!("Cannot write to destination: {e}")))?;
		ctx.log(format!("Extracting {}", self.archive.display()));

		let total = tokio::fs::metadata(&self.archive)
			.await
			.map(|m| m.len())
			.unwrap_or(0);
		let (archive, work_temp) = (self.archive.clone(), temp.clone());
		let extract_temp = work_temp.clone();
		supervise(&ctx, "Extracting", total, work_temp, move |control| {
			engine::extract(&archive, format, &extract_temp, control)
		})
		.await?;

		let destination = self.destination.clone();
		let placed = tokio::task::spawn_blocking(move || -> std::io::Result<PathBuf> {
			let mut children = std::fs::read_dir(&temp)?.collect::<Result<Vec<_>, _>>()?;
			if children.len() == 1 {
				let only = children.pop().expect("one child");
				let name = only.file_name().to_string_lossy().into_owned();
				let placed = place(&only.path(), &destination, &name)?;
				std::fs::remove_dir(&temp)?;
				Ok(placed)
			} else {
				place(&temp, &destination, &stem)
			}
		})
		.await
		.map_err(|e| JobError::execution(e.to_string()))?
		.map_err(|e| JobError::execution(format!("Failed to place extracted files: {e}")))?;
		ctx.log(format!("Extracted to {}", placed.display()));
		Ok(output_json(&placed))
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn free_names_keep_archive_suffixes() {
		let dir = tempfile::tempdir().unwrap();
		assert_eq!(
			free_path(dir.path(), "a.tar.gz"),
			dir.path().join("a.tar.gz")
		);
		std::fs::write(dir.path().join("a.tar.gz"), "").unwrap();
		assert_eq!(
			free_path(dir.path(), "a.tar.gz"),
			dir.path().join("a 2.tar.gz")
		);
		std::fs::create_dir(dir.path().join("photos")).unwrap();
		assert_eq!(free_path(dir.path(), "photos"), dir.path().join("photos 2"));
	}

	#[test]
	fn byte_sizes_are_readable() {
		assert_eq!(human_bytes(512), "512 B");
		assert_eq!(human_bytes(1536), "1.5 KB");
		assert_eq!(human_bytes(1_048_576_000), "1000.0 MB");
	}

	#[test]
	fn place_never_replaces() {
		let dir = tempfile::tempdir().unwrap();
		std::fs::create_dir(dir.path().join("out")).unwrap();
		let temp = dir.path().join(".tmp");
		std::fs::create_dir(&temp).unwrap();
		std::fs::write(temp.join("f"), "x").unwrap();
		let placed = place(&temp, dir.path(), "out").unwrap();
		assert_eq!(placed, dir.path().join("out 2"));
		assert!(placed.join("f").exists());
	}
}
