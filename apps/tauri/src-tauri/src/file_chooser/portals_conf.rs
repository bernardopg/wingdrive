//! Opting in to WingDrive as the FileChooser backend by editing
//! `xdg-desktop-portal`'s `portals.conf`.
//!
//! `xdg-desktop-portal` reads one configuration file per directory: the first
//! `<desktop>-portals.conf` for an entry of `$XDG_CURRENT_DESKTOP` (lowercased)
//! that has a `[preferred]` key, otherwise `portals.conf`. The user's directory
//! comes first, so editing a `portals.conf` next to a `hyprland-portals.conf`
//! would change nothing. The switch therefore edits the file the portal reads.
//!
//! Enabling writes `wingdrive` first and the previous backend after it, and
//! remembers the previous value. A list only falls back when the first backend
//! is not installed (its `.portal` file is missing), not when it fails to
//! start: `xdg-desktop-portal` 1.22 picks one backend per interface at start-up
//! (`xdp_portal_config_find` in `src/xdp-portal-config.c`).

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// The key `portals.conf` uses for the FileChooser backend.
pub const INTERFACE_KEY: &str = "org.freedesktop.impl.portal.FileChooser";
/// The backend's name: `wingdrive.portal` without its extension.
pub const BACKEND: &str = "wingdrive";
/// The portal description file packages install.
pub const PORTAL_FILE: &str = "wingdrive.portal";

const SECTION: &str = "preferred";

/// The lowercased desktop names of `$XDG_CURRENT_DESKTOP`, as the portal reads them.
pub fn current_desktops(value: Option<&str>) -> Vec<String> {
	value
		.unwrap_or_default()
		.split(':')
		.map(str::trim)
		// The portal skips names that could escape the directory.
		.filter(|name| {
			!name.is_empty()
				&& name
					.chars()
					.all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_'))
		})
		.map(str::to_ascii_lowercase)
		.collect()
}

/// The file the portal reads in `config_dir`, which the switch edits.
pub fn config_file(config_dir: &Path, desktops: &[String]) -> PathBuf {
	desktops
		.iter()
		.map(|desktop| config_dir.join(format!("{desktop}-portals.conf")))
		.find(|path| std::fs::read_to_string(path).is_ok_and(|text| has_preferred_keys(&text)))
		.unwrap_or_else(|| config_dir.join("portals.conf"))
}

/// Whether the portal would use this text: it needs a `[preferred]` key.
pub fn has_preferred_keys(text: &str) -> bool {
	let mut in_section = false;
	for line in text.lines() {
		match parse_line(line) {
			Line::Section(name) => in_section = name == SECTION,
			Line::Key(_, _) if in_section => return true,
			_ => {}
		}
	}
	false
}

enum Line<'a> {
	Section(&'a str),
	Key(&'a str, &'a str),
	Other,
}

fn parse_line(line: &str) -> Line<'_> {
	let trimmed = line.trim();
	if trimmed.starts_with('#') || trimmed.is_empty() {
		return Line::Other;
	}
	if let Some(name) = trimmed
		.strip_prefix('[')
		.and_then(|rest| rest.strip_suffix(']'))
	{
		return Line::Section(name.trim());
	}
	match trimmed.split_once('=') {
		Some((key, value)) => Line::Key(key.trim(), value.trim()),
		None => Line::Other,
	}
}

/// The FileChooser value in `[preferred]`, if the file sets one.
pub fn file_chooser_value(text: &str) -> Option<String> {
	let mut in_section = false;
	let mut value = None;
	for line in text.lines() {
		match parse_line(line) {
			Line::Section(name) => in_section = name == SECTION,
			// GKeyFile keeps the last of duplicate keys.
			Line::Key(key, found) if in_section && key == INTERFACE_KEY => {
				value = Some(found.to_string())
			}
			_ => {}
		}
	}
	value
}

/// Sets (or with `None` removes) the FileChooser line, keeping every other
/// line, comment and section as it was.
pub fn set_file_chooser(text: &str, value: Option<&str>) -> String {
	let mut lines: Vec<String> = Vec::new();
	let mut in_section = false;
	let mut section_end: Option<usize> = None;
	let mut written = false;
	for line in text.lines() {
		match parse_line(line) {
			Line::Section(name) => {
				if in_section && section_end.is_none() {
					section_end = Some(lines.len());
				}
				in_section = name == SECTION;
			}
			Line::Key(key, _) if in_section && key == INTERFACE_KEY => {
				if let (Some(value), false) = (value, written) {
					lines.push(format!("{INTERFACE_KEY}={value}"));
					written = true;
				}
				continue;
			}
			_ => {}
		}
		lines.push(line.to_string());
	}
	if let (Some(value), false) = (value, written) {
		let entry = format!("{INTERFACE_KEY}={value}");
		match (in_section, section_end) {
			(true, None) => {
				// The section runs to the end; keep trailing blank lines after the key.
				let end = lines
					.iter()
					.rposition(|line| !line.trim().is_empty())
					.map_or(lines.len(), |index| index + 1);
				lines.insert(end, entry);
			}
			(_, Some(end)) => {
				let end = lines[..end]
					.iter()
					.rposition(|line| !line.trim().is_empty())
					.map_or(end, |index| index + 1);
				lines.insert(end, entry);
			}
			(false, None) => {
				if lines.last().is_some_and(|line| !line.trim().is_empty()) {
					lines.push(String::new());
				}
				lines.push(format!("[{SECTION}]"));
				lines.push(entry);
			}
		}
	}
	let mut output = lines.join("\n");
	if !output.is_empty() {
		output.push('\n');
	}
	output
}

/// The entries of a `;`-separated list.
fn entries(value: &str) -> Vec<&str> {
	value
		.split(';')
		.map(str::trim)
		.filter(|entry| !entry.is_empty())
		.collect()
}

/// Whether a value puts WingDrive first.
pub fn is_enabled(value: Option<&str>) -> bool {
	value.is_some_and(|value| entries(value).first() == Some(&BACKEND))
}

/// The list that puts WingDrive first and keeps the previous backends after
/// it, so the portal still finds one when WingDrive is uninstalled.
pub fn enabled_value(previous: Option<&str>) -> String {
	let mut list = vec![BACKEND];
	if let Some(previous) = previous {
		list.extend(
			entries(previous)
				.into_iter()
				.filter(|entry| *entry != BACKEND),
		);
	}
	list.join(";")
}

/// The value to write back when the switch turns off.
///
/// The stored previous value wins. Without one (state lost, or written by an
/// older WingDrive), dropping `wingdrive` from the list gives the same result
/// for a list the switch wrote.
pub fn restored_value(current: Option<&str>, stored: Option<&StoredPrevious>) -> Option<String> {
	if let Some(stored) = stored {
		return stored.previous.clone();
	}
	let rest: Vec<&str> = entries(current.unwrap_or_default())
		.into_iter()
		.filter(|entry| *entry != BACKEND)
		.collect();
	(!rest.is_empty()).then(|| rest.join(";"))
}

/// The previous FileChooser value, kept in WingDrive's data folder.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoredPrevious {
	/// The file the switch edited.
	pub file: PathBuf,
	/// Its FileChooser value before; `None` when it had no such line.
	pub previous: Option<String>,
}

/// What the Settings switch shows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PortalStatus {
	pub enabled: bool,
	/// The file the switch edits.
	pub config_file: PathBuf,
	/// Its current FileChooser value.
	pub value: Option<String>,
	/// Whether `wingdrive.portal` is installed where the portal looks.
	pub backend_installed: bool,
}

/// Paths the switch works with; tests point them at temporary folders.
#[derive(Debug, Clone)]
pub struct PortalPaths {
	/// `$XDG_CONFIG_HOME/xdg-desktop-portal`.
	pub config_dir: PathBuf,
	pub desktops: Vec<String>,
	/// Where the previous value is stored.
	pub state_file: PathBuf,
	/// Folders that may hold `wingdrive.portal`.
	pub portal_dirs: Vec<PathBuf>,
}

impl PortalPaths {
	/// The paths of this session. `XDG_DESKTOP_PORTAL_DIR` replaces every
	/// portal folder, as it does for the portal itself.
	pub fn from_env(state_file: PathBuf) -> Self {
		Self::from_vars(state_file, |name| std::env::var_os(name))
	}

	/// The paths for an environment read through `var`, so tests never touch
	/// the process environment.
	pub fn from_vars(
		state_file: PathBuf,
		var: impl Fn(&str) -> Option<std::ffi::OsString>,
	) -> Self {
		let home = var("HOME").map(PathBuf::from).unwrap_or_default();
		let env_dir = |name: &str| {
			var(name)
				.map(PathBuf::from)
				.filter(|path| path.is_absolute())
		};
		let text = |name: &str| var(name).and_then(|value| value.into_string().ok());
		let config_home = env_dir("XDG_CONFIG_HOME").unwrap_or_else(|| home.join(".config"));
		let data_home = env_dir("XDG_DATA_HOME").unwrap_or_else(|| home.join(".local/share"));
		let portal_dirs = match env_dir("XDG_DESKTOP_PORTAL_DIR") {
			Some(dir) => vec![dir],
			None => {
				let data_dirs = text("XDG_DATA_DIRS")
					.filter(|value| !value.is_empty())
					.unwrap_or_else(|| "/usr/local/share:/usr/share".to_string());
				std::iter::once(data_home)
					.chain(data_dirs.split(':').map(PathBuf::from))
					.map(|dir| dir.join("xdg-desktop-portal/portals"))
					.collect()
			}
		};
		Self {
			config_dir: config_home.join("xdg-desktop-portal"),
			desktops: current_desktops(text("XDG_CURRENT_DESKTOP").as_deref()),
			state_file,
			portal_dirs,
		}
	}

	fn backend_installed(&self) -> bool {
		self.portal_dirs
			.iter()
			.any(|dir| dir.join(PORTAL_FILE).is_file())
	}

	fn load_state(&self) -> Option<StoredPrevious> {
		let text = std::fs::read_to_string(&self.state_file).ok()?;
		serde_json::from_str(&text).ok()
	}

	/// The file the switch edits: the one it edited last while enabled.
	fn target(&self) -> PathBuf {
		self.load_state()
			.map(|state| state.file)
			.unwrap_or_else(|| config_file(&self.config_dir, &self.desktops))
	}

	pub fn status(&self) -> PortalStatus {
		let config_file = self.target();
		let value = std::fs::read_to_string(&config_file)
			.ok()
			.and_then(|text| file_chooser_value(&text));
		PortalStatus {
			enabled: is_enabled(value.as_deref()),
			config_file,
			value,
			backend_installed: self.backend_installed(),
		}
	}

	/// Puts WingDrive first. Refuses while `wingdrive.portal` is missing,
	/// since the portal would skip the entry and the switch would do nothing.
	pub fn enable(&self) -> Result<PortalStatus, String> {
		if !self.backend_installed() {
			return Err(format!(
				"{PORTAL_FILE} is not installed; reinstall the WingDrive package"
			));
		}
		let file = config_file(&self.config_dir, &self.desktops);
		let text = read_optional(&file)?;
		let current = file_chooser_value(&text);
		if !is_enabled(current.as_deref()) {
			let state = StoredPrevious {
				file: file.clone(),
				previous: current.clone(),
			};
			write_file(
				&self.state_file,
				&serde_json::to_string_pretty(&state).map_err(|e| e.to_string())?,
			)?;
			write_file(
				&file,
				&set_file_chooser(&text, Some(&enabled_value(current.as_deref()))),
			)?;
		}
		Ok(self.status())
	}

	/// Writes the previous value back and forgets it.
	pub fn disable(&self) -> Result<PortalStatus, String> {
		let state = self.load_state();
		let file = state
			.as_ref()
			.map(|state| state.file.clone())
			.unwrap_or_else(|| config_file(&self.config_dir, &self.desktops));
		let text = read_optional(&file)?;
		let current = file_chooser_value(&text);
		if is_enabled(current.as_deref()) {
			let restored = restored_value(current.as_deref(), state.as_ref());
			write_file(&file, &set_file_chooser(&text, restored.as_deref()))?;
		}
		match std::fs::remove_file(&self.state_file) {
			Ok(()) => {}
			Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
			Err(error) => return Err(error.to_string()),
		}
		Ok(self.status())
	}
}

fn read_optional(path: &Path) -> Result<String, String> {
	match std::fs::read_to_string(path) {
		Ok(text) => Ok(text),
		Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
		Err(error) => Err(format!("{}: {error}", path.display())),
	}
}

/// Writes through a temporary file so a crash never leaves half a config.
fn write_file(path: &Path, contents: &str) -> Result<(), String> {
	// Write through a symlink (dotfile managers link these files) instead of replacing it.
	let resolved = std::fs::canonicalize(path).ok();
	let path = resolved.as_deref().unwrap_or(path);
	let parent = path
		.parent()
		.ok_or_else(|| format!("{} has no parent folder", path.display()))?;
	std::fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
	let temp = parent.join(format!(
		".{}.wingdrive-tmp",
		path.file_name().unwrap_or_default().to_string_lossy()
	));
	std::fs::write(&temp, contents).map_err(|e| format!("{}: {e}", temp.display()))?;
	std::fs::rename(&temp, path).map_err(|e| format!("{}: {e}", path.display()))
}
