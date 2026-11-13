//! Option parsing, results, URIs, `portals.conf` editing and the broker.

mod bus;
mod portals_conf;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use zbus::{OwnedVariant as OwnedValue, Variant as Value};

use super::options::*;
use super::*;

fn owned(value: Value<'_>) -> OwnedValue {
	OwnedValue::try_from(value).unwrap()
}

fn options(entries: Vec<(&str, Value<'_>)>) -> HashMap<String, OwnedValue> {
	entries
		.into_iter()
		.map(|(key, value)| (key.to_string(), owned(value)))
		.collect()
}

fn nul_terminated(path: &str) -> Vec<u8> {
	let mut bytes = path.as_bytes().to_vec();
	bytes.push(0);
	bytes
}

fn images() -> (String, Vec<(u32, String)>) {
	(
		"Images".to_string(),
		vec![(0, "*.png".to_string()), (1, "image/jpeg".to_string())],
	)
}

#[test]
fn missing_options_take_the_spec_defaults() {
	let parsed = parse(PickerMode::Open, &HashMap::new()).unwrap();
	assert_eq!(parsed, PickerOptions::default());
	assert!(parsed.modal, "modal defaults to yes");
	assert!(!parsed.multiple);
	assert!(!parsed.directory);
}

#[test]
fn open_options_are_parsed() {
	let parsed = parse(
		PickerMode::Open,
		&options(vec![
			("accept_label", Value::from("_Upload")),
			("modal", Value::from(false)),
			("multiple", Value::from(true)),
			("directory", Value::from(true)),
			("filters", Value::from(vec![images()])),
			("current_filter", Value::from(images())),
			("current_folder", Value::from(nul_terminated("/tmp/a b"))),
			("handle_token", Value::from("ignored")),
		]),
	)
	.unwrap();
	assert_eq!(parsed.accept_label.as_deref(), Some("_Upload"));
	assert!(!parsed.modal);
	assert!(parsed.multiple);
	assert!(parsed.directory);
	assert_eq!(
		parsed.filters,
		[FileFilter {
			name: "Images".into(),
			rules: vec![
				FilterRule::Glob("*.png".into()),
				FilterRule::MimeType("image/jpeg".into())
			],
			globs: vec![],
		}]
	);
	assert_eq!(parsed.current_filter, Some(0));
	assert_eq!(parsed.current_folder, Some(PathBuf::from("/tmp/a b")));
}

#[test]
fn a_current_filter_outside_the_list_is_added_to_it() {
	let text = ("Text".to_string(), vec![(0u32, "*.txt".to_string())]);
	let parsed = parse(
		PickerMode::Open,
		&options(vec![
			("filters", Value::from(vec![images()])),
			("current_filter", Value::from(text)),
		]),
	)
	.unwrap();
	assert_eq!(parsed.filters.len(), 2);
	assert_eq!(parsed.current_filter, Some(1));
	assert_eq!(parsed.filters[1].name, "Text");
}

#[test]
fn save_options_suggest_a_name_and_folder() {
	let parsed = parse(
		PickerMode::Save,
		&options(vec![
			("current_name", Value::from("report.pdf")),
			("current_folder", Value::from(nul_terminated("/srv/out"))),
			("multiple", Value::from(true)),
		]),
	)
	.unwrap();
	assert_eq!(parsed.suggested_name().as_deref(), Some("report.pdf"));
	assert_eq!(parsed.current_folder, Some(PathBuf::from("/srv/out")));
	assert!(!parsed.multiple, "SaveFile has no multiple option");

	let existing = parse(
		PickerMode::Save,
		&options(vec![
			("current_name", Value::from("ignored.txt")),
			(
				"current_file",
				Value::from(nul_terminated("/srv/old/notes.txt")),
			),
		]),
	)
	.unwrap();
	assert_eq!(existing.suggested_name().as_deref(), Some("notes.txt"));
}

#[test]
fn the_start_folder_prefers_the_current_file_then_the_folder_then_home() {
	let dir = tempfile::tempdir().unwrap();
	let sub = dir.path().join("sub");
	std::fs::create_dir(&sub).unwrap();
	let home = Path::new("/home/someone");
	let mut parsed = PickerOptions::default();
	assert_eq!(parsed.start_folder(home), home);
	parsed.current_folder = Some(dir.path().join("missing"));
	assert_eq!(
		parsed.start_folder(home),
		home,
		"a missing folder falls back"
	);
	parsed.current_folder = Some(dir.path().to_path_buf());
	assert_eq!(parsed.start_folder(home), dir.path());
	parsed.current_file = Some(sub.join("a.txt"));
	assert_eq!(parsed.start_folder(home), sub);
}

#[test]
fn choices_are_parsed() {
	let choices = vec![
		(
			"encoding".to_string(),
			"Encoding".to_string(),
			vec![
				("utf8".to_string(), "Unicode".to_string()),
				("latin1".to_string(), "Western".to_string()),
			],
			"latin1".to_string(),
		),
		(
			"reencode".to_string(),
			"Reencode".to_string(),
			vec![],
			"false".to_string(),
		),
	];
	let parsed = parse(
		PickerMode::Save,
		&options(vec![("choices", Value::from(choices))]),
	)
	.unwrap();
	assert_eq!(parsed.choices.len(), 2);
	assert_eq!(parsed.choices[0].options[1].id, "latin1");
	assert_eq!(parsed.choices[0].initial, "latin1");
	assert!(parsed.choices[1].options.is_empty(), "a check box");
}

#[test]
fn save_files_names_must_stay_in_the_folder() {
	let files = |names: &[&str]| {
		Value::from(
			names
				.iter()
				.map(|name| nul_terminated(name))
				.collect::<Vec<_>>(),
		)
	};
	let parsed = parse(
		PickerMode::SaveFiles,
		&options(vec![
			("files", files(&["a.txt", "b.txt"])),
			("filters", Value::from(vec![images()])),
		]),
	)
	.unwrap();
	assert_eq!(parsed.files, ["a.txt", "b.txt"]);
	assert!(parsed.directory);
	assert!(parsed.filters.is_empty(), "SaveFiles has no filters");
	for bad in ["../escape", "a/b", "..", ""] {
		assert!(
			parse(
				PickerMode::SaveFiles,
				&options(vec![("files", files(&[bad]))])
			)
			.is_err(),
			"{bad:?}"
		);
	}
}

#[test]
fn wrong_types_and_bad_values_are_errors() {
	let cases = vec![
		("multiple", Value::from("yes")),
		("filters", Value::from("*.png")),
		("current_folder", Value::from("/tmp")),
		(
			"current_folder",
			Value::from(nul_terminated("relative/path")),
		),
		(
			"filters",
			Value::from(vec![("Bad".to_string(), vec![(7u32, "*.png".to_string())])]),
		),
	];
	for (key, value) in cases {
		let description = format!("{key}={value:?}");
		assert!(
			parse(PickerMode::Open, &options(vec![(key, value)])).is_err(),
			"{description}"
		);
	}
}

#[test]
fn uris_percent_encode_reserved_and_non_utf8_bytes() {
	use std::os::unix::ffi::OsStrExt;
	assert_eq!(
		file_uri(Path::new("/tmp/a b/c#d%.txt")),
		"file:///tmp/a%20b/c%23d%25.txt"
	);
	assert_eq!(file_uri(Path::new("/home/ç")), "file:///home/%C3%A7");
	let raw = std::ffi::OsStr::from_bytes(b"/tmp/\xff");
	assert_eq!(file_uri(Path::new(raw)), "file:///tmp/%FF");
	assert_eq!(file_uri(Path::new("/a-b_c.~d")), "file:///a-b_c.~d");
}

fn uris(results: &HashMap<String, OwnedValue>) -> Vec<String> {
	Vec::<String>::try_from(results["uris"].try_clone().unwrap()).unwrap()
}

#[test]
fn open_results_check_the_request() {
	let dir = tempfile::tempdir().unwrap();
	let a = dir.path().join("a.png");
	let b = dir.path().join("b.png");
	std::fs::write(&a, "x").unwrap();
	std::fs::write(&b, "x").unwrap();
	let mut request = PickerOptions {
		filters: vec![FileFilter::default_for_test()],
		..PickerOptions::default()
	};
	let open = |paths: Vec<PathBuf>| Selection::Open {
		paths,
		choices: vec![],
		current_filter: Some(0),
	};

	let single = results(PickerMode::Open, &request, open(vec![a.clone()])).unwrap();
	assert_eq!(uris(&single), [file_uri(&a)]);
	assert!(single.contains_key("current_filter"));
	assert!(!bool::try_from(single["writable"].try_clone().unwrap()).unwrap());

	assert!(results(PickerMode::Open, &request, open(vec![a.clone(), b.clone()])).is_err());
	assert!(results(PickerMode::Open, &request, open(vec![])).is_err());
	assert!(results(PickerMode::Open, &request, open(vec![dir.path().into()])).is_err());
	assert!(results(PickerMode::Open, &request, open(vec!["rel.png".into()])).is_err());

	request.multiple = true;
	let both = results(PickerMode::Open, &request, open(vec![a.clone(), b.clone()])).unwrap();
	assert_eq!(uris(&both).len(), 2);

	request.directory = true;
	assert!(results(PickerMode::Open, &request, open(vec![a.clone()])).is_err());
	let folder = results(PickerMode::Open, &request, open(vec![dir.path().into()])).unwrap();
	assert_eq!(uris(&folder), [file_uri(dir.path())]);
}

impl FileFilter {
	fn default_for_test() -> Self {
		Self {
			name: "PNG".into(),
			rules: vec![FilterRule::Glob("*.png".into())],
			globs: vec!["*.png".into()],
		}
	}
}

#[test]
fn saving_over_an_existing_file_needs_confirmation() {
	let dir = tempfile::tempdir().unwrap();
	let existing = dir.path().join("taken.txt");
	std::fs::write(&existing, "x").unwrap();
	let request = PickerOptions::default();
	let save = |path: PathBuf, overwrite| Selection::Save {
		path,
		overwrite,
		choices: vec![],
		current_filter: None,
	};

	let fresh = results(
		PickerMode::Save,
		&request,
		save(dir.path().join("new.txt"), false),
	)
	.unwrap();
	assert_eq!(uris(&fresh), [file_uri(&dir.path().join("new.txt"))]);
	assert!(
		!fresh.contains_key("writable"),
		"SaveFile returns no writable flag"
	);

	assert_eq!(
		results(PickerMode::Save, &request, save(existing.clone(), false)).unwrap_err(),
		SelectionError::Exists {
			path: existing.display().to_string()
		}
	);
	let confirmed = results(PickerMode::Save, &request, save(existing.clone(), true)).unwrap();
	assert_eq!(uris(&confirmed), [file_uri(&existing)]);

	assert!(results(PickerMode::Save, &request, save(dir.path().into(), true)).is_err());
	assert!(results(
		PickerMode::Save,
		&request,
		save(dir.path().join("missing/x.txt"), false)
	)
	.is_err());
}

#[test]
fn save_files_returns_free_names_in_the_chosen_folder() {
	let dir = tempfile::tempdir().unwrap();
	std::fs::write(dir.path().join("a.txt"), "x").unwrap();
	let request = PickerOptions {
		files: vec![
			"a.txt".into(),
			"b.txt".into(),
			"b.txt".into(),
			".hidden".into(),
		],
		directory: true,
		..PickerOptions::default()
	};
	let chosen = results(
		PickerMode::SaveFiles,
		&request,
		Selection::SaveFiles {
			folder: dir.path().into(),
			choices: vec![],
		},
	)
	.unwrap();
	let expected: Vec<String> = ["a (1).txt", "b.txt", "b (1).txt", ".hidden"]
		.iter()
		.map(|name| file_uri(&dir.path().join(name)))
		.collect();
	assert_eq!(uris(&chosen), expected);
	std::fs::write(dir.path().join(".hidden"), "x").unwrap();
	assert_eq!(unique_name(dir.path(), ".hidden", &[]), ".hidden (1)");
}

#[test]
fn a_selection_of_another_mode_is_refused() {
	let selection = Selection::SaveFiles {
		folder: "/tmp".into(),
		choices: vec![],
	};
	assert!(results(PickerMode::Open, &PickerOptions::default(), selection).is_err());
}

#[test]
fn only_requested_choices_are_returned() {
	let dir = tempfile::tempdir().unwrap();
	let request = PickerOptions {
		choices: vec![Choice {
			id: "encoding".into(),
			label: "Encoding".into(),
			options: vec![],
			initial: "false".into(),
		}],
		..PickerOptions::default()
	};
	let chosen = results(
		PickerMode::Save,
		&request,
		Selection::Save {
			path: dir.path().join("x"),
			overwrite: false,
			choices: vec![
				("encoding".into(), "true".into()),
				("unknown".into(), "x".into()),
			],
			current_filter: Some(4),
		},
	)
	.unwrap();
	let choices =
		Vec::<(String, String)>::try_from(chosen["choices"].try_clone().unwrap()).unwrap();
	assert_eq!(choices, [("encoding".to_string(), "true".to_string())]);
	assert!(
		!chosen.contains_key("current_filter"),
		"index 4 does not exist"
	);
}

#[test]
fn mime_types_become_globs_from_the_shared_mime_table() {
	let globs2 = "\
# comment
50:image/png:*.png
50:image/jpeg:*.jpg
50:image/jpeg:*.jpeg
50:text/plain:*.txt
45:image/jpeg:*.jpg
50:application/pdf:*.pdf:cs
";
	assert_eq!(mime_globs(globs2, "image/jpeg"), ["*.jpg", "*.jpeg"]);
	assert_eq!(mime_globs(globs2, "IMAGE/*"), ["*.png", "*.jpg", "*.jpeg"]);
	assert_eq!(mime_globs(globs2, "application/pdf"), ["*.pdf"]);
	assert!(mime_globs(globs2, "audio/ogg").is_empty());

	let mut filters = vec![FileFilter {
		name: "Pictures".into(),
		rules: vec![
			FilterRule::Glob("*.svg".into()),
			FilterRule::MimeType("image/png".into()),
		],
		globs: vec![],
	}];
	resolve_mime_globs(&mut filters, globs2);
	assert_eq!(filters[0].globs, ["*.svg", "*.png"]);
}

#[test]
fn outcomes_map_to_portal_responses() {
	assert_eq!(Outcome::Chosen(HashMap::new()).into_reply().0, 0);
	assert_eq!(Outcome::Cancelled.into_reply().0, 1);
	assert_eq!(Outcome::Failed("x".into()).into_reply().0, 2);
	assert!(Outcome::Cancelled.into_reply().1.is_empty());
}

#[test]
fn window_labels_carry_the_request_id() {
	assert_eq!(request_id("file-chooser-12"), Some("12"));
	assert_eq!(request_id("file-chooser-"), None);
	assert_eq!(request_id("main"), None);
	assert!(disabled_by_env(Some(" OFF ")));
	assert!(disabled_by_env(Some("0")));
	assert!(!disabled_by_env(Some("1")));
	assert!(!disabled_by_env(None));
}

/// Records what the broker asks of its windows.
#[derive(Clone, Default)]
pub(super) struct FakeHost {
	pub shown: Arc<Mutex<Vec<PickerRequest>>>,
	pub dismissed: Arc<Mutex<Vec<String>>>,
	pub fail: bool,
}

impl PickerHost for FakeHost {
	fn show(&self, request: &PickerRequest) -> Result<(), String> {
		if self.fail {
			return Err("no display".into());
		}
		self.shown.lock().unwrap().push(request.clone());
		Ok(())
	}

	fn dismiss(&self, id: &str) {
		self.dismissed.lock().unwrap().push(id.to_string());
	}
}

fn request(broker: &Broker, mode: PickerMode, options: PickerOptions) -> PickerRequest {
	PickerRequest {
		id: broker.next_id(),
		mode,
		app_id: "org.example.App".into(),
		parent_window: String::new(),
		title: "Open".into(),
		start_folder: "/".into(),
		suggested_name: None,
		options,
	}
}

async fn wait_for_show(host: &FakeHost) -> PickerRequest {
	for _ in 0..200 {
		if let Some(request) = host.shown.lock().unwrap().last().cloned() {
			return request;
		}
		tokio::time::sleep(std::time::Duration::from_millis(5)).await;
	}
	panic!("the picker never opened");
}

#[tokio::test]
async fn the_broker_answers_with_the_choice_and_closes_the_window() {
	let dir = tempfile::tempdir().unwrap();
	let file = dir.path().join("a.txt");
	std::fs::write(&file, "x").unwrap();
	let host = FakeHost::default();
	let broker = Broker::new(host.clone());
	let request = request(&broker, PickerMode::Open, PickerOptions::default());
	let task = tokio::spawn({
		let broker = broker.clone();
		async move { broker.run(request).await }
	});
	let shown = wait_for_show(&host).await;
	assert_eq!(broker.request(&shown.id), Some(shown.clone()));

	let wrong = broker.respond(
		&shown.id,
		Selection::Open {
			paths: vec![dir.path().into()],
			choices: vec![],
			current_filter: None,
		},
	);
	assert!(wrong.is_err());
	assert!(
		broker.is_pending(&shown.id),
		"a refused selection keeps the request open"
	);

	broker
		.respond(
			&shown.id,
			Selection::Open {
				paths: vec![file.clone()],
				choices: vec![],
				current_filter: None,
			},
		)
		.unwrap();
	let (code, results) = task.await.unwrap().into_reply();
	assert_eq!(code, 0);
	assert_eq!(uris(&results), [file_uri(&file)]);
	assert_eq!(
		*host.dismissed.lock().unwrap(),
		std::slice::from_ref(&shown.id)
	);
	assert!(broker.request(&shown.id).is_none());
}

#[tokio::test]
async fn cancelling_or_closing_answers_one_and_a_failed_window_answers_two() {
	let host = FakeHost::default();
	let broker = Broker::new(host.clone());
	let first = request(&broker, PickerMode::Save, PickerOptions::default());
	let task = tokio::spawn({
		let broker = broker.clone();
		async move { broker.run(first).await }
	});
	let shown = wait_for_show(&host).await;
	broker.cancel(&shown.id);
	assert_eq!(task.await.unwrap().into_reply().0, 1);
	assert_eq!(host.dismissed.lock().unwrap().len(), 1);

	host.shown.lock().unwrap().clear();
	let second = request(&broker, PickerMode::Save, PickerOptions::default());
	let task = tokio::spawn({
		let broker = broker.clone();
		async move { broker.run(second).await }
	});
	let shown = wait_for_show(&host).await;
	broker.window_closed(&shown.id);
	assert_eq!(task.await.unwrap().into_reply().0, 1);
	assert_eq!(
		host.dismissed.lock().unwrap().len(),
		1,
		"a closed window is not closed again"
	);
	broker.cancel(&shown.id);
	assert_eq!(
		host.dismissed.lock().unwrap().len(),
		1,
		"cancel after close is a no-op"
	);

	let failing = Broker::new(FakeHost {
		fail: true,
		..FakeHost::default()
	});
	let third = request(&failing, PickerMode::Open, PickerOptions::default());
	let id = third.id.clone();
	assert_eq!(failing.run(third).await.into_reply().0, 2);
	assert!(!failing.is_pending(&id));
}

#[tokio::test]
async fn picker_commands_restrict_windows_and_return_the_selected_file() {
	use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

	let dir = tempfile::tempdir().unwrap();
	let file = dir.path().join("chosen.txt");
	std::fs::write(&file, "data").unwrap();
	let host = FakeHost::default();
	let broker = Broker::new(host.clone());
	let request = request(&broker, PickerMode::Open, PickerOptions::default());
	let id = request.id.clone();
	let app = tauri::test::mock_app();
	let picker =
		WebviewWindowBuilder::new(&app, format!("{WINDOW_PREFIX}{id}"), WebviewUrl::default())
			.build()
			.unwrap();
	let other = WebviewWindowBuilder::new(&app, "other", WebviewUrl::default())
		.build()
		.unwrap();
	let picker_window = picker.as_ref().window().clone();
	let other_window = other.as_ref().window().clone();
	assert!(file_chooser_request(app.handle().clone(), picker_window.clone()).is_err());
	app.manage(broker.clone());
	assert!(file_chooser_request(app.handle().clone(), other_window.clone()).is_err());
	assert!(file_chooser_cancel(app.handle().clone(), other_window).is_err());

	let task = tokio::spawn(async move { broker.run(request).await });
	wait_for_show(&host).await;
	assert_eq!(
		file_chooser_request(app.handle().clone(), picker_window.clone())
			.unwrap()
			.unwrap()
			.id,
		id
	);
	file_chooser_respond(
		app.handle().clone(),
		picker_window.clone(),
		Selection::Open {
			paths: vec![file.clone()],
			choices: vec![],
			current_filter: None,
		},
	)
	.unwrap();
	let (code, result) = task.await.unwrap().into_reply();
	assert_eq!(code, response::SUCCESS);
	assert_eq!(uris(&result), [file_uri(&file)]);
	assert!(file_chooser_request(app.handle().clone(), picker_window)
		.unwrap()
		.is_none());
}

#[tokio::test]
async fn tauri_host_creates_a_picker_and_answers_cancellation() {
	use tauri::Manager;

	let app = tauri::test::mock_app();
	let host = TauriHost {
		app: app.handle().clone(),
	};
	let broker = Broker::new(host);
	app.manage(broker.clone());
	let request = request(&broker, PickerMode::Open, PickerOptions::default());
	let id = request.id.clone();
	let label = format!("{WINDOW_PREFIX}{id}");
	let task = tokio::spawn({
		let broker = broker.clone();
		async move { broker.run(request).await }
	});
	tokio::time::timeout(std::time::Duration::from_secs(5), async {
		while app.get_webview_window(&label).is_none() {
			tokio::task::yield_now().await;
		}
	})
	.await
	.unwrap();
	broker.cancel(&id);
	assert_eq!(task.await.unwrap().into_reply().0, response::CANCELLED);
	assert!(broker.request(&id).is_none());
}

#[test]
fn glob_tables_follow_xdg_data_folders() {
	let map: HashMap<&str, &str> = HashMap::from([("HOME", "/h"), ("XDG_DATA_DIRS", "/x:/y")]);
	let paths =
		super::portal::globs2_paths_from(|name| map.get(name).map(std::ffi::OsString::from));
	assert_eq!(
		paths,
		[
			PathBuf::from("/h/.local/share/mime/globs2"),
			PathBuf::from("/x/mime/globs2"),
			PathBuf::from("/y/mime/globs2"),
		]
	);
	let empty = HashMap::from([("XDG_DATA_HOME", "/d"), ("XDG_DATA_DIRS", "")]);
	let paths =
		super::portal::globs2_paths_from(|name| empty.get(name).map(std::ffi::OsString::from));
	assert_eq!(paths[0], PathBuf::from("/d/mime/globs2"));
	assert_eq!(paths.last(), Some(&PathBuf::from("/usr/share/mime/globs2")));
}

#[tokio::test]
async fn restarting_the_portal_reports_the_command_result() {
	assert!(restart_portal(tokio::process::Command::new("true")).await);
	assert!(!restart_portal(tokio::process::Command::new("false")).await);
	assert!(!restart_portal(tokio::process::Command::new("/nonexistent/wing-systemctl")).await);
	let command = systemctl_restart();
	let args: Vec<_> = command.as_std().get_args().collect();
	assert_eq!(command.as_std().get_program(), "systemctl");
	assert_eq!(
		args,
		["--user", "try-restart", "xdg-desktop-portal.service"]
	);
}

#[tokio::test]
async fn the_switch_runs_off_the_runtime_and_reports_errors() {
	let dir = tempfile::tempdir().unwrap();
	let paths = super::portals_conf::PortalPaths {
		config_dir: dir.path().join("config"),
		desktops: vec![],
		state_file: dir.path().join("state.json"),
		portal_dirs: vec![dir.path().join("portals")],
	};
	// The backend file is missing, so enabling is refused.
	assert!(switch_portal(paths.clone(), true).await.is_err());
	assert!(!switch_portal(paths, false).await.unwrap().enabled);
}
