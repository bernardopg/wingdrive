//! # File Copy Job
//!
//! Implements file copy and move operations using the Strategy Pattern with real-time
//! progress tracking and transfer speed calculation. Supports resume on interruption.

use super::{database::CopyDatabaseQuery, input::CopyMethod, routing::CopyStrategyRouter};
use crate::{
	domain::addressing::{WingPath, WingPathBatch},
	infra::job::generic_progress::{GenericProgress, ToGenericProgress},
	infra::job::prelude::*,
};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::{
	collections::HashMap,
	path::PathBuf,
	sync::{Arc, Mutex},
	time::{Duration, Instant},
};
use tracing::info;
use uuid::Uuid;

/// Tracks transfer speed with exponential moving average for stable ETA calculation.
#[derive(Debug)]
pub struct SpeedTracker {
	start_time: Instant,
	last_update: Instant,
	last_bytes: u64,
	current_rate: f32,
	avg_rate: f32,
	/// Exponential smoothing factor (0.3 balances responsiveness and stability)
	alpha: f32,
}

impl SpeedTracker {
	pub fn new() -> Self {
		let now = Instant::now();
		Self {
			start_time: now,
			last_update: now,
			last_bytes: 0,
			current_rate: 0.0,
			avg_rate: 0.0,
			alpha: 0.3,
		}
	}

	/// Update speed tracker with current bytes copied. Returns current rate in bytes/sec.
	pub fn update(&mut self, bytes_copied: u64) -> f32 {
		let now = Instant::now();
		let elapsed = now.duration_since(self.last_update).as_secs_f32();

		// Throttle updates to at least 50ms intervals
		if elapsed < 0.05 {
			return self.current_rate;
		}

		let delta_bytes = bytes_copied.saturating_sub(self.last_bytes);
		let rate = delta_bytes as f32 / elapsed;

		// Exponential moving average for smoother ETA
		self.avg_rate = if self.avg_rate == 0.0 {
			rate
		} else {
			self.alpha * rate + (1.0 - self.alpha) * self.avg_rate
		};

		self.current_rate = rate;
		self.last_update = now;
		self.last_bytes = bytes_copied;

		rate
	}

	/// Calculate estimated time remaining based on average rate.
	pub fn calculate_eta(&self, bytes_remaining: u64) -> Option<Duration> {
		// Require minimum rate to avoid division issues
		if self.avg_rate < 1.0 {
			return None;
		}
		Some(Duration::from_secs_f32(
			bytes_remaining as f32 / self.avg_rate,
		))
	}

	/// Get elapsed time since tracker was created.
	pub fn elapsed(&self) -> Duration {
		Instant::now().duration_since(self.start_time)
	}

	/// Get current instantaneous rate in bytes/sec.
	pub fn current_rate(&self) -> f32 {
		self.current_rate
	}

	/// Get smoothed average rate in bytes/sec (better for ETA calculation).
	pub fn avg_rate(&self) -> f32 {
		self.avg_rate
	}
}

impl Default for SpeedTracker {
	fn default() -> Self {
		Self::new()
	}
}

/// Move operation modes for UI context
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MoveMode {
	/// Standard move operation
	Move,
	/// Rename a single file/directory
	Rename,
	/// Cut and paste operation (same as move but different UX context)
	Cut,
}

/// Options for file copy operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopyOptions {
	pub overwrite: bool,
	pub verify_checksum: bool,
	pub preserve_timestamps: bool,
	pub delete_after_copy: bool,
	pub move_mode: Option<MoveMode>,
	pub copy_method: CopyMethod,
	pub conflict_resolution: Option<super::action::FileConflictResolution>,
}

impl Default for CopyOptions {
	fn default() -> Self {
		Self {
			overwrite: false,
			verify_checksum: false,
			preserve_timestamps: true,
			delete_after_copy: false,
			move_mode: None,
			copy_method: CopyMethod::Auto,
			conflict_resolution: None,
		}
	}
}

/// File copy job using the Strategy Pattern
#[derive(Debug, Serialize, Deserialize, Job)]
pub struct FileCopyJob {
	pub sources: WingPathBatch,
	pub destination: WingPath,
	#[serde(default)]
	pub options: CopyOptions,

	// Internal state for resumption
	#[serde(default)]
	pub completed_indices: Vec<usize>,
	#[serde(skip, default = "Instant::now")]
	started_at: Instant,

	/// Queryable metadata about this copy operation (collected during preparation)
	#[serde(default)]
	pub job_metadata: super::metadata::CopyJobMetadata,
}

impl Job for FileCopyJob {
	const NAME: &'static str = "file_copy";
	const RESUMABLE: bool = true;
	const DESCRIPTION: Option<&'static str> = Some("Copy or move files to a destination");
}

impl crate::infra::job::traits::DynJob for FileCopyJob {
	fn job_name(&self) -> &'static str {
		Self::NAME
	}
}

#[async_trait::async_trait]
impl JobHandler for FileCopyJob {
	type Output = FileCopyOutput;

	async fn run(&mut self, ctx: JobContext<'_>) -> JobResult<Self::Output> {
		let validate_layout = |sources: &WingPathBatch, destination: &WingPath| -> JobResult<()> {
			let errors = super::safety::recursive_copy_errors(sources, destination);
			if errors.is_empty() {
				Ok(())
			} else {
				Err(JobError::execution(errors.join("; ")))
			}
		};

		validate_layout(&self.sources, &self.destination)?;

		let destination =
			self.destination.resolve_in_job(&ctx).await.map_err(|e| {
				JobError::execution(format!("Failed to resolve destination path: {e}"))
			})?;
		let mut sources = Vec::with_capacity(self.sources.paths.len());
		for source in &self.sources.paths {
			sources.push(
				source.resolve_in_job(&ctx).await.map_err(|e| {
					JobError::execution(format!("Failed to resolve source path: {e}"))
				})?,
			);
		}

		let resolved_sources = WingPathBatch::new(sources);
		if resolved_sources
			.paths
			.iter()
			.any(|path| matches!(path, WingPath::Cloud { .. }))
			|| matches!(destination, WingPath::Cloud { .. })
		{
			// Cloud volumes move bytes through their backend, not the filesystem
			// strategies below.
			let is_move = self.options.delete_after_copy
				|| matches!(self.options.move_mode, Some(MoveMode::Rename));
			let conflict = self
				.options
				.conflict_resolution
				.or(if self.options.overwrite {
					Some(super::action::FileConflictResolution::Overwrite)
				} else {
					None
				});
			return super::cloud::run_cloud_batch(
				&ctx,
				&resolved_sources.paths,
				&destination,
				self.options.move_mode.as_ref(),
				is_move,
				conflict,
				self.started_at,
			)
			.await;
		}
		validate_layout(&resolved_sources, &destination)?;
		self.destination = destination;

		ctx.log(format!(
			"Starting copy operation on {} files",
			self.sources.paths.len()
		));

		// Phase 1: Initializing
		let progress = CopyProgress {
			phase: CopyPhase::Initializing,
			current_file: String::new(),
			current_source_path: None,
			files_copied: 0,
			total_files: 0,
			bytes_copied: 0,
			total_bytes: 0,
			current_operation: "Initializing copy operation".to_string(),
			estimated_remaining: None,
			preparation_complete: false,
			error_count: 0,
			transfer_rate: 0.0,
			elapsed: None,
			strategy_metadata: None,
		};
		ctx.progress(Progress::generic(progress.to_generic_progress()));

		// Group by device for efficient processing
		let by_device: HashMap<String, Vec<WingPath>> = self
			.sources
			.by_device()
			.into_iter()
			.map(|(device_slug, paths)| (device_slug, paths.into_iter().cloned().collect()))
			.collect();

		let mut copied_count = 0;
		let mut skipped_count = 0;
		let mut total_bytes = 0u64;
		let mut failed_copies = Vec::new();
		let is_move = self.options.delete_after_copy;
		let volume_manager = ctx.volume_manager();

		// Phase 2: Database Query - Try to get instant estimates
		let progress = CopyProgress {
			phase: CopyPhase::DatabaseQuery,
			current_file: String::new(),
			current_source_path: None,
			files_copied: 0,
			total_files: 0,
			bytes_copied: 0,
			total_bytes: 0,
			current_operation: "Querying database for file information...".to_string(),
			estimated_remaining: None,
			preparation_complete: false,
			error_count: 0,
			transfer_rate: 0.0,
			elapsed: None,
			strategy_metadata: None,
		};
		ctx.progress(Progress::generic(progress.to_generic_progress()));

		// Try to get estimates from database
		let db_estimates = {
			let db_query = CopyDatabaseQuery::new(ctx.library_db().clone());
			match db_query.get_estimates_for_paths(&self.sources.paths).await {
				Ok(estimates) => {
					ctx.log(format!(
						"Database estimates: {} files, {} bytes ({:.0}% coverage)",
						estimates.file_count,
						estimates.total_size,
						estimates.confidence() * 100.0
					));
					Some(estimates)
				}
				Err(e) => {
					ctx.log(format!(
						"Database query failed, will calculate from filesystem: {}",
						e
					));
					None
				}
			}
		};

		// Use database estimates if available, otherwise use source path count for initial display
		let (estimated_files, estimated_bytes) = if let Some(ref estimates) = db_estimates {
			if estimates.is_complete() {
				// We have complete information from database
				(estimates.file_count as usize, estimates.total_size)
			} else {
				// Partial information - still useful for initial display
				(
					self.sources.paths.len().max(estimates.file_count as usize),
					estimates.total_size,
				)
			}
		} else {
			(self.sources.paths.len(), 0)
		};

		// Phase 3: Preparation - Calculate actual total size
		let progress = CopyProgress {
			phase: CopyPhase::Preparation,
			current_file: String::new(),
			current_source_path: None,
			files_copied: 0,
			total_files: estimated_files,
			bytes_copied: 0,
			total_bytes: estimated_bytes,
			current_operation: if db_estimates.is_some() {
				"Verifying file sizes...".to_string()
			} else {
				"Calculating total size...".to_string()
			},
			estimated_remaining: None,
			preparation_complete: false,
			error_count: 0,
			transfer_rate: 0.0,
			elapsed: None,
			strategy_metadata: None,
		};
		ctx.progress(Progress::generic(progress.to_generic_progress()));

		// Calculate actual file count and total size, and collect file metadata
		let actual_file_count = self.count_total_files().await?;
		let estimated_total_bytes = self.calculate_total_size(&ctx).await?;

		// Refuse before writing anything: a copy that runs the destination out of
		// space leaves truncated files behind, and filling a system volume can take
		// the machine down with it.
		if let Some(destination) = self.destination.as_local_path() {
			if let Err(error) =
				crate::infra::fs::free_space::ensure_space_for(destination, estimated_total_bytes)
			{
				return Err(JobError::execution(format!(
					"Cannot copy to {}: {error}",
					destination.display()
				)));
			}
		}

		// Collect file metadata for queryable list
		self.collect_file_metadata(&ctx).await?;

		// Persist job metadata to database for querying
		self.persist_job_state_to_db(&ctx).await?;

		ctx.log(format!(
			"Preparing to copy {} files ({}) from {} source paths",
			actual_file_count,
			format_bytes(estimated_total_bytes),
			self.sources.paths.len()
		));

		// Update progress with calculated size and file count
		let progress = CopyProgress {
			phase: CopyPhase::Preparation,
			current_file: String::new(),
			current_source_path: None,
			files_copied: 0,
			total_files: actual_file_count,
			bytes_copied: 0,
			total_bytes: estimated_total_bytes,
			current_operation: "Preparation complete".to_string(),
			estimated_remaining: None,
			preparation_complete: true,
			error_count: 0,
			transfer_rate: 0.0,
			elapsed: None,
			strategy_metadata: None,
		};
		ctx.progress(Progress::generic(progress.to_generic_progress()));

		let operation = match (&self.options.move_mode, is_move) {
			(Some(MoveMode::Rename), _) => "rename",
			(_, true) => "move",
			_ => "copy",
		};
		let journal = crate::ops::files::journal::Journal::open(&ctx, operation).await;

		// Create progress aggregator for tracking overall progress
		let mut progress_aggregator =
			ProgressAggregator::new(&ctx, actual_file_count, estimated_total_bytes);

		// Process each source using the appropriate strategy
		let sources = self.sources.paths.clone();
		for (index, source) in sources.iter().enumerate() {
			ctx.check_interrupt().await?;

			// One unresolvable source must not discard the items already copied,
			// so it is reported as a failure and the batch continues.
			let resolved_source = match source.resolve_in_job(&ctx).await {
				Ok(resolved) => resolved,
				Err(e) => {
					let error = format!("Failed to resolve source path: {}", e);
					failed_copies.push(CopyError {
						source: source.path().cloned().unwrap_or_default(),
						destination: self.destination.path().cloned().unwrap_or_default(),
						error: error.clone(),
					});
					ctx.add_non_critical_error(format!("{}: {}", source.display(), error));
					continue;
				}
			};

			// Skip files that have already been completed (resume logic)
			if self.completed_indices.contains(&index) {
				ctx.log(format!(
					"Skipping already completed file: {}",
					resolved_source.display()
				));

				// Update progress aggregator to account for already completed files
				let files_in_source = if let Some(local_path) = resolved_source.as_local_path() {
					let file_size = self.get_path_size(local_path).await.unwrap_or(0);
					let file_count = self.count_files_in_path(local_path).await.unwrap_or(1);
					progress_aggregator.skip_completed_file(file_size, file_count);
					total_bytes += file_size;
					file_count
				} else {
					1
				};

				copied_count += files_in_source; // Count actual files as copied for progress tracking

				// Mark as completed in metadata (already done during previous run)
				self.job_metadata
					.update_status(&resolved_source, super::metadata::CopyFileStatus::Completed);
				continue;
			}

			// Determine the final destination path for this source. The rules live in
			// `files::plan`, which the read-only preflight shares.
			// A rename names its destination exactly. Treating an existing folder
			// there as a drop target moved the item inside it.
			let final_destination = if matches!(self.options.move_mode, Some(MoveMode::Rename)) {
				self.destination.clone()
			} else {
				crate::ops::files::plan::final_destination(
					&resolved_source,
					&self.destination,
					self.sources.paths.len(),
				)
				.await
			};

			ctx.log(format!(
				"Final destination calculated: {} -> {}",
				resolved_source.display(),
				final_destination.display()
			));

			// Count files in this source path for accurate progress tracking
			let files_in_source = if let Some(local_path) = resolved_source.as_local_path() {
				self.count_files_in_path(local_path).await.unwrap_or(1)
			} else {
				1
			};

			// Mark file as currently copying in metadata
			self.job_metadata
				.update_status(&resolved_source, super::metadata::CopyFileStatus::Copying);

			// Persist immediately so UI can show "copying" status in real-time
			self.persist_job_state_to_db(&ctx).await?;

			// Update aggregator with current file info
			let operation_description = CopyStrategyRouter::describe_strategy(
				&resolved_source,
				&final_destination,
				is_move,
				&self.options.copy_method,
				volume_manager.as_deref(),
			)
			.await;

			progress_aggregator.start_file(
				resolved_source.display(),
				resolved_source.clone(),
				operation_description,
			);
			progress_aggregator.set_error_count(failed_copies.len());

			// Update progress - show files already completed
			let files_completed_count = *progress_aggregator.files_completed.lock().unwrap();
			let bytes_completed_snapshot = *progress_aggregator
				.bytes_completed_before_current
				.lock()
				.unwrap();
			let (current_rate, current_elapsed) = {
				let tracker = progress_aggregator.speed_tracker.lock().unwrap();
				(tracker.current_rate(), Some(tracker.elapsed()))
			}; // MutexGuard dropped here before any await
			let current_strategy_metadata = progress_aggregator
				.strategy_metadata
				.lock()
				.unwrap()
				.clone();
			let progress = CopyProgress {
				phase: CopyPhase::Copying,
				current_file: resolved_source.display(),
				current_source_path: Some(resolved_source.clone()),
				files_copied: files_completed_count,
				total_files: actual_file_count,
				bytes_copied: bytes_completed_snapshot,
				total_bytes: estimated_total_bytes,
				current_operation: progress_aggregator.current_operation.clone(),
				estimated_remaining: None,
				preparation_complete: true,
				error_count: failed_copies.len(),
				transfer_rate: current_rate,
				elapsed: current_elapsed,
				strategy_metadata: current_strategy_metadata,
			};
			ctx.progress(Progress::generic(progress.to_generic_progress()));

			// 1. Select the strategy with metadata
			let (strategy, strategy_metadata) = CopyStrategyRouter::select_strategy_with_metadata(
				&resolved_source,
				&final_destination,
				is_move,
				&self.options.copy_method,
				volume_manager.as_deref(),
			)
			.await;

			// Store strategy metadata for progress updates
			progress_aggregator.set_strategy_metadata(strategy_metadata);

			info!(
				"[JOB] About to execute strategy for {} -> {}",
				resolved_source.display(),
				final_destination.display()
			);

			// Handle conflict resolution before copying, with the same decision the
			// preflight plan reports (`files::plan::decide`).
			let decision = match crate::ops::files::plan::decide(
				&final_destination,
				self.options.conflict_resolution,
			)
			.await
			{
				Ok(decision) => decision,
				Err(e) => {
					failed_copies.push(CopyError {
						source: resolved_source.path().cloned().unwrap_or_default(),
						destination: final_destination.path().cloned().unwrap_or_default(),
						error: e.to_string(),
					});
					self.job_metadata.set_error(&resolved_source, e.to_string());
					continue;
				}
			};
			let may_replace = matches!(
				decision,
				crate::ops::files::plan::PlanDecision::Overwrite
					| crate::ops::files::plan::PlanDecision::Ask
			);
			let final_destination = match decision {
				crate::ops::files::plan::PlanDecision::Skip => {
					ctx.log(format!(
						"Skipping existing file: {}",
						final_destination.display()
					));
					self.job_metadata
						.update_status(&resolved_source, super::metadata::CopyFileStatus::Skipped);
					progress_aggregator.complete_source();
					skipped_count += files_in_source;
					self.completed_indices.push(index);
					continue;
				}
				crate::ops::files::plan::PlanDecision::Rename(unique_dest) => WingPath::Physical {
					device_slug: final_destination
						.device_slug()
						.unwrap_or_default()
						.to_string(),
					path: unique_dest,
				},
				crate::ops::files::plan::PlanDecision::Abort => {
					// Stop at the first conflict but keep what was already
					// copied visible: the rest of the batch counts as skipped.
					failed_copies.push(CopyError {
						source: resolved_source.path().cloned().unwrap_or_default(),
						destination: final_destination.path().cloned().unwrap_or_default(),
						error: "Destination already exists; operation aborted".to_string(),
					});
					for remaining in &sources[index + 1..] {
						skipped_count += match remaining.as_local_path() {
							Some(path) => self.count_files_in_path(path).await.unwrap_or(1),
							None => 1,
						};
					}
					break;
				}
				// The action asks about conflicts before queueing the job, so one found
				// here appeared since then. Keep it unless overwriting was chosen; a
				// case-only rename on a case-insensitive volume finds the source itself.
				crate::ops::files::plan::PlanDecision::Ask
					if !self.options.overwrite
						&& !same_entry(&resolved_source, &final_destination).await =>
				{
					let error = "Destination already exists".to_string();
					failed_copies.push(CopyError {
						source: resolved_source.path().cloned().unwrap_or_default(),
						destination: final_destination.path().cloned().unwrap_or_default(),
						error: error.clone(),
					});
					self.job_metadata.set_error(&resolved_source, error);
					continue;
				}
				crate::ops::files::plan::PlanDecision::Write
				| crate::ops::files::plan::PlanDecision::Overwrite
				| crate::ops::files::plan::PlanDecision::Ask => final_destination,
			};

			// A case-only rename on a case-insensitive volume finds itself there.
			let in_place = same_entry(&resolved_source, &final_destination).await;

			// What this item replaces is moved aside, not destroyed, so undo
			// can bring it back. A folder written into a folder is a merge.
			let mut merged = false;
			let mut stash = None;
			if may_replace && !in_place {
				if let Some(dest_path) = final_destination.as_local_path() {
					if let Ok(existing) = tokio::fs::symlink_metadata(dest_path).await {
						if existing.is_dir() {
							merged = true;
						} else {
							match journal.stash(dest_path).await {
								Ok(kept) => stash = Some(kept),
								Err(e) => {
									let error =
										format!("Could not set the existing file aside: {e}");
									failed_copies.push(CopyError {
										source: resolved_source.path().cloned().unwrap_or_default(),
										destination: dest_path.to_path_buf(),
										error: error.clone(),
									});
									self.job_metadata.set_error(&resolved_source, error);
									continue;
								}
							}
						}
					}
				}
			}

			// 2. Execute the strategy with progress callback
			match strategy
				.execute(
					&ctx,
					&resolved_source,
					&final_destination,
					self.options.verify_checksum,
					Some(&progress_aggregator.create_callback()),
				)
				.await
			{
				Ok(bytes) => {
					// Mark source as complete (bytes/files already updated by callback)
					progress_aggregator.complete_source();

					// Update totals
					copied_count += files_in_source;
					total_bytes += bytes;

					// Track successful completion for resume
					self.completed_indices.push(index);

					// Mark as completed in metadata
					self.job_metadata.update_status(
						&resolved_source,
						super::metadata::CopyFileStatus::Completed,
					);

					// A copy is a new file with a new modification time unless the
					// source's is put back. A renamed source is gone and kept its own.
					if self.options.preserve_timestamps {
						if let (Some(source_path), Some(dest_path)) = (
							resolved_source.as_local_path(),
							final_destination.as_local_path(),
						) {
							if source_path != dest_path {
								copy_modification_times(source_path, dest_path).await;
							}
						}
					}

					// Journal last, so the seal describes the finished item.
					let journal_after = |source_gone: bool| {
						let (source, destination) = (
							resolved_source.as_local_path().map(|p| p.to_path_buf()),
							final_destination.as_local_path().map(|p| p.to_path_buf()),
						);
						let journal = &journal;
						async move {
							use crate::ops::files::journal::{seal_of, Effect};
							let (Some(source), Some(destination)) = (source, destination) else {
								return;
							};
							if source == destination {
								return;
							}
							if merged {
								journal.record(&Effect::Merged { path: destination }).await;
								return;
							}
							let seal = match seal_of(&destination).await {
								Ok(seal) => seal,
								Err(error) => {
									tracing::warn!(path = %destination.display(), %error, "Could not seal a finished item; it cannot be undone");
									return;
								}
							};
							let effect = if source_gone || in_place {
								Effect::Moved {
									from: source,
									to: destination,
									seal,
								}
							} else {
								Effect::Created {
									path: destination,
									seal,
								}
							};
							journal.record(&effect).await;
						}
					};

					// A move whose strategy copied rather than renamed still has its
					// source. Whether the rename happened is read from the filesystem,
					// not inferred from volume knowledge: a streaming move on one
					// volume, or a job without a volume manager, used to skip this
					// delete and leave the file in both places while reporting success.
					// Ported from upstream Spacedrive 15d7df180.
					if is_move && !resolved_source.is_cross_device(&final_destination) {
						if let (Some(source_path), Some(dest_path)) = (
							resolved_source.as_local_path(),
							final_destination.as_local_path(),
						) {
							let still_there =
								tokio::fs::symlink_metadata(source_path).await.is_ok();
							if still_there && source_path != dest_path {
								if let Err(e) = self.delete_source_file(source_path).await {
									failed_copies.push(CopyError {
										source: resolved_source.path().cloned().unwrap_or_default(),
										destination: final_destination
											.path()
											.cloned()
											.unwrap_or_default(),
										error: format!(
											"Copy succeeded but failed to delete source: {}",
											e
										),
									});
									ctx.add_non_critical_error(format!(
										"Failed to delete source after move {}: {}",
										resolved_source.display(),
										e
									));
								}
							}
						}
					}

					let source_gone = match resolved_source.as_local_path() {
						Some(source_path) => {
							tokio::fs::symlink_metadata(source_path).await.is_err()
						}
						None => false,
					};
					journal_after(source_gone).await;
				}
				Err(e) => {
					// The replacement never arrived; put the previous item back.
					if let Some(kept) = stash.take() {
						journal.unstash(kept).await;
					}

					// A cancelled or paused job stops here. Counting the item as
					// failed let the job carry on and finish as completed.
					if super::strategy::is_interruption(&e) {
						return Err(JobError::Interrupted);
					}

					failed_copies.push(CopyError {
						source: resolved_source.path().cloned().unwrap_or_default(),
						destination: final_destination.path().cloned().unwrap_or_default(),
						error: e.to_string(),
					});
					ctx.add_non_critical_error(format!(
						"Failed to {} {}: {}",
						if is_move { "move" } else { "copy" },
						resolved_source.display(),
						e
					));

					// Mark as failed in metadata
					self.job_metadata.set_error(&resolved_source, e.to_string());
				}
			}

			// Persist after every file so UI can show real-time checkbox updates
			self.persist_job_state_to_db(&ctx).await?;

			// Checkpoint every 20 files to save job state to disk
			if copied_count % 20 == 0 {
				ctx.checkpoint().await?;
			}
		}

		// Phase 4: Complete
		let final_elapsed = progress_aggregator.speed_tracker.lock().unwrap().elapsed();
		let final_strategy_metadata = progress_aggregator
			.strategy_metadata
			.lock()
			.unwrap()
			.clone();
		let progress = CopyProgress {
			phase: CopyPhase::Complete,
			current_file: String::new(),
			current_source_path: None,
			files_copied: copied_count,
			total_files: actual_file_count,
			bytes_copied: total_bytes,
			total_bytes: estimated_total_bytes,
			current_operation: "Copy operation complete".to_string(),
			estimated_remaining: Some(Duration::ZERO),
			preparation_complete: true,
			error_count: failed_copies.len(),
			transfer_rate: 0.0,
			elapsed: Some(final_elapsed),
			strategy_metadata: final_strategy_metadata,
		};
		ctx.progress(Progress::generic(progress.to_generic_progress()));

		// Persist final job state with all file statuses
		self.persist_job_state_to_db(&ctx).await?;

		ctx.log(format!(
			"Copy operation completed: {} copied, {} failed, {} skipped",
			copied_count,
			failed_copies.len(),
			skipped_count
		));

		Ok(FileCopyOutput {
			copied_count,
			failed_count: failed_copies.len(),
			skipped_count,
			total_bytes,
			duration: self.started_at.elapsed(),
			failed_copies,
			is_move_operation: self.options.delete_after_copy,
		})
	}
}

/// Copy operation phases
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Type)]
pub enum CopyPhase {
	Initializing,
	DatabaseQuery,
	Preparation,
	Copying,
	Complete,
}

impl CopyPhase {
	fn as_str(&self) -> &'static str {
		match self {
			CopyPhase::Initializing => "Initializing",
			CopyPhase::DatabaseQuery => "Database Query",
			CopyPhase::Preparation => "Preparation",
			CopyPhase::Copying => "Copying",
			CopyPhase::Complete => "Complete",
		}
	}
}

/// Copy progress information with real-time transfer metrics.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CopyProgress {
	pub phase: CopyPhase,
	pub current_file: String,
	/// Current source path being processed (for GenericProgress.current_path)
	#[serde(default)]
	pub current_source_path: Option<WingPath>,
	pub files_copied: usize,
	pub total_files: usize,
	pub bytes_copied: u64,
	pub total_bytes: u64,
	pub current_operation: String,
	pub estimated_remaining: Option<Duration>,
	pub preparation_complete: bool,
	pub error_count: usize,
	/// Current transfer rate in bytes per second
	#[serde(default)]
	pub transfer_rate: f32,
	/// Time elapsed since copy started
	#[serde(default)]
	pub elapsed: Option<Duration>,
	/// Strategy metadata for UI display
	#[serde(default)]
	pub strategy_metadata: Option<super::routing::CopyStrategyMetadata>,
}

impl JobProgress for CopyProgress {}

/// Progress aggregator that tracks overall copy job progress with real-time speed calculation.
struct ProgressAggregator<'a> {
	ctx: &'a JobContext<'a>,
	current_file_index: usize,
	total_files: usize,
	bytes_completed_before_current: Arc<Mutex<u64>>,
	total_bytes: u64,
	current_file_path: String,
	current_source_path: Arc<Mutex<Option<WingPath>>>,
	current_operation: String,
	error_count: usize,
	files_completed: Arc<Mutex<usize>>,
	speed_tracker: Arc<Mutex<SpeedTracker>>,
	strategy_metadata: Arc<Mutex<Option<super::routing::CopyStrategyMetadata>>>,
}

impl<'a> ProgressAggregator<'a> {
	fn new(ctx: &'a JobContext<'a>, total_files: usize, total_bytes: u64) -> Self {
		Self {
			ctx,
			current_file_index: 0,
			total_files,
			bytes_completed_before_current: Arc::new(Mutex::new(0)),
			total_bytes,
			current_file_path: String::new(),
			current_source_path: Arc::new(Mutex::new(None)),
			current_operation: String::new(),
			error_count: 0,
			files_completed: Arc::new(Mutex::new(0)),
			speed_tracker: Arc::new(Mutex::new(SpeedTracker::new())),
			strategy_metadata: Arc::new(Mutex::new(None)),
		}
	}

	/// Start processing a new file with strategy metadata
	fn start_file(&mut self, file_path: String, source_path: WingPath, current_operation: String) {
		self.current_file_path = file_path;
		*self.current_source_path.lock().unwrap() = Some(source_path);
		self.current_operation = current_operation;
	}

	/// Set the current strategy metadata
	fn set_strategy_metadata(&mut self, metadata: super::routing::CopyStrategyMetadata) {
		*self.strategy_metadata.lock().unwrap() = Some(metadata);
	}

	/// Complete the current source item and update index
	fn complete_source(&mut self) {
		self.current_file_index += 1;
		// Note: bytes and file counts are already updated by the callback
	}

	/// Account for files that were already completed (for resume)
	fn skip_completed_file(&mut self, bytes_copied: u64, files_in_operation: usize) {
		*self.bytes_completed_before_current.lock().unwrap() += bytes_copied;
		*self.files_completed.lock().unwrap() += files_in_operation;
	}

	/// Create a progress callback for strategy implementations with speed tracking.
	fn create_callback(&self) -> Box<dyn Fn(u64, u64) + Send + Sync + 'a> {
		let ctx = self.ctx;
		let files_completed = self.files_completed.clone();
		let total_files = self.total_files;
		let bytes_before = self.bytes_completed_before_current.clone();
		let total_bytes = self.total_bytes;
		let current_file = self.current_file_path.clone();
		let current_source_path = self.current_source_path.clone();
		let current_operation = self.current_operation.clone();
		let error_count = self.error_count;
		let speed_tracker = self.speed_tracker.clone();
		let strategy_metadata = self.strategy_metadata.clone();

		Box::new(move |bytes_value: u64, signal_value: u64| {
			// Signal: u64::MAX means a file has finished, bytes_value is its size
			if signal_value == u64::MAX {
				// Update backend state
				let mut files = files_completed.lock().unwrap();
				*files += 1;
				drop(files);
				let mut bytes = bytes_before.lock().unwrap();
				*bytes += bytes_value;
				drop(bytes);
			}

			// Read current backend state for progress event (AFTER mutations above)
			let files_completed_count = *files_completed.lock().unwrap();
			let bytes_before_snapshot = *bytes_before.lock().unwrap();

			// Calculate total bytes for this progress event
			let total_bytes_copied = if signal_value == u64::MAX {
				// File just completed - bytes were already added to bytes_before above
				bytes_before_snapshot
			} else {
				// Byte-level progress - bytes_value is current file progress within current file
				bytes_before_snapshot + bytes_value
			};

			// Update speed tracker and get current rate
			let mut tracker = speed_tracker.lock().unwrap();
			let rate = tracker.update(total_bytes_copied);
			let bytes_remaining = total_bytes.saturating_sub(total_bytes_copied);
			let estimated_remaining = tracker.calculate_eta(bytes_remaining);
			let elapsed = tracker.elapsed();
			drop(tracker);

			let current_strategy_metadata = strategy_metadata.lock().unwrap().clone();
			let current_src_path = current_source_path.lock().unwrap().clone();

			let copy_progress = CopyProgress {
				phase: CopyPhase::Copying,
				current_file: current_file.clone(),
				current_source_path: current_src_path,
				files_copied: files_completed_count,
				total_files,
				bytes_copied: total_bytes_copied,
				total_bytes,
				current_operation: current_operation.clone(),
				estimated_remaining,
				preparation_complete: true,
				error_count,
				transfer_rate: rate,
				elapsed: Some(elapsed),
				strategy_metadata: current_strategy_metadata,
			};

			// Log progress details every 100MB or on file completion
			if signal_value == u64::MAX {
				ctx.log(format!(
					"File completed. Total files: {}/{}, Total bytes: {}",
					files_completed_count, total_files, total_bytes_copied
				));
			} else if total_bytes_copied % (100 * 1024 * 1024) < bytes_value {
				ctx.log(format!(
					"Progress: {} / {} bytes ({:.1}%), {}/s, ETA: {}",
					total_bytes_copied,
					total_bytes,
					(total_bytes_copied as f64 / total_bytes as f64) * 100.0,
					format_bytes(rate as u64),
					estimated_remaining
						.map(|d| format!("{:.0}s", d.as_secs_f32()))
						.unwrap_or_else(|| "calculating...".to_string())
				));
			}

			ctx.progress(Progress::generic(copy_progress.to_generic_progress()));
		})
	}

	fn set_error_count(&mut self, count: usize) {
		self.error_count = count;
	}
}

impl ToGenericProgress for CopyProgress {
	fn to_generic_progress(&self) -> GenericProgress {
		// Calculate percentage based on bytes if available, otherwise use file count
		let percentage = if self.total_bytes > 0 {
			(self.bytes_copied as f32 / self.total_bytes as f32).clamp(0.0, 1.0)
		} else if self.total_files > 0 {
			(self.files_copied as f32 / self.total_files as f32).clamp(0.0, 1.0)
		} else {
			0.0
		};

		// Create appropriate message based on phase
		let message = match self.phase {
			CopyPhase::Initializing => "Initializing copy operation...".to_string(),
			CopyPhase::DatabaseQuery => "Querying database for file information...".to_string(),
			CopyPhase::Preparation => {
				if self.total_files > 0 {
					format!("Preparing to copy {} files...", self.total_files)
				} else {
					"Preparing copy operation...".to_string()
				}
			}
			CopyPhase::Copying => {
				if !self.current_file.is_empty() {
					format!("Copying: {}", self.current_file)
				} else {
					self.current_operation.clone()
				}
			}
			CopyPhase::Complete => format!("Copy complete: {} files", self.files_copied),
		};

		// Build generic progress
		let mut progress = GenericProgress::new(percentage, self.phase.as_str(), message);

		// Only set completion if we're not using byte-based progress
		// (to avoid overriding the percentage)
		if self.total_bytes == 0 {
			progress = progress.with_completion(self.files_copied as u64, self.total_files as u64);
		} else {
			// For byte-based progress, just set the completion counts without recalculating percentage
			progress.completion.completed = self.files_copied as u64;
			progress.completion.total = self.total_files as u64;
		}

		progress = progress.with_bytes(self.bytes_copied, self.total_bytes);

		// Add performance metrics with real transfer rate
		if self.phase == CopyPhase::Copying && self.bytes_copied > 0 {
			progress = progress.with_performance(
				self.transfer_rate,
				self.estimated_remaining,
				self.elapsed,
			);
		}

		// Add error count if any
		if self.error_count > 0 {
			progress = progress.with_errors(self.error_count as u64, 0);
		}

		// Add current path if available
		if let Some(ref path) = self.current_source_path {
			progress = progress.with_current_path(path.clone());
		}

		// Add strategy metadata for UI display
		if let Some(ref strategy_metadata) = self.strategy_metadata {
			progress = progress.with_metadata(serde_json::json!({
				"strategy": strategy_metadata
			}));
		}

		progress
	}
}

/// Error information for failed copies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopyError {
	pub source: PathBuf,
	pub destination: PathBuf,
	pub error: String,
}

impl FileCopyJob {
	/// Create a new file copy job with sources and destination
	pub fn new(sources: WingPathBatch, destination: WingPath) -> Self {
		Self {
			sources,
			destination,
			options: Default::default(),
			completed_indices: Vec::new(),
			started_at: Instant::now(),
			job_metadata: super::metadata::CopyJobMetadata::default(),
		}
	}

	/// Create an empty job (used by derive macro)
	pub fn empty() -> Self {
		Self {
			sources: WingPathBatch::new(Vec::new()),
			destination: WingPath::local(PathBuf::new()),
			options: Default::default(),
			completed_indices: Vec::new(),
			started_at: Instant::now(),
			job_metadata: super::metadata::CopyJobMetadata::default(),
		}
	}

	/// Create from individual paths
	pub fn from_paths(sources: Vec<WingPath>, destination: WingPath) -> Self {
		Self::new(WingPathBatch::new(sources), destination)
	}

	/// Set copy options
	pub fn with_options(mut self, options: CopyOptions) -> Self {
		self.options = options;
		self
	}

	/// Create a move job using the copy job with delete_after_copy
	pub fn new_move(sources: WingPathBatch, destination: WingPath, move_mode: MoveMode) -> Self {
		let mut options = CopyOptions::default();
		options.delete_after_copy = true;
		options.move_mode = Some(move_mode);
		Self {
			sources,
			destination,
			options,
			completed_indices: Vec::new(),
			started_at: Instant::now(),
			job_metadata: super::metadata::CopyJobMetadata::new(true),
		}
	}

	/// Create a rename operation
	pub fn new_rename(source: WingPath, new_name: String) -> Self {
		let destination = match &source {
			WingPath::Physical { device_slug, path } => WingPath::Physical {
				device_slug: device_slug.clone(),
				path: path.with_file_name(&new_name),
			},
			WingPath::Cloud {
				service,
				identifier,
				path,
			} => WingPath::Cloud {
				service: *service,
				identifier: identifier.clone(),
				path: match path.trim_end_matches('/').rsplit_once('/') {
					Some((parent, _)) => format!("{parent}/{new_name}"),
					None => new_name.clone(),
				},
			},
			WingPath::Content { .. } => panic!("Cannot rename a content-addressed path"),
			WingPath::Sidecar { .. } => panic!("Cannot rename a sidecar path"),
		};

		Self::new_move(
			WingPathBatch::new(vec![source]),
			destination,
			MoveMode::Rename,
		)
	}

	/// Calculate total size for progress reporting
	async fn calculate_total_size(&self, ctx: &JobContext<'_>) -> JobResult<u64> {
		use crate::ops::indexing::PathResolver;

		let mut total = 0u64;

		for source in &self.sources.paths {
			if let Some(local_path) = source.as_local_path() {
				// Local path - calculate directly from filesystem
				total += self.get_path_size(local_path).await.unwrap_or(0);
			} else {
				// Non-local path - query database for synced metadata
				match PathResolver::resolve_to_entry(ctx.library_db(), source).await {
					Ok(Some(entry)) => {
						let size = match entry.kind {
							0 => entry.size as u64,           // File
							1 => entry.aggregate_size as u64, // Directory
							_ => 0,
						};
						total += size;
						ctx.log(format!(
							"Remote source '{}': found in database ({} bytes)",
							source.display(),
							size
						));
					}
					Ok(None) => {
						ctx.log(format!(
							"Remote source '{}': not indexed, size unknown",
							source.display()
						));
					}
					Err(e) => {
						ctx.log(format!(
							"Remote source '{}': database error: {}",
							source.display(),
							e
						));
					}
				}
			}
		}

		Ok(total)
	}

	/// Persist the job state to the database so metadata can be queried
	async fn persist_job_state_to_db(&self, ctx: &JobContext<'_>) -> JobResult<()> {
		use sea_orm::{ActiveModelTrait, ActiveValue::Set};

		// Serialize current job state
		let job_state = rmp_serde::to_vec(self).map_err(|e| {
			JobError::serialization(format!("Failed to serialize job state: {}", e))
		})?;

		// Update the jobs.state field in the database
		let job_db = ctx.library.jobs().database();
		let mut job_model = crate::infra::job::database::jobs::ActiveModel {
			id: Set(ctx.id.to_string()),
			state: Set(job_state.clone()),
			..Default::default()
		};

		job_model
			.update(job_db.conn())
			.await
			.map_err(|e| JobError::execution(format!("Failed to persist job state: {}", e)))?;

		ctx.log(format!(
			"Persisted job metadata to database ({} files in queue)",
			self.job_metadata.files.len()
		));

		Ok(())
	}

	/// Collect file metadata for the queryable file list.
	/// Directories are represented as a single entry (not flattened).
	async fn collect_file_metadata(&mut self, ctx: &JobContext<'_>) -> JobResult<()> {
		use super::metadata::{CopyFileEntry, CopyFileStatus};
		use crate::ops::indexing::PathResolver;

		self.job_metadata = super::metadata::CopyJobMetadata::new(self.options.delete_after_copy);

		for source in &self.sources.paths {
			let resolved_source = source.resolve_in_job(ctx).await.map_err(|e| {
				JobError::execution(format!("Failed to resolve source path: {}", e))
			})?;

			let (size_bytes, is_directory, entry_id) = if let Some(local_path) =
				resolved_source.as_local_path()
			{
				// Local path - get from filesystem
				let metadata = selection_metadata(local_path)
					.await
					.map_err(|e| JobError::execution(format!("Failed to read metadata: {}", e)))?;

				let size = if metadata.is_dir() {
					self.get_path_size(local_path).await.unwrap_or(0)
				} else {
					metadata.len()
				};

				// Try to find entry UUID for local paths too
				let entry_id = PathResolver::resolve_to_entry(ctx.library_db(), &resolved_source)
					.await
					.ok()
					.flatten()
					.and_then(|e| e.uuid);

				(size, metadata.is_dir(), entry_id)
			} else {
				// Remote path - query database for synced metadata
				match PathResolver::resolve_to_entry(ctx.library_db(), &resolved_source).await {
					Ok(Some(entry)) => {
						let size = match entry.kind {
							0 => entry.size as u64,
							1 => entry.aggregate_size as u64,
							_ => 0,
						};
						let is_dir = entry.kind == 1;
						(size, is_dir, entry.uuid)
					}
					Ok(None) | Err(_) => {
						// Entry not found - skip this file
						ctx.log(format!(
							"Warning: Could not find metadata for remote source: {}",
							resolved_source.display()
						));
						continue;
					}
				}
			};

			// Calculate destination path
			let dest_path = if let Some(dest_local) = self.destination.as_local_path() {
				if dest_local.is_dir() || self.sources.paths.len() > 1 {
					self.destination
						.join(resolved_source.file_name().unwrap_or_default().as_ref())
				} else {
					self.destination.clone()
				}
			} else {
				self.destination
					.join(resolved_source.file_name().unwrap_or_default().as_ref())
			};

			let entry = CopyFileEntry {
				source_path: resolved_source.clone(),
				dest_path,
				size_bytes,
				is_directory,
				status: CopyFileStatus::Pending,
				error: None,
				entry_id,
			};

			self.job_metadata.add_file(entry);
		}

		ctx.log(format!(
			"Collected metadata for {} source items ({} total bytes)",
			self.job_metadata.files.len(),
			self.job_metadata.total_bytes
		));

		Ok(())
	}

	/// Count total number of files to be copied (including files within directories)
	async fn count_total_files(&self) -> JobResult<usize> {
		let mut total_count = 0;

		for source in &self.sources.paths {
			if let Some(local_path) = source.as_local_path() {
				// Local path - count directly from filesystem
				total_count += self.count_files_in_path(local_path).await.unwrap_or(0);
			} else {
				// Non-local path - use database estimate
				// For now, count as 1 item (will be refined during actual transfer)
				total_count += 1;
			}
		}

		Ok(total_count)
	}

	/// Count files in a path (recursive for directories)
	async fn count_files_in_path(&self, path: &std::path::Path) -> Result<usize, std::io::Error> {
		Ok(tree_totals(path).await?.0)
	}

	/// List all files in a directory (recursive)
	async fn list_files_in_directory(&self, path: &std::path::Path) -> JobResult<Vec<PathBuf>> {
		let mut files = Vec::new();
		let mut stack = vec![path.to_path_buf()];

		while let Some(current_path) = stack.pop() {
			let metadata = tokio::fs::metadata(&current_path)
				.await
				.map_err(|e| JobError::execution(format!("Failed to read metadata: {}", e)))?;

			if metadata.is_file() {
				files.push(current_path);
			} else if metadata.is_dir() {
				let mut dir = tokio::fs::read_dir(&current_path)
					.await
					.map_err(|e| JobError::execution(format!("Failed to read directory: {}", e)))?;
				while let Some(entry) = dir.next_entry().await.map_err(|e| {
					JobError::execution(format!("Failed to read directory entry: {}", e))
				})? {
					stack.push(entry.path());
				}
			}
		}

		Ok(files)
	}

	/// Get size of a path (file or directory) using iterative approach
	async fn get_path_size(&self, path: &std::path::Path) -> Result<u64, std::io::Error> {
		Ok(tree_totals(path).await?.1)
	}

	/// Delete the source of a move whose strategy copied it.
	///
	/// Reads the link itself, not its target, so a moved symlink to a directory
	/// is unlinked rather than followed.
	async fn delete_source_file(&self, source: &std::path::Path) -> Result<(), std::io::Error> {
		let metadata = tokio::fs::symlink_metadata(source).await?;

		if metadata.is_dir() {
			tokio::fs::remove_dir_all(source).await
		} else {
			tokio::fs::remove_file(source).await
		}
	}
}

/// Metadata of a selected path. On Unix a symbolic link is the item itself,
/// as the copy strategies treat it.
async fn selection_metadata(path: &std::path::Path) -> Result<std::fs::Metadata, std::io::Error> {
	#[cfg(unix)]
	{
		tokio::fs::symlink_metadata(path).await
	}
	#[cfg(not(unix))]
	{
		tokio::fs::metadata(path).await
	}
}

/// Gives a finished copy the modification times of its source, folders included.
///
/// Best effort: the copy is complete either way, so a failure is only logged.
/// Links are left alone.
async fn copy_modification_times(source: &std::path::Path, destination: &std::path::Path) {
	fn apply(source: &std::path::Path, destination: &std::path::Path) -> std::io::Result<()> {
		let metadata = std::fs::symlink_metadata(source)?;
		if metadata.file_type().is_symlink() {
			return Ok(());
		}
		if metadata.is_dir() {
			// Children first: writing into a folder changes its own time.
			for entry in std::fs::read_dir(source)? {
				let entry = entry?;
				let _ = apply(&entry.path(), &destination.join(entry.file_name()));
			}
		}
		let modified = metadata.modified()?;
		// Windows needs write access to set times; a read-only file on Unix
		// only opens for reading, which is enough for its owner there.
		let file = std::fs::OpenOptions::new()
			.write(true)
			.open(destination)
			.or_else(|_| std::fs::File::open(destination))?;
		file.set_modified(modified)
	}

	let (source, destination) = (source.to_path_buf(), destination.to_path_buf());
	let shown = destination.clone();
	match tokio::task::spawn_blocking(move || apply(&source, &destination)).await {
		Ok(Ok(())) => {}
		Ok(Err(error)) => {
			tracing::warn!(path = %shown.display(), %error, "Could not keep the modification time of a copy")
		}
		Err(error) => {
			tracing::warn!(path = %shown.display(), %error, "Could not keep the modification time of a copy")
		}
	}
}

/// Count the items under `path` and add up their sizes, as the copy will see them.
///
/// Links count as one item of their own size, the selected path included on
/// Unix: the copy recreates them instead of reading their targets, and
/// following them walked the whole disk for a link to `/` and never ended for a
/// link to an ancestor.
pub(crate) async fn tree_totals(path: &std::path::Path) -> Result<(usize, u64), std::io::Error> {
	let mut count = 0;
	let mut size = 0u64;
	let mut stack = vec![(path.to_path_buf(), true)];

	while let Some((current, is_selection)) = stack.pop() {
		let metadata = if is_selection {
			selection_metadata(&current).await?
		} else {
			tokio::fs::symlink_metadata(&current).await?
		};

		if metadata.is_dir() {
			let mut dir = tokio::fs::read_dir(&current).await?;
			while let Some(entry) = dir.next_entry().await? {
				stack.push((entry.path(), false));
			}
		} else {
			count += 1;
			size += metadata.len();
		}
	}

	Ok((count, size))
}

/// Whether both paths name the same entry on disk, without following links:
/// a link renamed to its target's name is not the target.
pub(crate) async fn same_entry(a: &WingPath, b: &WingPath) -> bool {
	let (Some(a), Some(b)) = (a.as_local_path(), b.as_local_path()) else {
		return false;
	};
	#[cfg(unix)]
	{
		use std::os::unix::fs::MetadataExt;
		match (
			tokio::fs::symlink_metadata(a).await,
			tokio::fs::symlink_metadata(b).await,
		) {
			(Ok(a), Ok(b)) => a.dev() == b.dev() && a.ino() == b.ino(),
			_ => false,
		}
	}
	#[cfg(not(unix))]
	{
		// Windows names are case-insensitive, so compare the folder and then the name.
		let (Some(a_parent), Some(b_parent)) = (a.parent(), b.parent()) else {
			return false;
		};
		let same_folder = match (
			tokio::fs::canonicalize(a_parent).await,
			tokio::fs::canonicalize(b_parent).await,
		) {
			(Ok(a), Ok(b)) => a == b,
			_ => false,
		};
		same_folder
			&& a.file_name().map(|n| n.to_string_lossy().to_lowercase())
				== b.file_name().map(|n| n.to_string_lossy().to_lowercase())
	}
}

/// Output from file copy job
#[derive(Debug, Serialize, Deserialize)]
pub struct FileCopyOutput {
	pub copied_count: usize,
	pub failed_count: usize,
	#[serde(default)]
	pub skipped_count: usize,
	pub total_bytes: u64,
	pub duration: Duration,
	pub failed_copies: Vec<CopyError>,
	pub is_move_operation: bool,
}

/// Failure messages kept in a job output; the full list stays in the job log.
const MAX_REPORTED_ERRORS: usize = 20;

fn summarize_errors<'a>(errors: impl Iterator<Item = (&'a PathBuf, &'a str)>) -> Vec<String> {
	errors
		.take(MAX_REPORTED_ERRORS)
		.map(|(source, error)| format!("{}: {}", source.display(), error))
		.collect()
}

impl From<FileCopyOutput> for JobOutput {
	fn from(output: FileCopyOutput) -> Self {
		let errors = summarize_errors(
			output
				.failed_copies
				.iter()
				.map(|e| (&e.source, e.error.as_str())),
		);
		if output.is_move_operation {
			JobOutput::FileMove {
				moved_count: output.copied_count,
				failed_count: output.failed_count,
				skipped_count: output.skipped_count,
				total_bytes: output.total_bytes,
				errors,
			}
		} else {
			JobOutput::FileCopy {
				copied_count: output.copied_count,
				failed_count: output.failed_count,
				skipped_count: output.skipped_count,
				total_bytes: output.total_bytes,
				errors,
			}
		}
	}
}

/// Backward compatibility wrapper for move operations
#[derive(Debug, Serialize, Deserialize, Job)]
pub struct MoveJob {
	pub sources: WingPathBatch,
	pub destination: WingPath,
	pub mode: MoveMode,
	pub overwrite: bool,
	pub preserve_timestamps: bool,
}

impl Job for MoveJob {
	const NAME: &'static str = "move_files";
	const RESUMABLE: bool = true;
	const DESCRIPTION: Option<&'static str> = Some("Move or rename files and directories");
}

impl crate::infra::job::traits::DynJob for MoveJob {
	fn job_name(&self) -> &'static str {
		Self::NAME
	}
}

#[async_trait::async_trait]
impl JobHandler for MoveJob {
	type Output = MoveOutput;

	async fn run(&mut self, ctx: JobContext<'_>) -> JobResult<Self::Output> {
		// Convert to FileCopyJob with move options
		let mut copy_options = CopyOptions::default();
		copy_options.delete_after_copy = true;
		copy_options.move_mode = Some(self.mode.clone());
		copy_options.overwrite = self.overwrite;
		copy_options.preserve_timestamps = self.preserve_timestamps;

		let mut copy_job = FileCopyJob {
			sources: self.sources.clone(),
			destination: self.destination.clone(),
			options: copy_options,
			completed_indices: Vec::new(),
			started_at: Instant::now(),
			job_metadata: super::metadata::CopyJobMetadata::new(true),
		};

		// Run the copy job
		let copy_output = copy_job.run(ctx).await?;

		// Convert output to move format
		Ok(MoveOutput {
			moved_count: copy_output.copied_count,
			failed_count: copy_output.failed_count,
			skipped_count: copy_output.skipped_count,
			total_bytes: copy_output.total_bytes,
			duration: copy_output.duration,
			failed_moves: copy_output
				.failed_copies
				.into_iter()
				.map(|e| MoveError {
					source: e.source,
					destination: e.destination,
					error: e.error,
				})
				.collect(),
		})
	}
}

impl MoveJob {
	/// Create a new move job
	pub fn new(sources: WingPathBatch, destination: WingPath, mode: MoveMode) -> Self {
		Self {
			sources,
			destination,
			mode,
			overwrite: false,
			preserve_timestamps: true,
		}
	}

	/// Create an empty job (used by derive macro)
	pub fn empty() -> Self {
		Self {
			sources: WingPathBatch::new(Vec::new()),
			destination: WingPath::local(PathBuf::new()),
			mode: MoveMode::Move,
			overwrite: false,
			preserve_timestamps: true,
		}
	}

	/// Create a rename operation
	pub fn rename(source: WingPath, new_name: String) -> Self {
		let destination = match &source {
			WingPath::Physical { device_slug, path } => WingPath::Physical {
				device_slug: device_slug.clone(),
				path: path.with_file_name(&new_name),
			},
			WingPath::Cloud {
				service,
				identifier,
				path,
			} => WingPath::Cloud {
				service: *service,
				identifier: identifier.clone(),
				path: match path.trim_end_matches('/').rsplit_once('/') {
					Some((parent, _)) => format!("{parent}/{new_name}"),
					None => new_name.clone(),
				},
			},
			WingPath::Content { .. } => panic!("Cannot rename a content-addressed path"),
			WingPath::Sidecar { .. } => panic!("Cannot rename a sidecar path"),
		};

		Self::new(
			WingPathBatch::new(vec![source]),
			destination,
			MoveMode::Rename,
		)
	}
}

/// Error information for failed moves
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveError {
	pub source: PathBuf,
	pub destination: PathBuf,
	pub error: String,
}

/// Output from move operations
#[derive(Debug, Serialize, Deserialize)]
pub struct MoveOutput {
	pub moved_count: usize,
	pub failed_count: usize,
	#[serde(default)]
	pub skipped_count: usize,
	pub total_bytes: u64,
	pub duration: Duration,
	pub failed_moves: Vec<MoveError>,
}

impl From<MoveOutput> for JobOutput {
	fn from(output: MoveOutput) -> Self {
		JobOutput::FileMove {
			moved_count: output.moved_count,
			failed_count: output.failed_count,
			skipped_count: output.skipped_count,
			total_bytes: output.total_bytes,
			errors: summarize_errors(
				output
					.failed_moves
					.iter()
					.map(|e| (&e.source, e.error.as_str())),
			),
		}
	}
}

// Helper function to format bytes in human-readable format
fn format_bytes(bytes: u64) -> String {
	const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
	let mut size = bytes as f64;
	let mut unit_idx = 0;

	while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
		size /= 1024.0;
		unit_idx += 1;
	}

	if unit_idx == 0 {
		format!("{} {}", size as u64, UNITS[unit_idx])
	} else {
		format!("{:.2} {}", size, UNITS[unit_idx])
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::thread;

	#[cfg(unix)]
	#[tokio::test]
	async fn totals_count_inner_links_once_without_following_them() {
		let root = tempfile::tempdir().unwrap();
		let folder = root.path().join("folder");
		std::fs::create_dir(&folder).unwrap();
		std::fs::write(folder.join("data.bin"), [0u8; 100]).unwrap();
		// A link to an ancestor never ends when followed; one to `/` walks the disk.
		std::os::unix::fs::symlink(root.path(), folder.join("ancestor")).unwrap();
		std::os::unix::fs::symlink("/", folder.join("root")).unwrap();
		std::os::unix::fs::symlink(folder.join("missing"), folder.join("dangling")).unwrap();

		let (count, size) = tokio::time::timeout(Duration::from_secs(5), tree_totals(&folder))
			.await
			.expect("walking the folder must not follow its links")
			.unwrap();
		assert_eq!(count, 4);
		let links: u64 = ["ancestor", "root", "dangling"]
			.iter()
			.map(|name| std::fs::symlink_metadata(folder.join(name)).unwrap().len())
			.sum();
		assert_eq!(size, 100 + links);

		// A selected link is one item too, whether or not its target exists.
		let selected_link = root.path().join("selected");
		std::os::unix::fs::symlink(&folder, &selected_link).unwrap();
		let link_size = std::fs::symlink_metadata(&selected_link).unwrap().len();
		assert_eq!(tree_totals(&selected_link).await.unwrap(), (1, link_size));
		assert_eq!(tree_totals(&folder.join("dangling")).await.unwrap().0, 1);
	}

	#[test]
	fn test_speed_tracker_creation() {
		let tracker = SpeedTracker::new();
		assert_eq!(tracker.current_rate(), 0.0);
		assert_eq!(tracker.avg_rate(), 0.0);
	}

	#[test]
	fn test_speed_tracker_update() {
		let mut tracker = SpeedTracker::new();

		// Wait to ensure we exceed the 50ms throttle
		thread::sleep(Duration::from_millis(60));

		// Simulate 1MB transferred
		let rate = tracker.update(1024 * 1024);

		// Rate should be positive (bytes transferred over time)
		assert!(rate > 0.0, "Rate should be positive after transfer");
		assert!(tracker.current_rate() > 0.0);
		assert!(tracker.avg_rate() > 0.0);
	}

	#[test]
	fn test_speed_tracker_throttling() {
		let mut tracker = SpeedTracker::new();

		// First call should return 0 (no time elapsed)
		let rate1 = tracker.update(1000);
		assert_eq!(rate1, 0.0, "First immediate call should be throttled");

		// Call again immediately - should still be throttled
		let rate2 = tracker.update(2000);
		assert_eq!(rate2, 0.0, "Immediate second call should be throttled");
	}

	#[test]
	fn test_speed_tracker_eta() {
		let mut tracker = SpeedTracker::new();

		// Wait and update to establish a rate
		thread::sleep(Duration::from_millis(60));
		tracker.update(1024 * 1024); // 1MB

		// Calculate ETA for remaining bytes
		let eta = tracker.calculate_eta(1024 * 1024 * 10); // 10MB remaining

		// Should have an ETA now
		assert!(
			eta.is_some(),
			"ETA should be calculable after rate established"
		);
	}

	#[test]
	fn test_speed_tracker_eta_zero_rate() {
		let tracker = SpeedTracker::new();

		// No updates, rate is 0
		let eta = tracker.calculate_eta(1024 * 1024);

		// Should return None when rate is too low
		assert!(eta.is_none(), "ETA should be None when rate is 0");
	}

	#[test]
	fn test_speed_tracker_elapsed() {
		let tracker = SpeedTracker::new();

		thread::sleep(Duration::from_millis(50));

		let elapsed = tracker.elapsed();
		assert!(
			elapsed >= Duration::from_millis(50),
			"Elapsed time should be at least 50ms"
		);
	}

	#[test]
	fn test_speed_tracker_exponential_smoothing() {
		let mut tracker = SpeedTracker::new();

		// Simulate varying speeds
		thread::sleep(Duration::from_millis(60));
		tracker.update(1024 * 100); // 100KB

		thread::sleep(Duration::from_millis(60));
		let rate1 = tracker.update(1024 * 200); // 200KB total

		thread::sleep(Duration::from_millis(60));
		let rate2 = tracker.update(1024 * 400); // 400KB total

		// Average rate should be smoothed (not jumping wildly)
		let avg = tracker.avg_rate();
		assert!(avg > 0.0, "Average rate should be positive");

		// The smoothed average should be between the current rates
		// (exponential smoothing prevents extreme jumps)
	}

	#[test]
	fn test_copy_progress_to_generic() {
		let progress = CopyProgress {
			phase: CopyPhase::Copying,
			current_file: "test.txt".to_string(),
			current_source_path: None,
			files_copied: 5,
			total_files: 10,
			bytes_copied: 1024 * 1024 * 50, // 50MB
			total_bytes: 1024 * 1024 * 100, // 100MB
			current_operation: "Copying files".to_string(),
			estimated_remaining: Some(Duration::from_secs(30)),
			preparation_complete: true,
			error_count: 0,
			transfer_rate: 10.0 * 1024.0 * 1024.0, // 10 MB/s
			elapsed: Some(Duration::from_secs(5)),
			strategy_metadata: None,
		};

		let generic = progress.to_generic_progress();

		assert_eq!(generic.percentage, 0.5); // 50%
		assert_eq!(generic.phase, "Copying");
		assert_eq!(generic.completion.completed, 5);
		assert_eq!(generic.completion.total, 10);
		assert_eq!(generic.completion.bytes_completed, Some(1024 * 1024 * 50));
		assert_eq!(generic.performance.rate, 10.0 * 1024.0 * 1024.0);
		assert_eq!(
			generic.performance.estimated_remaining,
			Some(Duration::from_secs(30))
		);
	}

	#[test]
	fn test_copy_progress_with_strategy_metadata() {
		use crate::ops::files::copy::input::CopyMethod;
		use crate::ops::files::copy::routing::CopyStrategyMetadata;

		let metadata = CopyStrategyMetadata {
			strategy_name: "FastCopy".to_string(),
			strategy_description: "Fast copy (APFS clone)".to_string(),
			is_cross_device: false,
			is_cross_volume: false,
			is_fast_operation: true,
			copy_method: CopyMethod::Auto,
		};

		let progress = CopyProgress {
			phase: CopyPhase::Copying,
			current_file: "test.txt".to_string(),
			current_source_path: None,
			files_copied: 1,
			total_files: 5,
			bytes_copied: 1024,
			total_bytes: 5120,
			current_operation: "Fast copy (APFS clone)".to_string(),
			estimated_remaining: Some(Duration::from_secs(10)),
			preparation_complete: true,
			error_count: 0,
			transfer_rate: 512.0,
			elapsed: Some(Duration::from_secs(2)),
			strategy_metadata: Some(metadata.clone()),
		};

		assert!(progress.strategy_metadata.is_some());
		let meta = progress.strategy_metadata.unwrap();
		assert_eq!(meta.strategy_name, "FastCopy");
		assert!(meta.is_fast_operation);
		assert!(!meta.is_cross_device);
	}
}
