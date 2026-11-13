use super::*;

#[cfg(target_os = "linux")]
struct SingleInstanceRecorder(std::sync::mpsc::Sender<(Vec<String>, String)>);

#[cfg(target_os = "linux")]
#[zbus::interface(name = "org.SingleInstance.DBus")]
impl SingleInstanceRecorder {
	async fn execute_callback(&self, argv: Vec<String>, cwd: String) {
		self.0.send((argv, cwd)).unwrap();
	}
}

#[cfg(target_os = "linux")]
#[tokio::test]
async fn forwarding_delivers_arguments_and_working_directory_on_a_private_bus() {
	use std::io::{BufRead, BufReader};
	use std::process::{Child, Command, Stdio};

	struct Bus(Child);
	impl Drop for Bus {
		fn drop(&mut self) {
			let _ = self.0.kill();
			let _ = self.0.wait();
		}
	}
	let mut bus = Bus(Command::new("dbus-daemon")
		.args(["--session", "--nofork", "--print-address=1"])
		.stdout(Stdio::piped())
		.stderr(Stdio::null())
		.spawn()
		.unwrap());
	let mut address = String::new();
	BufReader::new(bus.0.stdout.take().unwrap())
		.read_line(&mut address)
		.unwrap();
	let address = address.trim();
	let id = single_instance_id(Some("test-instance"));
	let (sent, received) = std::sync::mpsc::channel();
	let _server = zbus::connection::Builder::address(address)
		.name(format!("{id}.SingleInstance"))
		.serve_at(
			format!("/{}/SingleInstance", id.replace('.', "/")),
			SingleInstanceRecorder(sent),
		)
		.build()
		.await
		.unwrap();
	let client = zbus::connection::Builder::address(address)
		.build()
		.await
		.unwrap();
	let argv = vec!["wingdrive".into(), "a b".into(), "--hidden".into()];
	assert!(call_instance_on(&client, &id, argv.clone(), "/other cwd".into()).await);
	assert_eq!(
		received
			.recv_timeout(std::time::Duration::from_secs(2))
			.unwrap(),
		(argv, "/other cwd".into())
	);
	assert!(
		!call_instance_on(
			&client,
			&single_instance_id(Some("absent")),
			vec![],
			String::new()
		)
		.await
	);
}

#[test]
fn delivery_queues_each_batch_and_notifies_the_frontend() {
	use std::sync::Arc;
	use tauri::{Listener, Manager};

	let app = tauri::test::mock_app();
	app.manage(PendingOpenRequests::new(Vec::new()));
	let events = Arc::new(Mutex::new(0));
	let seen = events.clone();
	app.listen_any(OPEN_REQUESTS_EVENT, move |_| *seen.lock().unwrap() += 1);
	let first = OpenRequest {
		directory: PathBuf::from("/first"),
		select: None,
	};
	let second = reveal_request(PathBuf::from("/second/file"));

	deliver(app.handle(), vec![first.clone()]);
	deliver(app.handle(), Vec::new());
	deliver(app.handle(), vec![second.clone()]);
	assert_eq!(take_open_requests(app.state()), vec![first, second]);
	assert_eq!(*events.lock().unwrap(), 2);
	deliver(app.handle(), vec![]);
	assert!(take_open_requests(app.state()).is_empty());
}

#[test]
fn draining_open_requests_keeps_order_and_allows_new_batches() {
	use tauri::Manager;
	let root = tempfile::tempdir().unwrap();
	let folder = OpenRequest {
		directory: root.path().join("a"),
		select: None,
	};
	let reveal = reveal_request(root.path().join("b/file"));
	let app = tauri::test::mock_app();
	app.manage(PendingOpenRequests::new(vec![
		folder.clone(),
		reveal.clone(),
		folder.clone(),
	]));
	assert_eq!(take_initial_open_request(app.state()), Some(folder.clone()));
	assert_eq!(take_initial_open_request(app.state()), None);
	assert_eq!(
		take_open_requests(app.state()),
		[reveal.clone(), folder.clone()]
	);
	app.state::<PendingOpenRequests>()
		.0
		.lock()
		.unwrap()
		.extend([reveal.clone(), folder.clone()]);
	assert_eq!(take_open_requests(app.state()), [reveal, folder]);
	assert!(take_open_requests(app.state()).is_empty());
}

#[cfg(unix)]
#[test]
fn launch_skips_dangling_links_and_keeps_argument_order_and_duplicates() {
	let root = tempfile::tempdir().unwrap();
	let file = root.path().join("a #%.txt");
	let folder = root.path().join("folder");
	std::fs::write(&file, []).unwrap();
	std::fs::create_dir(&folder).unwrap();
	std::os::unix::fs::symlink(root.path().join("missing"), root.path().join("dangling")).unwrap();
	let options = parse_args(
		[
			"dangling".into(),
			tauri::Url::from_file_path(&file).unwrap().to_string(),
			"folder".into(),
			"a #%.txt".into(),
		],
		root.path(),
	);
	assert_eq!(
		options.requests,
		[
			reveal_request(file.clone()),
			OpenRequest {
				directory: folder,
				select: None
			},
			reveal_request(file)
		]
	);
}

#[test]
fn hidden_start_is_consumed_once() {
	let hidden = StartHidden(AtomicBool::new(true));
	assert!(hidden.is_set());
	assert!(hidden.consume());
	assert!(!hidden.is_set());
	assert!(!hidden.consume());
	assert!(!StartHidden::default().consume());
}

#[test]
fn local_uris_decode_unicode_spaces_and_literal_percent_signs() {
	let cwd = Path::new("/tmp");
	for (uri, expected) in [
		("file:///tmp/a%20b", "/tmp/a b"),
		("file://localhost/tmp/a%25b", "/tmp/a%b"),
		("file:///tmp/caf%C3%A9", "/tmp/café"),
		("file:///tmp/a+b", "/tmp/a+b"),
	] {
		assert_eq!(local_path(uri, cwd), Some(PathBuf::from(expected)), "{uri}");
	}
}

#[test]
fn nonlocal_and_malformed_uris_do_not_become_filesystem_paths() {
	for uri in [
		"file://remote/tmp/a",
		"file://localhost.evil/tmp/a",
		"file:///tmp/%",
		"file:///tmp/%0",
		"file:///tmp/%zz",
		"file:///tmp/%FF",
		"https://host/a",
		"smb://host/share",
		"--unknown",
	] {
		assert_eq!(local_path(uri, Path::new("/tmp")), None, "{uri}");
	}
}

#[test]
fn relative_paths_resolve_against_the_forwarded_working_directory() {
	assert_eq!(
		local_path("../other/./notes.txt", Path::new("/tmp/forwarded")),
		Some(PathBuf::from("/tmp/other/notes.txt"))
	);
	assert_eq!(
		local_path("/../../notes.txt", Path::new("/tmp")),
		Some(PathBuf::from("/notes.txt"))
	);
}

#[test]
fn missing_files_are_skipped_without_losing_valid_requests_or_hidden_flag() {
	let root = tempfile::tempdir().unwrap();
	let directory = root.path().join("folder");
	std::fs::create_dir(&directory).unwrap();
	let options = parse_args(
		["missing", "folder", "--hidden", "--unknown"].map(str::to_owned),
		root.path(),
	);
	assert!(options.hidden);
	assert_eq!(
		options.requests,
		vec![OpenRequest {
			directory,
			select: None
		}]
	);
}

#[test]
fn reveal_selects_the_file_in_its_parent() {
	let request = reveal_request(PathBuf::from("/tmp/notes.txt"));
	assert_eq!(request.directory, Path::new("/tmp"));
	assert_eq!(request.select, Some(PathBuf::from("/tmp/notes.txt")));
}

#[cfg(unix)]
#[test]
fn launch_preserves_symlink_paths_instead_of_canonicalizing_them() {
	let root = tempfile::tempdir().unwrap();
	let target = root.path().join("target");
	let link = root.path().join("link");
	std::fs::create_dir(&target).unwrap();
	std::os::unix::fs::symlink(&target, &link).unwrap();
	let options = parse_args(["link".to_owned()], root.path());
	assert_eq!(
		options.requests,
		vec![OpenRequest {
			directory: link,
			select: None
		}]
	);
}

#[test]
fn valid_instance_names_do_not_collide_on_the_session_bus() {
	let names = ["work-a", "work_a", "work_2da", "café", "cafê"];
	let identifiers: std::collections::HashSet<_> = names
		.iter()
		.map(|name| single_instance_id(Some(name)))
		.collect();
	assert_eq!(identifiers.len(), names.len());
	#[cfg(target_os = "linux")]
	for id in identifiers {
		assert!(zbus::names::WellKnownName::try_from(format!("{id}.SingleInstance")).is_ok());
	}
	#[cfg(target_os = "linux")]
	assert!(zbus::names::WellKnownName::try_from(format!(
		"{}.SingleInstance",
		single_instance_id(Some(&"é".repeat(32)))
	))
	.is_ok());
}

#[test]
fn file_uri_rejects_signed_percent_escapes() {
	for uri in ["file:///tmp/%+A", "file:///tmp/%+1"] {
		assert_eq!(local_path(uri, Path::new("/tmp")), None, "{uri}");
	}
}

#[test]
fn initial_request_is_taken_only_when_it_opens_a_folder() {
	use tauri::Manager;

	let folder = |path: &str| OpenRequest {
		directory: PathBuf::from(path),
		select: None,
	};
	let reveal = OpenRequest {
		directory: PathBuf::from("/b"),
		select: Some(PathBuf::from("/b/file")),
	};

	let app = tauri::test::mock_app();
	app.manage(PendingOpenRequests::new(vec![reveal.clone(), folder("/c")]));
	// A reveal at the head stays queued for the frontend, with what follows it.
	assert_eq!(take_initial_open_request(app.state()), None);
	assert_eq!(take_open_requests(app.state()), vec![reveal, folder("/c")]);
	assert!(take_open_requests(app.state()).is_empty());
	assert_eq!(take_initial_open_request(app.state()), None);

	let app = tauri::test::mock_app();
	app.manage(PendingOpenRequests::new(vec![folder("/a"), folder("/d")]));
	assert_eq!(take_initial_open_request(app.state()), Some(folder("/a")));
	assert_eq!(take_open_requests(app.state()), vec![folder("/d")]);
}
