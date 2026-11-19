use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use tempfile::TempDir;
use uuid::Uuid;
use wing_core::{
	config::AppConfig,
	infra::{daemon::rpc::RpcServer, db::entities},
	Core,
};

#[tokio::test]
async fn cloud_removal_leaves_local_volumes_tracked() -> anyhow::Result<()> {
	let temp_dir = TempDir::new()?;
	let path = temp_dir.path().to_path_buf();
	let mut config = AppConfig::default_with_dir(path.clone());
	config.services.networking_enabled = false;
	let core = std::sync::Arc::new(
		Core::new_with_config(path, Some(config), None)
			.await
			.unwrap(),
	);
	let library = core.libraries.list().await[0].clone();
	let db = library.db().conn();
	let device_id = core.device.device_id()?;
	let mut volumes = Vec::new();
	for (fingerprint, kind) in [("fixture-local", "Primary"), ("fixture-cloud", "Cloud")] {
		let now = chrono::Utc::now();
		let volume = entities::volume::ActiveModel {
			uuid: Set(Uuid::new_v4()),
			device_id: Set(device_id),
			fingerprint: Set(fingerprint.to_string()),
			tracked_at: Set(now),
			last_seen_at: Set(now),
			is_online: Set(true),
			volume_type: Set(Some(kind.to_string())),
			..Default::default()
		}
		.insert(db)
		.await?;
		let credential = entities::cloud_credential::ActiveModel {
			volume_fingerprint: Set(fingerprint.to_string()),
			encrypted_credential: Set(vec![0]),
			service_type: Set("fixture".to_string()),
			created_at: Set(now),
			updated_at: Set(now),
			..Default::default()
		}
		.insert(db)
		.await?;
		volumes.push((volume.id, credential.id));
	}

	let remove = |fingerprint| {
		RpcServer::execute_json_operation(
			"action:volumes.remove_cloud.input",
			Some(library.id()),
			serde_json::json!({ "fingerprint": fingerprint }),
			&core,
		)
	};
	let local = remove("fixture-local").await;
	assert!(local.is_err(), "a local volume must not be removed");
	assert!(entities::volume::Entity::find_by_id(volumes[0].0)
		.one(db)
		.await?
		.is_some());

	let cloud = remove("fixture-cloud").await.map_err(anyhow::Error::msg)?;
	assert_eq!(cloud["fingerprint"], "fixture-cloud");
	assert!(entities::volume::Entity::find_by_id(volumes[1].0)
		.one(db)
		.await?
		.is_none());

	assert!(entities::cloud_credential::Entity::find_by_id(volumes[0].1)
		.one(db)
		.await?
		.is_some());
	assert!(entities::cloud_credential::Entity::find_by_id(volumes[1].1)
		.one(db)
		.await?
		.is_none());

	core.shutdown().await.unwrap();
	Ok(())
}
