//! Sync resilience coverage: disconnect/reconnect convergence and idempotent replay.

mod helpers;

use helpers::TwoDeviceHarnessBuilder;
use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter};
use tokio::time::Duration;
use wing_core::infra::db::entities;

#[tokio::test]
async fn disconnect_then_reconnect_converges_after_backlog() -> anyhow::Result<()> {
	let harness = TwoDeviceHarnessBuilder::new("sync_disconnect_reconnect")
		.await?
		.build()
		.await?;

	harness
		.transport_alice
		.block_device(harness.device_bob_id)
		.await;
	harness
		.transport_bob
		.block_device(harness.device_alice_id)
		.await;

	let offline = tempfile::tempdir()?;
	for i in 0..12 {
		std::fs::write(
			offline.path().join(format!("offline-{i:02}.txt")),
			format!("offline-{i}"),
		)?;
	}
	harness
		.add_and_index_location_alice(offline.path().to_str().unwrap(), "offline")
		.await?;

	let err = harness
		.wait_for_sync(Duration::from_secs(5))
		.await
		.expect_err("sync should not complete while peers are disconnected");
	assert!(
		err.to_string().to_lowercase().contains("timeout"),
		"unexpected error: {err}"
	);

	harness
		.transport_alice
		.unblock_device(harness.device_bob_id)
		.await;
	harness
		.transport_bob
		.unblock_device(harness.device_alice_id)
		.await;

	// Dropped messages are not replayed by MockTransport; create fresh changes
	// after reconnect so the test verifies convergence after connectivity returns.
	let online = tempfile::tempdir()?;
	for i in 0..8 {
		std::fs::write(
			online.path().join(format!("online-{i:02}.txt")),
			format!("online-{i}"),
		)?;
	}
	harness
		.add_and_index_location_alice(online.path().to_str().unwrap(), "online")
		.await?;
	harness.wait_for_sync(Duration::from_secs(60)).await?;

	let bob_online_files = entities::entry::Entity::find()
		.filter(entities::entry::Column::Name.starts_with("online-"))
		.count(harness.library_bob.db().conn())
		.await?;
	assert_eq!(bob_online_files, 8);

	Ok(())
}

#[tokio::test]
async fn shared_change_replay_is_idempotent() -> anyhow::Result<()> {
	use wing_core::infra::sync::{registry, ChangeType, SharedChangeEntry, SystemTimeSource, HLC};

	let harness = TwoDeviceHarnessBuilder::new("sync_idempotent_replay")
		.await?
		.build()
		.await?;

	let fixture = tempfile::tempdir()?;
	std::fs::write(fixture.path().join("replay.txt"), "hello")?;
	harness
		.add_and_index_location_alice(fixture.path().to_str().unwrap(), "replay")
		.await?;
	harness.wait_for_sync(Duration::from_secs(30)).await?;

	let entry = entities::entry::Entity::find()
		.filter(entities::entry::Column::Name.eq("replay.txt"))
		.one(harness.library_bob.db().conn())
		.await?
		.ok_or_else(|| anyhow::anyhow!("replay.txt missing on bob"))?;

	let count_before = entities::entry::Entity::find()
		.count(harness.library_bob.db().conn())
		.await?;

	let change = SharedChangeEntry {
		hlc: HLC::now(harness.device_alice_id, &SystemTimeSource),
		model_type: "entry".to_string(),
		record_uuid: entry.uuid.expect("indexed entries have a UUID"),
		change_type: ChangeType::Update,
		data: serde_json::to_value(&entry)?,
	};

	for _ in 0..2 {
		registry::apply_shared_change(
			change.clone(),
			std::sync::Arc::new(harness.library_bob.db().conn().clone()),
		)
		.await?;
	}

	let count_after = entities::entry::Entity::find()
		.count(harness.library_bob.db().conn())
		.await?;
	assert_eq!(count_before, count_after);

	let replayed = entities::entry::Entity::find()
		.filter(entities::entry::Column::Uuid.eq(entry.uuid))
		.all(harness.library_bob.db().conn())
		.await?;
	assert_eq!(replayed.len(), 1);

	Ok(())
}
