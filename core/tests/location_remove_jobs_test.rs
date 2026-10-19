//! Removing a location stops the jobs that work on it
//!
//! A large location keeps its indexer and thumbnail jobs busy for minutes.
//! Before this test those jobs kept running after the location was removed,
//! and their writes held the SQLite lock the removal needed.

mod helpers;

use helpers::IndexingHarnessBuilder;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use tokio::fs;
use wing_core::{
	domain::addressing::WingPath,
	infra::db::entities,
	location::{IndexMode, LocationManager},
	ops::indexing::{self, IndexerJob, IndexerJobConfig},
};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn removing_a_location_cancels_its_jobs() -> anyhow::Result<()> {
	let harness = IndexingHarnessBuilder::new("location_remove_jobs")
		.disable_watcher()
		.build()
		.await?;

	let test_location = harness.create_test_location("Busy").await?;
	let location = test_location.index("Busy", IndexMode::Shallow).await?;

	// Enough files that a deep walk is still running when it is cancelled.
	for dir in 0..40 {
		let dir_path = location.path.join(format!("dir-{dir}"));
		fs::create_dir_all(&dir_path).await?;
		for file in 0..100 {
			fs::write(
				dir_path.join(format!("file-{file}.txt")),
				format!("{dir}-{file}"),
			)
			.await?;
		}
	}

	let job = IndexerJob::new(IndexerJobConfig::new(
		location.uuid,
		WingPath::local(location.path.clone()),
		indexing::IndexMode::Deep,
	));
	let handle = harness.library.jobs().dispatch(job).await?;

	let cancelled = harness
		.library
		.jobs()
		.cancel_location_jobs(location.uuid)
		.await;
	assert_eq!(cancelled, 1, "the location's indexer job was not cancelled");
	assert!(
		!handle.status().is_active(),
		"cancelled job still reports {:?}",
		handle.status()
	);

	// Jobs of other locations are left alone.
	assert_eq!(
		harness
			.library
			.jobs()
			.cancel_location_jobs(uuid::Uuid::new_v4())
			.await,
		0
	);

	LocationManager::new((*harness.core.events).clone())
		.remove_location(&harness.library, location.uuid)
		.await?;
	let remaining = entities::location::Entity::find()
		.filter(entities::location::Column::Uuid.eq(location.uuid))
		.one(harness.library.db().conn())
		.await?;
	assert!(remaining.is_none(), "location still in the database");

	harness.shutdown().await
}
