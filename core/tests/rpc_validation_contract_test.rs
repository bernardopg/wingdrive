//! # RPC validation contracts
//!
//! Exercise registered handlers and action validation with an isolated library so
//! malformed client input cannot reach filesystem operations.

mod helpers;

use std::time::Duration;

use serde_json::json;
use uuid::Uuid;

use helpers::IndexingHarnessBuilder;
use wing_core::{
	domain::addressing::WingPath,
	infra::{
		action::{error::ActionError, LibraryAction},
		api::{dispatcher::ApiDispatcher, error::ApiError, SessionContext},
		wire::registry::{CORE_ACTIONS, CORE_QUERIES, LIBRARY_ACTIONS, LIBRARY_QUERIES},
	},
	ops::files::{
		create_folder::{action::CreateFolderAction, input::CreateFolderInput},
		query::file_by_id::FileByIdQuery,
	},
};

#[tokio::test]
async fn every_registered_handler_rejects_scalar_payloads_without_panicking() -> anyhow::Result<()>
{
	let harness = IndexingHarnessBuilder::new("rpc_malformed")
		.disable_watcher()
		.build()
		.await?;
	let session = SessionContext::device_session(harness.device_id, "contract".into())
		.with_library(harness.library.id());
	for (method, handler) in CORE_ACTIONS.iter() {
		let result = tokio::time::timeout(
			Duration::from_secs(5),
			handler(harness.core.context.clone(), json!(true)),
		)
		.await?;
		assert!(result.is_err(), "{method} accepted a boolean payload");
	}
	for (method, handler) in LIBRARY_ACTIONS.iter() {
		let result = tokio::time::timeout(
			Duration::from_secs(5),
			handler(harness.core.context.clone(), session.clone(), json!(true)),
		)
		.await?;
		assert!(result.is_err(), "{method} accepted a boolean payload");
	}
	for registry in [&*CORE_QUERIES, &*LIBRARY_QUERIES] {
		for (method, handler) in registry {
			let result = tokio::time::timeout(
				Duration::from_secs(5),
				handler(harness.core.context.clone(), session.clone(), json!(true)),
			)
			.await?;
			assert!(result.is_err(), "{method} accepted a boolean payload");
		}
	}
	harness.shutdown().await?;
	Ok(())
}

#[tokio::test]
async fn dispatcher_distinguishes_missing_library_from_invalid_payload() -> anyhow::Result<()> {
	let harness = IndexingHarnessBuilder::new("rpc_context")
		.disable_watcher()
		.build()
		.await?;
	let dispatcher = ApiDispatcher::new(harness.core.context.clone());
	let execute = wing_core::infra::daemon::rpc::RpcServer::execute_json_operation;
	let error = execute("query:missing.contract", None, json!({}), &harness.core)
		.await
		.unwrap_err();
	assert_eq!(error, "Unknown method: query:missing.contract");
	for (method, expected) in [
		("query:files.by_id", "Library ID required for library query"),
		(
			"action:files.createFolder.input",
			"Library ID required for library action",
		),
	] {
		let error = execute(method, None, json!({}), &harness.core)
			.await
			.unwrap_err();
		assert_eq!(error, expected);
	}
	let session = SessionContext::device_session(harness.device_id, "contract".into());
	let error = dispatcher
		.execute_library_query::<FileByIdQuery>(FileByIdQuery::new(Uuid::nil()), session.clone())
		.await
		.unwrap_err();
	assert!(matches!(error, ApiError::NoLibrarySelected));
	assert_eq!(error.status_code(), 404);
	let absent_id = Uuid::from_u128(7);
	let error = dispatcher
		.execute_library_query::<FileByIdQuery>(
			FileByIdQuery::new(Uuid::nil()),
			session.clone().with_library(absent_id),
		)
		.await
		.unwrap_err();
	assert!(
		matches!(&error, ApiError::LibraryNotFound { library_id } if library_id == &absent_id.to_string())
	);
	assert_eq!(error.status_code(), 404);
	let handler = LIBRARY_QUERIES["query:files.by_id"];
	let error = handler(
		harness.core.context.clone(),
		session.clone(),
		json!({"file_id": "not-a-uuid"}),
	)
	.await
	.unwrap_err();
	assert!(
		error.contains("UUID") || error.contains("invalid character"),
		"{error}"
	);
	let error = handler(
		harness.core.context.clone(),
		session,
		json!({"file_id": Uuid::nil()}),
	)
	.await
	.unwrap_err();
	assert_eq!(error, "No library selected");
	let session = SessionContext::device_session(harness.device_id, "contract".into())
		.with_library(harness.library.id());
	let value = handler(
		harness.core.context.clone(),
		session,
		json!({"file_id": Uuid::nil()}),
	)
	.await
	.map_err(anyhow::Error::msg)?;
	assert_eq!(value, serde_json::Value::Null);
	harness.shutdown().await?;
	Ok(())
}

#[tokio::test]
async fn folder_validation_rejects_invalid_names_and_address_kinds() -> anyhow::Result<()> {
	let harness = IndexingHarnessBuilder::new("folder_validation")
		.disable_watcher()
		.build()
		.await?;
	for name in ["", ".", "..", "../escape", "a\\b", "bad\0name", "bad:name"] {
		let action = CreateFolderAction::from_input(CreateFolderInput::new(
			WingPath::local(harness.temp_path()),
			name,
		))
		.unwrap();
		assert!(
			matches!(action.validate(&harness.library, harness.core.context.clone()).await, Err(ActionError::Validation { field, .. }) if field == "name"),
			"{name:?}"
		);
	}
	for parent in [
		WingPath::content(Uuid::nil()),
		WingPath::from_uri("sidecar://00000000-0000-0000-0000-000000000000/ocr/default.json")?,
	] {
		let action = CreateFolderAction::new(parent, "new folder");
		assert!(
			matches!(action.validate(&harness.library, harness.core.context.clone()).await, Err(ActionError::Validation { field, .. }) if field == "parent")
		);
	}
	let input: CreateFolderInput = serde_json::from_value(
		json!({"parent": WingPath::local(harness.temp_path()), "name": "café"}),
	)?;
	assert!(input.items.is_empty());
	let action = CreateFolderAction::from_input(input).unwrap();
	action
		.validate(&harness.library, harness.core.context.clone())
		.await?;
	let output = action
		.execute(harness.library.clone(), harness.core.context.clone())
		.await?;
	assert!(output.folder_path.as_local_path().unwrap().is_dir());
	let value = serde_json::to_value(&output)?;
	assert_eq!(
		value["folder_path"],
		serde_json::to_value(WingPath::local(harness.temp_path().join("café")))?
	);
	// The folder is journaled so it can be undone; the id is a fresh UUID.
	assert!(value["undo_id"]
		.as_str()
		.is_some_and(|id| Uuid::parse_str(id).is_ok()));
	harness.shutdown().await?;
	Ok(())
}

#[tokio::test]
async fn cloud_sync_update_preserves_null_and_clears_zero_interval() -> anyhow::Result<()> {
	use sea_orm::{ActiveModelTrait, Set};
	use wing_core::infra::{daemon::rpc::RpcServer, db::entities::cloud_sync_pair};

	let harness = IndexingHarnessBuilder::new("cloud_sync_interval")
		.disable_watcher()
		.build()
		.await?;
	let id = Uuid::new_v4();
	let now = chrono::Utc::now();
	cloud_sync_pair::ActiveModel {
		uuid: Set(id),
		name: Set("Backup".into()),
		local_path: Set(harness.temp_path().to_string_lossy().into_owned()),
		remote_service: Set("gdrive".into()),
		remote_identifier: Set("fixture@example.test".into()),
		remote_path: Set("Backup".into()),
		mode: Set("two_way".into()),
		propagate_deletes: Set(false),
		interval_minutes: Set(Some(60)),
		enabled: Set(true),
		created_at: Set(now),
		updated_at: Set(now),
		..Default::default()
	}
	.insert(harness.library.db().conn())
	.await?;

	let update = |payload| {
		RpcServer::execute_json_operation(
			"action:cloud_sync.update.input",
			Some(harness.library.id()),
			payload,
			&harness.core,
		)
	};
	let result = update(json!({"id": id, "enabled": false, "interval_minutes": null}))
		.await
		.map_err(anyhow::Error::msg)?;
	assert_eq!(result["enabled"], false);
	assert_eq!(result["interval_minutes"], 60);

	let result = update(json!({"id": id, "interval_minutes": 15}))
		.await
		.map_err(anyhow::Error::msg)?;
	assert_eq!(result["interval_minutes"], 15);
	assert!(update(json!({"id": id, "interval_minutes": 1}))
		.await
		.is_err());

	let result = update(json!({"id": id, "interval_minutes": 0}))
		.await
		.map_err(anyhow::Error::msg)?;
	assert_eq!(result["interval_minutes"], serde_json::Value::Null);

	harness.shutdown().await?;
	Ok(())
}
