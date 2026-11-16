//! # Child processes without a console window
//!
//! On Windows the daemon runs without a console (TAURI-019), and every console
//! program it starts (`git`, `ffmpeg`, `cmd`) would otherwise open a window of
//! its own on the user's desktop. Commands built here never do. Elsewhere this
//! changes nothing.
//!
//! ## Example
//! ```rust,ignore
//! let mut command = std::process::Command::new("git");
//! hide_console_window(&mut command);
//! ```

/// `CREATE_NO_WINDOW` from the Windows process creation flags.
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// Keeps a console program started by the daemon from opening a window.
pub fn hide_console_window(command: &mut std::process::Command) -> &mut std::process::Command {
	#[cfg(windows)]
	{
		use std::os::windows::process::CommandExt;
		command.creation_flags(CREATE_NO_WINDOW);
	}
	command
}

/// [`hide_console_window`] for Tokio commands.
pub fn hide_console_window_async(
	command: &mut tokio::process::Command,
) -> &mut tokio::process::Command {
	#[cfg(windows)]
	command.creation_flags(CREATE_NO_WINDOW);
	command
}
