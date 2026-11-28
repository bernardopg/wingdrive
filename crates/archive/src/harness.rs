//! Contract harness: runs an adapter folder against its fixtures the way the daemon runs it.
//!
//! An adapter keeps its test cases in `fixtures/<case>/`. Each case has:
//!
//! - `config.json` (optional): the config a user would save for the source. In string values,
//!   `{fixture}` becomes the case folder's absolute path, so a case can point the adapter at
//!   files it ships, such as `fixtures/basic/vault`.
//! - `expected.json`: what the source database must hold afterwards ([`Expected`]).
//!
//! For each case the harness loads the adapter with [`ScriptAdapter::from_dir`], creates a
//! source in a temporary folder, and syncs it as the daemon does: same stdin payload, same
//! sandbox, the cursor of one run passed to the next. By default it syncs twice, and the second
//! run must leave every model with the same number of records, so an adapter that duplicates or
//! drops records on an incremental run fails.
//!
//! The registry's review checks, the templates' CI and `wing adapter test` all call
//! [`test_adapter`].

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::adapter::script::ScriptAdapter;
use crate::adapter::{Adapter, SyncReport};
use crate::error::{Error, Result};
use crate::source::SourceManager;

/// Folder of an adapter that holds its test cases.
pub const FIXTURES_DIR: &str = "fixtures";

/// What a case expects, read from `expected.json`.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Expected {
	/// How many times to sync; each run gets the cursor of the one before. Defaults to 2.
	#[serde(default = "default_runs")]
	pub runs: u32,
	/// The first run must fail with an error containing this text. No further run follows.
	#[serde(default)]
	pub error: Option<String>,
	/// Whether every run after the first must keep the record counts of the first. Defaults
	/// to true; a case whose fixture changes between runs cannot have that.
	#[serde(default = "default_true")]
	pub idempotent: bool,
	/// Number of records per model after the last run.
	#[serde(default)]
	pub counts: BTreeMap<String, i64>,
	/// Records that must exist after the last run, with the fields they must hold.
	#[serde(default)]
	pub records: Vec<ExpectedRecord>,
	/// Records that must not exist after the last run.
	#[serde(default)]
	pub absent: Vec<RecordRef>,
}

fn default_runs() -> u32 {
	2
}

fn default_true() -> bool {
	true
}

/// A record a case expects, and the fields it must hold.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExpectedRecord {
	pub model: String,
	pub external_id: String,
	/// Fields to compare; others are not checked. A value matches the stored text it turns
	/// into, so `1` matches a stored `"1"` and `null` matches a field the adapter left out.
	#[serde(default)]
	pub fields: serde_json::Map<String, serde_json::Value>,
}

/// A record named by model and external ID.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecordRef {
	pub model: String,
	pub external_id: String,
}

/// The outcome of one case.
#[derive(Debug, Clone)]
pub struct CaseReport {
	/// The case folder's name.
	pub name: String,
	/// The report of every run, in order.
	pub runs: Vec<SyncReport>,
	/// Every check that did not hold; empty when the case passed.
	pub failures: Vec<String>,
}

impl CaseReport {
	/// Whether every check of the case held.
	pub fn passed(&self) -> bool {
		self.failures.is_empty()
	}
}

/// Runs every case in `adapter_dir/fixtures`, in name order.
///
/// Fails when the adapter cannot be loaded, has no cases, or a case file cannot be read.
/// A case whose checks do not hold is not an error: its report lists the failures.
pub async fn test_adapter(adapter_dir: &Path) -> Result<Vec<CaseReport>> {
	let cases = case_dirs(adapter_dir)?;
	if cases.is_empty() {
		return Err(Error::Other(format!(
			"{} has no test cases: add fixtures/<case>/expected.json",
			adapter_dir.display()
		)));
	}
	let mut reports = Vec::with_capacity(cases.len());
	for case in cases {
		reports.push(run_case(adapter_dir, &case).await?);
	}
	Ok(reports)
}

/// Case folders of an adapter: the folders in `fixtures/` that hold an `expected.json`.
pub fn case_dirs(adapter_dir: &Path) -> Result<Vec<PathBuf>> {
	let fixtures = adapter_dir.join(FIXTURES_DIR);
	if !fixtures.is_dir() {
		return Ok(Vec::new());
	}
	let mut cases = Vec::new();
	for entry in std::fs::read_dir(&fixtures)? {
		let path = entry?.path();
		if path.join("expected.json").is_file() {
			cases.push(path);
		}
	}
	cases.sort();
	Ok(cases)
}

/// Runs one case folder against the adapter in `adapter_dir`.
pub async fn run_case(adapter_dir: &Path, case_dir: &Path) -> Result<CaseReport> {
	let case_dir = std::fs::canonicalize(case_dir)?;
	let name = case_dir
		.file_name()
		.map(|name| name.to_string_lossy().into_owned())
		.unwrap_or_default();
	let expected: Expected = read_json(&case_dir.join("expected.json"))?;
	let config = match read_optional_json(&case_dir.join("config.json"))? {
		Some(config) => substitute_fixture(config, &case_dir),
		None => serde_json::json!({}),
	};
	if !config.is_object() {
		return Err(Error::Other(format!(
			"{}: config.json must hold an object",
			case_dir.display()
		)));
	}

	let adapter = ScriptAdapter::from_dir(adapter_dir)?;
	let schema = adapter.schema();
	let workspace = tempfile::tempdir()?;
	let sources = SourceManager::new(workspace.path().join("sources"));
	sources.create("harness", schema).await?;
	let db = sources.open("harness").await?;
	let mut run_config = config;
	if let Some(object) = run_config.as_object_mut() {
		object.insert(
			"_data_dir".to_string(),
			serde_json::Value::String(sources.source_dir("harness").to_string_lossy().into_owned()),
		);
	}

	let mut report = CaseReport {
		name,
		runs: Vec::new(),
		failures: Vec::new(),
	};
	let runs = if expected.error.is_some() {
		1
	} else {
		expected.runs.max(1)
	};
	let mut first_counts = None;
	for run in 1..=runs {
		let result = adapter.sync(&db, &run_config).await?;
		let error = result.error.clone();
		report.runs.push(result);
		match (&expected.error, error) {
			(Some(wanted), Some(error)) if error.contains(wanted.as_str()) => {}
			(Some(wanted), Some(error)) => report.failures.push(format!(
				"run {run} failed with \"{error}\", expected \"{wanted}\""
			)),
			(Some(wanted), None) => report.failures.push(format!(
				"run {run} succeeded, expected an error with \"{wanted}\""
			)),
			(None, Some(error)) => {
				report.failures.push(format!("run {run} failed: {error}"));
				break;
			}
			(None, None) => {}
		}
		let counts = count_models(&db, schema.models.keys()).await?;
		match &first_counts {
			None => first_counts = Some(counts),
			Some(first) if expected.idempotent && *first != counts => {
				report.failures.push(format!(
					"run {run} changed the record counts from {} to {} on unchanged input",
					format_counts(first),
					format_counts(&counts)
				));
			}
			Some(_) => {}
		}
	}

	check_final_state(&db, schema, &expected, &mut report.failures).await?;
	Ok(report)
}

async fn check_final_state(
	db: &crate::db::SourceDb,
	schema: &crate::schema::DataTypeSchema,
	expected: &Expected,
	failures: &mut Vec<String>,
) -> Result<()> {
	for (model, wanted) in &expected.counts {
		if !schema.models.contains_key(model) {
			failures.push(format!("expected.json counts unknown model \"{model}\""));
			continue;
		}
		let count = db.count(model).await?;
		if count != *wanted {
			failures.push(format!("{model}: {count} records, expected {wanted}"));
		}
	}
	let mut stored = BTreeMap::new();
	let models = expected
		.records
		.iter()
		.map(|record| &record.model)
		.chain(expected.absent.iter().map(|record| &record.model));
	for model in models {
		if stored.contains_key(model) {
			continue;
		}
		if !schema.models.contains_key(model) {
			failures.push(format!("expected.json names unknown model \"{model}\""));
			stored.insert(model.clone(), Vec::new());
			continue;
		}
		stored.insert(model.clone(), db.records(model).await?);
	}
	for record in &expected.records {
		let Some(found) = stored[&record.model]
			.iter()
			.find(|stored| stored.external_id == record.external_id)
		else {
			failures.push(format!(
				"{} \"{}\" is missing",
				record.model, record.external_id
			));
			continue;
		};
		for (field, wanted) in &record.fields {
			let Some(actual) = found.fields.get(field) else {
				failures.push(format!(
					"{} \"{}\": the model has no field \"{field}\"",
					record.model, record.external_id
				));
				continue;
			};
			if !matches_stored(wanted, actual) {
				failures.push(format!(
					"{} \"{}\": {field} is {actual}, expected {wanted}",
					record.model, record.external_id
				));
			}
		}
	}
	for record in &expected.absent {
		if stored[&record.model]
			.iter()
			.any(|stored| stored.external_id == record.external_id)
		{
			failures.push(format!(
				"{} \"{}\" should not exist",
				record.model, record.external_id
			));
		}
	}
	Ok(())
}

/// Whether an expected JSON value matches the text a source database stored for it.
fn matches_stored(wanted: &serde_json::Value, stored: &serde_json::Value) -> bool {
	match (wanted, stored) {
		(serde_json::Value::Null, serde_json::Value::Null) => true,
		(serde_json::Value::Null, serde_json::Value::String(text)) => text.is_empty(),
		(_, serde_json::Value::Null) => false,
		(serde_json::Value::String(wanted), serde_json::Value::String(text)) => wanted == text,
		(wanted, serde_json::Value::String(text)) => {
			serde_json::from_str::<serde_json::Value>(text).is_ok_and(|stored| stored == *wanted)
		}
		(wanted, stored) => wanted == stored,
	}
}

async fn count_models<'a>(
	db: &crate::db::SourceDb,
	models: impl Iterator<Item = &'a String>,
) -> Result<BTreeMap<String, i64>> {
	let mut counts = BTreeMap::new();
	for model in models {
		counts.insert(model.clone(), db.count(model).await?);
	}
	Ok(counts)
}

fn format_counts(counts: &BTreeMap<String, i64>) -> String {
	counts
		.iter()
		.map(|(model, count)| format!("{model}={count}"))
		.collect::<Vec<_>>()
		.join(", ")
}

/// Replaces `{fixture}` in every string of `value` with the case folder's path.
fn substitute_fixture(value: serde_json::Value, case_dir: &Path) -> serde_json::Value {
	let fixture = case_dir.to_string_lossy();
	match value {
		serde_json::Value::String(text) => {
			serde_json::Value::String(text.replace("{fixture}", &fixture))
		}
		serde_json::Value::Array(items) => serde_json::Value::Array(
			items
				.into_iter()
				.map(|item| substitute_fixture(item, case_dir))
				.collect(),
		),
		serde_json::Value::Object(object) => serde_json::Value::Object(
			object
				.into_iter()
				.map(|(key, item)| (key, substitute_fixture(item, case_dir)))
				.collect(),
		),
		other => other,
	}
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
	let text = std::fs::read_to_string(path)?;
	serde_json::from_str(&text)
		.map_err(|error| Error::Other(format!("{}: {error}", path.display())))
}

fn read_optional_json(path: &Path) -> Result<Option<serde_json::Value>> {
	if path.is_file() {
		read_json(path).map(Some)
	} else {
		Ok(None)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use serde_json::json;

	#[test]
	fn expected_values_match_the_text_the_database_stores() {
		assert!(matches_stored(&json!("a"), &json!("a")));
		assert!(matches_stored(&json!(1), &json!("1")));
		assert!(matches_stored(&json!(true), &json!("true")));
		assert!(matches_stored(&json!(["x"]), &json!("[\"x\"]")));
		assert!(matches_stored(&json!(null), &json!(null)));
		assert!(matches_stored(&json!(null), &json!("")));
		assert!(!matches_stored(&json!("a"), &json!(null)));
		assert!(!matches_stored(&json!(2), &json!("1")));
		assert!(!matches_stored(&json!("1"), &json!("01")));
	}

	#[test]
	fn fixture_placeholder_becomes_the_case_folder() {
		let config = json!({"vault": "{fixture}/vault", "list": ["{fixture}"], "n": 3});
		assert_eq!(
			substitute_fixture(config, Path::new("/cases/basic")),
			json!({"vault": "/cases/basic/vault", "list": ["/cases/basic"], "n": 3})
		);
	}
}
