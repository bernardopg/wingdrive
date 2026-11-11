use super::*;
use serde_json::{json, Value};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use wing_core::infra::daemon::types::{DaemonError, DaemonRequest, DaemonResponse};

const LIBRARY: &str = "550e8400-e29b-41d4-a716-446655440000";
const TAG: &str = "550e8400-e29b-41d4-a716-446655440001";

struct Step {
	method: &'static str,
	library: Option<uuid::Uuid>,
	response: DaemonResponse,
}

fn success(method: &'static str, library: Option<uuid::Uuid>, result: Value) -> Step {
	Step {
		method,
		library,
		response: DaemonResponse::JsonOk(result),
	}
}

async fn fixture(steps: Vec<Step>) -> (String, tokio::task::JoinHandle<Vec<Value>>) {
	let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
	let addr = listener.local_addr().unwrap().to_string();
	let task = tokio::spawn(async move {
		let mut payloads = Vec::new();
		for step in steps {
			let (mut stream, _) = tokio::time::timeout(Duration::from_secs(5), listener.accept())
				.await
				.unwrap()
				.unwrap();
			let mut bytes = Vec::new();
			stream.read_to_end(&mut bytes).await.unwrap();
			let req: DaemonRequest = serde_json::from_slice(&bytes).unwrap();
			let (method, library, payload) = match req {
				DaemonRequest::Query {
					method,
					library_id,
					payload,
				}
				| DaemonRequest::Action {
					method,
					library_id,
					payload,
				} => (method, library_id, payload),
				other => panic!("unexpected request: {other:?}"),
			};
			assert_eq!(method, step.method);
			assert_eq!(library, step.library);
			payloads.push(payload);
			stream
				.write_all(&serde_json::to_vec(&step.response).unwrap())
				.await
				.unwrap();
		}
		payloads
	});
	(addr, task)
}

fn library() -> uuid::Uuid {
	uuid::Uuid::parse_str(LIBRARY).unwrap()
}
fn listing() -> Step {
	success(
		"query:libraries.list",
		None,
		json!([{ "id": LIBRARY, "name": "isolated", "path": "fixture", "stats": null }]),
	)
}

async fn run(
	args: &[&str],
	format: OutputFormat,
	steps: Vec<Step>,
) -> (anyhow::Result<()>, Vec<Value>) {
	let root = tempfile::tempdir().unwrap();
	let (addr, server) = fixture(steps).await;
	let cli = Cli::try_parse_from(args).unwrap();
	let result = tokio::time::timeout(
		Duration::from_secs(8),
		run_client_command(cli.command, format, root.path().to_path_buf(), None, addr),
	)
	.await
	.expect("CLI request timed out");
	let received = tokio::time::timeout(Duration::from_secs(8), server)
		.await
		.expect("fixture timed out")
		.expect("fixture failed");
	(result, received)
}

#[tokio::test]
async fn create_tag_dispatches_name_and_namespace() {
	let (result, requests) = run(
		&["wing", "tag", "create", "report", "--namespace", "work"],
		OutputFormat::Human,
		vec![
			listing(),
			success(
				"action:tags.create.input",
				Some(library()),
				json!({"tag_id":TAG,"canonical_name":"report","namespace":"work","message":"created"}),
			),
		],
	)
	.await;
	result.unwrap();
	assert_eq!(requests[1]["canonical_name"], "report");
	assert_eq!(requests[1]["namespace"], "work");
}

#[tokio::test]
async fn search_tags_passes_filters_to_daemon() {
	let (result, requests) = run(&["wing", "tag", "search", "report", "--namespace", "work", "--limit", "5", "--include-archived"], OutputFormat::Human, vec![listing(), success("query:tags.search", Some(library()), json!({"tags":[],"total_found":0,"disambiguated":false,"query":"report","filters":{"namespace":"work","tag_type":null,"include_archived":true,"limit":5}}))]).await;
	result.unwrap();
	assert_eq!(requests[1]["query"], "report");
	assert_eq!(requests[1]["namespace"], "work");
	assert_eq!(requests[1]["limit"], 5);
	assert_eq!(requests[1]["include_archived"], true);
}

#[tokio::test]
async fn device_list_sends_detail_and_paired_filters() {
	let (result, requests) = run(
		&["wing", "devices", "list", "--detailed", "--show-paired"],
		OutputFormat::Human,
		vec![
			listing(),
			success("query:devices.list", Some(library()), json!([])),
		],
	)
	.await;
	result.unwrap();
	assert_eq!(requests[1]["include_details"], true);
	assert_eq!(requests[1]["show_paired"], true);
}

#[tokio::test]
async fn network_status_is_a_core_query() {
	let (result, _) = run(&["wing", "network", "status"], OutputFormat::Human, vec![listing(), success("query:network.status", None, json!({"running":true,"node_id":"fixture-node","addresses":["127.0.0.1"],"paired_devices":2,"connected_devices":1,"version":"test","relay_url":null}))]).await;
	result.unwrap();
}

#[tokio::test]
async fn daemon_error_is_propagated_by_tag_create() {
	let (result, _) = run(
		&["wing", "tag", "create", "report"],
		OutputFormat::Json,
		vec![
			listing(),
			Step {
				method: "action:tags.create.input",
				library: Some(library()),
				response: DaemonResponse::Error(DaemonError::ValidationError(
					"duplicate tag".into(),
				)),
			},
		],
	)
	.await;
	assert!(result.unwrap_err().to_string().contains("duplicate tag"));
}

#[tokio::test]
async fn file_search_forwards_pagination_and_scope() {
	let response = wing_core::ops::search::output::FileSearchOutput::empty("report");
	let (result, requests) = run(
		&[
			"wing",
			"search",
			"files",
			"report",
			"--limit",
			"7",
			"--offset",
			"3",
			"--location",
			LIBRARY,
		],
		OutputFormat::Human,
		vec![
			listing(),
			success(
				"query:search.files",
				Some(library()),
				serde_json::to_value(response).unwrap(),
			),
		],
	)
	.await;
	result.unwrap();
	assert_eq!(requests[1]["pagination"]["limit"], 7);
	assert_eq!(requests[1]["pagination"]["offset"], 3);
}

#[tokio::test]
async fn location_add_passes_name_mode_and_path() {
	let out = wing_core::ops::locations::add::output::LocationAddOutput::new(
		library(),
		wing_core::domain::addressing::WingPath::local("fixture"),
		Some("Photos".into()),
	);
	let (result, requests) = run(
		&[
			"wing", "location", "add", "fixture", "--name", "Photos", "--mode", "shallow",
		],
		OutputFormat::Human,
		vec![
			listing(),
			success(
				"action:locations.add.input",
				Some(library()),
				serde_json::to_value(out).unwrap(),
			),
		],
	)
	.await;
	result.unwrap();
	assert_eq!(requests[1]["name"], "Photos");
	assert_eq!(requests[1]["mode"], "Shallow");
}

#[tokio::test]
async fn location_list_empty_succeeds() {
	let (result, _) = run(
		&["wing", "location", "list"],
		OutputFormat::Human,
		vec![
			listing(),
			success(
				"query:locations.list",
				Some(library()),
				json!({"locations":[]}),
			),
		],
	)
	.await;
	result.unwrap();
}

#[tokio::test]
async fn location_removal_requires_explicit_confirmation_flag() {
	let (result, requests) = run(
		&["wing", "location", "remove", TAG, "--yes"],
		OutputFormat::Human,
		vec![
			listing(),
			success(
				"action:locations.remove.input",
				Some(library()),
				json!({"location_id":TAG,"path":"fixture"}),
			),
		],
	)
	.await;
	result.unwrap();
	assert_eq!(requests[1]["location_id"], TAG);
}

#[tokio::test]
async fn location_rescan_forwards_force_flag() {
	let (result, requests) = run(
		&["wing", "location", "rescan", TAG, "--force"],
		OutputFormat::Human,
		vec![
			listing(),
			success(
				"action:locations.rescan.input",
				Some(library()),
				json!({"location_id":TAG,"location_path":"fixture","job_id":LIBRARY,"full_rescan":true}),
			),
		],
	)
	.await;
	result.unwrap();
	assert_eq!(requests[1]["location_id"], TAG);
	assert_eq!(requests[1]["full_rescan"], true);
}

#[tokio::test]
async fn file_rename_dispatches_absolute_path_and_name() {
	let (result, requests) = run(
		&["wing", "file", "rename", "old", "new"],
		OutputFormat::Human,
		vec![
			listing(),
			success(
				"action:files.rename.input",
				Some(library()),
				json!({"id":TAG,"job_name":"rename"}),
			),
		],
	)
	.await;
	result.unwrap();
	assert_eq!(requests[1]["new_name"], "new");
	assert!(requests[1]["target"].to_string().contains("old"));
}

#[tokio::test]
async fn permanent_delete_dry_run_only_queries_preflight() {
	let (result, requests) = run(&["wing", "file", "delete", "--permanent", "--dry-run", "fixture"], OutputFormat::Human, vec![listing(), success("query:files.preflight", Some(library()), json!({"files":1,"bytes":10,"conflicts":0,"items":[],"truncated":false,"last_copies":1,"unknown_files":0}))]).await;
	result.unwrap();
	assert_eq!(requests[1]["operation"], "Delete");
	assert!(requests[1]["destination"].is_null());
}

#[tokio::test]
async fn network_pair_cancel_forwards_session_id() {
	let (result, requests) = run(
		&["wing", "network", "pair", "cancel", TAG],
		OutputFormat::Human,
		vec![
			listing(),
			success(
				"action:network.pair.cancel.input",
				Some(library()),
				json!({"cancelled":true}),
			),
		],
	)
	.await;
	result.unwrap();
	assert_eq!(requests[1]["session_id"], TAG);
}

#[tokio::test]
async fn wingdrop_accept_is_a_core_action() {
	let (result, requests) = run(
		&["wing", "network", "wingdrop", "accept", TAG],
		OutputFormat::Human,
		vec![
			listing(),
			success(
				"action:network.wingdrop.respond.input",
				None,
				json!({"destinations":["fixture"]}),
			),
		],
	)
	.await;
	result.unwrap();
	assert_eq!(requests[1]["drop_id"], TAG);
	assert_eq!(requests[1]["accept"], true);
}

#[tokio::test]
async fn wingdrop_decline_is_a_core_action() {
	let (result, requests) = run(
		&["wing", "network", "wingdrop", "decline", TAG],
		OutputFormat::Human,
		vec![
			listing(),
			success(
				"action:network.wingdrop.respond.input",
				None,
				json!({"destinations":[]}),
			),
		],
	)
	.await;
	result.unwrap();
	assert_eq!(requests[1]["drop_id"], TAG);
	assert_eq!(requests[1]["accept"], false);
}

#[tokio::test]
async fn job_pause_forwards_id() {
	let (result, requests) = run(
		&["wing", "job", "pause", TAG],
		OutputFormat::Human,
		vec![
			listing(),
			success(
				"action:jobs.pause.input",
				Some(library()),
				json!({"job_id":TAG,"success":true}),
			),
		],
	)
	.await;
	result.unwrap();
	assert_eq!(requests[1]["job_id"], TAG);
}

#[tokio::test]
async fn job_resume_forwards_id() {
	let (result, requests) = run(
		&["wing", "job", "resume", TAG],
		OutputFormat::Human,
		vec![
			listing(),
			success(
				"action:jobs.resume.input",
				Some(library()),
				json!({"job_id":TAG,"success":true}),
			),
		],
	)
	.await;
	result.unwrap();
	assert_eq!(requests[1]["job_id"], TAG);
}

#[tokio::test]
async fn job_cancel_forwards_id() {
	let (result, requests) = run(
		&["wing", "job", "cancel", TAG],
		OutputFormat::Human,
		vec![
			listing(),
			success(
				"action:jobs.cancel.input",
				Some(library()),
				json!({"job_id":TAG,"success":true}),
			),
		],
	)
	.await;
	result.unwrap();
	assert_eq!(requests[1]["job_id"], TAG);
}

#[tokio::test]
async fn redundancy_at_risk_forwards_volume_filter() {
	let out = wing_core::ops::search::output::FileSearchOutput::empty("");
	let (result, requests) = run(
		&[
			"wing",
			"redundancy",
			"at-risk",
			"--volume",
			TAG,
			"--limit",
			"3",
		],
		OutputFormat::Human,
		vec![
			listing(),
			success(
				"query:search.files",
				Some(library()),
				serde_json::to_value(out).unwrap(),
			),
		],
	)
	.await;
	result.unwrap();
	assert_eq!(requests[1]["filters"]["at_risk"], true);
	assert_eq!(requests[1]["filters"]["on_volumes"], json!([TAG]));
	assert_eq!(requests[1]["pagination"]["limit"], 3);
}

#[tokio::test]
async fn redundancy_compare_shared_requires_both_volumes() {
	let out = wing_core::ops::search::output::FileSearchOutput::empty("");
	let (result, requests) = run(
		&[
			"wing",
			"redundancy",
			"compare",
			LIBRARY,
			TAG,
			"--mode",
			"shared",
		],
		OutputFormat::Human,
		vec![
			listing(),
			success(
				"query:search.files",
				Some(library()),
				serde_json::to_value(out).unwrap(),
			),
		],
	)
	.await;
	result.unwrap();
	assert_eq!(requests[1]["filters"]["on_volumes"], json!([LIBRARY, TAG]));
	assert_eq!(requests[1]["filters"]["min_volume_count"], 2);
}

#[tokio::test]
async fn failed_pause_returns_error() {
	let (result, _) = run(
		&["wing", "job", "pause", TAG],
		OutputFormat::Human,
		vec![
			listing(),
			success(
				"action:jobs.pause.input",
				Some(library()),
				json!({"job_id":TAG,"success":false}),
			),
		],
	)
	.await;
	assert!(result.is_err());
}

#[tokio::test]
async fn failed_resume_returns_error() {
	let (result, _) = run(
		&["wing", "job", "resume", TAG],
		OutputFormat::Json,
		vec![
			listing(),
			success(
				"action:jobs.resume.input",
				Some(library()),
				json!({"job_id":TAG,"success":false}),
			),
		],
	)
	.await;
	assert!(result.is_err());
}

#[tokio::test]
async fn failed_cancel_returns_error() {
	let (result, _) = run(
		&["wing", "job", "cancel", TAG],
		OutputFormat::Human,
		vec![
			listing(),
			success(
				"action:jobs.cancel.input",
				Some(library()),
				json!({"job_id":TAG,"success":false}),
			),
		],
	)
	.await;
	assert!(result.is_err());
}

#[tokio::test]
async fn named_instance_reads_its_own_logs() {
	let root = tempfile::tempdir().unwrap();
	let named_logs = root.path().join("instances/check/logs");
	std::fs::create_dir_all(&named_logs).unwrap();
	std::fs::write(named_logs.join("daemon.log"), [0xff]).unwrap();
	let (addr, server) = fixture(vec![listing()]).await;
	let cli = Cli::try_parse_from(["wing", "logs", "show"]).unwrap();
	let result = run_client_command(
		cli.command,
		OutputFormat::Human,
		root.path().to_path_buf(),
		Some("check".into()),
		addr,
	)
	.await;
	assert!(
		result.is_err(),
		"the invalid UTF-8 fixture must be read from the named instance"
	);
	server.await.unwrap();
}

#[tokio::test]
async fn index_verify_renders_long_unicode_paths_without_panicking() {
	use wing_core::ops::indexing::verify::output::{
		IndexVerifyOutput, IntegrityDifference, IntegrityReport,
	};
	let long_path = std::path::PathBuf::from("é".repeat(60));
	let mut report = IntegrityReport::new();
	report
		.missing_from_index
		.push(IntegrityDifference::missing_from_index(long_path.clone()));
	report
		.stale_in_index
		.push(IntegrityDifference::stale_in_index(long_path));
	let output = IndexVerifyOutput {
		is_valid: false,
		report,
		path: "fixture".into(),
		duration_secs: 0.1,
	};
	let (result, _) = run(
		&["wing", "index", "verify", "fixture"],
		OutputFormat::Human,
		vec![
			listing(),
			success(
				"action:indexing.verify.input",
				Some(library()),
				serde_json::to_value(output).unwrap(),
			),
		],
	)
	.await;
	result.unwrap();
}

fn core_status(node_id: &str) -> Value {
	json!({
	  "version":"test", "built_at":"now", "library_count":0,
	  "device_info":{"id":LIBRARY,"name":"fixture","slug":"fixture","os":"test","hardware_model":null,"created_at":"2026-01-01T00:00:00Z"},
	  "libraries":[],
	  "services":{"location_watcher":{"running":true,"details":null},"networking":{"running":true,"details":null},"volume_monitor":{"running":false,"details":null},"file_sharing":{"running":false,"details":null}},
	  "network":{"running":true,"node_id":node_id,"addresses":["127.0.0.1"],"paired_devices":1,"connected_devices":1,"version":"test","relay_url":null},
	  "system":{"uptime":3660,"data_directory":"fixture","instance_name":"fixture","current_library":null}
	})
}

#[tokio::test]
async fn status_renders_long_unicode_node_id_without_panicking() {
	let node = "é".repeat(60);
	let (result, _) = run(
		&["wing", "status"],
		OutputFormat::Human,
		vec![
			listing(),
			success("query:core.status", None, core_status(&node)),
		],
	)
	.await;
	result.unwrap();
}

fn sync_metrics() -> Value {
	use wing_core::service::sync::metrics::snapshot::{
		DataVolumeSnapshot, ErrorSnapshot, OperationSnapshot, PerformanceSnapshot,
		SyncMetricsSnapshot, SyncStateSnapshot,
	};
	let mut data = DataVolumeSnapshot::default();
	data.entries_synced.insert("entry".into(), 12);
	data.bytes_sent = 2048;
	let mut operations = OperationSnapshot::default();
	operations.backfill_sessions_completed = 1;
	operations.backfill_pagination_rounds = 3;
	serde_json::to_value(SyncMetricsSnapshot {
		timestamp: chrono::Utc::now(),
		state: SyncStateSnapshot::default(),
		operations,
		data_volume: data,
		performance: PerformanceSnapshot::default(),
		errors: ErrorSnapshot::default(),
	})
	.unwrap()
}

#[tokio::test]
async fn sync_metrics_passes_filters_and_renders_snapshot() {
	let (result, requests) = run(
		&[
			"wing",
			"sync",
			"metrics",
			"--since",
			"2025-12-03 10:00:00",
			"--peer",
			TAG,
			"--model",
			"entry",
		],
		OutputFormat::Human,
		vec![
			listing(),
			success(
				"query:sync.metrics",
				Some(library()),
				json!({"metrics":sync_metrics()}),
			),
		],
	)
	.await;
	result.unwrap();
	assert_eq!(requests[1]["peer_id"], TAG);
	assert_eq!(requests[1]["model_type"], "entry");
	assert!(requests[1]["since"]
		.as_str()
		.unwrap()
		.starts_with("2025-12-03T10:00:00"));
}

#[tokio::test]
async fn sync_metrics_json_and_state_flags_are_forwarded() {
	let (result, requests) = run(
		&["wing", "sync", "metrics", "--json", "--state", "--errors"],
		OutputFormat::Human,
		vec![
			listing(),
			success(
				"query:sync.metrics",
				Some(library()),
				json!({"metrics":sync_metrics()}),
			),
		],
	)
	.await;
	result.unwrap();
	assert_eq!(requests[1]["state_only"], true);
	assert_eq!(requests[1]["errors_only"], true);
}

#[tokio::test]
async fn sync_partners_renders_debug_information() {
	let output = json!({"partners":[{"device_uuid":TAG,"device_name":"remote","is_paired":true}],"debug_info":{"total_devices":1,"sync_enabled_devices":1,"paired_devices":1,"final_sync_partners":1,"device_details":[{"uuid":TAG,"name":"remote","sync_enabled":true,"has_node_id":true,"node_id":"node"}]}});
	let (result, _) = run(
		&["wing", "sync", "partners"],
		OutputFormat::Human,
		vec![
			listing(),
			success("query:sync.partners", Some(library()), output),
		],
	)
	.await;
	result.unwrap();
}

fn sync_event() -> wing_core::infra::sync::SyncEventLog {
	use wing_core::infra::sync::{SyncEventLog, SyncEventType};
	let mut event = SyncEventLog::new(library(), SyncEventType::StateTransition, "it's a fixture");
	event.details = Some(json!({"text":"fixture"}));
	event.record_count = Some(4);
	event.duration_ms = Some(30);
	event
}

#[tokio::test]
async fn sync_event_json_export_writes_filtered_fixture() {
	let root = tempfile::tempdir().unwrap();
	let output = root.path().join("events.json");
	let events = json!({"events":[sync_event()]});
	let (result, requests) = run(
		&[
			"wing",
			"sync",
			"events",
			"--format",
			"json",
			"--output",
			output.to_str().unwrap(),
			"--severity",
			"info",
			"--limit",
			"2",
		],
		OutputFormat::Human,
		vec![
			listing(),
			success("query:sync.eventLog", Some(library()), events),
		],
	)
	.await;
	result.unwrap();
	assert_eq!(requests[1]["limit"], 2);
	let exported: Value = serde_json::from_slice(&std::fs::read(output).unwrap()).unwrap();
	assert_eq!(exported[0]["summary"], "it's a fixture");
	assert!(exported[0].get("device_id").is_none());
}

#[tokio::test]
async fn sync_event_sql_export_escapes_summary() {
	let root = tempfile::tempdir().unwrap();
	let output = root.path().join("events.sql");
	let (result, _) = run(
		&[
			"wing",
			"sync",
			"events",
			"--format",
			"sql",
			"--output",
			output.to_str().unwrap(),
		],
		OutputFormat::Human,
		vec![
			listing(),
			success(
				"query:sync.eventLog",
				Some(library()),
				json!({"events":[sync_event()]}),
			),
		],
	)
	.await;
	result.unwrap();
	assert!(std::fs::read_to_string(output)
		.unwrap()
		.contains("it''s a fixture"));
}

#[tokio::test]
async fn sync_event_markdown_export_lists_devices() {
	let root = tempfile::tempdir().unwrap();
	let output = root.path().join("events.md");
	let (result, _) = run(
		&[
			"wing",
			"sync",
			"events",
			"--format",
			"markdown",
			"--with-device",
			"--output",
			output.to_str().unwrap(),
		],
		OutputFormat::Human,
		vec![
			listing(),
			success(
				"query:sync.eventLog",
				Some(library()),
				json!({"events":[sync_event()]}),
			),
		],
	)
	.await;
	result.unwrap();
	let text = std::fs::read_to_string(output).unwrap();
	assert!(text.contains("it's a fixture"));
	assert!(text.contains(LIBRARY));
}

#[tokio::test]
async fn sync_event_rejects_unknown_severity_before_query() {
	let (result, _) = run(
		&["wing", "sync", "events", "--severity", "fatal"],
		OutputFormat::Human,
		vec![listing()],
	)
	.await;
	assert!(result.unwrap_err().to_string().contains("Invalid severity"));
}

#[tokio::test]
async fn index_start_dispatches_to_selected_library() {
	let (result, requests) = run(
		&[
			"wing",
			"index",
			"start",
			"folder",
			"--library",
			LIBRARY,
			"--mode",
			"shallow",
			"--scope",
			"current",
			"--persistent",
		],
		OutputFormat::Human,
		vec![
			listing(),
			success(
				"action:indexing.start.input",
				Some(library()),
				json!({"id":TAG,"job_name":"index"}),
			),
		],
	)
	.await;
	result.unwrap();
	assert_eq!(requests[1]["library_id"], LIBRARY);
	assert_eq!(requests[1]["paths"], json!(["folder"]));
	assert_eq!(requests[1]["persistence"], "Persistent");
}

#[tokio::test]
async fn index_quick_scan_loads_library_and_dispatches() {
	let (result, requests) = run(
		&["wing", "index", "quick-scan", "folder"],
		OutputFormat::Human,
		vec![
			listing(),
			listing(),
			success(
				"action:indexing.start.input",
				Some(library()),
				json!({"id":TAG,"job_name":"index"}),
			),
		],
	)
	.await;
	result.unwrap();
	assert_eq!(requests[2]["paths"], json!(["folder"]));
	assert_eq!(requests[2]["scope"], "Current");
}

#[tokio::test]
async fn index_browse_content_dispatches() {
	let (result, requests) = run(
		&["wing", "index", "browse", "folder", "--content"],
		OutputFormat::Human,
		vec![
			listing(),
			listing(),
			success(
				"action:indexing.start.input",
				Some(library()),
				json!({"id":TAG,"job_name":"index"}),
			),
		],
	)
	.await;
	result.unwrap();
	assert_eq!(requests[2]["mode"], "Content");
}

#[tokio::test]
async fn index_verify_displays_valid_report() {
	use wing_core::ops::indexing::verify::output::{IndexVerifyOutput, IntegrityReport};
	let output = IndexVerifyOutput {
		is_valid: true,
		report: IntegrityReport::new(),
		path: "fixture".into(),
		duration_secs: 0.1,
	};
	let (result, requests) = run(
		&["wing", "index", "verify", "fixture", "--verify-content"],
		OutputFormat::Human,
		vec![
			listing(),
			success(
				"action:indexing.verify.input",
				Some(library()),
				serde_json::to_value(output).unwrap(),
			),
		],
	)
	.await;
	result.unwrap();
	assert_eq!(requests[1]["verify_content"], true);
}

#[tokio::test]
async fn index_cache_displays_detailed_stats_and_paths() {
	let response = json!({
	  "indexed_paths_count":1,"indexing_in_progress_count":1,
	  "index_stats":{"total_entries":42,"path_index_count":4,"unique_names":12,"interned_strings":13,"content_kinds":2,"uuid_count":3,"memory_bytes":1024,"total_file_bytes":2048,"age_seconds":5.0,"idle_seconds":1.0,
	  "memory_breakdown":{"arena":1,"cache":2,"registry":3,"path_index_overhead":4,"path_index_entries":5,"entry_uuids_overhead":6,"entry_uuids_entries":7,"content_kinds_overhead":8,"content_kinds_entries":9}},
	  "indexed_paths":[{"path":"indexed","child_count":3}],"paths_in_progress":["pending"],"total_indexes":null,"indexing_in_progress":null,"indexes":[]
	});
	let (result, requests) = run(
		&[
			"wing",
			"index",
			"ephemeral-cache",
			"--detailed",
			"--filter",
			"fixture",
		],
		OutputFormat::Human,
		vec![
			listing(),
			success("query:core.ephemeral_status", None, response),
		],
	)
	.await;
	result.unwrap();
	assert_eq!(requests[1]["detailed"], true);
	assert_eq!(requests[1]["path_filter"], "fixture");
}

#[tokio::test]
async fn redundancy_summary_renders_library_and_volume_totals() {
	let response = json!({"volumes":[{"volume_uuid":TAG,"display_name":"fixture","total_bytes":1000,"at_risk_bytes":400,"at_risk_file_count":2,"redundant_bytes":600,"redundant_file_count":3,"total_file_count":5}],"library_totals":{"total_unique_content_bytes":1000,"total_at_risk_bytes":400,"total_redundant_bytes":600,"replication_score":0.6}});
	let (result, requests) = run(
		&["wing", "redundancy", "summary", "--volume", TAG],
		OutputFormat::Human,
		vec![
			listing(),
			success("query:redundancy.summary", Some(library()), response),
		],
	)
	.await;
	result.unwrap();
	assert_eq!(requests[1]["volume_uuids"], json!([TAG]));
}

#[tokio::test]
async fn library_create_selects_new_library() {
	let (result, requests) = run(
		&["wing", "library", "create", "new"],
		OutputFormat::Human,
		vec![
			listing(),
			success(
				"action:libraries.create.input",
				None,
				json!({"library_id":TAG,"name":"new","path":"fixture"}),
			),
		],
	)
	.await;
	result.unwrap();
	assert_eq!(requests[1]["name"], "new");
}

#[tokio::test]
async fn library_list_queries_core() {
	let (result, _) = run(
		&["wing", "library", "list"],
		OutputFormat::Human,
		vec![listing(), listing()],
	)
	.await;
	result.unwrap();
}

#[tokio::test]
async fn library_switch_by_name_checks_list() {
	let (result, _) = run(
		&["wing", "library", "switch", "--name", "isolated"],
		OutputFormat::Human,
		vec![listing(), listing()],
	)
	.await;
	result.unwrap();
}

#[tokio::test]
async fn library_switch_unknown_name_returns_error() {
	let (result, _) = run(
		&["wing", "library", "switch", "--name", "missing"],
		OutputFormat::Human,
		vec![listing(), listing()],
	)
	.await;
	assert!(result.unwrap_err().to_string().contains("not found"));
}

#[tokio::test]
async fn library_info_uses_explicit_library_id() {
	let now = chrono::Utc::now();
	let info = wing_core::domain::Library {
		id: uuid::Uuid::parse_str(TAG).unwrap(),
		name: "selected".into(),
		description: None,
		path: "fixture".into(),
		created_at: now,
		updated_at: now,
		settings: Default::default(),
		statistics: Default::default(),
	};
	let (result, _) = run(
		&["wing", "library", "info", TAG],
		OutputFormat::Human,
		vec![
			listing(),
			success(
				"query:libraries.info",
				Some(uuid::Uuid::parse_str(TAG).unwrap()),
				serde_json::to_value(info).unwrap(),
			),
		],
	)
	.await;
	result.unwrap();
}

#[tokio::test]
async fn library_delete_forwards_confirmation_and_data_policy() {
	let (result, req) = run(
		&["wing", "library", "delete", TAG, "--yes", "--delete-data"],
		OutputFormat::Human,
		vec![
			listing(),
			success(
				"action:libraries.delete.input",
				Some(library()),
				json!({"library_id":TAG,"name":"fixture"}),
			),
		],
	)
	.await;
	result.unwrap();
	assert_eq!(req[1]["library_id"], TAG);
	assert_eq!(req[1]["delete_data"], true);
}

#[tokio::test]
async fn job_list_queries_library_and_renders_jobs() {
	let output = json!({"jobs":[{"id":TAG,"name":"index","device_id":LIBRARY,"status":"running","progress":0.5,"action_type":null,"action_context":null,"created_at":"2026-01-01T00:00:00Z","started_at":null,"completed_at":null}]});
	let (result, req) = run(
		&["wing", "job", "list", "--status", "running"],
		OutputFormat::Human,
		vec![
			listing(),
			listing(),
			success("query:jobs.list", Some(library()), output),
		],
	)
	.await;
	result.unwrap();
	assert_eq!(req[2]["status"], "running");
}

#[tokio::test]
async fn job_info_renders_found_job() {
	let output = json!({"id":TAG,"name":"copy","status":"completed","progress":1.0,"created_at":"2026-01-01T00:00:00Z","started_at":null,"completed_at":null,"error_message":null});
	let (result, req) = run(
		&["wing", "job", "info", TAG],
		OutputFormat::Human,
		vec![
			listing(),
			listing(),
			success("query:jobs.info", Some(library()), output),
		],
	)
	.await;
	result.unwrap();
	assert_eq!(req[2]["job_id"], TAG);
}

#[tokio::test]
async fn network_devices_renders_connected_peer() {
	let output = json!({"devices":[{"id":TAG,"name":"peer","deviceType":"desktop","osVersion":"test","appVersion":"1","isConnected":true,"lastSeen":"2026-01-01T00:00:00Z"}],"total":1,"connected":1});
	let (result, req) = run(
		&["wing", "network", "devices", "--connected"],
		OutputFormat::Human,
		vec![
			listing(),
			success("query:network.devices.list", None, output),
		],
	)
	.await;
	result.unwrap();
	assert_eq!(req[1]["connectedOnly"], true);
}

#[tokio::test]
async fn network_pair_join_passes_code_and_node() {
	let (result, req) = run(
		&[
			"wing",
			"network",
			"pair",
			"join",
			"one two",
			"--node-id",
			"fixture-node",
		],
		OutputFormat::Human,
		vec![
			listing(),
			success(
				"action:network.pair.join.input",
				Some(library()),
				json!({"paired_device_id":TAG,"device_name":"peer"}),
			),
		],
	)
	.await;
	result.unwrap();
	let value: Value = serde_json::from_str(req[1]["code"].as_str().unwrap()).unwrap();
	assert_eq!(value["words"], "one two");
	assert_eq!(value["node_id"], "fixture-node");
}

#[tokio::test]
async fn network_revoke_forwards_device_and_preserves_library() {
	let (result, req) = run(
		&["wing", "network", "revoke", TAG, "--yes"],
		OutputFormat::Human,
		vec![
			listing(),
			success(
				"action:network.device.revoke.input",
				Some(library()),
				json!({"revoked":true,"peer_notified":false}),
			),
		],
	)
	.await;
	result.unwrap();
	assert_eq!(req[1]["device_id"], TAG);
	assert_eq!(req[1]["remove_from_library"], false);
}

#[tokio::test]
async fn file_history_renders_operation_and_retention() {
	let out = wing_core::ops::files::undo::UndoHistoryOutput {
		operations: vec![wing_core::ops::files::undo::UndoableOperation {
			job_id: uuid::Uuid::parse_str(TAG).unwrap(),
			operation: "copy".into(),
			started_at: 0,
			effects: 2,
			pending: 1,
			example: "fixture".into(),
		}],
		retention_days: 7,
	};
	let (result, req) = run(
		&["wing", "file", "history", "--limit", "2"],
		OutputFormat::Human,
		vec![
			listing(),
			success(
				"query:files.undo_history",
				Some(library()),
				serde_json::to_value(out).unwrap(),
			),
		],
	)
	.await;
	result.unwrap();
	assert_eq!(req[1]["limit"], 2);
}

#[tokio::test]
async fn file_undo_forwards_job_id() {
	let (result, req) = run(
		&["wing", "file", "undo", TAG],
		OutputFormat::Human,
		vec![
			listing(),
			success(
				"action:files.undo.input",
				Some(library()),
				json!({"id":TAG,"job_name":"undo"}),
			),
		],
	)
	.await;
	result.unwrap();
	assert_eq!(req[1]["job_id"], TAG);
}

#[tokio::test]
async fn file_info_handles_missing_file() {
	let (result, _) = run(
		&["wing", "file", "info", "fixture"],
		OutputFormat::Human,
		vec![
			listing(),
			success("query:files.by_path", Some(library()), Value::Null),
		],
	)
	.await;
	result.unwrap();
}

#[tokio::test]
async fn file_list_forwards_sort_and_pagination() {
	let (result, _) = run(
		&[
			"wing",
			"file",
			"list",
			"fixture",
			"--sort-by",
			"size",
			"--limit",
			"5",
			"--include-hidden",
		],
		OutputFormat::Human,
		vec![
			listing(),
			success(
				"query:files.directory_listing",
				Some(library()),
				json!({"files":[],"total_count":0,"has_more":false}),
			),
		],
	)
	.await;
	result.unwrap();
}

fn fixture_file(
	kind: wing_core::domain::file::EntryKind,
	name: &str,
) -> wing_core::domain::file::File {
	use wing_core::domain::{addressing::WingPath, ContentKind};
	let now = chrono::Utc::now();
	wing_core::domain::file::File {
		id: uuid::Uuid::new_v4(),
		wing_path: WingPath::local(name),
		kind,
		name: name.into(),
		extension: Some("txt".into()),
		size: 2048,
		directory_size: None,
		content_identity: None,
		alternate_paths: vec![],
		tags: vec![],
		favorite: false,
		sidecars: vec![],
		image_media_data: None,
		video_media_data: None,
		audio_media_data: None,
		created_at: now,
		modified_at: now,
		accessed_at: None,
		content_kind: ContentKind::Unknown,
		is_local: true,
		duration_seconds: None,
		archive: None,
	}
}

#[tokio::test]
async fn file_list_renders_files_directories_and_links() {
	use wing_core::domain::file::EntryKind;
	let files = vec![
		fixture_file(EntryKind::File, "report"),
		fixture_file(EntryKind::Directory, "folder"),
		fixture_file(EntryKind::Symlink, "link"),
	];
	let (result, req) = run(
		&[
			"wing",
			"file",
			"list",
			"fixture",
			"--sort-by",
			"modified",
			"--limit",
			"5",
			"--include-hidden",
		],
		OutputFormat::Human,
		vec![
			listing(),
			success(
				"query:files.directory_listing",
				Some(library()),
				json!({"files":files,"total_count":3,"has_more":false}),
			),
		],
	)
	.await;
	result.unwrap();
	assert_eq!(req[1]["sort_by"], "modified");
	assert_eq!(req[1]["limit"], 5);
	assert_eq!(req[1]["include_hidden"], true);
}

#[tokio::test]
async fn file_info_renders_matching_file() {
	let file = fixture_file(wing_core::domain::file::EntryKind::File, "report");
	let (result, _) = run(
		&["wing", "file", "info", "report"],
		OutputFormat::Human,
		vec![
			listing(),
			success(
				"query:files.by_path",
				Some(library()),
				serde_json::to_value(file).unwrap(),
			),
		],
	)
	.await;
	result.unwrap();
}

#[tokio::test]
async fn file_mkdir_dispatches_parent_and_name() {
	let out = wing_core::ops::files::create_folder::output::CreateFolderOutput::without_items(
		wing_core::domain::addressing::WingPath::local("fixture/new"),
	);
	let (result, req) = run(
		&["wing", "file", "mkdir", "fixture", "new"],
		OutputFormat::Human,
		vec![
			listing(),
			success(
				"action:files.createFolder.input",
				Some(library()),
				serde_json::to_value(out).unwrap(),
			),
		],
	)
	.await;
	result.unwrap();
	assert_eq!(req[1]["name"], "new");
}

#[tokio::test]
async fn permanent_delete_confirms_and_dispatches_after_preflight() {
	let plan = json!({"files":1,"bytes":10,"conflicts":0,"items":[],"truncated":false,"last_copies":1,"unknown_files":0});
	let (result, req) = run(
		&["wing", "file", "delete", "fixture", "--permanent", "--yes"],
		OutputFormat::Human,
		vec![
			listing(),
			success("query:files.preflight", Some(library()), plan),
			success(
				"action:files.delete.input",
				Some(library()),
				json!({"id":TAG,"job_name":"delete"}),
			),
		],
	)
	.await;
	result.unwrap();
	assert_eq!(req[2]["permanent"], true);
}

#[tokio::test]
async fn file_copy_and_move_dispatch_after_preflight() {
	let root = tempfile::tempdir().unwrap();
	let source = root.path().join("source");
	let destination = root.path().join("dest");
	std::fs::write(&source, b"fixture").unwrap();
	let plan = json!({"files":1,"bytes":7,"conflicts":0,"items":[],"truncated":false,"last_copies":null,"unknown_files":null});
	let (result, req) = run(
		&[
			"wing",
			"file",
			"copy",
			source.to_str().unwrap(),
			"--destination",
			destination.to_str().unwrap(),
			"--move-files",
		],
		OutputFormat::Human,
		vec![
			listing(),
			success("query:files.preflight", Some(library()), plan),
			success(
				"action:files.copy.input",
				Some(library()),
				json!({"id":TAG,"job_name":"copy"}),
			),
		],
	)
	.await;
	result.unwrap();
	assert_eq!(req[1]["operation"], "Move");
	assert_eq!(req[2]["move_files"], true);
	assert_eq!(std::fs::read(source).unwrap(), b"fixture");
}

#[tokio::test]
async fn volume_add_cloud_dispatches_synthetic_s3_configuration() {
	use wing_core::ops::volumes::add_cloud::VolumeAddCloudOutput;
	use wing_core::volume::{backend::CloudServiceType, VolumeFingerprint};
	let out = VolumeAddCloudOutput::new(
		VolumeFingerprint::from_hex("0123456789abcdef"),
		"fixture".into(),
		CloudServiceType::S3,
	);
	let (result, req) = run(
		&[
			"wing",
			"volume",
			"add-cloud",
			"fixture",
			"--service",
			"s3",
			"--bucket",
			"bucket",
			"--region",
			"region",
			"--access-key-id",
			"fake",
			"--secret-access-key",
			"fake",
		],
		OutputFormat::Human,
		vec![
			listing(),
			success(
				"action:volumes.add_cloud.input",
				Some(library()),
				serde_json::to_value(out).unwrap(),
			),
		],
	)
	.await;
	result.unwrap();
	assert_eq!(req[1]["display_name"], "fixture");
	assert_eq!(req[1]["config"]["bucket"], "bucket");
}

#[tokio::test]
async fn volume_remove_cloud_dispatches_fingerprint() {
	use wing_core::ops::volumes::remove_cloud::VolumeRemoveCloudOutput;
	use wing_core::volume::VolumeFingerprint;
	let out = VolumeRemoveCloudOutput::new(VolumeFingerprint::from_hex("0123456789abcdef"));
	let (result, req) = run(
		&[
			"wing",
			"volume",
			"remove-cloud",
			"0123456789abcdef",
			"--yes",
		],
		OutputFormat::Human,
		vec![
			listing(),
			success(
				"action:volumes.remove_cloud.input",
				Some(library()),
				serde_json::to_value(out).unwrap(),
			),
		],
	)
	.await;
	result.unwrap();
	assert_eq!(req[1]["fingerprint"], "0123456789abcdef");
}

#[tokio::test]
async fn volume_list_handles_empty_library() {
	let (result, _) = run(
		&["wing", "volume", "list"],
		OutputFormat::Human,
		vec![
			listing(),
			success("query:volumes.list", Some(library()), json!({"volumes":[]})),
		],
	)
	.await;
	result.unwrap();
}

#[tokio::test]
async fn volume_scan_does_not_issue_mutating_request() {
	let (result, _) = run(
		&["wing", "volume", "scan"],
		OutputFormat::Human,
		vec![listing()],
	)
	.await;
	result.unwrap();
}

#[tokio::test]
async fn spaces_list_handles_empty_response() {
	let (result, _) = run(
		&["wing", "spaces", "list"],
		OutputFormat::Human,
		vec![
			listing(),
			success("query:spaces.list", Some(library()), json!({"spaces":[]})),
		],
	)
	.await;
	result.unwrap();
}

#[tokio::test]
async fn spaces_layout_forwards_id() {
	let (result, req) = run(
		&["wing", "spaces", "layout", TAG],
		OutputFormat::Human,
		vec![
			listing(),
			success(
				"query:spaces.get_layout",
				Some(library()),
				json!({"groups":[]}),
			),
		],
	)
	.await;
	result.unwrap();
	assert_eq!(req[1]["space_id"], TAG);
}

#[tokio::test]
async fn volume_list_renders_tracked_volume() {
	let mut volume = wing_core::domain::volume::Volume::new(
		library(),
		wing_core::domain::volume::VolumeFingerprint::from_hex("0123456789abcdef"),
		"fixture".into(),
		"mount".into(),
	);
	volume.track(Some(library()));
	volume.total_capacity = 2048;
	volume.available_space = 1024;
	let (result, _) = run(
		&["wing", "volume", "list"],
		OutputFormat::Human,
		vec![
			listing(),
			success(
				"query:volumes.list",
				Some(library()),
				json!({"volumes":[volume]}),
			),
		],
	)
	.await;
	result.unwrap();
}

#[tokio::test]
async fn spaces_list_renders_a_wing() {
	let space = wing_core::domain::Space::new("fixture".into(), "star".into(), "#112233".into());
	let (result, _) = run(
		&["wing", "spaces", "list"],
		OutputFormat::Human,
		vec![
			listing(),
			success(
				"query:spaces.list",
				Some(library()),
				json!({"spaces":[space]}),
			),
		],
	)
	.await;
	result.unwrap();
}

#[tokio::test]
async fn no_library_selection_blocks_library_specific_command() {
	let (result, _) = run(
		&["wing", "file", "rename", "old", "new"],
		OutputFormat::Human,
		vec![success("query:libraries.list", None, json!([]))],
	)
	.await;
	assert!(result
		.unwrap_err()
		.to_string()
		.contains("No active library"));
}

#[tokio::test]
async fn library_info_defaults_to_current_library() {
	let now = chrono::Utc::now();
	let info = wing_core::domain::Library {
		id: library(),
		name: "fixture".into(),
		description: Some("description".into()),
		path: "fixture".into(),
		created_at: now,
		updated_at: now,
		settings: Default::default(),
		statistics: Default::default(),
	};
	let (result, _) = run(
		&["wing", "library", "info"],
		OutputFormat::Human,
		vec![
			listing(),
			success(
				"query:libraries.info",
				Some(library()),
				serde_json::to_value(info).unwrap(),
			),
		],
	)
	.await;
	result.unwrap();
}

#[tokio::test]
async fn network_pair_status_lists_sessions() {
	let output = json!({"sessions":[{"id":TAG,"state":"Completed","remote_device_id":LIBRARY,"expires_at":null}]});
	let (result, _) = run(
		&["wing", "network", "pair", "status"],
		OutputFormat::Human,
		vec![
			listing(),
			success("query:network.pair.status", None, output),
		],
	)
	.await;
	result.unwrap();
}

#[tokio::test]
async fn network_devices_handles_empty_list() {
	let (result, _) = run(
		&["wing", "network", "devices"],
		OutputFormat::Human,
		vec![
			listing(),
			success(
				"query:network.devices.list",
				None,
				json!({"devices":[],"total":0,"connected":0}),
			),
		],
	)
	.await;
	result.unwrap();
}

#[tokio::test]
async fn location_export_forwards_output_and_renders_statistics() {
	let output = json!({"location_uuid":TAG,"location_name":"photos","export_path":"fixture.sql","file_size_bytes":500,"stats":{"entries":4,"content_identities":3,"user_metadata":2,"tags":1,"media_data":1}});
	let (result, req) = run(
		&["wing", "location", "export", TAG, "--output", "fixture.sql"],
		OutputFormat::Human,
		vec![
			listing(),
			success("action:locations.export.input", Some(library()), output),
		],
	)
	.await;
	result.unwrap();
	assert_eq!(req[1]["export_path"], "fixture.sql");
}

#[tokio::test]
async fn location_import_forwards_name_and_skip_policy() {
	let output = json!({"location_uuid":TAG,"location_name":"photos","import_path":"fixture.sql","stats":{"entries_imported":3,"entries_skipped":1,"content_identities":2,"user_metadata":0,"tags":0,"media_data":0}});
	let (result, req) = run(
		&[
			"wing",
			"location",
			"import",
			"fixture.sql",
			"--name",
			"photos",
			"--skip-existing",
		],
		OutputFormat::Human,
		vec![
			listing(),
			success("action:locations.import.input", Some(library()), output),
		],
	)
	.await;
	result.unwrap();
	assert_eq!(req[1]["new_name"], "photos");
	assert_eq!(req[1]["skip_existing"], true);
}

#[tokio::test]
async fn library_sync_discovery_renders_remote_libraries() {
	let stats = json!({"total_files":0,"total_size":0,"location_count":0,"tag_count":0,"thumbnail_count":0,"database_size":0,"last_indexed":null,"updated_at":"2026-01-01T00:00:00Z"});
	let out = json!({"deviceId":TAG,"deviceName":"peer","libraries":[{"id":LIBRARY,"name":"remote","description":"fixture","createdAt":"2026-01-01T00:00:00Z","statistics":stats}],"isOnline":true});
	let (result, _) = run(
		&["wing", "library", "sync-setup", "discover", TAG],
		OutputFormat::Human,
		vec![
			listing(),
			success("query:network.sync_setup.discover", None, out),
		],
	)
	.await;
	result.unwrap();
}

#[tokio::test]
async fn library_sync_setup_dispatches_explicit_device() {
	let out = json!({"success":true,"localLibraryId":LIBRARY,"remoteLibraryId":TAG,"devicesRegistered":true,"message":"fixture"});
	let (result, _) = run(
		&[
			"wing",
			"library",
			"sync-setup",
			"setup",
			"--local-library",
			LIBRARY,
			"--remote-device",
			TAG,
			"--local-device",
			LIBRARY,
			"--action",
			"share-local",
			"--name",
			"fixture",
		],
		OutputFormat::Human,
		vec![
			listing(),
			success("action:network.sync_setup.input", None, out),
		],
	)
	.await;
	result.unwrap();
}

#[tokio::test]
async fn pairing_generate_renders_synthetic_code() {
	let out = json!({"code":"fixture words","session_id":TAG,"expires_at":"2026-12-01T00:00:00Z","qr_json":"fixture","node_id":"node"});
	let (result, _) = run(
		&["wing", "network", "pair", "generate"],
		OutputFormat::Human,
		vec![
			listing(),
			success("action:network.pair.generate.input", Some(library()), out),
		],
	)
	.await;
	result.unwrap();
}

#[tokio::test]
async fn wingdrop_pending_handles_no_offers() {
	let (result, _) = run(
		&["wing", "network", "wingdrop", "pending"],
		OutputFormat::Human,
		vec![
			listing(),
			success("query:network.wingdrop.pending", None, json!([])),
		],
	)
	.await;
	result.unwrap();
}

#[tokio::test]
async fn wingdrop_cancel_forwards_drop_id() {
	let (result, req) = run(
		&["wing", "network", "wingdrop", "cancel", TAG],
		OutputFormat::Human,
		vec![
			listing(),
			success(
				"action:network.wingdrop.cancel.input",
				None,
				json!({"cancelled":true}),
			),
		],
	)
	.await;
	result.unwrap();
	assert_eq!(req[1]["drop_id"], TAG);
}

#[tokio::test]
async fn library_selection_replaces_stale_id_and_persists() {
	let root = tempfile::tempdir().unwrap();
	let (addr, server) = fixture(vec![listing()]).await;
	let mut ctx = Context::new(
		CoreClient::new(addr.clone()),
		OutputFormat::Human,
		root.path().to_path_buf(),
		addr,
	)
	.unwrap();
	ctx.set_library_id(uuid::Uuid::parse_str(TAG).unwrap())
		.unwrap();
	ctx.validate_and_fix_library().await.unwrap();
	assert_eq!(ctx.get_current_library_id(), Some(library()));
	assert_eq!(
		CliConfig::load(&root.path().to_path_buf())
			.unwrap()
			.current_library_id,
		Some(library())
	);
	server.await.unwrap();
}

#[tokio::test]
async fn library_selection_clears_stale_id_when_empty() {
	let root = tempfile::tempdir().unwrap();
	let (addr, server) = fixture(vec![success("query:libraries.list", None, json!([]))]).await;
	let mut ctx = Context::new(
		CoreClient::new(addr.clone()),
		OutputFormat::Human,
		root.path().to_path_buf(),
		addr,
	)
	.unwrap();
	ctx.set_library_id(library()).unwrap();
	ctx.validate_and_fix_library().await.unwrap();
	assert_eq!(ctx.get_current_library_id(), None);
	assert_eq!(
		CliConfig::load(&root.path().to_path_buf())
			.unwrap()
			.current_library_id,
		None
	);
	server.await.unwrap();
}

#[tokio::test]
async fn library_selection_without_available_libraries_stays_empty() {
	let root = tempfile::tempdir().unwrap();
	let (addr, server) = fixture(vec![success("query:libraries.list", None, json!([]))]).await;
	let mut ctx = Context::new(
		CoreClient::new(addr.clone()),
		OutputFormat::Human,
		root.path().to_path_buf(),
		addr,
	)
	.unwrap();
	ctx.validate_and_fix_library().await.unwrap();
	assert!(!ctx.has_active_library());
	assert!(ctx.require_current_library().is_err());
	server.await.unwrap();
}

#[tokio::test]
async fn library_selection_keeps_existing_id() {
	let root = tempfile::tempdir().unwrap();
	let (addr, server) = fixture(vec![listing()]).await;
	let mut ctx = Context::new(
		CoreClient::new(addr.clone()),
		OutputFormat::Human,
		root.path().to_path_buf(),
		addr,
	)
	.unwrap();
	ctx.set_library_id(library()).unwrap();
	ctx.validate_and_fix_library().await.unwrap();
	assert_eq!(ctx.require_current_library().unwrap(), library());
	server.await.unwrap();
}
