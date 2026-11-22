//! # Administrator retries
//!
//! File operations that fail for lack of permission can be repeated with
//! administrator rights. On Linux, the app runs one fixed coreutils program
//! through `pkexec`, so polkit asks for the password. On Windows, it starts a
//! small helper with the `runas` verb, so Windows shows one UAC prompt. The
//! helper reads its operation from a file, never from its command line, and file
//! paths are passed to filesystem APIs as data, never through a shell.

use std::path::PathBuf;

use serde::Serialize;

pub use wing_elevate::{AdminItemFailure, AdminOperation};

/// Outcome returned to the frontend after an administrator retry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum AdminRunOutcome {
	/// Every requested item completed.
	Completed,
	/// The user declined or the system refused the elevation prompt.
	Cancelled,
	/// Some items completed and some failed.
	Partial { failures: Vec<AdminItemFailure> },
}

/// A stale or crafted request must not widen what runs as root.
#[cfg(target_os = "linux")]
use std::path::{Component, Path};

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
pub async fn run_as_admin(operation: AdminOperation) -> Result<AdminRunOutcome, String> {
	let args = pkexec_args(&operation)?;
	tracing::info!(?operation, "Repeating an operation as administrator");
	run_elevated("pkexec", args).await
}

/// Runs `program` with `args` and turns its exit into the command's result:
/// polkit's dismissal codes become a cancellation, other failures report the
/// first line the program printed to stderr.
#[cfg(target_os = "linux")]
async fn run_elevated(program: &'static str, args: Vec<String>) -> Result<AdminRunOutcome, String> {
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
		return Ok(AdminRunOutcome::Completed);
	}
	if output
		.status
		.code()
		.is_some_and(|code| PKEXEC_DISMISSED.contains(&code))
	{
		return Ok(AdminRunOutcome::Cancelled);
	}
	let stderr = String::from_utf8_lossy(&output.stderr);
	Err(stderr
		.trim()
		.lines()
		.next()
		.unwrap_or("The command failed")
		.to_string())
}

/// Runs `operation` as administrator through a Windows UAC prompt.
#[cfg(target_os = "windows")]
#[tauri::command]
pub async fn run_as_admin(operation: AdminOperation) -> Result<AdminRunOutcome, String> {
	let operation_for_log = operation.clone();
	tracing::info!(?operation_for_log, "Repeating an operation as administrator");
	tauri::async_runtime::spawn_blocking(move || run_elevated_windows(&operation))
		.await
		.map_err(|e| e.to_string())?
}

#[cfg(target_os = "windows")]
fn run_elevated_windows(operation: &AdminOperation) -> Result<AdminRunOutcome, String> {
	use std::fs;
	use wing_elevate::{AdminRequest, AdminResponse, AdminStatus};

	let helper = find_elevate_binary()?;
	let temp_dir = std::env::temp_dir().join(format!("wingdrive-admin-{}", std::process::id()));
	fs::create_dir_all(&temp_dir).map_err(|e| format!("Failed to create temp dir: {e}"))?;

	let unique = format!(
		"{}-{}",
		std::process::id(),
		std::time::SystemTime::now()
			.duration_since(std::time::UNIX_EPOCH)
			.unwrap_or_default()
			.as_nanos()
	);
	let request_path = temp_dir.join(format!("request-{unique}.json"));
	let response_path = temp_dir.join(format!("response-{unique}.json"));

	let request = AdminRequest {
		operation: operation.clone(),
	};
	let request_json = serde_json::to_vec(&request).map_err(|e| e.to_string())?;
	fs::write(&request_path, request_json).map_err(|e| format!("Failed to write request: {e}"))?;

	let params = format!(
		"--request \"{}\" --response \"{}\"",
		request_path.display(),
		response_path.display()
	);

	let launch = shell_execute_runas(&helper, &params);
	// Always clean up request; keep response until read.
	let _ = fs::remove_file(&request_path);

	let exit_code = match launch {
		Ok(code) => code,
		Err(ElevationLaunchError::Cancelled) => {
			let _ = fs::remove_file(&response_path);
			let _ = fs::remove_dir(&temp_dir);
			return Ok(AdminRunOutcome::Cancelled);
		}
		Err(ElevationLaunchError::Message(message)) => {
			let _ = fs::remove_file(&response_path);
			let _ = fs::remove_dir(&temp_dir);
			return Err(message);
		}
	};

	let response_bytes = fs::read(&response_path).map_err(|e| {
		let _ = fs::remove_dir(&temp_dir);
		format!("Failed to read elevated helper response: {e}")
	})?;
	let _ = fs::remove_file(&response_path);
	let _ = fs::remove_dir(&temp_dir);
	let response: AdminResponse = serde_json::from_slice(&response_bytes)
		.map_err(|e| format!("Failed to parse elevated helper response: {e}"))?;

	match response.status {
		AdminStatus::Completed => Ok(AdminRunOutcome::Completed),
		AdminStatus::Failed { message } => {
			if exit_code == 1223 {
				Ok(AdminRunOutcome::Cancelled)
			} else {
				Err(message)
			}
		}
		AdminStatus::Partial { failures } => Ok(AdminRunOutcome::Partial { failures }),
	}
}

#[cfg(target_os = "windows")]
enum ElevationLaunchError {
	Cancelled,
	Message(String),
}

#[cfg(target_os = "windows")]
fn shell_execute_runas(
	helper: &std::path::Path,
	parameters: &str,
) -> Result<u32, ElevationLaunchError> {
	use std::os::windows::ffi::OsStrExt;

	use windows_sys::Win32::Foundation::{CloseHandle, GetLastError, HANDLE, HWND};
	use windows_sys::Win32::System::Threading::{GetExitCodeProcess, WaitForSingleObject, INFINITE};
	use windows_sys::Win32::UI::Shell::{
		ShellExecuteExW, SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW,
	};
	use windows_sys::Win32::UI::WindowsAndMessaging::SW_HIDE;

	fn wide(value: &std::ffi::OsStr) -> Vec<u16> {
		value.encode_wide().chain(std::iter::once(0)).collect()
	}

	const ERROR_CANCELLED: u32 = 1223;

	let verb = wide(std::ffi::OsStr::new("runas"));
	let file = wide(helper.as_os_str());
	let params = wide(std::ffi::OsStr::new(parameters));
	let dir = helper
		.parent()
		.map(|p| wide(p.as_os_str()))
		.unwrap_or_else(|| wide(std::ffi::OsStr::new("")));

	let mut info: SHELLEXECUTEINFOW = unsafe { std::mem::zeroed() };
	info.cbSize = std::mem::size_of::<SHELLEXECUTEINFOW>() as u32;
	info.fMask = SEE_MASK_NOCLOSEPROCESS;
	info.hwnd = 0 as HWND;
	info.lpVerb = verb.as_ptr();
	info.lpFile = file.as_ptr();
	info.lpParameters = params.as_ptr();
	info.lpDirectory = dir.as_ptr();
	info.nShow = SW_HIDE as i32;

	let ok = unsafe { ShellExecuteExW(&mut info) };
	if ok == 0 {
		let err = unsafe { GetLastError() };
		return if err == ERROR_CANCELLED {
			Err(ElevationLaunchError::Cancelled)
		} else {
			Err(ElevationLaunchError::Message(format!(
				"Failed to start elevated helper: {err}"
			)))
		};
	}

	let process: HANDLE = info.hProcess;
	if process == 0 {
		return Err(ElevationLaunchError::Message(
			"Elevated helper started without a process handle".into(),
		));
	}

	unsafe {
		WaitForSingleObject(process, INFINITE);
	}
	let mut exit_code: u32 = 1;
	let got_code = unsafe { GetExitCodeProcess(process, &mut exit_code) };
	unsafe {
		CloseHandle(process);
	}
	if got_code == 0 {
		return Err(ElevationLaunchError::Message(
			"Failed to read elevated helper exit code".into(),
		));
	}
	Ok(exit_code)
}

#[cfg(target_os = "windows")]
fn find_elevate_binary() -> Result<PathBuf, String> {
	let exe_path =
		std::env::current_exe().map_err(|e| format!("Failed to get current exe: {e}"))?;
	let bin_dir = exe_path.parent().ok_or("No parent directory for exe")?;
	find_elevate_binary_in(bin_dir)
}

#[cfg(target_os = "windows")]
fn find_elevate_binary_in(bin_dir: &std::path::Path) -> Result<PathBuf, String> {
	let triple = env!("WING_TARGET_TRIPLE");
	let with_triple = format!("wing-elevate-{}.exe", triple);
	let plain = "wing-elevate.exe".to_string();
	let order = if cfg!(debug_assertions) {
		[plain.as_str(), with_triple.as_str()]
	} else {
		[with_triple.as_str(), plain.as_str()]
	};
	order
		.iter()
		.map(|name| bin_dir.join(name))
		.find(|p| p.exists())
		.ok_or_else(|| {
			format!(
				"Elevated helper not found. Checked {} and {} in {:?}",
				with_triple, plain, bin_dir
			)
		})
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
#[tauri::command]
pub async fn run_as_admin(operation: AdminOperation) -> Result<AdminRunOutcome, String> {
	let _ = operation;
	Err("Administrator retries are not available on this platform".into())
}

#[cfg(all(test, target_os = "linux"))]
#[path = "admin/tests/mod.rs"]
mod tests;
