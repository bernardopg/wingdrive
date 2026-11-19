//! Tests recovery and removal of sidecar metadata in a temporary library.

use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use tempfile::TempDir;
use uuid::Uuid;
use wing_core::{
	infra::db::entities::{content_identity, content_kind, sidecar_availability},
	ops::sidecar::{SidecarFormat, SidecarKind, SidecarStatus, SidecarVariant},
	service::sidecar_manager::SidecarResult,
	Core,
};

#[tokio::test]
async fn bootstrap_recovers_sidecars_and_removal_clears_local_availability(
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
	let root = TempDir::new()?;
	let core = Core::new(root.path().join("daemon")).await?;
	let library = core
		.libraries
		.create_library("Sidecar fixture", None, core.context.clone())
		.await?;
	let manager = &core.services.sidecar_manager;
	let db = library.db().conn();
	let content = Uuid::from_u128(0x1234_0000_0000_0000_0000_0000_0000_0001);
	let kind_row = content_kind::Entity::find().one(db).await?.unwrap();
	let now = chrono::Utc::now();
	content_identity::ActiveModel {
		uuid: Set(Some(content)),
		content_hash: Set("sidecar-fixture-content".into()),
		kind_id: Set(kind_row.id),
		total_size: Set(1),
		entry_count: Set(1),
		first_seen_at: Set(now),
		last_verified_at: Set(now),
		..Default::default()
	}
	.insert(db)
	.await?;

	let kind = SidecarKind::Thumb;
	let variant = SidecarVariant::new("grid@1x");
	let format = SidecarFormat::Webp;
	let path = manager
		.compute_path(&library.id(), &content, &kind, &variant, &format)
		.await?;
	assert_eq!(path.shards, ("12".into(), "34".into()));
	assert!(path
		.absolute_path
		.starts_with(library.path().join("sidecars")));
	assert!(
		!manager
			.exists(&library.id(), &content, &kind, &variant, &format)
			.await?
	);
	manager.bootstrap_scan(&library).await?;

	tokio::fs::create_dir_all(path.absolute_path.parent().unwrap()).await?;
	tokio::fs::write(&path.absolute_path, b"offline sidecar fixture").await?;
	assert!(
		manager
			.exists(&library.id(), &content, &kind, &variant, &format)
			.await?
	);
	assert!(matches!(
		manager.get_or_enqueue(&library, &content, &kind, &variant, &format).await?,
		SidecarResult::Ready(relative) if relative == path.relative_path,
	));

	manager.bootstrap_scan(&library).await?;
	let presence = manager
		.get_presence(&library, &[content], &kind, std::slice::from_ref(&variant))
		.await?;
	let recovered = &presence[&content][variant.as_str()];
	assert!(recovered.local);
	assert_eq!(recovered.path.as_ref(), Some(&path.relative_path));
	assert_eq!(recovered.status, SidecarStatus::Ready);
	let device_id = core.device.current_device().await.id;
	assert_eq!(recovered.devices, [device_id]);

	manager
		.record_sidecar(&library, &content, &kind, &variant, &format, 30, None)
		.await?;
	let availability = sidecar_availability::Entity::find()
		.filter(sidecar_availability::Column::ContentUuid.eq(content))
		.filter(sidecar_availability::Column::DeviceUuid.eq(device_id));
	let row = availability.clone().one(db).await?.unwrap();
	assert!(row.has);
	assert_eq!(row.size, Some(30));

	tokio::fs::remove_file(&path.absolute_path).await?;
	manager
		.remove_sidecar(&library, &content, &kind, &variant)
		.await?;
	assert!(manager
		.get_presence(&library, &[content], &kind, std::slice::from_ref(&variant))
		.await?
		.is_empty());
	let removed = availability.one(db).await?.unwrap();
	assert!(!removed.has);
	assert_eq!(removed.size, None);

	manager.deinit_library(&library.id()).await;
	let error = manager
		.compute_path(&library.id(), &content, &kind, &variant, &format)
		.await
		.unwrap_err();
	assert!(error.to_string().contains("not initialized"));
	core.shutdown().await?;
	Ok(())
}
