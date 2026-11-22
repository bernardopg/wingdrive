//! Backfill + Live Events Race Condition Test
//!
//! Tests the hypothesis that live events arriving during backfill can advance
//! watermarks past unbackfilled entries, causing permanent data loss.
//!
//! Scenario:
//! 1. Alice indexes a location (creates entries)
//! 2. Bob starts backfilling from Alice
//! 3. WHILE backfill is in progress, Alice indexes ANOTHER location
//! 4. Live events from Alice arrive at Bob (buffered during backfill)
//! 5. After backfill completes, buffered events are processed
//! 6. If watermarks advance incorrectly, some entries will be permanently skipped
//!
//! Expected: Bob should have ALL entries from BOTH locations
//! Bug hypothesis: Bob will be missing entries from location 1

mod helpers;

use helpers::{
	add_and_index_location, init_test_tracing, register_device, MockTransport, TestConfigBuilder,
	TestDataDir,
};
use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QuerySelect};
use std::{collections::BTreeSet, path::PathBuf, sync::Arc};
use tokio::{fs, time::Duration};
use uuid::Uuid;
use wing_core::{
	infra::{
		db::entities,
		sync::{ChangeType, NetworkTransport, SharedChangeEntry, SystemTimeSource, HLC},
	},
	library::Library,
	service::{sync::state::DeviceSyncState, Service},
	Core,
};

/// Test harness for backfill race condition testing
struct BackfillRaceHarness {
	test_data_alice: TestDataDir,
	_test_data_bob: TestDataDir,
	core_alice: Core,
	core_bob: Core,
	library_alice: Arc<Library>,
	library_bob: Arc<Library>,
	device_alice_id: Uuid,
	device_bob_id: Uuid,
	transport_alice: Arc<MockTransport>,
	transport_bob: Arc<MockTransport>,
	snapshot_dir: PathBuf,
}

impl BackfillRaceHarness {
	/// Create test harness - Bob will need to backfill (not set to Ready)
	async fn new(test_name: &str) -> anyhow::Result<Self> {
		// Use TestDataDir helper for proper cross-platform directory management.
		// TestDataDir paths are deterministic on Linux, so two tests in this binary
		// running in parallel would otherwise share one persisted device_id and
		// sync.db, corrupting each other's backfill state.
		let test_data_alice = TestDataDir::new(format!("{test_name}_alice"))?;
		let test_data_bob = TestDataDir::new(format!("{test_name}_bob"))?;

		let snapshot_dir = test_data_alice.logs_path();
		init_test_tracing(test_name, &snapshot_dir)?;
		let temp_dir_alice = test_data_alice.core_data_path();
		let temp_dir_bob = test_data_bob.core_data_path();

		tracing::info!(
			snapshot_dir = %snapshot_dir.display(),
			alice_dir = %temp_dir_alice.display(),
			bob_dir = %temp_dir_bob.display(),
			"Starting backfill race condition test"
		);

		TestConfigBuilder::new(temp_dir_alice.clone()).build()?;
		TestConfigBuilder::new(temp_dir_bob.clone()).build()?;

		let core_alice = Core::new(temp_dir_alice.clone())
			.await
			.map_err(|e| anyhow::anyhow!("Failed to create Alice core: {}", e))?;
		let device_alice_id = core_alice.device.device_id()?;

		let core_bob = Core::new(temp_dir_bob.clone())
			.await
			.map_err(|e| anyhow::anyhow!("Failed to create Bob core: {}", e))?;
		let device_bob_id = core_bob.device.device_id()?;

		// Initialize volume managers
		tracing::info!("Initializing volume managers");
		core_alice.volumes.initialize().await?;
		core_bob.volumes.initialize().await?;

		let library_alice = core_alice
			.libraries
			.create_library_no_sync("Backfill Race Test", None, core_alice.context.clone())
			.await?;

		let library_bob = core_bob
			.libraries
			.create_library_no_sync("Backfill Race Test", None, core_bob.context.clone())
			.await?;

		register_device(&library_alice, device_bob_id, "Bob").await?;
		register_device(&library_bob, device_alice_id, "Alice").await?;

		tracing::info!(
			alice_device = %device_alice_id,
			bob_device = %device_bob_id,
			"Devices registered - Alice synced, Bob needs backfill"
		);

		let (transport_alice, transport_bob) =
			MockTransport::new_pair(device_alice_id, device_bob_id);

		// CRITICAL: Block Bob from receiving messages BEFORE starting services
		transport_alice.block_device(device_bob_id).await;
		transport_bob.block_device(device_bob_id).await;

		tracing::info!("Bob blocked from receiving messages");

		library_alice
			.init_sync_service(
				device_alice_id,
				transport_alice.clone() as Arc<dyn NetworkTransport>,
			)
			.await?;

		library_bob
			.init_sync_service(
				device_bob_id,
				transport_bob.clone() as Arc<dyn NetworkTransport>,
			)
			.await?;

		transport_alice
			.register_sync_service(
				device_alice_id,
				Arc::downgrade(library_alice.sync_service().unwrap()),
			)
			.await;
		transport_bob
			.register_sync_service(
				device_bob_id,
				Arc::downgrade(library_bob.sync_service().unwrap()),
			)
			.await;

		library_alice.sync_service().unwrap().start().await?;
		library_bob.sync_service().unwrap().start().await?;

		// Set Alice to Ready
		library_alice
			.sync_service()
			.unwrap()
			.peer_sync()
			.set_state_for_test(DeviceSyncState::Ready)
			.await;

		let bob_state = library_bob
			.sync_service()
			.unwrap()
			.peer_sync()
			.state()
			.await;

		tracing::info!(
			bob_state = ?bob_state,
			"Initial states set - Alice ready, Bob {:?} (BLOCKED)",
			bob_state
		);

		Ok(Self {
			test_data_alice,
			_test_data_bob: test_data_bob,
			core_alice,
			core_bob,
			library_alice,
			library_bob,
			device_alice_id,
			device_bob_id,
			transport_alice,
			transport_bob,
			snapshot_dir,
		})
	}

	/// Trigger backfill on Bob (also unblocks Bob)
	async fn trigger_bob_backfill(&self) -> anyhow::Result<()> {
		let sync_service = self.library_bob.sync_service().unwrap();
		let peer_sync = sync_service.peer_sync();

		// Unblock Bob
		tracing::info!("Unblocking Bob for backfill");
		self.transport_alice
			.unblock_device(self.device_bob_id)
			.await;
		self.transport_bob.unblock_device(self.device_bob_id).await;

		tracing::info!("Triggering backfill on Bob");

		peer_sync
			.set_state_for_test(DeviceSyncState::Backfilling {
				peer: self.device_alice_id,
				progress: 0,
			})
			.await;

		let backfill_manager = sync_service.backfill_manager();

		let peer_info = wing_core::service::sync::state::PeerInfo {
			device_id: self.device_alice_id,
			is_online: true,
			latency_ms: 1.0,
			has_complete_state: true,
			active_syncs: 0,
		};

		backfill_manager
			.start_backfill(vec![peer_info])
			.await
			.map_err(|e| anyhow::anyhow!("Backfill failed: {}", e))?;

		Ok(())
	}

	async fn fixture(&self, name: &str) -> anyhow::Result<PathBuf> {
		let path = self.test_data_alice.locations_path().join(name);
		fs::create_dir(&path).await?;
		for number in 0..64 {
			let stem = format!("coverage-{name}-{number:03}");
			fs::write(path.join(format!("{stem}.txt")), stem).await?;
		}
		Ok(path)
	}

	async fn wait_for_sync(&self) -> anyhow::Result<()> {
		let deadline = tokio::time::Instant::now() + Duration::from_secs(30);
		let expected = ["first", "second"]
			.into_iter()
			.flat_map(|name| (0..64).map(move |number| format!("coverage-{name}-{number:03}")))
			.collect::<BTreeSet<_>>();
		loop {
			let mut complete = true;
			for library in [&self.library_alice, &self.library_bob] {
				let names = entities::entry::Entity::find()
					.filter(entities::entry::Column::Name.starts_with("coverage-"))
					.select_only()
					.column(entities::entry::Column::Name)
					.into_tuple::<String>()
					.all(library.db().conn())
					.await?;
				let count = names.len();
				let received = names.into_iter().collect::<BTreeSet<_>>();
				if received != expected || count != expected.len() {
					complete = false;
					anyhow::ensure!(
						tokio::time::Instant::now() < deadline,
						"Peer library {} received {count} fixture rows, {} distinct, expected {}",
						library.id(),
						received.len(),
						expected.len()
					);
				}
			}
			if complete {
				return Ok(());
			}
			tokio::time::sleep(Duration::from_millis(100)).await;
		}
	}

	async fn shutdown(&self) -> anyhow::Result<()> {
		self.core_bob.shutdown().await.map_err(anyhow::Error::msg)?;
		self.core_alice
			.shutdown()
			.await
			.map_err(anyhow::Error::msg)?;
		Ok(())
	}

	/// Capture test snapshot
	async fn capture_snapshot(&self, name: &str) -> anyhow::Result<()> {
		let snapshot_path = self.snapshot_dir.join(name);
		fs::create_dir_all(&snapshot_path).await?;

		let alice_dir = snapshot_path.join("alice");
		let bob_dir = snapshot_path.join("bob");
		fs::create_dir_all(&alice_dir).await?;
		fs::create_dir_all(&bob_dir).await?;

		// Copy databases
		let alice_db = self.library_alice.path().join("database.db");
		let bob_db = self.library_bob.path().join("database.db");

		if alice_db.exists() {
			fs::copy(&alice_db, alice_dir.join("database.db")).await?;
		}
		if bob_db.exists() {
			fs::copy(&bob_db, bob_dir.join("database.db")).await?;
		}

		// Write summary
		let entries_alice = entities::entry::Entity::find()
			.count(self.library_alice.db().conn())
			.await?;
		let entries_bob = entities::entry::Entity::find()
			.count(self.library_bob.db().conn())
			.await?;

		let locations_alice = entities::location::Entity::find()
			.count(self.library_alice.db().conn())
			.await?;
		let locations_bob = entities::location::Entity::find()
			.count(self.library_bob.db().conn())
			.await?;

		let summary = format!(
			r#"# Backfill Race Test Snapshot: {}

## Alice (Device {})
- Locations: {}
- Entries: {}

## Bob (Device {})
- Locations: {}
- Entries: {}

## Diff
- Entry difference: {} (Alice - Bob)
"#,
			name,
			self.device_alice_id,
			locations_alice,
			entries_alice,
			self.device_bob_id,
			locations_bob,
			entries_bob,
			entries_alice as i64 - entries_bob as i64
		);

		fs::write(snapshot_path.join("summary.md"), summary).await?;

		Ok(())
	}
}

async fn wait_for_device(
	library: &Arc<Library>,
	device_id: Uuid,
	timeout: Duration,
) -> anyhow::Result<entities::device::Model> {
	let deadline = std::time::Instant::now() + timeout;

	while std::time::Instant::now() < deadline {
		if let Some(device) = entities::device::Entity::find()
			.filter(entities::device::Column::Uuid.eq(device_id))
			.one(library.db().conn())
			.await?
		{
			return Ok(device);
		}

		tokio::time::sleep(Duration::from_millis(100)).await;
	}

	anyhow::bail!("device {device_id} did not arrive through backfill")
}

/// A device state snapshot must not abort when its slug is already held locally.
#[tokio::test]
async fn test_backfill_renames_colliding_device_slug() -> anyhow::Result<()> {
	let harness = BackfillRaceHarness::new("device_slug_collision").await?;
	let incoming_device = Uuid::new_v4();
	let local_holder = Uuid::new_v4();

	register_device(&harness.library_alice, incoming_device, "shared-host").await?;
	register_device(&harness.library_bob, local_holder, "shared-host").await?;

	harness.trigger_bob_backfill().await?;

	let synced = wait_for_device(
		&harness.library_bob,
		incoming_device,
		Duration::from_secs(30),
	)
	.await?;
	let holder = entities::device::Entity::find()
		.filter(entities::device::Column::Uuid.eq(local_holder))
		.one(harness.library_bob.db().conn())
		.await?
		.expect("local device holding the slug must remain");

	assert_eq!(holder.slug, "shared-host");
	assert_eq!(synced.slug, "shared-host-2");

	harness.shutdown().await?;
	Ok(())
}

/// Test: Backfill + concurrent indexing race condition
#[tokio::test]
async fn test_backfill_with_concurrent_indexing() -> anyhow::Result<()> {
	let harness = BackfillRaceHarness::new("backfill_race").await?;

	let first_path = harness.fixture("first").await?;
	let second_path = harness.fixture("second").await?;
	add_and_index_location(
		&harness.library_alice,
		&harness.core_alice.volumes,
		first_path.to_str().unwrap(),
		"first",
	)
	.await?;

	let alice_entries_after_loc1 = entities::entry::Entity::find()
		.count(harness.library_alice.db().conn())
		.await?;

	tracing::info!(
		entries = alice_entries_after_loc1,
		"Alice has entries after first location"
	);

	let bob_entries_before = entities::entry::Entity::find()
		.count(harness.library_bob.db().conn())
		.await?;

	tracing::info!(
		bob_entries_before = bob_entries_before,
		alice_entries_after_loc1 = alice_entries_after_loc1,
		"Entry counts before concurrent phase"
	);

	assert!(
		bob_entries_before < 10,
		"Bob should have almost no entries initially (has {})",
		bob_entries_before
	);

	// Step 2: Start backfill on Bob while Alice continues indexing
	tracing::info!("Step 2: Starting Bob's backfill AND Alice's second indexing concurrently");

	let backfill_future = harness.trigger_bob_backfill();
	let indexing_future = add_and_index_location(
		&harness.library_alice,
		&harness.core_alice.volumes,
		second_path.to_str().unwrap(),
		"second",
	);

	tokio::try_join!(backfill_future, indexing_future)?;

	// Step 3: Wait for everything to stabilize
	tracing::info!("Step 3: Waiting for sync to stabilize");
	harness.wait_for_sync().await?;

	// Capture snapshot
	harness.capture_snapshot("final_state").await?;

	// Step 4: Compare results
	let entries_alice = entities::entry::Entity::find()
		.count(harness.library_alice.db().conn())
		.await?;
	let entries_bob = entities::entry::Entity::find()
		.count(harness.library_bob.db().conn())
		.await?;

	let locations_alice = entities::location::Entity::find()
		.count(harness.library_alice.db().conn())
		.await?;
	let locations_bob = entities::location::Entity::find()
		.count(harness.library_bob.db().conn())
		.await?;

	tracing::info!(
		alice_entries = entries_alice,
		bob_entries = entries_bob,
		alice_locations = locations_alice,
		bob_locations = locations_bob,
		"Final counts"
	);

	let diff = (entries_alice as i64 - entries_bob as i64).abs();

	if diff > 5 {
		tracing::error!(
			diff = diff,
			alice = entries_alice,
			bob = entries_bob,
			"RACE CONDITION CONFIRMED: Bob is missing {} entries!",
			diff
		);
	}

	assert!(
		diff <= 5,
		"Entry count mismatch: Alice has {}, Bob has {} (diff: {}). \
		This suggests watermarks advanced past unbackfilled entries.",
		entries_alice,
		entries_bob,
		diff
	);

	harness.shutdown().await?;
	Ok(())
}

/// Test: Sequential indexing (control - should always pass)
#[tokio::test]
async fn test_sequential_backfill_control() -> anyhow::Result<()> {
	let harness = BackfillRaceHarness::new("sequential_control").await?;

	let first_path = harness.fixture("first").await?;
	let second_path = harness.fixture("second").await?;

	tracing::info!("Indexing both locations on Alice first");

	add_and_index_location(
		&harness.library_alice,
		&harness.core_alice.volumes,
		first_path.to_str().unwrap(),
		"core",
	)
	.await?;
	add_and_index_location(
		&harness.library_alice,
		&harness.core_alice.volumes,
		second_path.to_str().unwrap(),
		"apps",
	)
	.await?;

	let alice_entries = entities::entry::Entity::find()
		.count(harness.library_alice.db().conn())
		.await?;

	tracing::info!(entries = alice_entries, "Alice has all entries");

	// Now trigger backfill on Bob
	tracing::info!("Starting Bob's backfill (no concurrent indexing)");
	harness.trigger_bob_backfill().await?;

	// Wait for sync
	harness.wait_for_sync().await?;

	// Capture snapshot
	harness.capture_snapshot("final_state").await?;

	// Compare
	let entries_alice = entities::entry::Entity::find()
		.count(harness.library_alice.db().conn())
		.await?;
	let entries_bob = entities::entry::Entity::find()
		.count(harness.library_bob.db().conn())
		.await?;

	tracing::info!(
		alice = entries_alice,
		bob = entries_bob,
		"Final counts (sequential)"
	);

	let diff = (entries_alice as i64 - entries_bob as i64).abs();
	assert!(
		diff <= 5,
		"Sequential backfill should result in similar counts: Alice {}, Bob {} (diff: {})",
		entries_alice,
		entries_bob,
		diff
	);

	harness.shutdown().await?;
	Ok(())
}

/// Changes received during an incremental catch-up are buffered, and the
/// catch-up then returns to Ready without going through
/// `transition_to_ready`, which is the only other place the buffer is
/// drained. Ending the catch-up must apply them, or they are lost.
#[tokio::test]
async fn test_changes_buffered_during_catch_up_are_applied() -> anyhow::Result<()> {
	let test_data = TestDataDir::new("catch_up_buffer")?;
	let dir = test_data.core_data_path();
	TestConfigBuilder::new(dir.clone()).build()?;

	let core = Core::new(dir)
		.await
		.map_err(|e| anyhow::anyhow!("Failed to create core: {}", e))?;
	let device_id = core.device.device_id()?;
	let library = core
		.libraries
		.create_library_no_sync("Catch Up Buffer", None, core.context.clone())
		.await?;
	library
		.init_sync_service(
			device_id,
			MockTransport::new_single(device_id) as Arc<dyn NetworkTransport>,
		)
		.await?;
	let peer_sync = library.sync_service().unwrap().peer_sync();

	let peer = Uuid::new_v4();
	peer_sync
		.set_state_for_test(DeviceSyncState::CatchingUp { buffered_count: 0 })
		.await;

	let tag_uuid = Uuid::new_v4();
	let now = chrono::Utc::now();
	peer_sync
		.on_shared_change_received(SharedChangeEntry {
			hlc: HLC::now(peer, &SystemTimeSource),
			model_type: "tag".to_string(),
			record_uuid: tag_uuid,
			change_type: ChangeType::Insert,
			data: serde_json::json!({
				"uuid": tag_uuid,
				"canonical_name": "from-peer",
				"display_name": "From peer",
				"tag_type": "Standard",
				"privacy_level": "Normal",
				"is_organizational_anchor": false,
				"search_weight": 0,
				"icon": "Planet",
				"color": "#000000",
				"created_at": now,
				"updated_at": now,
			}),
		})
		.await?;

	let tag_count = || async {
		entities::tag::Entity::find()
			.filter(entities::tag::Column::Uuid.eq(tag_uuid))
			.count(library.db().conn())
			.await
	};
	assert_eq!(
		tag_count().await?,
		0,
		"the change must be buffered while catching up"
	);

	peer_sync.finish_catch_up(peer, "catchup_completed").await;

	assert!(peer_sync.state().await.is_ready());
	assert_eq!(
		tag_count().await?,
		1,
		"the change buffered during catch-up was never applied"
	);

	Ok(())
}

#[tokio::test]
async fn idle_peer_without_locations_does_not_catch_up_each_tick() -> anyhow::Result<()> {
	use wing_core::service::network::protocol::sync::messages::SyncMessage;

	let harness = BackfillRaceHarness::new("catchup_empty_peer").await?;
	let location = harness.fixture("first").await?;
	add_and_index_location(
		&harness.library_alice,
		&harness.core_alice.volumes,
		location.to_str().unwrap(),
		"first",
	)
	.await?;
	harness.trigger_bob_backfill().await?;

	let alice = harness.library_alice.sync_service().unwrap().peer_sync();
	let bob_id = entities::device::Entity::find()
		.filter(entities::device::Column::Uuid.eq(harness.device_bob_id))
		.one(harness.library_bob.db().conn())
		.await?
		.unwrap()
		.id;
	assert_eq!(
		entities::location::Entity::find()
			.filter(entities::location::Column::DeviceId.eq(bob_id))
			.count(harness.library_bob.db().conn())
			.await?,
		0
	);
	let catchup_requests = || async {
		harness
			.transport_alice
			.get_messages_between(harness.device_alice_id, harness.device_bob_id)
			.await
			.into_iter()
			.filter(|request| matches!(request, SyncMessage::SharedChangeRequest { .. }))
			.count()
	};
	let first = tokio::time::timeout(Duration::from_secs(85), async {
		loop {
			let count = catchup_requests().await;
			if count > 0 && alice.state().await.is_ready() {
				break count;
			}
			tokio::time::sleep(Duration::from_millis(100)).await;
		}
	})
	.await?;
	// Three default poll intervals fit inside the per-peer catch-up cooldown.
	tokio::time::sleep(Duration::from_secs(16)).await;
	assert_eq!(
		catchup_requests().await,
		first,
		"idle peer restarted catch-up"
	);
	harness.shutdown().await?;
	Ok(())
}

#[tokio::test]
async fn offline_updates_to_the_same_tag_converge_on_the_later_hlc() -> anyhow::Result<()> {
	use sea_orm::{ActiveModelTrait, Set};
	use wing_core::{ops::tags::manager::TagManager, service::Service};

	let harness = BackfillRaceHarness::new("offline_same_tag").await?;
	let alice = &harness.library_alice;
	let bob = &harness.library_bob;
	let manager = TagManager::new(Arc::new(alice.db().conn().clone()));
	let tag = manager
		.create_tag_entity("offline-edit".into(), None, harness.device_alice_id)
		.await?;
	alice.sync_model(&tag, ChangeType::Insert).await?;
	harness.trigger_bob_backfill().await?;
	let sync_a = alice.sync_service().unwrap();
	let sync_b = bob.sync_service().unwrap();
	let bob_tag = entities::tag::Entity::find()
		.filter(entities::tag::Column::Uuid.eq(tag.uuid))
		.one(bob.db().conn())
		.await?
		.expect("backfill replicated the tag");

	// Stop polling and drop broadcasts while each profile edits its own copy.
	sync_a.stop().await?;
	sync_b.stop().await?;
	harness
		.transport_alice
		.block_device(harness.device_alice_id)
		.await;
	harness
		.transport_bob
		.block_device(harness.device_bob_id)
		.await;
	let watermark_a = sync_a.peer_sync().get_watermarks().await.1;
	let watermark_b = sync_b.peer_sync().get_watermarks().await.1;

	for (library, tag, name) in [(alice, tag.clone(), "Alice"), (bob, bob_tag, "Bob")] {
		let mut active: entities::tag::ActiveModel = tag.into();
		active.display_name = Set(Some(name.to_owned()));
		active.updated_at = Set(chrono::Utc::now());
		let updated = active.update(library.db().conn()).await?;
		library.sync_model(&updated, ChangeType::Update).await?;
	}
	let hlc_a = sync_a
		.peer_sync()
		.peer_log()
		.get_latest_hlc_for_record(tag.uuid)
		.await?
		.unwrap();
	let hlc_b = sync_b
		.peer_sync()
		.peer_log()
		.get_latest_hlc_for_record(tag.uuid)
		.await?
		.unwrap();
	assert_ne!(hlc_a, hlc_b);
	let winner = if hlc_a > hlc_b { "Alice" } else { "Bob" };

	harness
		.transport_alice
		.unblock_device(harness.device_alice_id)
		.await;
	harness
		.transport_bob
		.unblock_device(harness.device_bob_id)
		.await;

	sync_a
		.backfill_manager()
		.catch_up_from_peer(
			harness.device_bob_id,
			None,
			watermark_a.map(|hlc| hlc.to_string()),
		)
		.await?;
	sync_b
		.backfill_manager()
		.catch_up_from_peer(
			harness.device_alice_id,
			None,
			watermark_b.map(|hlc| hlc.to_string()),
		)
		.await?;
	for library in [alice, bob] {
		let after = entities::tag::Entity::find()
			.filter(entities::tag::Column::Uuid.eq(tag.uuid))
			.one(library.db().conn())
			.await?
			.unwrap();
		assert_eq!(after.display_name.as_deref(), Some(winner));
	}
	harness.shutdown().await?;
	Ok(())
}
