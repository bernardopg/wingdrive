//! Verifies duplicate detection against ordinary files and overlapping selections.

use std::collections::HashSet;

use tempfile::TempDir;
use wing_core::{
	domain::addressing::{WingPath, WingPathBatch},
	infra::job::output::JobOutput,
	ops::files::duplicate_detection::{DetectionMode, DuplicateDetectionJob},
	Core,
};

#[tokio::test]
async fn overlapping_roots_do_not_list_the_same_file_as_two_copies(
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
	let root = TempDir::new()?;
	let core = Core::new(root.path().join("daemon")).await?;
	let library = core.libraries.list().await.remove(0);
	let scanned = root.path().join("scan");
	let nested = scanned.join("nested");
	tokio::fs::create_dir_all(&nested).await?;
	let first = nested.join("copy.txt");
	let second = scanned.join("copy.txt");
	let different = scanned.join("different.txt");
	let contents = vec![b'a'; 2048];
	tokio::fs::write(&first, &contents).await?;
	tokio::fs::write(&second, &contents).await?;
	tokio::fs::write(&different, vec![b'b'; contents.len()]).await?;
	#[cfg(unix)]
	tokio::fs::symlink(&first, scanned.join("shortcut.txt")).await?;
	let batch = WingPathBatch {
		paths: [scanned, nested.clone(), first.clone(), nested.join("..")]
			.into_iter()
			.map(WingPath::local)
			.collect(),
	};

	for (mode, expected) in [
		(
			DetectionMode::ContentHash,
			vec![first.clone(), second.clone()],
		),
		(DetectionMode::DeepScan, vec![first.clone(), second.clone()]),
		(
			DetectionMode::NameAndSize,
			vec![first.clone(), second.clone()],
		),
		(
			DetectionMode::SizeOnly,
			vec![first.clone(), second.clone(), different.clone()],
		),
	] {
		let job = library
			.jobs()
			.dispatch(DuplicateDetectionJob::new(batch.clone(), mode))
			.await?;
		let JobOutput::DuplicateDetection {
			groups,
			total_duplicates,
			potential_savings,
			..
		} = tokio::time::timeout(std::time::Duration::from_secs(20), job.wait()).await??
		else {
			panic!("unexpected job output");
		};

		assert_eq!(groups.len(), 1);
		let extra_copies = expected.len() - 1;
		assert_eq!(
			total_duplicates, extra_copies,
			"overlapping roots overcounted copies"
		);
		assert_eq!(potential_savings, (extra_copies * contents.len()) as u64);
		assert_eq!(groups[0].paths.len(), expected.len());
		let returned: HashSet<_> = groups[0]
			.paths
			.iter()
			.filter_map(WingPath::as_local_path)
			.collect();
		assert_eq!(
			returned,
			expected
				.iter()
				.map(std::path::PathBuf::as_path)
				.collect::<HashSet<_>>()
		);
	}

	core.shutdown().await?;
	Ok(())
}
