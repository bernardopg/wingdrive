//! # Terminal emulator
//!
//! "Open Terminal Here" and `Terminal=true` applications need the user's
//! terminal. Linux has no single setting for it, so this follows the order
//! most desktops use: an explicit override, the `xdg-terminal-exec` proposal,
//! `$TERMINAL`, then well-known emulators.

use std::path::Path;
use std::process::Command;

use crate::desktop_entry::program_exists;
use crate::host_env::use_host_environment;

/// Emulators tried when nothing is configured, with the flag that runs a
/// command (`None` when the rest of the arguments are the command).
const KNOWN: &[(&str, Option<&str>)] = &[
	("ghostty", Some("-e")),
	("kitty", None),
	("foot", None),
	("alacritty", Some("-e")),
	("wezterm", Some("start --")),
	("konsole", Some("-e")),
	("gnome-terminal", Some("--")),
	("kgx", Some("--")),
	("ptyxis", Some("--")),
	("xfce4-terminal", Some("-x")),
	("tilix", Some("-e")),
	("terminator", Some("-x")),
	("xterm", Some("-e")),
];

/// A terminal command and how it takes a command to run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Terminal {
	pub program: Vec<String>,
	pub exec_flag: Option<Vec<String>>,
}

/// Picks the terminal to use. `override_command` is a full command line from
/// settings, such as `kitty --single-instance`.
pub fn resolve(override_command: Option<&str>) -> Option<Terminal> {
	if let Some(command) = override_command.map(str::trim).filter(|c| !c.is_empty()) {
		let program: Vec<String> = command.split_whitespace().map(str::to_string).collect();
		let flag = flag_for(program.first()?);
		return Some(Terminal {
			program,
			exec_flag: flag,
		});
	}
	if program_exists("xdg-terminal-exec") {
		return Some(Terminal {
			program: vec!["xdg-terminal-exec".to_string()],
			exec_flag: None,
		});
	}
	if let Ok(terminal) = std::env::var("TERMINAL") {
		let program: Vec<String> = terminal.split_whitespace().map(str::to_string).collect();
		if program.first().is_some_and(|p| program_exists(p)) {
			let flag = flag_for(&program[0]);
			return Some(Terminal {
				program,
				exec_flag: flag,
			});
		}
	}
	KNOWN
		.iter()
		.find(|(name, _)| program_exists(name))
		.map(|(name, flag)| Terminal {
			program: vec![name.to_string()],
			exec_flag: flag.map(|f| f.split(' ').map(str::to_string).collect()),
		})
}

fn flag_for(program: &str) -> Option<Vec<String>> {
	let name = Path::new(program).file_name()?.to_str()?;
	KNOWN
		.iter()
		.find(|(known, _)| *known == name)
		.map(|(_, flag)| flag.map(|f| f.split(' ').map(str::to_string).collect()))
		.unwrap_or_else(|| Some(vec!["-e".to_string()]))
}

impl Terminal {
	/// Command that opens the terminal in `directory`, optionally running `command`.
	pub fn command(&self, directory: &Path, command: &[String]) -> Command {
		let mut cmd = Command::new(&self.program[0]);
		use_host_environment(&mut cmd)
			.args(&self.program[1..])
			.current_dir(directory);
		if !command.is_empty() {
			cmd.args(self.exec_flag.iter().flatten()).args(command);
		}
		cmd
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn override_keeps_arguments_and_knows_exec_flags() {
		let kitty = resolve(Some("kitty --single-instance")).unwrap();
		assert_eq!(kitty.program, vec!["kitty", "--single-instance"]);
		assert_eq!(kitty.exec_flag, None);

		let wezterm = resolve(Some("/usr/bin/wezterm")).unwrap();
		assert_eq!(wezterm.exec_flag, Some(vec!["start".into(), "--".into()]));

		let unknown = resolve(Some("myterm")).unwrap();
		assert_eq!(unknown.exec_flag, Some(vec!["-e".into()]));
	}

	#[test]
	fn command_runs_in_directory_with_exec_flag() {
		let terminal = Terminal {
			program: vec!["xterm".into()],
			exec_flag: Some(vec!["-e".into()]),
		};
		let cmd = terminal.command(Path::new("/tmp"), &["htop".into()]);
		let args: Vec<_> = cmd
			.get_args()
			.map(|a| a.to_string_lossy().into_owned())
			.collect();
		assert_eq!(args, vec!["-e", "htop"]);
		assert_eq!(cmd.get_current_dir(), Some(Path::new("/tmp")));
	}
}
