//! `files.preflight` against the jobs it describes.
//!
//! The plan and the copy job share `ops::files::plan`; these tests run both over
//! one fixture and check that what the plan announced is what the job did, and
//! that a delete plan counts the last copies of content from the index.

mod helpers;

use helpers::*;
use tokio::{fs, time::Duration};
use wing_core::{
	domain::addressing::{WingPath, WingPathBatch},
	infra::{api::SessionContext, query::LibraryQuery},
	location::IndexMode,
	ops::files::{
		copy::{
			action::FileConflictResolution,
			job::{CopyOptions, FileCopyJob},
		},
		plan::PlanDecision,
		query::preflight::{
			FilePreflightInput, FilePreflightOutput, FilePreflightQuery, PreflightOperation,
		},
	},
};

async fn preflight(
	harness: &IndexingHarness,
	input: FilePreflightInput,
) -> anyhow::Result<FilePreflightOutput> {
	let query = FilePreflightQuery::from_input(input)?;
	let mut session = SessionContext::device_session(
		wing_core::device::get_current_device_id(),
		wing_core::device::get_current_device_slug(),
	);
	session.current_library_id = Some(harness.library.id());
	Ok(query.execute(harness.core.context.clone(), session).await?)
}

async fn write(path: &std::path::Path, content: &str) -> std::io::Result<()> {
	if let Some(parent) = path.parent() {
		fs::create_dir_all(parent).await?;
	}
	fs::write(path, content).await
}

#[tokio::test]
async fn copy_plan_matches_what_the_job_does() -> anyhow::Result<()> {
	let harness = IndexingHarnessBuilder::new("preflight_copy")
		.disable_watcher()
		.build()
		.await?;
	let root = harness.temp_path().join("preflight_copy");
	let source = root.join("source");
	write(&source.join("kept.txt"), "new").await?;
	write(&source.join("fresh.txt"), "fresh!").await?;

	for (policy, folder) in [
		(FileConflictResolution::Skip, "skip"),
		(FileConflictResolution::AutoModifyName, "rename"),
	] {
		let dest = root.join(folder);
		write(&dest.join("kept.txt"), "original").await?;
		let sources = vec![
			WingPath::local(source.join("kept.txt")),
			WingPath::local(source.join("fresh.txt")),
		];

		let plan = preflight(
			&harness,
			FilePreflightInput {
				operation: PreflightOperation::Copy,
				sources: sources.clone(),
				destination: Some(WingPath::local(&dest)),
				on_conflict: Some(policy),
			},
		)
		.await?;
		assert_eq!((plan.files, plan.bytes, plan.conflicts), (2, 9, 1));
		let decisions: Vec<_> = plan
			.items
			.iter()
			.map(|i| i.decision.clone().unwrap())
			.collect();

		harness
			.library
			.jobs()
			.dispatch(
				FileCopyJob::new(WingPathBatch::new(sources), WingPath::local(&dest)).with_options(
					CopyOptions {
						conflict_resolution: Some(policy),
						..CopyOptions::default()
					},
				),
			)
			.await?
			.wait()
			.await?;

		assert_eq!(decisions[1], PlanDecision::Write);
		assert_eq!(fs::read_to_string(dest.join("fresh.txt")).await?, "fresh!");
		assert_eq!(fs::read_to_string(dest.join("kept.txt")).await?, "original");
		match &decisions[0] {
			PlanDecision::Skip => {
				assert_eq!(policy, FileConflictResolution::Skip);
				assert!(!dest.join("kept (1).txt").exists());
			}
			PlanDecision::Rename(path) => {
				assert_eq!(policy, FileConflictResolution::AutoModifyName);
				assert_eq!(path, &dest.join("kept (1).txt"));
				assert_eq!(fs::read_to_string(path).await?, "new");
			}
			other => panic!("unexpected decision {other:?}"),
		}
	}

	// Nothing chosen yet: the conflict is planned as a question, and nothing moved.
	let plan = preflight(
		&harness,
		FilePreflightInput {
			operation: PreflightOperation::Move,
			sources: vec![WingPath::local(source.join("kept.txt"))],
			destination: Some(WingPath::local(root.join("skip"))),
			on_conflict: None,
		},
	)
	.await?;
	assert_eq!(plan.items[0].decision, Some(PlanDecision::Ask));
	assert!(source.join("kept.txt").exists());

	harness.shutdown().await?;
	Ok(())
}

#[tokio::test]
async fn delete_plan_counts_last_copies() -> anyhow::Result<()> {
	let harness = IndexingHarnessBuilder::new("preflight_delete")
		.disable_watcher()
		.build()
		.await?;
	let location = harness.create_test_location("preflight_delete").await?;
	location.write_file("twin-a.txt", "same content").await?;
	location.write_file("twin-b.txt", "same content").await?;
	location.write_file("only.txt", "unique content").await?;
	location
		.write_file("folder/inside.txt", "inside content")
		.await?;
	location.index("Preflight", IndexMode::Content).await?;
	tokio::time::sleep(Duration::from_millis(500)).await;

	let root = location.path().to_path_buf();
	let delete = |paths: Vec<std::path::PathBuf>| FilePreflightInput {
		operation: PreflightOperation::Delete,
		sources: paths.into_iter().map(WingPath::local).collect(),
		destination: None,
		on_conflict: None,
	};

	// One twin goes, the other keeps the content.
	let plan = preflight(&harness, delete(vec![root.join("twin-a.txt")])).await?;
	assert_eq!((plan.last_copies, plan.unknown_files), (Some(0), Some(0)));

	// Both twins and the unique file go: all three are last copies.
	let plan = preflight(
		&harness,
		delete(vec![
			root.join("twin-a.txt"),
			root.join("twin-b.txt"),
			root.join("only.txt"),
		]),
	)
	.await?;
	assert_eq!(plan.files, 3);
	assert_eq!(plan.last_copies, Some(3));

	// A folder counts the files below it.
	let plan = preflight(&harness, delete(vec![root.join("folder")])).await?;
	assert_eq!((plan.files, plan.last_copies), (1, Some(1)));

	// A file the index has never seen is unknown, not safe.
	let stray = harness.temp_path().join("stray.txt");
	write(&stray, "not indexed").await?;
	let plan = preflight(&harness, delete(vec![stray])).await?;
	assert_eq!((plan.last_copies, plan.unknown_files), (Some(0), Some(1)));

	harness.shutdown().await?;
	Ok(())
}
