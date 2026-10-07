//! # MIME associations
//!
//! Finds the applications that open a file the way the desktop does: detect
//! the file's MIME type, widen it through shared-mime-info's subclass tree
//! (a Python script is also `text/plain`), collect desktop entries that
//! declare any of those types, then order them by the user's `mimeapps.list`
//! defaults and added associations.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::desktop_entry::{self, DesktopEntry};
use crate::host_env::use_host_environment;

/// XDG base directories, injectable so tests run against fixtures.
#[derive(Debug, Clone)]
pub struct XdgDirs {
	pub config_home: PathBuf,
	pub config_dirs: Vec<PathBuf>,
	pub data_home: PathBuf,
	pub data_dirs: Vec<PathBuf>,
	/// Lowercased `XDG_CURRENT_DESKTOP` names, for `<desktop>-mimeapps.list`.
	pub desktops: Vec<String>,
}

impl XdgDirs {
	pub fn from_env() -> Self {
		let home = std::env::var_os("HOME")
			.map(PathBuf::from)
			.unwrap_or_default();
		let dir = |var: &str, fallback: PathBuf| {
			std::env::var_os(var)
				.map(PathBuf::from)
				.filter(|p| p.is_absolute())
				.unwrap_or(fallback)
		};
		let dirs = |var: &str, fallback: &str| -> Vec<PathBuf> {
			let value = std::env::var(var).ok().filter(|v| !v.is_empty());
			value
				.as_deref()
				.unwrap_or(fallback)
				.split(':')
				.map(PathBuf::from)
				.filter(|p| p.is_absolute())
				.collect()
		};
		Self {
			config_home: dir("XDG_CONFIG_HOME", home.join(".config")),
			config_dirs: dirs("XDG_CONFIG_DIRS", "/etc/xdg"),
			data_home: dir("XDG_DATA_HOME", home.join(".local/share")),
			data_dirs: dirs("XDG_DATA_DIRS", "/usr/local/share:/usr/share"),
			desktops: std::env::var("XDG_CURRENT_DESKTOP")
				.unwrap_or_default()
				.split(':')
				.filter(|d| !d.is_empty())
				.map(str::to_lowercase)
				.collect(),
		}
	}

	fn data_search(&self) -> impl Iterator<Item = &PathBuf> {
		std::iter::once(&self.data_home).chain(&self.data_dirs)
	}

	/// mimeapps.list files in precedence order, highest first.
	fn mimeapps_files(&self) -> Vec<PathBuf> {
		let mut files = Vec::new();
		let mut add_dir = |dir: &Path| {
			for desktop in &self.desktops {
				files.push(dir.join(format!("{desktop}-mimeapps.list")));
			}
			files.push(dir.join("mimeapps.list"));
		};
		add_dir(&self.config_home);
		for dir in &self.config_dirs {
			add_dir(dir);
		}
		for dir in self.data_search() {
			add_dir(&dir.join("applications"));
		}
		files
	}
}

/// All launchable desktop entries, keyed by desktop file ID.
///
/// Earlier data directories win, so a user's copy in `~/.local/share`
/// overrides the system one, including a `Hidden=true` copy that removes it.
pub fn load_entries(dirs: &XdgDirs) -> HashMap<String, DesktopEntry> {
	let locales = desktop_entry::locale_keys();
	let mut seen = HashSet::new();
	let mut entries = HashMap::new();
	for data_dir in dirs.data_search() {
		let root = data_dir.join("applications");
		for path in desktop_files(&root) {
			let Ok(relative) = path.strip_prefix(&root) else {
				continue;
			};
			let id = relative.to_string_lossy().replace('/', "-");
			if !seen.insert(id.clone()) {
				continue;
			}
			let Ok(contents) = std::fs::read_to_string(&path) else {
				continue;
			};
			if let Some(entry) = desktop_entry::parse(&id, &path, &contents, &locales) {
				entries.insert(id, entry);
			}
		}
	}
	entries
}

fn desktop_files(root: &Path) -> Vec<PathBuf> {
	let mut files = Vec::new();
	let mut stack = vec![root.to_path_buf()];
	while let Some(dir) = stack.pop() {
		let Ok(read) = std::fs::read_dir(&dir) else {
			continue;
		};
		for item in read.flatten() {
			let path = item.path();
			if path.is_dir() {
				stack.push(path);
			} else if path.extension().is_some_and(|ext| ext == "desktop") {
				files.push(path);
			}
		}
	}
	files.sort();
	files
}

/// Associations from every mimeapps.list, merged by precedence.
#[derive(Debug, Default)]
pub struct Associations {
	defaults: HashMap<String, Vec<String>>,
	added: HashMap<String, Vec<String>>,
	removed: HashMap<String, HashSet<String>>,
}

impl Associations {
	pub fn load(dirs: &XdgDirs) -> Self {
		let mut associations = Self::default();
		for file in dirs.mimeapps_files() {
			if let Ok(contents) = std::fs::read_to_string(&file) {
				associations.merge(&contents);
			}
		}
		associations
	}

	/// Adds a lower-precedence file: existing entries stay first.
	fn merge(&mut self, contents: &str) {
		let mut group = "";
		for line in contents.lines().map(str::trim) {
			if line.starts_with('[') {
				group = line;
				continue;
			}
			let Some((mime, apps)) = line.split_once('=') else {
				continue;
			};
			let apps = apps.split(';').map(str::trim).filter(|a| !a.is_empty());
			let mime = mime.trim().to_string();
			match group {
				"[Default Applications]" => {
					extend_unique(self.defaults.entry(mime).or_default(), apps)
				}
				"[Added Associations]" => extend_unique(self.added.entry(mime).or_default(), apps),
				"[Removed Associations]" => self
					.removed
					.entry(mime)
					.or_default()
					.extend(apps.map(str::to_string)),
				_ => {}
			}
		}
	}
}

fn extend_unique<'a>(list: &mut Vec<String>, apps: impl Iterator<Item = &'a str>) {
	for app in apps {
		if !list.iter().any(|existing| existing == app) {
			list.push(app.to_string());
		}
	}
}

/// The MIME type of `path` as the desktop sees it.
///
/// Like GIO, the file name's shared-mime-info glob decides first and content
/// sniffing only runs when no glob matches, so `README.md` stays Markdown
/// even when it contains HTML.
pub fn detect_mime(dirs: &XdgDirs, path: &Path) -> Option<String> {
	if path.is_dir() {
		return Some("inode/directory".to_string());
	}
	if let Some(mime) = path
		.file_name()
		.and_then(|name| name.to_str())
		.and_then(|name| glob_mime(dirs, name))
	{
		return Some(mime);
	}
	let query = |program: &str, args: &[&str]| {
		let output = use_host_environment(&mut Command::new(program))
			.args(args)
			.arg(path)
			.output()
			.ok()?;
		let mime = String::from_utf8(output.stdout).ok()?.trim().to_string();
		(output.status.success() && mime.contains('/')).then_some(mime)
	};
	query("xdg-mime", &["query", "filetype"]).or_else(|| query("file", &["--brief", "--mime-type"]))
}

/// Best `globs2` match for a file name: highest weight, then longest pattern.
///
/// Only literal names and `*.suffix` patterns are matched; they cover nearly
/// every glob shared-mime-info ships, and the rest fall back to sniffing.
fn glob_mime(dirs: &XdgDirs, name: &str) -> Option<String> {
	let lower = name.to_lowercase();
	let mut best: Option<(u32, usize, String)> = None;
	for dir in dirs.data_search() {
		let Ok(text) = std::fs::read_to_string(dir.join("mime/globs2")) else {
			continue;
		};
		for line in text.lines().filter(|l| !l.starts_with('#')) {
			let mut fields = line.split(':');
			let (Some(weight), Some(mime), Some(glob)) =
				(fields.next(), fields.next(), fields.next())
			else {
				continue;
			};
			let case_sensitive = fields
				.next()
				.is_some_and(|flags| flags.split(',').any(|f| f == "cs"));
			let subject = if case_sensitive { name } else { lower.as_str() };
			let pattern = if case_sensitive {
				glob.to_string()
			} else {
				glob.to_lowercase()
			};
			let matched = match pattern.strip_prefix('*') {
				Some(suffix) if !suffix.contains(['*', '?', '[']) => subject.ends_with(suffix),
				_ if !pattern.contains(['*', '?', '[']) => subject == pattern,
				_ => false,
			};
			let Ok(weight) = weight.parse::<u32>() else {
				continue;
			};
			if matched
				&& best
					.as_ref()
					.is_none_or(|(w, len, _)| (weight, glob.len()) > (*w, *len))
			{
				best = Some((weight, glob.len(), mime.to_string()));
			}
		}
	}
	best.map(|(_, _, mime)| mime)
}

/// The type itself, then its ancestors from shared-mime-info, nearest first.
pub fn mime_lineage(dirs: &XdgDirs, mime: &str) -> Vec<String> {
	let mut parents: HashMap<String, Vec<String>> = HashMap::new();
	let mut aliases: HashMap<String, String> = HashMap::new();
	for dir in dirs.data_search() {
		if let Ok(text) = std::fs::read_to_string(dir.join("mime/subclasses")) {
			for (child, parent) in text.lines().filter_map(|l| l.split_once(' ')) {
				parents
					.entry(child.to_string())
					.or_default()
					.push(parent.to_string());
			}
		}
		if let Ok(text) = std::fs::read_to_string(dir.join("mime/aliases")) {
			for (alias, canonical) in text.lines().filter_map(|l| l.split_once(' ')) {
				aliases
					.entry(alias.to_string())
					.or_insert_with(|| canonical.to_string());
			}
		}
	}
	let start = aliases
		.get(mime)
		.cloned()
		.unwrap_or_else(|| mime.to_string());
	let mut lineage = vec![start.clone()];
	let mut index = 0;
	while index < lineage.len() {
		let current = lineage[index].clone();
		let mut next: Vec<String> = parents.get(&current).cloned().unwrap_or_default();
		// Implicit rules from the shared-mime-info spec.
		if current.starts_with("text/") && current != "text/plain" {
			next.push("text/plain".to_string());
		}
		if current != "application/octet-stream" && !current.starts_with("inode/") {
			next.push("application/octet-stream".to_string());
		}
		for parent in next {
			if !lineage.contains(&parent) {
				lineage.push(parent);
			}
		}
		index += 1;
	}
	// The catch-all handler belongs at the very end.
	if let Some(pos) = lineage.iter().position(|m| m == "application/octet-stream") {
		let catch_all = lineage.remove(pos);
		lineage.push(catch_all);
	}
	lineage
}

/// The configured default handler: the first valid `[Default Applications]`
/// entry along the lineage, as GIO resolves it.
pub fn default_app(
	lineage: &[String],
	entries: &HashMap<String, DesktopEntry>,
	associations: &Associations,
) -> Option<String> {
	lineage.iter().find_map(|mime| {
		associations
			.defaults
			.get(mime)?
			.iter()
			.find(|id| entries.contains_key(*id))
			.cloned()
	})
}

/// Applications for one file, default first, without duplicates.
pub fn apps_for_mime(
	lineage: &[String],
	entries: &HashMap<String, DesktopEntry>,
	associations: &Associations,
) -> Vec<DesktopEntry> {
	let mut ordered: Vec<String> = Vec::new();
	let mut add = |id: &str, mime: &str| {
		let removed = associations
			.removed
			.get(mime)
			.is_some_and(|set| set.contains(id));
		if !removed && entries.contains_key(id) && !ordered.iter().any(|e| e == id) {
			ordered.push(id.to_string());
		}
	};
	for mime in lineage {
		for id in associations.defaults.get(mime).into_iter().flatten() {
			add(id, mime);
		}
	}
	for mime in lineage {
		for id in associations.added.get(mime).into_iter().flatten() {
			add(id, mime);
		}
		let mut declared: Vec<&DesktopEntry> = entries
			.values()
			.filter(|e| !e.no_display && e.mime_types.iter().any(|m| m == mime))
			.collect();
		declared.sort_by(|a, b| {
			a.name
				.to_lowercase()
				.cmp(&b.name.to_lowercase())
				.then(a.id.cmp(&b.id))
		});
		for entry in declared {
			add(&entry.id, mime);
		}
	}
	ordered
		.iter()
		.filter_map(|id| entries.get(id).cloned())
		.collect()
}

#[cfg(test)]
mod tests {
	use super::*;

	fn write(path: &Path, contents: &str) {
		std::fs::create_dir_all(path.parent().unwrap()).unwrap();
		std::fs::write(path, contents).unwrap();
	}

	fn app(name: &str, mimes: &str) -> String {
		format!(
			"[Desktop Entry]\nType=Application\nName={name}\nExec={name} %F\nMimeType={mimes}\n"
		)
	}

	#[test]
	fn globs_pick_weight_then_longest_pattern() {
		let root = tempfile::tempdir().unwrap();
		let dirs = XdgDirs {
			config_home: root.path().join("config"),
			config_dirs: vec![],
			data_home: root.path().join("home"),
			data_dirs: vec![root.path().join("system")],
			desktops: vec![],
		};
		write(
			&root.path().join("system/mime/globs2"),
			"50:text/markdown:*.md\n10:application/x-genesis-rom:*.md\n50:application/x-compressed-tar:*.tar.gz\n50:application/gzip:*.gz\n50:text/x-makefile:Makefile:cs\n",
		);
		assert_eq!(
			glob_mime(&dirs, "README.MD").as_deref(),
			Some("text/markdown")
		);
		assert_eq!(
			glob_mime(&dirs, "a.tar.gz").as_deref(),
			Some("application/x-compressed-tar")
		);
		assert_eq!(
			glob_mime(&dirs, "Makefile").as_deref(),
			Some("text/x-makefile")
		);
		assert_eq!(glob_mime(&dirs, "makefile"), None);
	}

	#[test]
	fn defaults_added_declared_and_inherited_handlers_in_order() {
		let root = tempfile::tempdir().unwrap();
		let dirs = XdgDirs {
			config_home: root.path().join("config"),
			config_dirs: vec![],
			data_home: root.path().join("home"),
			data_dirs: vec![root.path().join("system")],
			desktops: vec!["hyprland".into()],
		};
		let system = root.path().join("system");
		write(
			&system.join("mime/subclasses"),
			"text/x-python text/plain\n",
		);
		write(
			&system.join("applications/zed.desktop"),
			&app("Zed", "text/x-python;"),
		);
		write(
			&system.join("applications/editor.desktop"),
			&app("Editor", "text/plain;"),
		);
		write(
			&system.join("applications/hex.desktop"),
			&app("Hex", "application/octet-stream;"),
		);
		write(
			&system.join("applications/kde/viewer.desktop"),
			&app("Viewer", "text/plain;"),
		);
		write(
			&system.join("applications/gone.desktop"),
			&app("Gone", "text/x-python;"),
		);
		// The user's copy hides the system entry.
		write(
			&root.path().join("home/applications/gone.desktop"),
			"[Desktop Entry]\nType=Application\nName=Gone\nExec=gone\nHidden=true\n",
		);
		write(
			&root.path().join("config/mimeapps.list"),
			"[Default Applications]\ntext/plain=kde-viewer.desktop\n[Removed Associations]\ntext/plain=editor.desktop\n",
		);
		write(
			&root.path().join("config/hyprland-mimeapps.list"),
			"[Default Applications]\ntext/x-python=hex.desktop\n",
		);

		let entries = load_entries(&dirs);
		let lineage = mime_lineage(&dirs, "text/x-python");
		assert_eq!(
			lineage,
			vec!["text/x-python", "text/plain", "application/octet-stream"]
		);
		let ids: Vec<_> = apps_for_mime(&lineage, &entries, &Associations::load(&dirs))
			.into_iter()
			.map(|e| e.id)
			.collect();
		assert_eq!(
			ids,
			vec!["hex.desktop", "kde-viewer.desktop", "zed.desktop"]
		);
	}
}
