//! Test for job pausing during shutdown

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use std::time::Duration;
use tempfile::TempDir;
use tokio::time::sleep;
use wing_core::{
	infra::db::entities,
	infra::job::{
		database::{init_database, JobDb},
		types::{JobId, JobStatus},
	},
	location::{create_location, IndexMode, LocationCreateArgs},
	Core,
};

#[tokio::test]
async fn test_jobs_paused_on_shutdown() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
	// Setup test environment
	let temp_dir = TempDir::new()?;
	let core_dir = temp_dir.path().join("core");
	tokio::fs::create_dir_all(&core_dir).await?;

	let core = Core::new(core_dir).await?;

	// Create library
	let library = core
		.libraries
		.create_library("Test Shutdown Library", None, core.context.clone())
		.await?;

	// Create test location with many files to ensure job runs long enough
	let test_location_dir = temp_dir.path().join("test_location");
	tokio::fs::create_dir_all(&test_location_dir).await?;

	// Content identification hashes ~400 files/s in a debug build. 310 entries finished
	// within about a second, so the job could complete before shutdown and leave nothing
	// to pause; ~3000 keeps it running for several seconds.
	for i in 0..2000 {
		let file_path = test_location_dir.join(format!("test_file_{}.txt", i));
		tokio::fs::write(&file_path, format!("Test content {}", i)).await?;

		// Create some subdirectories with files
		if i % 200 == 0 {
			let subdir = test_location_dir.join(format!("subdir_{}", i));
			tokio::fs::create_dir_all(&subdir).await?;
			for j in 0..100 {
				let subfile = subdir.join(format!("subfile_{}.txt", j));
				tokio::fs::write(&subfile, format!("Subcontent {} {}", i, j)).await?;
			}
		}
	}

	// create_library already registers this device, so reuse that row; inserting it
	// again would violate the unique slug constraint.
	let device = core.device.to_device()?;
	let device_record = entities::device::Entity::find()
		.filter(entities::device::Column::Uuid.eq(device.id))
		.one(library.db().conn())
		.await?
		.ok_or("create_library should register the current device")?;

	// Create location to trigger indexing
	let location_args = LocationCreateArgs {
		path: test_location_dir.clone(),
		name: Some("Test Location".to_string()),
		index_mode: IndexMode::Deep,
	};

	create_location(
		library.clone(),
		&core.events,
		location_args,
		device_record.id,
	)
	.await?;

	// Wait for indexing to start
	sleep(Duration::from_millis(500)).await;

	// Verify we have running jobs
	let job_manager = library.jobs();
	let running_jobs = job_manager.list_jobs(Some(JobStatus::Running)).await?;
	assert!(
		!running_jobs.is_empty(),
		"Should have at least one running job"
	);

	let job_ids: Vec<JobId> = running_jobs.iter().map(|j| JobId(j.id)).collect();
	println!("Found {} running jobs before shutdown", job_ids.len());

	// Shutdown the core, which should pause all jobs
	println!("Shutting down core...");
	core.shutdown().await?;

	// Shutdown closes the library and its job database pool, so the live job manager
	// can no longer answer. Paused state only matters if it was persisted for resume,
	// so read it back from the library's jobs.db.
	let jobs_db = JobDb::new(init_database(&library.path().join("jobs.db")).await?);
	for job_id in &job_ids {
		let job = jobs_db
			.get_job(*job_id)
			.await?
			.ok_or_else(|| format!("Job {} missing from jobs.db", job_id.0))?;
		assert_eq!(
			job.status,
			JobStatus::Paused.to_string(),
			"Job {} should be persisted as paused after shutdown",
			job_id.0
		);
	}

	Ok(())
}

#[tokio::test]
async fn test_shutdown_with_no_running_jobs() -> Result<(), Box<dyn std::error::Error + Send + Sync>>
{
	// This test ensures shutdown works correctly when no jobs are running
	let temp_dir = TempDir::new()?;
	let core = Core::new(temp_dir.path().to_path_buf()).await?;

	let library = core
		.libraries
		.create_library("Empty Library", None, core.context.clone())
		.await?;

	// Verify no running jobs
	let job_manager = library.jobs();
	let running_jobs = job_manager.list_jobs(Some(JobStatus::Running)).await?;
	assert!(running_jobs.is_empty());

	// Shutdown should complete without errors
	core.shutdown().await?;
	println!("✓ Shutdown completed successfully with no running jobs");

	Ok(())
}
