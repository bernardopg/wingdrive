//! Script adapter tests: the contract 1 stdin payload, the cursor round trip and deleting
//! linked records.
//!
//! The adapter under test is a short Python script, so these tests need `python3` on PATH.

use std::path::Path;

use wing_archive::adapter::script::ScriptAdapter;
use wing_archive::adapter::Adapter;
use wing_archive::source::SourceManager;

const MANIFEST: &str = r#"[adapter]
id = "echo"
name = "Echo"
summary = "Echoes its stdin back as records."
version = "1.0.0"
author = "WingDrive"
license = "Apache-2.0"
contract = 1
platforms = ["linux", "macos", "windows"]

[adapter.runtime]
kind = "python3"
entry = "sync.py"

[data_type]
id = "echo"
name = "Echo"

[models.note]
fields.title = "string"
fields.body = "text"

[models.note.relations]
many_to_many = ["note"]

[search]
primary_model = "note"
title = "title"
preview = "body"
search_fields = ["title", "body"]
"#;

/// Records what it received, links two notes, and on the second run deletes one of them.
const SCRIPT: &str = r#"import json, sys
data = json.load(sys.stdin)
def emit(op):
    print(json.dumps(op), flush=True)
emit({"upsert": "note", "external_id": "input", "fields": {"title": "input", "body": json.dumps(data, sort_keys=True)}})
if data["cursor"] is None:
    emit({"upsert": "note", "external_id": "a", "fields": {"title": "a", "body": ""}})
    emit({"upsert": "note", "external_id": "b", "fields": {"title": "b", "body": ""}})
    emit({"link": "note", "id": "a", "to": "note", "to_id": "b"})
    emit({"cursor": "one"})
else:
    emit({"delete": "note", "external_id": "b"})
    emit({"cursor": "two"})
print("a line on stderr", file=sys.stderr)
"#;

fn write_adapter(dir: &Path) {
	std::fs::write(dir.join("adapter.toml"), MANIFEST).unwrap();
	std::fs::write(dir.join("sync.py"), SCRIPT).unwrap();
}

async fn input_record(db: &wing_archive::db::SourceDb) -> serde_json::Value {
	let items = db.list_items(10, 0).await.unwrap();
	let input = items.iter().find(|i| i.external_id == "input").unwrap();
	serde_json::from_str(input.preview.as_deref().unwrap()).unwrap()
}

#[tokio::test]
async fn contract_one_gets_config_and_cursor_and_deletes_linked_records() {
	let tmp = tempfile::tempdir().unwrap();
	let adapter_dir = tmp.path().join("echo");
	std::fs::create_dir_all(&adapter_dir).unwrap();
	write_adapter(&adapter_dir);

	let adapter = ScriptAdapter::from_dir(&adapter_dir).unwrap();
	let sources = SourceManager::new(tmp.path().join("sources"));
	sources.create("s1", adapter.schema()).await.unwrap();
	let db = sources.open("s1").await.unwrap();

	let config = serde_json::json!({ "vault_path": "/notes", "_data_dir": "/data/s1" });

	let first = adapter.sync(&db, &config).await.unwrap();
	assert_eq!(first.error, None);
	assert_eq!(first.records_upserted, 3);
	assert_eq!(first.links_created, 1);
	let input = input_record(&db).await;
	assert_eq!(input["contract"], 1);
	assert_eq!(
		input["config"],
		serde_json::json!({ "vault_path": "/notes" })
	);
	assert_eq!(input["cursor"], serde_json::Value::Null);
	assert_eq!(input["data_dir"], "/data/s1");
	assert_eq!(
		db.get_cursor("default").await.unwrap().as_deref(),
		Some("one")
	);

	// "b" is still linked from "a"; the delete has to clear the junction row first.
	let second = adapter.sync(&db, &config).await.unwrap();
	assert_eq!(second.error, None);
	assert_eq!(second.records_deleted, 1);
	assert_eq!(input_record(&db).await["cursor"], "one");
	assert_eq!(db.count("note").await.unwrap(), 2);
	assert_eq!(
		db.get_cursor("default").await.unwrap().as_deref(),
		Some("two")
	);
}

#[tokio::test]
async fn failed_run_reports_the_last_stderr_line() {
	let tmp = tempfile::tempdir().unwrap();
	let adapter_dir = tmp.path().join("echo");
	std::fs::create_dir_all(&adapter_dir).unwrap();
	write_adapter(&adapter_dir);
	std::fs::write(
		adapter_dir.join("sync.py"),
		"import sys\nprint('vault_path is not a folder', file=sys.stderr)\nsys.exit(2)\n",
	)
	.unwrap();

	let adapter = ScriptAdapter::from_dir(&adapter_dir).unwrap();
	let sources = SourceManager::new(tmp.path().join("sources"));
	sources.create("s1", adapter.schema()).await.unwrap();
	let db = sources.open("s1").await.unwrap();

	let report = adapter.sync(&db, &serde_json::json!({})).await.unwrap();
	let error = report.error.unwrap();
	assert!(error.ends_with("vault_path is not a folder"), "{error}");
}

#[tokio::test]
async fn opening_a_source_repairs_the_old_search_delete_trigger() {
	let tmp = tempfile::tempdir().unwrap();
	let adapter_dir = tmp.path().join("echo");
	std::fs::create_dir_all(&adapter_dir).unwrap();
	write_adapter(&adapter_dir);
	let adapter = ScriptAdapter::from_dir(&adapter_dir).unwrap();
	let sources = SourceManager::new(tmp.path().join("sources"));
	sources.create("s1", adapter.schema()).await.unwrap();

	// The trigger as databases created before the fix have it.
	let db = sources.open("s1").await.unwrap();
	sqlx::raw_sql(
		"DROP TRIGGER search_index_ad; \
		 CREATE TRIGGER search_index_ad AFTER DELETE ON \"notes\" BEGIN \
		 INSERT INTO search_index(search_index, rowid, \"title\", \"body\") \
		 VALUES ('delete', old.rowid, old.\"title\", old.\"body\"); END",
	)
	.execute(db.pool())
	.await
	.unwrap();
	db.pool().close().await;
	drop(db);

	let db = sources.open("s1").await.unwrap();
	db.upsert(
		"note",
		"n",
		&serde_json::json!({ "title": "n", "body": "" }),
	)
	.await
	.unwrap();
	db.delete("note", "n").await.unwrap();
	assert_eq!(db.count("note").await.unwrap(), 0);
}

#[tokio::test]
async fn engine_uses_adapters_installed_outside_its_data_dir() {
	let tmp = tempfile::tempdir().unwrap();
	// Where the store installs adapters, not `<engine data dir>/adapters`.
	let installed = tmp.path().join("installed").join("echo");
	std::fs::create_dir_all(&installed).unwrap();
	write_adapter(&installed);

	let engine = wing_archive::Engine::new(wing_archive::EngineConfig {
		data_dir: tmp.path().join("library"),
	})
	.await
	.unwrap();
	engine
		.adapters()
		.register(std::sync::Arc::new(ScriptAdapter::from_dir(&installed).unwrap()));

	assert!(engine.adapter_config_fields("echo").unwrap().is_empty());
	let source = engine
		.create_source("Echo", "echo", serde_json::json!({}))
		.await
		.unwrap();
	let report = engine.sync(&source.id).await.unwrap();
	assert_eq!(report.error, None);
	assert_eq!(report.records_upserted, 3);
}
