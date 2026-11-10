//! # Undo for copy, move, rename and trash
//!
//! `files.undo` reverses what one earlier job recorded in the
//! [journal](super::journal), newest effect first, and `files.undo_history`
//! lists the jobs it can still work on.
//!
//! Undo never guesses. Each item is compared with the seal the job left; one
//! that changed, moved or vanished since is skipped and reported, and stays
//! in the journal so a later undo can try again.

use std::{
	path::{Path, PathBuf},
	sync::Arc,
};

use serde::{Deserialize, Serialize};
use specta::Type;
use uuid::Uuid;

use super::journal::{self, check_seal, Effect};
use crate::{
	context::CoreContext,
	infra::{
		action::{error::ActionError, LibraryAction},
		job::{handle::JobReceipt, prelude::*},
		query::{LibraryQuery, QueryError, QueryResult},
	},
	library::Library,
};

/// Skip reasons kept in the job output; the full list is in the job log.
const MAX_REPORTED_SKIPS: usize = 20;

/// Operations `files.undo_history` returns when no limit is given.
const DEFAULT_HISTORY_LIMIT: u32 = 20;

/// Reverses the journaled effects of one file job.
#[derive(Debug, Serialize, Deserialize, Job)]
pub struct FileUndoJob {
	/// The copy, move, rename or delete job to undo.
	pub job_id: Uuid,
}

impl Job for FileUndoJob {
	const NAME: &'static str = "file_undo";
	const RESUMABLE: bool = true;
	const DESCRIPTION: Option<&'static str> = Some("Undo a copy, move, rename or trash operation");
}

impl crate::infra::job::traits::DynJob for FileUndoJob {
	fn job_name(&self) -> &'static str {
		Self::NAME
	}
}

/// What an undo job did.
#[derive(Debug, Serialize, Deserialize)]
pub struct FileUndoOutput {
	pub undone_count: usize,
	pub skipped: Vec<String>,
}

impl From<FileUndoOutput> for JobOutput {
	fn from(output: FileUndoOutput) -> Self {
		JobOutput::FileUndo {
			undone_count: output.undone_count,
			skipped_count: output.skipped.len(),
			skipped: output
				.skipped
				.into_iter()
				.take(MAX_REPORTED_SKIPS)
				.collect(),
		}
	}
}

#[async_trait::async_trait]
impl JobHandler for FileUndoJob {
	type Output = FileUndoOutput;

	async fn run(&mut self, ctx: JobContext<'_>) -> JobResult<Self::Output> {
		let db = ctx.library().jobs().database().conn().clone();
		// Effects already undone are not pending, so a resumed job continues
		// where it stopped.
		let effects = journal::pending(&db, self.job_id)
			.await
			.map_err(JobError::execution)?;
		ctx.log(format!(
			"Undoing {} recorded changes of job {}",
			effects.len(),
			self.job_id
		));

		let total = effects.len();
		let mut undone_count = 0;
		let mut skipped = Vec::new();
		for (index, (row, effect)) in effects.into_iter().enumerate() {
			ctx.check_interrupt().await?;
			ctx.progress(Progress::percentage(index as f32 / total.max(1) as f32));

			match reverse(&effect).await {
				Ok(()) => {
					journal::mark_undone(&db, row)
						.await
						.map_err(JobError::execution)?;
					undone_count += 1;
				}
				Err(reason) => {
					let message = format!("{}: {reason}", effect.subject().display());
					ctx.log(format!("Left alone: {message}"));
					skipped.push(message);
				}
			}
		}

		ctx.progress(Progress::percentage(1.0));
		ctx.log(format!(
			"Undo finished: {undone_count} undone, {} left alone",
			skipped.len()
		));
		Ok(FileUndoOutput {
			undone_count,
			skipped,
		})
	}
}

/// Reverses one effect, or says why it was left alone.
async fn reverse(effect: &Effect) -> Result<(), String> {
	match effect {
		Effect::Created { path, seal } => {
			check_seal(seal, path).await?;
			remove_tree(path).await
		}
		Effect::Moved { from, to, seal } => {
			check_seal(seal, to).await?;
			// A case-only rename on a case-insensitive volume finds itself.
			if exists(from).await && !same_entry(from, to).await {
				return Err(format!("{} is taken again", from.display()));
			}
			move_back(to, from).await
		}
		Effect::Replaced { path, stash } => {
			if !exists(stash).await {
				return Err("the replaced file is no longer kept".to_string());
			}
			if exists(path).await {
				return Err("what replaced it is still there".to_string());
			}
			tokio::fs::rename(stash, path)
				.await
				.map_err(|error| error.to_string())?;
			if let Some(folder) = stash.parent() {
				// Only goes when it was the last stashed item.
				let _ = tokio::fs::remove_dir(folder).await;
			}
			Ok(())
		}
		Effect::Trashed { original, trash_id } => {
			if exists(original).await {
				return Err("something else is at its place now".to_string());
			}
			crate::ops::trash::restore_one(trash_id.clone(), original.clone()).await
		}
		Effect::Merged { .. } => Err(
			"it was merged into a folder that already existed, which cannot be undone".to_string(),
		),
	}
}

async fn exists(path: &Path) -> bool {
	tokio::fs::symlink_metadata(path).await.is_ok()
}

async fn same_entry(a: &Path, b: &Path) -> bool {
	super::copy::job::same_entry(
		&crate::domain::addressing::WingPath::local(a.to_path_buf()),
		&crate::domain::addressing::WingPath::local(b.to_path_buf()),
	)
	.await
}

async fn remove_tree(path: &Path) -> Result<(), String> {
	let metadata = tokio::fs::symlink_metadata(path)
		.await
		.map_err(|error| error.to_string())?;
	let removed = if metadata.is_dir() {
		tokio::fs::remove_dir_all(path).await
	} else {
		tokio::fs::remove_file(path).await
	};
	removed.map_err(|error| error.to_string())
}

/// Moves `to` back to `from`, across volumes when a rename cannot.
async fn move_back(to: &Path, from: &Path) -> Result<(), String> {
	if let Some(parent) = from.parent() {
		tokio::fs::create_dir_all(parent)
			.await
			.map_err(|error| error.to_string())?;
	}
	match tokio::fs::rename(to, from).await {
		Ok(()) => Ok(()),
		Err(error) if error.kind() == std::io::ErrorKind::CrossesDevices => {
			let (source, destination) = (to.to_path_buf(), from.to_path_buf());
			tokio::task::spawn_blocking(move || copy_back(&source, &destination))
				.await
				.map_err(|error| error.to_string())?
				.map_err(|error| error.to_string())
		}
		Err(error) => Err(error.to_string()),
	}
}

/// Copies `source` to `destination` on another volume, then removes `source`.
///
/// The copy is built under a temporary name, so a failure leaves `source`
/// whole and nothing half-written under the real name.
fn copy_back(source: &Path, destination: &Path) -> std::io::Result<()> {
	let partial = destination.with_file_name(format!(".wing-part-{}", Uuid::new_v4().simple()));
	let copied = copy_tree(source, &partial).and_then(|()| std::fs::rename(&partial, destination));
	if let Err(error) = copied {
		let _ = if partial.is_dir() {
			std::fs::remove_dir_all(&partial)
		} else {
			std::fs::remove_file(&partial)
		};
		return Err(error);
	}
	if std::fs::symlink_metadata(source)?.is_dir() {
		std::fs::remove_dir_all(source)
	} else {
		std::fs::remove_file(source)
	}
}

fn copy_tree(source: &Path, destination: &Path) -> std::io::Result<()> {
	let metadata = std::fs::symlink_metadata(source)?;
	#[cfg(unix)]
	if metadata.file_type().is_symlink() {
		return std::os::unix::fs::symlink(std::fs::read_link(source)?, destination);
	}
	if metadata.is_dir() {
		std::fs::create_dir(destination)?;
		for entry in std::fs::read_dir(source)? {
			let entry = entry?;
			copy_tree(&entry.path(), &destination.join(entry.file_name()))?;
		}
	} else {
		std::fs::copy(source, destination)?;
	}
	if let Ok(modified) = metadata.modified() {
		// Best effort, as for any copy.
		let _ = std::fs::File::open(destination).and_then(|file| file.set_modified(modified));
	}
	Ok(())
}

/// Input for undoing a file operation.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct FileUndoInput {
	/// The job to undo, as `files.undo_history` lists it. Without it, the
	/// most recent operation that still has something to undo.
	#[serde(default)]
	#[specta(optional)]
	pub job_id: Option<Uuid>,
}

/// Queues an undo job for one earlier copy, move, rename or trash job.
pub struct FileUndoAction {
	input: FileUndoInput,
}

impl LibraryAction for FileUndoAction {
	type Input = FileUndoInput;
	type Output = JobReceipt;

	fn from_input(input: Self::Input) -> Result<Self, String> {
		Ok(Self { input })
	}

	async fn execute(
		self,
		library: Arc<Library>,
		_context: Arc<CoreContext>,
	) -> Result<Self::Output, ActionError> {
		let db = library.jobs().database().conn();
		let job_id = match self.input.job_id {
			Some(job_id) => job_id,
			None => journal::history(db, 50)
				.await
				.map_err(ActionError::Internal)?
				.into_iter()
				.find(|operation| operation.pending > 0)
				.map(|operation| operation.job_id)
				.ok_or_else(|| ActionError::Validation {
					field: "job_id".to_string(),
					message: "Nothing to undo".to_string(),
				})?,
		};

		let pending = journal::pending(db, job_id)
			.await
			.map_err(ActionError::Internal)?;
		if pending.is_empty() {
			return Err(ActionError::Validation {
				field: "job_id".to_string(),
				message: format!("Job {job_id} has nothing left to undo"),
			});
		}

		let handle = library
			.jobs()
			.dispatch(FileUndoJob { job_id })
			.await
			.map_err(ActionError::Job)?;
		Ok(handle.into())
	}

	fn action_kind(&self) -> &'static str {
		"files.undo"
	}
}

crate::register_library_action!(FileUndoAction, "files.undo");

/// Input for listing what can be undone.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct UndoHistoryInput {
	/// Most operations to return, newest first.
	#[serde(default)]
	#[specta(optional)]
	pub limit: Option<u32>,
}

/// One earlier operation and how much of it undo can still reverse.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct UndoableOperation {
	pub job_id: Uuid,
	/// `copy`, `move`, `rename` or `delete`.
	pub operation: String,
	/// Seconds since the Unix epoch.
	pub started_at: i64,
	/// Changes the job recorded.
	pub effects: u32,
	/// Changes undo can still reverse; zero once undone.
	pub pending: u32,
	/// One path from the operation, to recognise it by.
	pub example: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct UndoHistoryOutput {
	/// Newest first.
	pub operations: Vec<UndoableOperation>,
	/// Days a replaced file and its record are kept.
	pub retention_days: u32,
}

/// Lists recent file operations from the journal.
pub struct UndoHistoryQuery {
	input: UndoHistoryInput,
}

impl LibraryQuery for UndoHistoryQuery {
	type Input = UndoHistoryInput;
	type Output = UndoHistoryOutput;

	fn from_input(input: Self::Input) -> QueryResult<Self> {
		Ok(Self { input })
	}

	async fn execute(
		self,
		context: Arc<CoreContext>,
		session: crate::infra::api::SessionContext,
	) -> QueryResult<Self::Output> {
		let library_id = session
			.current_library_id
			.ok_or_else(|| QueryError::Internal("No library in session".to_string()))?;
		let library = context
			.libraries()
			.await
			.get_library(library_id)
			.await
			.ok_or_else(|| QueryError::Internal("Library not found".to_string()))?;

		let limit = self.input.limit.unwrap_or(DEFAULT_HISTORY_LIMIT).min(200) as usize;
		let operations = journal::history(library.jobs().database().conn(), limit)
			.await
			.map_err(QueryError::Internal)?
			.into_iter()
			.map(|operation| UndoableOperation {
				job_id: operation.job_id,
				operation: operation.operation,
				started_at: operation.started_at.timestamp(),
				effects: operation.effects,
				pending: operation.pending,
				example: operation.example,
			})
			.collect();

		Ok(UndoHistoryOutput {
			operations,
			retention_days: journal::RETENTION_DAYS as u32,
		})
	}
}

crate::register_library_query!(UndoHistoryQuery, "files.undo_history");
