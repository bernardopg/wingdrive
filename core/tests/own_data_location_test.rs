//! CORE-021: a location inside the daemon's own data folder used to finish its
//! discovery with zero entries and no error, because the walk drops the data
//! folder's contents. The indexer now refuses that root with an error, and a
//! location next to the data folder indexes normally.
//!
//! This file holds one test so its Core is the first in the process: the own
//! data folder is recorded once per process, by the first Core.

use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter};
use std::{path::Path, sync::Arc, time::Duration};
use tempfile::TempDir;
use wing_core::{
	domain::WingPath,
	infra::{action::LibraryAction, db::entities, job::types::JobStatus},
	library::Library,
	ops::locations::add::action::{LocationAddAction, LocationAddInput},
	Core,
};

type TestResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

async fn add_location(core: &Core, library: &Arc<Library>, slug: &str, path: &Path) -> TestResult {
	let action = LocationAddAction::from_input(LocationAddInput {
		path: WingPath::Physical {
			device_slug: slug.to_string(),
			path: path.to_path_buf(),
		},
		name: None,
		mode: wing_core::ops::indexing::IndexMode::Deep,
		job_policies: None,
	})
	.map_err(|e| format!("invalid input: {e}"))?;
	action
		.execute(library.clone(), core.context.clone())
		.await
		.map_err(|e| format!("location add failed: {e:?}"))?;
	Ok(())
}

/// Waits until the jobs with `status` grow past `known`, and returns them.
async fn wait_for_jobs(
	library: &Arc<Library>,
	status: JobStatus,
	known: usize,
) -> Result<Vec<wing_core::infra::job::types::JobInfo>, Box<dyn std::error::Error + Send + Sync>> {
	let deadline = tokio::time::Instant::now() + Duration::from_secs(30);
	loop {
		let jobs = library.jobs().list_jobs(Some(status)).await?;
		if jobs.len() > known {
			return Ok(jobs);
		}
		if tokio::time::Instant::now() > deadline {
			return Err(format!("no new {status:?} job within 30 s").into());
		}
		tokio::time::sleep(Duration::from_millis(100)).await;
	}
}

#[tokio::test]
async fn a_location_inside_the_data_folder_fails_and_one_beside_it_indexes() -> TestResult {
	let temp_dir = TempDir::new()?;
	let data_dir = temp_dir.path().join("data");
	let core = Core::new(data_dir.clone()).await?;

	let inside = data_dir.join("inside");
	let beside = temp_dir.path().join("beside");
	for dir in [&inside, &beside] {
		std::fs::create_dir_all(dir.join("sub"))?;
		std::fs::write(dir.join("a.txt"), "a")?;
		std::fs::write(dir.join("sub/b.txt"), "b")?;
	}

	let library = core
		.libraries
		.create_library("Own data", None, core.context.clone())
		.await?;
	let device = core.device.to_device()?;
	let slug = device.slug.clone();
	if entities::device::Entity::find()
		.filter(entities::device::Column::Uuid.eq(device.id))
		.one(library.db().conn())
		.await?
		.is_none()
	{
		let model: entities::device::ActiveModel = device.into();
		model.insert(library.db().conn()).await?;
	}

	add_location(&core, &library, &slug, &inside).await?;
	let failed = wait_for_jobs(&library, JobStatus::Failed, 0).await?;
	let message = failed[0].error_message.clone().unwrap_or_default();
	assert!(
		message.contains("own data folder"),
		"unexpected failure: {message}"
	);

	let completed_before = library
		.jobs()
		.list_jobs(Some(JobStatus::Completed))
		.await?
		.len();
	add_location(&core, &library, &slug, &beside).await?;
	wait_for_jobs(&library, JobStatus::Completed, completed_before).await?;
	let entries = entities::entry::Entity::find()
		.count(library.db().conn())
		.await?;
	// The location root, `a.txt`, `sub` and `sub/b.txt`.
	assert!(entries >= 4, "only {entries} entries were indexed");

	core.shutdown().await?;
	Ok(())
}
