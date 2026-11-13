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

use std::path::PathBuf;
#[cfg(target_os = "linux")]
use std::path::{Component, Path};

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
#[cfg(target_os = "linux")]
const PROGRAM_DIRS: [&str; 2] = ["/usr/bin", "/bin"];

/// `pkexec` exits with these when the user dismisses or fails the prompt.
#[cfg(target_os = "linux")]
const PKEXEC_DISMISSED: [i32; 2] = [126, 127];

/// Builds `pkexec` arguments: the program's absolute path, then its arguments.
#[cfg(target_os = "linux")]
pub fn pkexec_args(operation: &AdminOperation) -> Result<Vec<String>, String> {
	pkexec_args_in(operation, &PROGRAM_DIRS)
}

/// [`pkexec_args`] with the folders the program is searched in.
#[cfg(target_os = "linux")]
fn pkexec_args_in(
	operation: &AdminOperation,
	program_dirs: &[&str],
) -> Result<Vec<String>, String> {
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

	let program = program_dirs
		.iter()
		.map(|dir| Path::new(dir).join(program))
		.find(|path| path.is_file())
		.ok_or_else(|| format!("{program} was not found"))?;
	Ok(std::iter::once(program.to_string_lossy().into_owned())
		.chain(args)
		.collect())
}

#[cfg(target_os = "linux")]
fn require_some(paths: &[PathBuf]) -> Result<(), String> {
	if paths.is_empty() {
		Err("No files were given".into())
	} else {
		Ok(())
	}
}

/// An absolute path below the root without `.` or `..` components.
#[cfg(target_os = "linux")]
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
#[cfg(target_os = "linux")]
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
	run_elevated("pkexec", args).await
}

/// Runs `program` with `args` and turns its exit into the command's result:
/// polkit's dismissal codes become a cancellation, other failures report the
/// first line the program printed to stderr.
#[cfg(target_os = "linux")]
async fn run_elevated(program: &'static str, args: Vec<String>) -> Result<(), String> {
	let output = tauri::async_runtime::spawn_blocking(move || {
		let mut command = std::process::Command::new(program);
		file_opening_linux::use_host_environment(&mut command);
		command
			.args(&args)
			.stdin(std::process::Stdio::null())
			.output()
	})
	.await
	.map_err(|e| e.to_string())?
	.map_err(|e| format!("{program} could not start: {e}"))?;

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

#[cfg(all(test, target_os = "linux"))]
#[path = "admin/tests/mod.rs"]
mod tests;
