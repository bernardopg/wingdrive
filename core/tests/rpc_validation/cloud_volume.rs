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
	for (fingerprint, kind, location) in [
		("fixture-local", "Primary", "/local"),
		("fixture-cloud", "Cloud", "gdrive://fixture@example.test"),
		("fixture-legacy-s3", "Network", "s3://fixture-bucket"),
		("fixture-lan", "Network", "smb://fixture/share"),
	] {
		let now = chrono::Utc::now();
		let volume = entities::volume::ActiveModel {
			uuid: Set(Uuid::new_v4()),
			device_id: Set(device_id),
			fingerprint: Set(fingerprint.to_string()),
			tracked_at: Set(now),
			last_seen_at: Set(now),
			is_online: Set(true),
			volume_type: Set(Some(kind.to_string())),
			mount_point: Set(Some(location.to_string())),
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
	for (index, fingerprint, removable) in [
		(0, "fixture-local", false),
		(1, "fixture-cloud", true),
		(2, "fixture-legacy-s3", true),
		(3, "fixture-lan", false),
	] {
		let result = remove(fingerprint).await;
		if removable {
			assert_eq!(
				result.map_err(anyhow::Error::msg)?["fingerprint"],
				fingerprint
			);
		} else {
			assert!(result.is_err(), "{fingerprint} must remain tracked");
		}
		let (volume_id, credential_id) = volumes[index];
		let tracked = entities::volume::Entity::find_by_id(volume_id)
			.one(db)
			.await?;
		let credential = entities::cloud_credential::Entity::find_by_id(credential_id)
			.one(db)
			.await?;
		assert_eq!(tracked.is_none(), removable, "{fingerprint}");
		assert_eq!(credential.is_none(), removable, "{fingerprint}");
	}

	core.shutdown().await.unwrap();
	Ok(())
}
