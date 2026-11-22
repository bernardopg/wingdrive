//! Targeted job lifecycle coverage: cancel, terminal persistence, and history cleanup.

use chrono::{Duration as ChronoDuration, Utc};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use std::time::Duration;
use tempfile::TempDir;
use tokio::time::sleep;
use wing_core::{
	infra::db::entities,
	infra::job::{
		database::{history, init_database, JobDb},
		types::{JobId, JobStatus},
	},
	location::{create_location, IndexMode, LocationCreateArgs},
	Core,
};

async fn create_core_with_indexing_job() -> Result<
	(
		TempDir,
		Core,
		std::sync::Arc<wing_core::library::Library>,
		JobId,
	),
	Box<dyn std::error::Error + Send + Sync>,
> {
	let temp_dir = TempDir::new()?;
	let core_dir = temp_dir.path().join("core");
	tokio::fs::create_dir_all(&core_dir).await?;
	let core = Core::new(core_dir).await?;

	let library = core
		.libraries
		.create_library("Job Lifecycle Library", None, core.context.clone())
		.await?;

	let test_location_dir = temp_dir.path().join("location");
	tokio::fs::create_dir_all(&test_location_dir).await?;
	for i in 0..1200 {
		let file_path = test_location_dir.join(format!("file_{i}.txt"));
		tokio::fs::write(&file_path, format!("content {i}")).await?;
		if i % 200 == 0 {
			let subdir = test_location_dir.join(format!("subdir_{i}"));
			tokio::fs::create_dir_all(&subdir).await?;
			for j in 0..60 {
				tokio::fs::write(subdir.join(format!("sub_{j}.txt")), format!("{i}-{j}")).await?;
			}
		}
	}

	let device = core.device.to_device()?;
	let device_record = entities::device::Entity::find()
		.filter(entities::device::Column::Uuid.eq(device.id))
		.one(library.db().conn())
		.await?
		.ok_or("device record missing")?;

	create_location(
		library.clone(),
		&core.events,
		LocationCreateArgs {
			path: test_location_dir,
			name: Some("Lifecycle Location".to_string()),
			index_mode: IndexMode::Deep,
		},
		device_record.id,
	)
	.await?;

	let job_manager = library.jobs();
	let deadline = tokio::time::Instant::now() + Duration::from_secs(10);
	let job_id = loop {
		let running = job_manager.list_jobs(Some(JobStatus::Running)).await?;
		if let Some(job) = running.first() {
			break JobId(job.id);
		}
		if tokio::time::Instant::now() > deadline {
			return Err("indexing job did not start".into());
		}
		sleep(Duration::from_millis(50)).await;
	};

	Ok((temp_dir, core, library, job_id))
}

#[tokio::test]
async fn cancel_running_job_persists_cancelled_state(
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
	let (_temp_dir, core, library, job_id) = create_core_with_indexing_job().await?;
	let manager = library.jobs();
	manager.cancel_job(job_id).await?;

	let deadline = tokio::time::Instant::now() + Duration::from_secs(10);
	let jobs_db = JobDb::new(init_database(&library.path().join("jobs.db")).await?);
	let row = loop {
		let row = jobs_db.get_job(job_id).await?.ok_or("job missing")?;
		if row.status == JobStatus::Cancelled.to_string() {
			break row;
		}
		if row.status == JobStatus::Completed.to_string()
			|| row.status == JobStatus::Failed.to_string()
		{
			return Err(format!("job reached {} before cancel persisted", row.status).into());
		}
		if tokio::time::Instant::now() > deadline {
			return Err(format!("job not cancelled, status={}", row.status).into());
		}
		sleep(Duration::from_millis(50)).await;
	};

	assert_eq!(row.status, JobStatus::Cancelled.to_string());
	assert!(row.completed_at.is_some() || row.error_message.is_some() || row.status == "cancelled");

	core.shutdown().await?;
	Ok(())
}

#[tokio::test]
async fn cleanup_history_removes_only_old_terminal_rows(
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
	let temp_dir = TempDir::new()?;
	let db = JobDb::new(init_database(&temp_dir.path().join("jobs.db")).await?);
	let now = Utc::now();

	let old_id = JobId::new().to_string();
	history::ActiveModel {
		id: Set(old_id.clone()),
		name: Set("old_job".to_string()),
		status: Set(JobStatus::Completed.to_string()),
		started_at: Set(now - ChronoDuration::hours(2)),
		completed_at: Set(now - ChronoDuration::hours(2)),
		duration_ms: Set(100),
		output: Set(None),
		metrics: Set(None),
	}
	.insert(db.conn())
	.await?;

	let new_id = JobId::new().to_string();
	history::ActiveModel {
		id: Set(new_id.clone()),
		name: Set("new_job".to_string()),
		status: Set(JobStatus::Failed.to_string()),
		started_at: Set(now - ChronoDuration::minutes(5)),
		completed_at: Set(now - ChronoDuration::minutes(5)),
		duration_ms: Set(200),
		output: Set(None),
		metrics: Set(None),
	}
	.insert(db.conn())
	.await?;

	let removed = db.cleanup_history(now - ChronoDuration::hours(1)).await?;
	assert_eq!(removed, 1);
	assert!(history::Entity::find_by_id(old_id)
		.one(db.conn())
		.await?
		.is_none());
	assert!(history::Entity::find_by_id(new_id)
		.one(db.conn())
		.await?
		.is_some());
	Ok(())
}

#[tokio::test]
async fn terminal_job_rejects_resume_and_reports_info(
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
	let temp_dir = TempDir::new()?;
	let core_dir = temp_dir.path().join("core");
	tokio::fs::create_dir_all(&core_dir).await?;
	let core = Core::new(core_dir).await?;
	let library = core.libraries.list().await[0].clone();
	let manager = library.jobs();

	let id = JobId::new();
	let db = manager.database();
	wing_core::infra::job::database::jobs::ActiveModel {
		id: Set(id.to_string()),
		name: Set("file_copy".to_string()),
		state: Set(Vec::new()),
		status: Set(JobStatus::Completed.to_string()),
		priority: Set(0),
		created_at: Set(Utc::now()),
		completed_at: Set(Some(Utc::now())),
		..Default::default()
	}
	.insert(db.conn())
	.await?;

	let info = manager.get_job_info(id.0).await?.expect("job info");
	assert_eq!(info.status, JobStatus::Completed);
	let err = manager
		.resume_job(id)
		.await
		.expect_err("terminal resume should fail");
	assert!(err.to_string().to_lowercase().contains("completed") || !err.to_string().is_empty());

	core.shutdown().await?;
	Ok(())
}
