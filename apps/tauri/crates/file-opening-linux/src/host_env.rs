//! # Host process environment
//!
//! AppImage launchers prepend the bundle's own libraries, GTK modules and data
//! directories to the environment so the app finds them. Host programs that
//! WingDrive spawns, such as `gdbus`, `xdg-open` or `systemctl`, inherit that
//! environment and load the bundle's older libraries instead of their own,
//! failing with errors like `undefined symbol: g_variant_builder_init_static`.
//! This module removes those bundle entries from a child's environment so the
//! child sees the host as if WingDrive had not been launched from a bundle.
//!
//! ## Example
//! ```rust,no_run
//! use file_opening_linux::use_host_environment;
//!
//! let mut command = std::process::Command::new("xdg-open");
//! use_host_environment(&mut command).arg("/tmp");
//! ```

use std::ffi::{OsStr, OsString};
use std::io;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::process::{Command, Stdio};

/// GDK backend order WingDrive sets for itself when the user chose none.
///
/// Children must not inherit it, or apps opened from WingDrive would start on
/// XWayland instead of the session's native backend.
pub const PREFERRED_GDK_BACKEND: &str = "x11,wayland";

/// Variables the bundle launcher sets that only make sense inside the bundle.
const LAUNCHER_ONLY: &[&str] = &[
	"APPDIR",
	"APPIMAGE",
	"ARGV0",
	"OWD",
	"GTK_THEME",
	"GST_REGISTRY_REUSE_PLUGIN_SCANNER",
];

/// Removes bundle-specific variables from `command`'s environment.
///
/// Outside a bundle (`APPDIR` unset) only the GDK backend preference that
/// WingDrive injected is removed, so native packages behave as before.
pub fn use_host_environment(command: &mut Command) -> &mut Command {
	for (key, value) in host_overrides(std::env::vars_os()) {
		match value {
			Some(value) => command.env(key, value),
			None => command.env_remove(key),
		};
	}
	command
}

/// Opens a path or URL with the host's default handler.
///
/// Mirrors `open::that`, which tries each desktop opener until one succeeds,
/// but runs every candidate with the host environment.
pub fn open_on_host(target: impl AsRef<OsStr>) -> io::Result<()> {
	let mut last_error = None;
	for mut command in open::commands(target) {
		let status = use_host_environment(&mut command)
			.stdin(Stdio::null())
			.stdout(Stdio::null())
			.stderr(Stdio::null())
			.status();
		match status {
			Ok(status) if status.success() => return Ok(()),
			Ok(status) => {
				last_error = Some(io::Error::other(format!(
					"{:?} exited with {status}",
					command.get_program()
				)))
			}
			Err(error) => last_error = Some(error),
		}
	}
	Err(last_error.unwrap_or_else(|| io::Error::other("no opener available")))
}

/// Computes the changes that turn `vars` into a host environment.
///
/// `Some` replaces a variable with its value minus bundle entries; `None`
/// removes it. List variables keep the user's own entries, so a user-set
/// `LD_LIBRARY_PATH` or `XDG_DATA_DIRS` survives.
fn host_overrides(
	vars: impl IntoIterator<Item = (OsString, OsString)>,
) -> Vec<(OsString, Option<OsString>)> {
	let vars: Vec<_> = vars.into_iter().collect();
	let app_dir = vars
		.iter()
		.find(|(key, _)| key == "APPDIR")
		.map(|(_, value)| Path::new(value).to_path_buf())
		.filter(|dir| dir.is_absolute() && dir != Path::new("/"));

	let mut overrides = Vec::new();
	for (key, value) in vars {
		if key == "GDK_BACKEND" && value == PREFERRED_GDK_BACKEND {
			overrides.push((key, None));
			continue;
		}
		let Some(app_dir) = &app_dir else {
			continue;
		};
		if LAUNCHER_ONLY.iter().any(|name| key == *name) {
			overrides.push((key, None));
			continue;
		}
		let entries: Vec<&[u8]> = value.as_bytes().split(|byte| *byte == b':').collect();
		let host: Vec<&[u8]> = entries
			.iter()
			.copied()
			.filter(|entry| {
				!entry.is_empty() && !Path::new(OsStr::from_bytes(entry)).starts_with(app_dir)
			})
			.collect();
		if host.len() == entries.len() {
			continue;
		}
		let value = (!host.is_empty()).then(|| OsStr::from_bytes(&host.join(&b':')).to_os_string());
		overrides.push((key, value));
	}
	overrides
}

#[cfg(test)]
mod tests {
	use super::*;

	fn overrides(vars: &[(&str, &str)]) -> Vec<(String, Option<String>)> {
		let vars = vars
			.iter()
			.map(|(k, v)| (OsString::from(k), OsString::from(v)));
		host_overrides(vars)
			.into_iter()
			.map(|(k, v)| {
				(
					k.into_string().unwrap(),
					v.map(|v| v.into_string().unwrap()),
				)
			})
			.collect()
	}

	#[test]
	fn strips_bundle_entries_and_keeps_user_entries() {
		let result = overrides(&[
			("APPDIR", "/opt/wingdrive"),
			(
				"LD_LIBRARY_PATH",
				"/opt/wingdrive/usr/lib/:/opt/wingdrive/lib/:/home/u/lib",
			),
			("XDG_DATA_DIRS", "/opt/wingdrive/usr/share:/usr/share:"),
			("GIO_MODULE_DIR", "/opt/wingdrive//usr/lib/gio/modules"),
			("GTK_THEME", "Adwaita:dark"),
			("HOME", "/home/u"),
			("PATH", "/usr/bin"),
		]);
		assert_eq!(
			result,
			vec![
				("APPDIR".into(), None),
				("LD_LIBRARY_PATH".into(), Some("/home/u/lib".into())),
				("XDG_DATA_DIRS".into(), Some("/usr/share".into())),
				("GIO_MODULE_DIR".into(), None),
				("GTK_THEME".into(), None),
			]
		);
	}

	#[test]
	fn sibling_directory_with_shared_prefix_is_kept() {
		let result = overrides(&[
			("APPDIR", "/opt/wingdrive"),
			("LD_LIBRARY_PATH", "/opt/wingdrive-extra/lib"),
		]);
		assert_eq!(result, vec![("APPDIR".into(), None)]);
	}

	#[test]
	fn outside_a_bundle_only_the_injected_gdk_backend_is_removed() {
		let result = overrides(&[
			("GDK_BACKEND", PREFERRED_GDK_BACKEND),
			("GTK_THEME", "Adwaita:dark"),
			("LD_LIBRARY_PATH", "/home/u/lib"),
		]);
		assert_eq!(result, vec![("GDK_BACKEND".into(), None)]);

		let user_choice = overrides(&[("GDK_BACKEND", "wayland")]);
		assert!(user_choice.is_empty());
	}
}
