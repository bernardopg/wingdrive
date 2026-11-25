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

	let data_dir = tmp.path().join("sources").join("s1");
	let config = serde_json::json!({ "vault_path": "/notes", "_data_dir": data_dir });

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
	assert_eq!(input["data_dir"], data_dir.join("state").to_str().unwrap());
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
	engine.adapters().register(std::sync::Arc::new(
		ScriptAdapter::from_dir(&installed).unwrap(),
	));

	assert!(engine.adapter_config_fields("echo").unwrap().is_empty());
	let source = engine
		.create_source("Echo", "echo", serde_json::json!({}))
		.await
		.unwrap();
	let report = engine.sync(&source.id).await.unwrap();
	assert_eq!(report.error, None);
	assert_eq!(report.records_upserted, 3);
}

#[tokio::test]
async fn detect_collects_suggestions_only_when_declared() {
	let tmp = tempfile::tempdir().unwrap();
	let dir = tmp.path().join("echo");
	std::fs::create_dir_all(&dir).unwrap();
	write_adapter(&dir);
	std::fs::write(
		dir.join("sync.py"),
		r#"import json, sys
data = json.load(sys.stdin)
assert data["mode"] == "detect" and data["config"] == {}
print("not json")
print(json.dumps({"suggest": {"vault_path": ["/a", "/b"]}}))
print(json.dumps({"suggest": {"vault_path": ["/b", "/c"], "other": [1, "x"]}}))
"#,
	)
	.unwrap();

	// Without `runtime.detect` the adapter is never asked.
	let adapter = ScriptAdapter::from_dir(&dir).unwrap();
	assert!(adapter.detect_suggestions().await.is_empty());

	let manifest = MANIFEST.replace("entry = \"sync.py\"", "entry = \"sync.py\"\ndetect = true");
	std::fs::write(dir.join("adapter.toml"), manifest).unwrap();
	let adapter = ScriptAdapter::from_dir(&dir).unwrap();
	let suggestions = adapter.detect_suggestions().await;
	assert_eq!(suggestions["vault_path"], vec!["/a", "/b", "/c"]);
	assert_eq!(suggestions["other"], vec!["x"]);
}

/// Runs `script` as a contract 1 adapter with `permissions`, returning the JSON it printed
/// as its only record's body.
#[cfg(target_os = "linux")]
async fn run_confined(
	tmp: &Path,
	permissions: &str,
	script: &str,
	config: serde_json::Value,
) -> serde_json::Value {
	let dir = tmp.join("probe");
	std::fs::create_dir_all(&dir).unwrap();
	let manifest = MANIFEST.replace(
		"[data_type]",
		&format!("[adapter.permissions]\n{permissions}\n\n[data_type]"),
	);
	std::fs::write(dir.join("adapter.toml"), manifest).unwrap();
	let body = format!(
		"import json, sys\ndata = json.load(sys.stdin)\nresult = {{}}\n{script}\nprint(json.dumps({{\"upsert\": \"note\", \"external_id\": \"r\", \"fields\": {{\"title\": \"r\", \"body\": json.dumps(result)}}}}))\n"
	);
	std::fs::write(dir.join("sync.py"), body).unwrap();
	let adapter = ScriptAdapter::from_dir(&dir).unwrap();
	let sources = SourceManager::new(tmp.join("sources"));
	let _ = sources.create("s1", adapter.schema()).await;
	let db = sources.open("s1").await.unwrap();
	let report = adapter.sync(&db, &config).await.unwrap();
	assert_eq!(report.error, None);
	let items = db.list_items(10, 0).await.unwrap();
	let item = items.iter().find(|i| i.external_id == "r").unwrap();
	serde_json::from_str(item.preview.as_deref().unwrap()).unwrap()
}

#[cfg(target_os = "linux")]
fn confinement_available() -> bool {
	match wing_archive::adapter::sandbox::isolation() {
		Ok(wing_archive::adapter::sandbox::Isolation::Bubblewrap) => true,
		_ => {
			eprintln!("bubblewrap is not available; skipping the confinement test");
			false
		}
	}
}

#[cfg(target_os = "linux")]
#[tokio::test]
async fn adapters_see_only_declared_paths_and_write_only_their_state() {
	if !confinement_available() {
		return;
	}
	let tmp = tempfile::tempdir().unwrap();
	let notes = tmp.path().join("Notes");
	std::fs::create_dir_all(&notes).unwrap();
	std::fs::write(notes.join("a.md"), "declared").unwrap();
	let secret = tmp.path().join("secret.txt");
	std::fs::write(&secret, "undeclared").unwrap();
	let data_dir = tmp.path().join("sources").join("s1");

	let script = format!(
		r#"import os
def read(path):
    try:
        return open(path).read()
    except OSError as error:
        return type(error).__name__
def write(path):
    try:
        open(path, "w").write("x")
        return "written"
    except OSError as error:
        return type(error).__name__
result["declared"] = read({notes:?} + "/a.md")
result["undeclared"] = read({secret:?})
result["passwd"] = read("/etc/shadow")
result["home"] = os.path.isdir(os.path.expanduser("~/.ssh"))
result["state"] = write(os.path.join(data["data_dir"], "cache.json"))
result["adapter_dir"] = write("sync.py")
result["data_dir"] = data["data_dir"]
result["isolation"] = os.environ.get("WINGDRIVE_ADAPTER_ISOLATION")
"#,
		notes = notes.to_str().unwrap(),
		secret = secret.to_str().unwrap(),
	);
	let result = run_confined(
		tmp.path(),
		"read = [\"{config.vault_path}\"]",
		&script,
		serde_json::json!({ "vault_path": notes, "_data_dir": data_dir }),
	)
	.await;
	assert_eq!(result["declared"], "declared");
	assert_eq!(result["undeclared"], "FileNotFoundError");
	assert_eq!(result["passwd"], "FileNotFoundError");
	assert_eq!(result["home"], false);
	assert_eq!(result["state"], "written");
	assert_ne!(result["adapter_dir"], "written");
	assert_eq!(result["isolation"], "bubblewrap");
	assert_eq!(result["data_dir"], data_dir.join("state").to_str().unwrap());
	assert!(data_dir.join("state").join("cache.json").is_file());
}

#[cfg(target_os = "linux")]
#[tokio::test]
async fn adapters_without_hosts_have_no_network() {
	if !confinement_available() {
		return;
	}
	let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
	let port = listener.local_addr().unwrap().port();
	let script = format!(
		r#"import socket
try:
    socket.create_connection(("127.0.0.1", {port}), timeout=3).close()
    result["connect"] = "connected"
except OSError as error:
    result["connect"] = type(error).__name__
"#
	);

	let offline = tempfile::tempdir().unwrap();
	let result = run_confined(offline.path(), "read = []", &script, serde_json::json!({})).await;
	assert_ne!(result["connect"], "connected");

	let online = tempfile::tempdir().unwrap();
	let result = run_confined(
		online.path(),
		"network = [\"api.example.org\"]",
		&script,
		serde_json::json!({}),
	)
	.await;
	assert_eq!(result["connect"], "connected");
	drop(listener);
}

#[tokio::test]
async fn adapters_stop_at_their_timeout_and_memory_limit() {
	let tmp = tempfile::tempdir().unwrap();
	let dir = tmp.path().join("slow");
	std::fs::create_dir_all(&dir).unwrap();
	std::fs::write(
		dir.join("adapter.toml"),
		MANIFEST.replace("entry = \"sync.py\"", "entry = \"sync.py\"\ntimeout = 1"),
	)
	.unwrap();
	std::fs::write(
		dir.join("sync.py"),
		"import json, sys, time\njson.load(sys.stdin)\n\
		 try:\n    bytearray(3 * 1024 ** 3)\n    memory = 'unlimited'\nexcept MemoryError:\n    memory = 'limited'\n\
		 print(json.dumps({'upsert': 'note', 'external_id': 'a', 'fields': {'title': 'a', 'body': memory}}), flush=True)\n\
		 time.sleep(30)\n",
	)
	.unwrap();
	let adapter = ScriptAdapter::from_dir(&dir).unwrap();
	let sources = SourceManager::new(tmp.path().join("sources"));
	sources.create("s1", adapter.schema()).await.unwrap();
	let db = sources.open("s1").await.unwrap();

	let started = std::time::Instant::now();
	let report = adapter.sync(&db, &serde_json::json!({})).await.unwrap();
	assert!(started.elapsed() < std::time::Duration::from_secs(10));
	assert_eq!(
		report.error.as_deref(),
		Some("adapter stopped after 1 s (runtime.timeout)")
	);
	// What it printed before the limit is kept; the 3 GiB allocation was refused.
	assert_eq!(report.records_upserted, 1);
	let items = db.list_items(10, 0).await.unwrap();
	assert_eq!(items[0].preview.as_deref(), Some("limited"));
}
