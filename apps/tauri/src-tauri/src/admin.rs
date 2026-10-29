//! # Administrator retries
//!
//! Dolphin's admin mode and Nautilus's `admin://` let you change files you do
//! not own after you authenticate. WingDrive does the same for the operations
//! that hit a permission error: it runs one fixed coreutils program through
//! `pkexec`, so polkit asks for the password and the command runs as root.
//!
//! Every argument is a validated absolute path or file name, passed after `--`
//! and never through a shell. Paths with `..` components, the filesystem root
//! and relative paths are refused, so a stale or crafted request cannot widen
//! what runs as root. Copies and moves never overwrite (`-n`), matching the
//! "skip" conflict choice, and deletes are permanent because the root user's
//! trash is not the user's.

use std::path::{Component, Path, PathBuf};

use serde::Deserialize;

/// One operation to repeat with administrator rights.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AdminOperation {
	/// Removes files and folders, recursively.
	Delete { paths: Vec<PathBuf> },
	/// Renames an entry within its folder.
	Rename { path: PathBuf, new_name: String },
	/// Creates an empty folder.
	CreateFolder { parent: PathBuf, name: String },
	/// Copies or moves entries into a folder, skipping names that exist.
	Transfer {
		sources: Vec<PathBuf>,
		destination: PathBuf,
		#[serde(default)]
		move_files: bool,
	},
}

/// Programs the operations run, searched in these folders only.
const PROGRAM_DIRS: [&str; 2] = ["/usr/bin", "/bin"];

/// `pkexec` exits with these when the user dismisses or fails the prompt.
const PKEXEC_DISMISSED: [i32; 2] = [126, 127];

/// Builds `pkexec` arguments: the program's absolute path, then its arguments.
pub fn pkexec_args(operation: &AdminOperation) -> Result<Vec<String>, String> {
	let (program, args): (&str, Vec<String>) = match operation {
		AdminOperation::Delete { paths } => {
			require_some(paths)?;
			let mut args = vec!["-rf".to_string(), "--".to_string()];
			for path in paths {
				args.push(checked_path(path)?);
			}
			("rm", args)
		}
		AdminOperation::Rename { path, new_name } => {
			let source = checked_path(path)?;
			let parent = Path::new(&source)
				.parent()
				.ok_or("The path has no folder")?;
			let target = checked_path(&parent.join(checked_name(new_name)?))?;
			(
				"mv",
				vec!["-n".into(), "-T".into(), "--".into(), source, target],
			)
		}
		AdminOperation::CreateFolder { parent, name } => {
			let target =
				checked_path(&PathBuf::from(checked_path(parent)?).join(checked_name(name)?))?;
			("mkdir", vec!["--".into(), target])
		}
		AdminOperation::Transfer {
			sources,
			destination,
			move_files,
		} => {
			require_some(sources)?;
			let mut args = if *move_files {
				vec!["-n".to_string()]
			} else {
				vec!["-a".to_string(), "-n".to_string()]
			};
			args.push("-t".into());
			args.push(checked_path(destination)?);
			args.push("--".into());
			for source in sources {
				args.push(checked_path(source)?);
			}
			(if *move_files { "mv" } else { "cp" }, args)
		}
	};

	let program = PROGRAM_DIRS
		.iter()
		.map(|dir| Path::new(dir).join(program))
		.find(|path| path.is_file())
		.ok_or_else(|| format!("{program} was not found"))?;
	Ok(std::iter::once(program.to_string_lossy().into_owned())
		.chain(args)
		.collect())
}

fn require_some(paths: &[PathBuf]) -> Result<(), String> {
	if paths.is_empty() {
		Err("No files were given".into())
	} else {
		Ok(())
	}
}

/// An absolute path below the root without `.` or `..` components.
fn checked_path(path: &Path) -> Result<String, String> {
	let text = path.to_str().ok_or("The path is not valid UTF-8")?;
	// `components()` drops `.` in the middle of a path, so check the text too.
	if text
		.split('/')
		.any(|segment| segment == "." || segment == "..")
	{
		return Err(format!("{text} is not a plain path"));
	}
	let mut components = path.components();
	if components.next() != Some(Component::RootDir) {
		return Err(format!("{text} is not an absolute path"));
	}
	let mut depth = 0;
	for component in components {
		match component {
			Component::Normal(_) => depth += 1,
			_ => return Err(format!("{text} is not a plain path")),
		}
	}
	if depth == 0 {
		return Err("The filesystem root cannot be changed this way".into());
	}
	Ok(text.trim_end_matches('/').to_string())
}

/// A single file name, not a path.
fn checked_name(name: &str) -> Result<&str, String> {
	if name.is_empty() || name == "." || name == ".." || name.contains('/') || name.contains('\0') {
		return Err(format!("{name:?} is not a valid name"));
	}
	Ok(name)
}

/// Runs `operation` as root through polkit, waiting for it to finish.
#[cfg(target_os = "linux")]
#[tauri::command]
pub async fn run_as_admin(operation: AdminOperation) -> Result<(), String> {
	let args = pkexec_args(&operation)?;
	tracing::info!(?operation, "Repeating an operation as administrator");

	let output = tauri::async_runtime::spawn_blocking(move || {
		let mut command = std::process::Command::new("pkexec");
		file_opening_linux::use_host_environment(&mut command);
		command
			.args(&args)
			.stdin(std::process::Stdio::null())
			.output()
	})
	.await
	.map_err(|e| e.to_string())?
	.map_err(|e| format!("pkexec could not start: {e}"))?;

	if output.status.success() {
		return Ok(());
	}
	if output
		.status
		.code()
		.is_some_and(|code| PKEXEC_DISMISSED.contains(&code))
	{
		return Err("Authentication was cancelled or refused".into());
	}
	let stderr = String::from_utf8_lossy(&output.stderr);
	Err(stderr
		.trim()
		.lines()
		.next()
		.unwrap_or("The command failed")
		.to_string())
}

#[cfg(not(target_os = "linux"))]
#[tauri::command]
pub async fn run_as_admin(operation: AdminOperation) -> Result<(), String> {
	let _ = operation;
	Err("Administrator retries are only available on Linux".into())
}

#[cfg(test)]
mod tests {
	use super::*;

	fn args(operation: AdminOperation) -> Vec<String> {
		// The program path depends on the system; the arguments do not.
		pkexec_args(&operation).unwrap().split_off(1)
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
}
