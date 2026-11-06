//! `config.app.update` validation, normalization, persistence and change reporting.

mod helpers;

use helpers::*;
use serde_json::json;
use wing_core::{
	config::AppConfig,
	infra::action::{error::ActionError, CoreAction},
	ops::config::app::{UpdateAppConfigAction, UpdateAppConfigInput, UpdateAppConfigOutput},
};

fn action(input: serde_json::Value) -> UpdateAppConfigAction {
	let input: UpdateAppConfigInput = serde_json::from_value(input).unwrap();
	UpdateAppConfigAction::from_input(input).unwrap()
}

async fn update(harness: &IndexingHarness, input: serde_json::Value) -> UpdateAppConfigOutput {
	let action = action(input);
	action.validate(harness.core.context.clone()).await.unwrap();
	action.execute(harness.core.context.clone()).await.unwrap()
}

async fn rejected_field(harness: &IndexingHarness, input: serde_json::Value) -> String {
	match action(input.clone())
		.validate(harness.core.context.clone())
		.await
	{
		Err(ActionError::Validation { field, .. }) => field,
		other => panic!("{input} was not rejected: {other:?}"),
	}
}

fn saved(harness: &IndexingHarness) -> AppConfig {
	AppConfig::load_from(&harness.core.context.data_dir).unwrap()
}

#[tokio::test]
async fn validation_rejects_each_invalid_field() {
	let harness = IndexingHarnessBuilder::new("app_config_validation")
		.build()
		.await
		.unwrap();
	for (input, field) in [
		(
			json!({"wingdrop_destination": "relative/dir"}),
			"wingdrop_destination",
		),
		(json!({"log_level": "verbose"}), "log_level"),
		(json!({"theme": "neon"}), "theme"),
		(json!({"language": "pt-BR"}), "language"),
		(json!({"language": "PT"}), "language"),
		(
			json!({"proxy_pairing_vouch_signature_max_age": 0}),
			"proxy_pairing_vouch_signature_max_age",
		),
		(
			json!({"proxy_pairing_vouch_response_timeout": 0}),
			"proxy_pairing_vouch_response_timeout",
		),
		(
			json!({"proxy_pairing_vouch_queue_retry_limit": 0}),
			"proxy_pairing_vouch_queue_retry_limit",
		),
		(json!({"spacebot_base_url": "  "}), "spacebot_base_url"),
		// Trailing slashes are trimmed before saving, so "/" would store an empty URL.
		(json!({"spacebot_base_url": "/"}), "spacebot_base_url"),
		(
			json!({"spacebot_default_agent_id": " "}),
			"spacebot_default_agent_id",
		),
		(
			json!({"spacebot_default_sender_name": ""}),
			"spacebot_default_sender_name",
		),
	] {
		assert_eq!(rejected_field(&harness, input).await, field);
	}
	harness.shutdown().await.unwrap();
}

#[tokio::test]
async fn updates_are_normalized_saved_and_reported() {
	let harness = IndexingHarnessBuilder::new("app_config_update")
		.build()
		.await
		.unwrap();

	let output = update(
		&harness,
		json!({
			"log_level": "error",
			"theme": "Dark",
			"language": "pt",
			"wingdrop_destination": "  ",
			"spacebot_base_url": " http://127.0.0.1:19898/ ",
			"spacebot_auth_token": "  ",
			"spacebot_default_agent_id": " main ",
			"fs_watcher_enabled": false,
		}),
	)
	.await;
	assert!(output.success);
	assert!(output.requires_restart);
	assert!(output.message.contains("log_level"), "{}", output.message);

	let config = saved(&harness);
	assert_eq!(config.log_level, "error");
	assert_eq!(config.preferences.theme, "dark");
	assert_eq!(config.preferences.language, "pt");
	assert_eq!(config.wingdrop.destination, None);
	assert_eq!(config.spacebot.base_url, "http://127.0.0.1:19898");
	assert_eq!(config.spacebot.auth_token, None);
	assert_eq!(config.spacebot.default_agent_id, "main");
	assert!(!config.services.fs_watcher_enabled);

	let output = update(&harness, json!({"theme": "dark", "language": "pt"})).await;
	assert_eq!(output.message, "No changes to apply");
	assert!(!output.requires_restart);

	harness.shutdown().await.unwrap();
}

/// Values are stored lowercase, so the same value in another case is no change
/// and must not ask for a restart.
#[tokio::test]
async fn same_value_in_another_case_is_not_a_change() {
	let harness = IndexingHarnessBuilder::new("app_config_case")
		.build()
		.await
		.unwrap();
	update(&harness, json!({"log_level": "warn", "theme": "nord"})).await;

	let output = update(&harness, json!({"log_level": "WARN", "theme": "Nord"})).await;

	assert_eq!(output.message, "No changes to apply");
	assert!(!output.requires_restart);
	assert_eq!(saved(&harness).log_level, "warn");
	harness.shutdown().await.unwrap();
}
