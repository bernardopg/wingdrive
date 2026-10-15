//! A tracked volume that detection lost is marked offline for this device only.

use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use tempfile::TempDir;
use uuid::Uuid;
use wing_core::{infra::db::entities, volume::VolumeFingerprint, Core};

async fn insert_volume(
	db: &sea_orm::DatabaseConnection,
	device_id: Uuid,
	fingerprint: &str,
) -> anyhow::Result<()> {
	let now = chrono::Utc::now();
	entities::volume::ActiveModel {
		uuid: Set(Uuid::new_v4()),
		device_id: Set(device_id),
		fingerprint: Set(fingerprint.into()),
		tracked_at: Set(now),
		last_seen_at: Set(now),
		is_online: Set(true),
		..Default::default()
	}
	.insert(db)
	.await?;
	Ok(())
}

async fn is_online(
	db: &sea_orm::DatabaseConnection,
	device_id: Uuid,
	fingerprint: &str,
) -> anyhow::Result<bool> {
	Ok(entities::volume::Entity::find()
		.filter(entities::volume::Column::DeviceId.eq(device_id))
		.filter(entities::volume::Column::Fingerprint.eq(fingerprint))
		.one(db)
		.await?
		.expect("volume row")
		.is_online)
}

#[tokio::test]
async fn marks_only_this_devices_row_offline() -> anyhow::Result<()> {
	let temp_dir = TempDir::new()?;
	let core = Core::new(temp_dir.path().to_path_buf()).await.unwrap();
	let library = core.libraries.list().await[0].clone();
	let db = library.db().conn();
	let this_device = core.device.device_id()?;
	// A paired device holding a drive with the same fingerprint, synced into this library.
	let peer_device = Uuid::new_v4();
	let mut peer: entities::device::ActiveModel = entities::device::Entity::find()
		.filter(entities::device::Column::Uuid.eq(this_device))
		.one(db)
		.await?
		.expect("this device's row")
		.into();
	peer.id = sea_orm::NotSet;
	peer.uuid = Set(peer_device);
	peer.slug = Set(format!("peer-{peer_device}"));
	peer.insert(db).await?;

	insert_volume(db, this_device, "usb").await?;
	insert_volume(db, this_device, "internal").await?;
	insert_volume(db, peer_device, "usb").await?;

	core.volumes
		.mark_tracked_volume_offline(&library, &VolumeFingerprint("usb".into()))
		.await?;

	assert!(!is_online(db, this_device, "usb").await?);
	assert!(is_online(db, this_device, "internal").await?);
	assert!(
		is_online(db, peer_device, "usb").await?,
		"a peer's row is left alone"
	);

	core.shutdown().await.unwrap();
	Ok(())
}
