//! Cloud sync job: runs one sync pair and stores its result.

use std::path::PathBuf;

use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::engine::{self, Manifest, SyncMode, SyncReport, SyncTarget};
use crate::infra::db::entities::cloud_sync_pair;
use crate::infra::job::prelude::*;

/// Runs a sync pair once.
///
/// Not resumable: the manifest already makes a rerun cheap, because files
/// synced before the interruption compare equal and are skipped.
#[derive(Debug, Serialize, Deserialize, Job)]
pub struct CloudSyncJob {
	pub pair_id: Uuid,
}

impl Job for CloudSyncJob {
	const NAME: &'static str = "cloud_sync";
	const RESUMABLE: bool = false;
	const DESCRIPTION: Option<&'static str> = Some("Sync a local folder with a cloud folder");
}

impl crate::infra::job::traits::DynJob for CloudSyncJob {
	fn job_name(&self) -> &'static str {
		Self::NAME
	}
}

impl From<SyncReport> for JobOutput {
	fn from(report: SyncReport) -> Self {
		JobOutput::Custom(serde_json::to_value(report).unwrap_or_default())
	}
}

impl CloudSyncJob {
	pub fn new(pair_id: Uuid) -> Self {
		Self { pair_id }
	}
}

#[async_trait::async_trait]
impl JobHandler for CloudSyncJob {
	type Output = SyncReport;

	async fn run(&mut self, ctx: JobContext<'_>) -> JobResult<Self::Output> {
		if !super::try_mark_running(self.pair_id) {
			return Err(JobError::execution("This sync pair is already running"));
		}
		let result = self.run_marked(&ctx).await;
		super::mark_finished(self.pair_id);
		result
	}
}

impl CloudSyncJob {
	async fn run_marked(&self, ctx: &JobContext<'_>) -> JobResult<SyncReport> {
		let db = ctx.library_db();
		let pair = cloud_sync_pair::Entity::find()
			.filter(cloud_sync_pair::Column::Uuid.eq(self.pair_id))
			.one(db)
			.await
			.map_err(|e| JobError::execution(e.to_string()))?
			.ok_or_else(|| JobError::execution("Sync pair not found"))?;
		ctx.log(format!("Syncing '{}'", pair.name));
		ctx.progress(Progress::Indeterminate(format!("Comparing {}", pair.name)));

		let context = ctx.library().core_context().clone();
		let remote = super::remote_path_of(&pair);
		let outcome = match super::resolve_remote(&context, &remote).await {
			Ok((backend, remote_root)) => {
				let manifest: Manifest = pair
					.manifest
					.as_deref()
					.and_then(|m| serde_json::from_str(m).ok())
					.unwrap_or_default();
				let target = SyncTarget {
					local_root: PathBuf::from(&pair.local_path),
					backend,
					remote_root,
					use_system_trash: true,
				};
				let progress = |done: usize, total: usize, path: &str| {
					if total > 0 {
						ctx.progress(Progress::Count {
							current: done,
							total,
						});
					}
					if !path.is_empty() {
						ctx.log_debug(format!("Syncing {path}"));
					}
				};
				engine::run(
					&target,
					SyncMode::parse(&pair.mode).unwrap_or(SyncMode::TwoWay),
					pair.propagate_deletes,
					&manifest,
					&progress,
				)
				.await
				.map_err(|e| e.to_string())
			}
			Err(error) => Err(error),
		};

		let mut active: cloud_sync_pair::ActiveModel = pair.clone().into();
		let now = chrono::Utc::now();
		active.last_run_at = Set(Some(now));
		active.updated_at = Set(now);
		let result = match outcome {
			Ok((report, manifest)) => {
				let status = if report.errors.is_empty() {
					"success"
				} else {
					"partial"
				};
				active.last_status = Set(Some(status.to_string()));
				active.last_error = Set(report.errors.first().cloned());
				active.last_report = Set(serde_json::to_string(&report).ok());
				active.manifest = Set(serde_json::to_string(&manifest).ok());
				for error in &report.errors {
					ctx.add_warning(error.clone());
				}
				ctx.log(format!(
					"Sync of '{}' done: {} uploaded, {} downloaded, {} deleted here, {} deleted in the cloud, {} conflicts",
					pair.name,
					report.uploaded,
					report.downloaded,
					report.deleted_local,
					report.deleted_remote,
					report.conflicts
				));
				Ok(report)
			}
			Err(error) => {
				active.last_status = Set(Some("failed".to_string()));
				active.last_error = Set(Some(error.clone()));
				Err(JobError::execution(error))
			}
		};
		if let Err(error) = active.update(db).await {
			ctx.add_warning(format!("Could not save the sync result: {error}"));
		}
		context.events.emit(crate::infra::event::Event::Custom {
			event_type: "cloud_sync_finished".to_string(),
			data: serde_json::json!({ "pair_id": self.pair_id }),
		});
		result
	}
}
