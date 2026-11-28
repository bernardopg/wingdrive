//! Contract harness tests: the fixture adapter in `tests/fixtures/harness/notes` passes its
//! cases, and the harness reports adapters that break the contract.
//!
//! The adapter is a short Python script, so these tests need `python3` on PATH.

use std::path::{Path, PathBuf};

use wing_archive::harness::{case_dirs, test_adapter};

fn fixture_adapter() -> PathBuf {
	Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/harness/notes")
}

/// Copies the fixture adapter into `dir`, so a test can break it.
fn copy_adapter(dir: &Path) -> PathBuf {
	fn copy(from: &Path, to: &Path) {
		std::fs::create_dir_all(to).unwrap();
		for entry in std::fs::read_dir(from).unwrap() {
			let path = entry.unwrap().path();
			let target = to.join(path.file_name().unwrap());
			if path.is_dir() {
				copy(&path, &target);
			} else {
				std::fs::copy(&path, &target).unwrap();
			}
		}
	}
	let adapter = dir.join("notes");
	copy(&fixture_adapter(), &adapter);
	adapter
}

#[tokio::test]
async fn the_fixture_adapter_passes_its_cases() {
	let reports = test_adapter(&fixture_adapter()).await.unwrap();
	let names: Vec<_> = reports.iter().map(|report| report.name.as_str()).collect();
	assert_eq!(names, ["missing-folder", "two-notes"]);
	for report in &reports {
		assert!(report.passed(), "{}: {:?}", report.name, report.failures);
	}
	let two_notes = &reports[1];
	assert_eq!(two_notes.runs.len(), 2);
	assert_eq!(two_notes.runs[0].records_upserted, 2);
	let missing = &reports[0];
	assert_eq!(missing.runs.len(), 1);
	assert!(missing.runs[0].error.is_some());
}

#[tokio::test]
async fn an_adapter_that_adds_records_on_an_unchanged_rerun_fails() {
	let tmp = tempfile::tempdir().unwrap();
	let adapter = copy_adapter(tmp.path());
	// Gives every run its own ids, as an adapter that keys records by run time would.
	let script = std::fs::read_to_string(adapter.join("sync.py")).unwrap();
	let script = script.replace(
		r#""external_id": name,"#,
		r#""external_id": f"{name}-{len(seen)}","#,
	);
	std::fs::write(adapter.join("sync.py"), script).unwrap();
	std::fs::remove_dir_all(adapter.join("fixtures/missing-folder")).unwrap();

	let reports = test_adapter(&adapter).await.unwrap();
	let failures = &reports[0].failures;
	assert!(
		failures
			.iter()
			.any(|failure| failure.contains("changed the record counts from note=2 to note=4")),
		"{failures:?}"
	);
	assert!(
		failures
			.iter()
			.any(|failure| failure.contains("\"groceries.txt\" is missing")),
		"{failures:?}"
	);
}

#[tokio::test]
async fn wrong_fields_and_unexpected_errors_are_reported() {
	let tmp = tempfile::tempdir().unwrap();
	let adapter = copy_adapter(tmp.path());
	std::fs::write(
		adapter.join("fixtures/two-notes/expected.json"),
		r#"{"counts": {"note": 3}, "records": [
			{"model": "note", "external_id": "todo.txt", "fields": {"words": 4, "colour": "red"}}
		], "absent": [{"model": "note", "external_id": "groceries.txt"}]}"#,
	)
	.unwrap();
	std::fs::write(
		adapter.join("fixtures/missing-folder/expected.json"),
		r#"{"counts": {"note": 0}}"#,
	)
	.unwrap();

	let reports = test_adapter(&adapter).await.unwrap();
	let missing = &reports[0].failures;
	assert_eq!(missing.len(), 1, "{missing:?}");
	assert!(missing[0].starts_with("run 1 failed: "), "{missing:?}");
	assert!(missing[0].ends_with("is not a folder"), "{missing:?}");
	let two_notes = &reports[1].failures;
	assert_eq!(
		two_notes,
		&[
			"note: 2 records, expected 3",
			// Fields are checked in key order.
			"note \"todo.txt\": the model has no field \"colour\"",
			"note \"todo.txt\": words is \"3\", expected 4",
			"note \"groceries.txt\" should not exist",
		]
	);
}

#[tokio::test]
async fn an_adapter_without_cases_is_an_error() {
	let tmp = tempfile::tempdir().unwrap();
	let adapter = copy_adapter(tmp.path());
	std::fs::remove_dir_all(adapter.join("fixtures")).unwrap();
	assert!(case_dirs(&adapter).unwrap().is_empty());
	let error = test_adapter(&adapter).await.unwrap_err().to_string();
	assert!(error.contains("has no test cases"), "{error}");
}
