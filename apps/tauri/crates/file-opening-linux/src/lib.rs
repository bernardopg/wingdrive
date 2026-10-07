#![cfg(target_os = "linux")]

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use file_opening::{FileOpener, OpenResult, OpenWithApp};

mod desktop_entry;
mod host_env;
mod mime_apps;
pub mod terminal;

use desktop_entry::DesktopEntry;
use mime_apps::XdgDirs;

pub use host_env::{open_on_host, use_host_environment, PREFERRED_GDK_BACKEND};

pub struct LinuxFileOpener;

impl FileOpener for LinuxFileOpener {
	fn get_apps_for_file(&self, path: &Path) -> Result<Vec<OpenWithApp>, String> {
		let dirs = XdgDirs::from_env();
		let Some(mime) = mime_apps::detect_mime(&dirs, path) else {
			return Ok(vec![]);
		};
		let lineage = mime_apps::mime_lineage(&dirs, &mime);
		let entries = mime_apps::load_entries(&dirs);
		let associations = mime_apps::Associations::load(&dirs);
		let default = mime_apps::default_app(&lineage, &entries, &associations);
		Ok(mime_apps::apps_for_mime(&lineage, &entries, &associations)
			.into_iter()
			.map(|entry| OpenWithApp {
				is_default: default.as_deref() == Some(entry.id.as_str()),
				id: entry.id,
				name: entry.name,
				icon: None,
			})
			.collect())
	}

	fn open_with_default(&self, path: &Path) -> Result<OpenResult, String> {
		if !path.exists() {
			return Ok(OpenResult::FileNotFound {
				path: path.to_string_lossy().to_string(),
			});
		}

		match open_on_host(path) {
			Ok(_) => Ok(OpenResult::Success),
			Err(e) => Ok(OpenResult::PlatformError {
				message: e.to_string(),
			}),
		}
	}

	fn open_with_app(&self, path: &Path, app_id: &str) -> Result<OpenResult, String> {
		let mut results = self.open_files_with_app(&[path.to_path_buf()], app_id)?;
		Ok(results.pop().unwrap_or(OpenResult::Success))
	}

	fn open_files_with_app(
		&self,
		paths: &[PathBuf],
		app_id: &str,
	) -> Result<Vec<OpenResult>, String> {
		if let Some(missing) = paths.iter().find(|path| !path.exists()) {
			return Ok(vec![OpenResult::FileNotFound {
				path: missing.to_string_lossy().to_string(),
			}]);
		}
		let entries = mime_apps::load_entries(&XdgDirs::from_env());
		let Some(entry) = entries.get(app_id) else {
			return Ok(vec![OpenResult::AppNotFound {
				app_id: app_id.to_string(),
			}]);
		};
		let mut results = Vec::new();
		for args in desktop_entry::command_lines(entry, paths)? {
			let result = match launch(entry, args) {
				Ok(()) => OpenResult::Success,
				Err(message) => OpenResult::PlatformError { message },
			};
			results.push(result);
		}
		Ok(results)
	}

	fn set_default_app(&self, path: &Path, app_id: &str) -> Result<(), String> {
		let mime = mime_apps::detect_mime(&XdgDirs::from_env(), path)
			.ok_or("Could not detect the file type")?;
		let output = use_host_environment(&mut Command::new("xdg-mime"))
			.args(["default", app_id, &mime])
			.output()
			.map_err(|e| e.to_string())?;
		if output.status.success() {
			Ok(())
		} else {
			Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
		}
	}
}

/// Starts an application without waiting for it, wrapping it in a terminal
/// when its entry asks for one.
fn launch(entry: &DesktopEntry, args: Vec<String>) -> Result<(), String> {
	let (program, rest) = args.split_first().ok_or("Empty Exec command")?;
	let mut command = if entry.terminal {
		let terminal = terminal::resolve(None).ok_or("No terminal emulator found")?;
		let home = std::env::var_os("HOME")
			.map(PathBuf::from)
			.unwrap_or_else(|| "/".into());
		terminal.command(&home, &args)
	} else {
		let mut command = Command::new(program);
		use_host_environment(&mut command).args(rest);
		command
	};
	let mut child = command
		.stdin(Stdio::null())
		.stdout(Stdio::null())
		.stderr(Stdio::null())
		.spawn()
		.map_err(|e| format!("Failed to start {}: {e}", entry.name))?;
	// Reap the child when it exits so it does not linger as a zombie.
	std::thread::spawn(move || {
		let _ = child.wait();
	});
	Ok(())
}
