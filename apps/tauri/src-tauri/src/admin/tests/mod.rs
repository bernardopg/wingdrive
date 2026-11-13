use super::*;

fn args(operation: AdminOperation) -> Vec<String> {
	// The program path depends on the system; the arguments do not.
	pkexec_args(&operation).unwrap().split_off(1)
}

#[tokio::test]
async fn elevated_exit_codes_become_results() {
	let sh = |script: &str| vec!["-c".to_string(), script.to_string()];
	run_elevated("sh", sh("exit 0")).await.unwrap();
	for code in PKEXEC_DISMISSED {
		assert_eq!(
			run_elevated("sh", sh(&format!("exit {code}")))
				.await
				.unwrap_err(),
			"Authentication was cancelled or refused"
		);
	}
	assert_eq!(
		run_elevated(
			"sh",
			sh("echo 'mv: cannot move' >&2; echo second >&2; exit 1")
		)
		.await
		.unwrap_err(),
		"mv: cannot move"
	);
	assert_eq!(
		run_elevated("sh", sh("exit 1")).await.unwrap_err(),
		"The command failed"
	);
	let error = run_elevated("/nonexistent/pkexec", vec![])
		.await
		.unwrap_err();
	assert!(
		error.starts_with("/nonexistent/pkexec could not start"),
		"{error}"
	);
}

#[test]
fn builds_each_command_with_fixed_flags() {
	assert_eq!(
		args(AdminOperation::Delete {
			paths: vec!["/etc/a b".into(), "/opt/x/".into()]
		}),
		["-rf", "--", "/etc/a b", "/opt/x"]
	);
	assert_eq!(
		args(AdminOperation::Rename {
			path: "/etc/old.conf".into(),
			new_name: "-new.conf".into()
		}),
		["-n", "-T", "--", "/etc/old.conf", "/etc/-new.conf"]
	);
	assert_eq!(
		args(AdminOperation::CreateFolder {
			parent: "/opt".into(),
			name: "tools".into()
		}),
		["--", "/opt/tools"]
	);
	assert_eq!(
		args(AdminOperation::Transfer {
			sources: vec!["/home/me/a".into()],
			destination: "/opt".into(),
			move_files: false
		}),
		["-a", "-n", "-t", "/opt", "--", "/home/me/a"]
	);
	assert_eq!(
		args(AdminOperation::Transfer {
			sources: vec!["/home/me/a".into()],
			destination: "/opt".into(),
			move_files: true
		}),
		["-n", "-t", "/opt", "--", "/home/me/a"]
	);
}

#[test]
fn the_program_is_an_absolute_system_path() {
	let full = pkexec_args(&AdminOperation::Delete {
		paths: vec!["/tmp/x".into()],
	})
	.unwrap();
	assert!(
		full[0] == "/usr/bin/rm" || full[0] == "/bin/rm",
		"{}",
		full[0]
	);
}

#[test]
fn refuses_paths_that_could_widen_the_command() {
	for path in ["relative/x", "/", "/etc/../root", "/etc/./x", ""] {
		assert!(
			pkexec_args(&AdminOperation::Delete {
				paths: vec![path.into()]
			})
			.is_err(),
			"{path} was accepted"
		);
	}
	assert!(pkexec_args(&AdminOperation::Delete { paths: vec![] }).is_err());
	assert!(pkexec_args(&AdminOperation::Transfer {
		sources: vec![],
		destination: "/opt".into(),
		move_files: false
	})
	.is_err());
}

#[test]
fn refuses_names_that_are_paths() {
	for name in ["", ".", "..", "a/b", "../x"] {
		assert!(
			pkexec_args(&AdminOperation::Rename {
				path: "/etc/a".into(),
				new_name: name.into()
			})
			.is_err(),
			"{name:?} was accepted"
		);
		assert!(pkexec_args(&AdminOperation::CreateFolder {
			parent: "/opt".into(),
			name: name.into()
		})
		.is_err());
	}
}

#[tokio::test]
async fn refused_operations_never_reach_pkexec() {
	// Validation fails before any program starts, so no password prompt appears.
	assert_eq!(
		run_as_admin(AdminOperation::Delete {
			paths: vec!["/".into()]
		})
		.await
		.unwrap_err(),
		"The filesystem root cannot be changed this way"
	);
	assert_eq!(
		run_as_admin(AdminOperation::Transfer {
			sources: vec!["/a".into()],
			destination: "opt".into(),
			move_files: true
		})
		.await
		.unwrap_err(),
		"opt is not an absolute path"
	);
}

#[test]
fn non_utf8_paths_and_missing_programs_are_refused() {
	use std::os::unix::ffi::OsStrExt;

	let path = PathBuf::from(std::ffi::OsStr::from_bytes(b"/etc/name\xff"));
	assert_eq!(
		pkexec_args(&AdminOperation::Delete { paths: vec![path] }).unwrap_err(),
		"The path is not valid UTF-8"
	);
	// A system without coreutils in the trusted folders gets a clear error, not
	// a program found through `PATH`.
	let empty = tempfile::tempdir().unwrap();
	let dirs = [empty.path().to_str().unwrap()];
	assert_eq!(
		pkexec_args_in(
			&AdminOperation::CreateFolder {
				parent: "/opt".into(),
				name: "tools".into()
			},
			&dirs
		)
		.unwrap_err(),
		"mkdir was not found"
	);
	// The first trusted folder that has the program wins.
	std::fs::write(empty.path().join("mkdir"), "").unwrap();
	let args = pkexec_args_in(
		&AdminOperation::CreateFolder {
			parent: "/opt".into(),
			name: "tools".into(),
		},
		&dirs,
	)
	.unwrap();
	assert_eq!(args[0], empty.path().join("mkdir").to_str().unwrap());
}

#[test]
fn reads_operations_sent_by_the_frontend() {
	let operation: AdminOperation = serde_json::from_str(
		r#"{"type":"transfer","sources":["/a"],"destination":"/opt","move_files":true}"#,
	)
	.unwrap();
	assert_eq!(
		operation,
		AdminOperation::Transfer {
			sources: vec!["/a".into()],
			destination: "/opt".into(),
			move_files: true
		}
	);
}

/// Runs the command `pkexec` would run, as the current user, on a temp tree.
fn run(operation: AdminOperation) {
	let args = pkexec_args(&operation).unwrap();
	let status = std::process::Command::new(&args[0])
		.args(&args[1..])
		.stderr(std::process::Stdio::null())
		.status()
		.unwrap();
	assert!(status.success(), "{args:?}");
}

#[test]
fn commands_do_what_they_describe() {
	let dir = tempfile::tempdir().unwrap();
	let root = dir.path();
	let path = |name: &str| root.join(name);
	std::fs::create_dir(path("dest")).unwrap();
	std::fs::write(path("a.txt"), "a").unwrap();
	std::fs::write(path("dest/a.txt"), "kept").unwrap();
	std::fs::write(path("b.txt"), "b").unwrap();

	run(AdminOperation::CreateFolder {
		parent: root.to_path_buf(),
		name: "new".into(),
	});
	assert!(path("new").is_dir());

	run(AdminOperation::Rename {
		path: path("b.txt"),
		new_name: "c.txt".into(),
	});
	assert_eq!(std::fs::read_to_string(path("c.txt")).unwrap(), "b");

	// Copies skip names that exist instead of replacing them.
	run(AdminOperation::Transfer {
		sources: vec![path("a.txt"), path("c.txt")],
		destination: path("dest"),
		move_files: false,
	});
	assert_eq!(std::fs::read_to_string(path("dest/a.txt")).unwrap(), "kept");
	assert_eq!(std::fs::read_to_string(path("dest/c.txt")).unwrap(), "b");

	run(AdminOperation::Transfer {
		sources: vec![path("new")],
		destination: path("dest"),
		move_files: true,
	});
	assert!(path("dest/new").is_dir() && !path("new").exists());

	run(AdminOperation::Delete {
		paths: vec![path("dest"), path("a.txt")],
	});
	assert!(!path("dest").exists() && !path("a.txt").exists());
}

/// Runs every operation through the real `pkexec` and polkit, in a folder of `/opt`
/// that only root can write. Ignored by default: it needs a polkit agent that answers
/// for this process, which `scripts/check-admin-retry.sh` registers.
#[test]
#[ignore = "needs a polkit agent; run scripts/check-admin-retry.sh"]
fn pkexec_round_trip_on_root_owned_files() {
	use std::os::unix::fs::MetadataExt;

	let run = |operation: AdminOperation| {
		tauri::async_runtime::block_on(run_as_admin(operation.clone()))
			.unwrap_or_else(|e| panic!("{operation:?}: {e}"));
	};
	let parent = PathBuf::from("/opt");
	assert!(
		std::fs::write(parent.join(".wingdrive-write-check"), "").is_err(),
		"/opt must not be writable by this user"
	);
	let name = format!("wingdrive-admin-check-{}", std::process::id());
	let root = parent.join(&name);
	let user_file = tempfile::NamedTempFile::new().unwrap();
	std::fs::write(user_file.path(), "from the user").unwrap();

	run(AdminOperation::CreateFolder {
		parent: parent.clone(),
		name: name.clone(),
	});
	assert_eq!(
		std::fs::metadata(&root).unwrap().uid(),
		0,
		"created as root"
	);

	run(AdminOperation::Transfer {
		sources: vec![user_file.path().to_path_buf()],
		destination: root.clone(),
		move_files: false,
	});
	let copied = root.join(user_file.path().file_name().unwrap());
	assert_eq!(std::fs::read_to_string(&copied).unwrap(), "from the user");

	run(AdminOperation::Rename {
		path: copied,
		new_name: "renamed.txt".into(),
	});
	assert!(root.join("renamed.txt").is_file());

	run(AdminOperation::Delete {
		paths: vec![root.clone()],
	});
	assert!(!root.exists());
}
