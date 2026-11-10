//! Integration tests for the indexing rules engine

use std::collections::HashSet;
use tempfile::TempDir;
use wing_core::ops::indexing::rules::{
	build_default_ruler, IndexerRule, IndexerRuler, RulePerKind, RuleToggles, RulerDecision,
};

fn touch(path: &std::path::Path) {
	std::fs::create_dir_all(path.parent().unwrap()).ok();
	std::fs::write(path, b"").unwrap();
}

#[tokio::test]
async fn test_gitignore_basic() {
	let tmp = TempDir::new().unwrap();
	let root = tmp.path();

	// Simulate a git repo
	std::fs::create_dir_all(root.join(".git")).unwrap();

	// Create a directory with a .gitignore
	let dir = root.join("a");
	std::fs::create_dir_all(&dir).unwrap();
	let gitignore = dir.join(".gitignore");
	std::fs::write(&gitignore, b"*.log\n!keep.log\n").unwrap();

	let f_ignored = dir.join("test.log");
	let f_kept = dir.join("keep.log");
	touch(&f_ignored);
	touch(&f_kept);

	let toggles = RuleToggles {
		no_system_files: false,
		no_hidden: false,
		no_git: false,
		gitignore: true,
		only_images: false,
		no_dev_dirs: false,
	};
	// current should be the directory containing the files (like discovery)
	let ruler = build_default_ruler(toggles, root, &dir).await;

	let d1 = ruler
		.evaluate_path(&f_ignored, &std::fs::metadata(&f_ignored).unwrap())
		.await
		.unwrap();
	assert!(
		matches!(d1, RulerDecision::Reject),
		"ignored file should reject"
	);

	let d2 = ruler
		.evaluate_path(&f_kept, &std::fs::metadata(&f_kept).unwrap())
		.await
		.unwrap();
	assert!(
		matches!(d2, RulerDecision::Accept),
		"negated pattern should accept"
	);
}

#[tokio::test]
async fn test_conflict_precedence_and_children_rules() {
	let tmp = TempDir::new().unwrap();
	let root = tmp.path();

	let file_txt = root.join("doc.txt");
	touch(&file_txt);

	// Build a custom ruler with both accept and reject globs matching *.txt
	let accept = RulePerKind::new_accept_files_by_globs_str(["**/*.txt"]).unwrap();
	let reject = RulePerKind::new_reject_files_by_globs_str(["**/*.txt"]).unwrap();
	let rule = IndexerRule {
		id: None,
		name: "conflict".to_string(),
		default: true,
		rules: vec![accept, reject],
		date_created: chrono::Utc::now(),
		date_modified: chrono::Utc::now(),
	};
	let ruler = IndexerRuler::new(vec![rule]);

	let d = ruler
		.evaluate_path(&file_txt, &std::fs::metadata(&file_txt).unwrap())
		.await
		.unwrap();
	// Reject should win over accept
	assert!(matches!(d, RulerDecision::Reject));

	// Children directory rules
	let dir = root.join("project");
	std::fs::create_dir_all(dir.join("src")).unwrap();

	let mut set: HashSet<String> = HashSet::new();
	set.insert("src".to_string());
	let child_reject = RulePerKind::RejectIfChildrenDirectoriesArePresent(set.clone());
	let rule_children = IndexerRule {
		id: None,
		name: "child_reject".to_string(),
		default: true,
		rules: vec![child_reject],
		date_created: chrono::Utc::now(),
		date_modified: chrono::Utc::now(),
	};
	let ruler_children = IndexerRuler::new(vec![rule_children]);
	// Discovery callers also consume the aggregated rule results directly.
	let acc = ruler_children
		.apply_all(&dir, &std::fs::metadata(&dir).unwrap())
		.await
		.unwrap();
	assert!(wing_core::ops::indexing::rules::IndexerRuler::rejected_by_children_directories(&acc));

	// A directory without the child should be accepted
	let other = root.join("other");
	std::fs::create_dir_all(&other).unwrap();
	let acc_other = ruler_children
		.apply_all(&other, &std::fs::metadata(&other).unwrap())
		.await
		.unwrap();
	assert!(
		!wing_core::ops::indexing::rules::IndexerRuler::rejected_by_children_directories(
			&acc_other
		)
	);
}

#[tokio::test]
async fn test_rule_toggles_basic() {
	let tmp = TempDir::new().unwrap();
	let root = tmp.path();

	// Files and dirs
	let path_normal = root.join("file.txt");
	touch(&path_normal);

	let path_hidden = root.join(".hidden");
	touch(&path_hidden);

	let img_jpg = root.join("photo.jpg");
	touch(&img_jpg);

	let dev_dir = root.join("node_modules");
	std::fs::create_dir_all(&dev_dir).unwrap();

	// 1) no_dev_dirs = true should reject node_modules directory
	let toggles = RuleToggles {
		no_system_files: false,
		no_hidden: false,
		no_git: false,
		gitignore: false,
		only_images: false,
		no_dev_dirs: true,
	};
	let ruler = build_default_ruler(toggles, root, root).await;
	let decision = ruler
		.evaluate_path(&dev_dir, &std::fs::metadata(&dev_dir).unwrap())
		.await
		.unwrap();
	assert!(matches!(decision, RulerDecision::Reject));

	// Normal file should be accepted
	let decision = ruler
		.evaluate_path(&path_normal, &std::fs::metadata(&path_normal).unwrap())
		.await
		.unwrap();
	assert!(matches!(decision, RulerDecision::Accept));

	// 2) no_hidden = true should reject hidden files
	let toggles = RuleToggles {
		no_system_files: false,
		no_hidden: true,
		no_git: false,
		gitignore: false,
		only_images: false,
		no_dev_dirs: false,
	};
	let ruler = build_default_ruler(toggles, root, root).await;
	let decision = ruler
		.evaluate_path(&path_hidden, &std::fs::metadata(&path_hidden).unwrap())
		.await
		.unwrap();
	assert!(matches!(decision, RulerDecision::Reject));

	// 3) only_images = true should accept images and reject non-images
	let toggles = RuleToggles {
		no_system_files: false,
		no_hidden: false,
		no_git: false,
		gitignore: false,
		only_images: true,
		no_dev_dirs: false,
	};
	let ruler = build_default_ruler(toggles, root, root).await;
	let decision_img = ruler
		.evaluate_path(&img_jpg, &std::fs::metadata(&img_jpg).unwrap())
		.await
		.unwrap();
	assert!(matches!(decision_img, RulerDecision::Accept));
	let decision_txt = ruler
		.evaluate_path(&path_normal, &std::fs::metadata(&path_normal).unwrap())
		.await
		.unwrap();
	assert!(matches!(decision_txt, RulerDecision::Reject));

	// 4) no_system_files = true should reject platform-specific system files
	let toggles = RuleToggles {
		no_system_files: true,
		no_hidden: false,
		no_git: false,
		gitignore: false,
		only_images: false,
		no_dev_dirs: false,
	};
	let _ruler = build_default_ruler(toggles, root, root).await;
	#[cfg(target_os = "macos")]
	{
		let ds_store = root.join(".DS_Store");
		touch(&ds_store);
		let decision = _ruler
			.evaluate_path(&ds_store, &std::fs::metadata(&ds_store).unwrap())
			.await
			.unwrap();
		assert!(matches!(decision, RulerDecision::Reject));
	}
	#[cfg(target_os = "windows")]
	{
		let thumbs = root.join("Thumbs.db");
		touch(&thumbs);
		let decision = _ruler
			.evaluate_path(&thumbs, &std::fs::metadata(&thumbs).unwrap())
			.await
			.unwrap();
		assert!(matches!(decision, RulerDecision::Reject));
	}
}

fn custom_rule(rules: Vec<RulePerKind>) -> IndexerRule {
	let timestamp = chrono::DateTime::from_timestamp(0, 0).unwrap();
	IndexerRule {
		id: None,
		name: "contract".into(),
		default: false,
		rules,
		date_created: timestamp,
		date_modified: timestamp,
	}
}

#[tokio::test]
async fn child_directory_rejection_reaches_the_public_evaluator() {
	let tmp = TempDir::new().unwrap();
	std::fs::create_dir(tmp.path().join("cache")).unwrap();
	let ruler = IndexerRuler::new(vec![custom_rule(vec![
		RulePerKind::RejectIfChildrenDirectoriesArePresent(HashSet::from(["cache".into()])),
	])]);
	assert_eq!(
		ruler
			.evaluate_path(tmp.path(), &std::fs::metadata(tmp.path()).unwrap())
			.await
			.unwrap(),
		RulerDecision::Reject
	);
	std::fs::remove_dir(tmp.path().join("cache")).unwrap();
	touch(&tmp.path().join("cache"));
	assert_eq!(
		ruler
			.evaluate_path(tmp.path(), &std::fs::metadata(tmp.path()).unwrap())
			.await
			.unwrap(),
		RulerDecision::Accept
	);
}

#[tokio::test]
async fn child_rules_distinguish_files_directories_and_missing_paths() {
	use wing_core::ops::indexing::rules::{Error, RuleKind};

	let tmp = TempDir::new().unwrap();
	let root = tmp.path();
	let accept = RulePerKind::AcceptIfChildrenDirectoriesArePresent(HashSet::from(["src".into()]));
	let reject = RulePerKind::RejectIfChildrenDirectoriesArePresent(HashSet::from(["src".into()]));
	touch(&root.join("src"));
	let file_meta = std::fs::metadata(root.join("src")).unwrap();
	assert_eq!(
		accept.apply(root.join("src"), &file_meta).await.unwrap(),
		(RuleKind::AcceptIfChildrenDirectoriesArePresent, false)
	);
	assert_eq!(
		reject.apply(root.join("src"), &file_meta).await.unwrap(),
		(RuleKind::RejectIfChildrenDirectoriesArePresent, true)
	);
	let dir_meta = std::fs::metadata(root).unwrap();
	assert!(!accept.apply(root, &dir_meta).await.unwrap().1);
	assert!(reject.apply(root, &dir_meta).await.unwrap().1);
	std::fs::remove_file(root.join("src")).unwrap();
	std::fs::create_dir(root.join("src")).unwrap();
	assert!(accept.apply(root, &dir_meta).await.unwrap().1);
	assert!(!reject.apply(root, &dir_meta).await.unwrap().1);
	for rule in [accept, reject] {
		assert!(matches!(
			rule.apply(root.join("missing"), &dir_meta).await,
			Err(Error::Io(_))
		));
	}
}

#[tokio::test]
async fn serialized_rules_rebuild_globsets_and_keep_child_rules() {
	let tmp = TempDir::new().unwrap();
	let file = tmp.path().join("café.txt");
	touch(&file);
	let metadata = std::fs::metadata(&file).unwrap();
	for rule in [
		RulePerKind::new_accept_files_by_globs_str(["**/*.txt", "**/文件.*"]).unwrap(),
		RulePerKind::new_reject_files_by_globs_str(["**/*.txt"]).unwrap(),
		RulePerKind::AcceptIfChildrenDirectoriesArePresent(HashSet::from(["src".into()])),
		RulePerKind::RejectIfChildrenDirectoriesArePresent(HashSet::from(["cache".into()])),
	] {
		let expected = rule.apply(&file, &metadata).await.unwrap();
		let decoded: RulePerKind =
			serde_json::from_value(serde_json::to_value(&rule).unwrap()).unwrap();
		assert_eq!(decoded.apply(&file, &metadata).await.unwrap(), expected);
		let decoded: RulePerKind =
			rmp_serde::from_slice(&rmp_serde::to_vec_named(&rule).unwrap()).unwrap();
		assert_eq!(decoded.apply(&file, &metadata).await.unwrap(), expected);
	}
}

#[test]
fn invalid_globs_and_serialized_rules_return_errors() {
	for pattern in ["[", "{one,two"] {
		assert!(RulePerKind::new_accept_files_by_globs_str([pattern]).is_err());
		assert!(RulePerKind::new_reject_files_by_globs_str([pattern]).is_err());
	}
	for value in [
		serde_json::json!({"Unknown": []}),
		serde_json::json!({"AcceptFilesByGlob": ["["]}),
		serde_json::json!({"RejectFilesByGlob": ["["]}),
		serde_json::json!({"AcceptIfChildrenDirectoriesArePresent": [1]}),
		serde_json::json!({"RejectIfChildrenDirectoriesArePresent": null}),
	] {
		assert!(
			serde_json::from_value::<RulePerKind>(value.clone()).is_err(),
			"{value}"
		);
	}
}

#[tokio::test]
async fn cloned_rulers_keep_base_rules_but_drop_directory_specific_extras() {
	let tmp = TempDir::new().unwrap();
	let path = tmp.path().join("café.log");
	touch(&path);
	let metadata = std::fs::metadata(&path).unwrap();
	let mut ruler = IndexerRuler::new(vec![custom_rule(vec![
		RulePerKind::new_accept_files_by_globs_str(["**/*.log"]).unwrap(),
	])]);
	ruler.extend([custom_rule(vec![
		RulePerKind::new_reject_files_by_globs_str(["**/café.log"]).unwrap(),
	])]);
	assert_eq!(
		ruler.evaluate_path(&path, &metadata).await.unwrap(),
		RulerDecision::Reject
	);
	assert_eq!(
		ruler.clone().evaluate_path(&path, &metadata).await.unwrap(),
		RulerDecision::Accept
	);
	let other = tmp.path().join("notes.txt");
	touch(&other);
	assert_eq!(
		ruler
			.clone()
			.evaluate_path(&other, &std::fs::metadata(&other).unwrap())
			.await
			.unwrap(),
		RulerDecision::Reject
	);
}

#[cfg(unix)]
#[tokio::test]
async fn child_rules_skip_non_utf8_names_and_do_not_follow_directory_links() {
	use std::os::unix::{ffi::OsStrExt, fs::symlink};

	let tmp = TempDir::new().unwrap();
	let root = tmp.path().join("root");
	let target = tmp.path().join("target");
	std::fs::create_dir(&root).unwrap();
	std::fs::create_dir(&target).unwrap();
	std::fs::create_dir(root.join(std::ffi::OsStr::from_bytes(b"caf\xff"))).unwrap();
	symlink(&target, root.join("src")).unwrap();
	symlink("missing", root.join("dangling")).unwrap();
	let metadata = std::fs::metadata(&root).unwrap();
	let accept = RulePerKind::AcceptIfChildrenDirectoriesArePresent(HashSet::from([
		"src".into(),
		"café".into(),
	]));
	assert!(!accept.apply(&root, &metadata).await.unwrap().1);
	let reject = RulePerKind::RejectIfChildrenDirectoriesArePresent(HashSet::from([
		"src".into(),
		"café".into(),
	]));
	assert!(reject.apply(&root, &metadata).await.unwrap().1);
}

#[tokio::test]
async fn git_exclude_rules_stay_inside_the_repository_and_respect_location_boundaries() {
	use wing_core::ops::indexing::rules::GitIgnoreRules;

	let tmp = TempDir::new().unwrap();
	let repo = tmp.path().join("repository");
	std::fs::create_dir_all(repo.join(".git/info")).unwrap();
	std::fs::write(repo.join(".git/info/exclude"), b"*.secret\n").unwrap();
	let nested = repo.join("nested");
	std::fs::create_dir(&nested).unwrap();
	let ignored = nested.join("data.secret");
	let kept = nested.join("data.txt");
	touch(&ignored);
	touch(&kept);
	let ruler = build_default_ruler(
		RuleToggles {
			gitignore: true,
			..RuleToggles::complete()
		},
		&repo,
		&nested,
	)
	.await;
	assert_eq!(
		ruler
			.evaluate_path(&ignored, &std::fs::metadata(&ignored).unwrap())
			.await
			.unwrap(),
		RulerDecision::Reject
	);
	assert_eq!(
		ruler
			.evaluate_path(&kept, &std::fs::metadata(&kept).unwrap())
			.await
			.unwrap(),
		RulerDecision::Accept
	);
	let outside = tmp.path().join("data.secret");
	touch(&outside);
	assert_eq!(
		ruler
			.evaluate_path(&outside, &std::fs::metadata(&outside).unwrap())
			.await
			.unwrap(),
		RulerDecision::Accept
	);
	assert!(GitIgnoreRules::get_rules_if_in_git_repo(&nested, &nested)
		.await
		.is_none());
}
