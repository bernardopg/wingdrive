//! # Service menus
//!
//! Dolphin lets users add commands to the context menu with service menus:
//! desktop files under `kio/servicemenus` in each XDG data directory, with a
//! `MimeType` filter, an `Actions` list and one `[Desktop Action id]` group per
//! command. WingDrive reads the same files, so menus written for Dolphin show
//! up here, and also reads `wingdrive/actions` in the same format for actions
//! meant only for WingDrive.
//!
//! As in KIO, a file in the user's own data directory only counts when it is
//! executable, so a desktop file dropped there by a download cannot add a
//! command without the user marking it. Files under the system data
//! directories are trusted, since only an administrator can write there.
//!
//! Commands run without a shell, with the `Exec` field codes of the Desktop
//! Entry Specification, so file names are never interpreted by one.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::desktop_entry::{self, DesktopEntry};
use crate::mime_apps::{self, XdgDirs};

/// Folders, relative to an XDG data directory, that hold service menus.
const MENU_DIRS: [&str; 3] = [
	"kio/servicemenus",
	"kservices5/ServiceMenus",
	"wingdrive/actions",
];

/// One command from a service menu, with the filters that decide where it shows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceAction {
	/// `<file name>#<action id>`, stable across launches.
	pub id: String,
	pub name: String,
	pub icon: Option<String>,
	/// `X-KDE-Submenu` of the menu file, which groups its actions.
	pub submenu: Option<String>,
	exec: String,
	path: PathBuf,
	mime_types: Vec<String>,
	min_files: usize,
	max_files: Option<usize>,
}

impl ServiceAction {
	/// Whether the action applies to a selection whose items have these MIME lineages.
	///
	/// Every item must match one of the action's types, as in Dolphin.
	pub fn matches(&self, lineages: &[Vec<String>]) -> bool {
		let count = lineages.len();
		if count == 0 || count < self.min_files || self.max_files.is_some_and(|max| count > max) {
			return false;
		}
		lineages.iter().all(|lineage| {
			let is_dir = lineage
				.first()
				.is_some_and(|mime| mime == "inode/directory");
			self.mime_types
				.iter()
				.any(|filter| mime_matches(filter, lineage, is_dir))
		})
	}

	/// Command lines that run the action on `paths`.
	pub fn command_lines(&self, paths: &[PathBuf]) -> Result<Vec<Vec<String>>, String> {
		desktop_entry::command_lines(&self.as_entry(), paths)
	}

	pub(crate) fn as_entry(&self) -> DesktopEntry {
		DesktopEntry {
			id: self.id.clone(),
			path: self.path.clone(),
			name: self.name.clone(),
			exec: self.exec.clone(),
			icon: self.icon.clone(),
			mime_types: self.mime_types.clone(),
			terminal: false,
			no_display: false,
		}
	}
}

fn mime_matches(filter: &str, lineage: &[String], is_dir: bool) -> bool {
	match filter {
		"all/all" => true,
		"all/allfiles" | "application/octet-stream" => !is_dir,
		_ => match filter.strip_suffix("/*") {
			Some(group) => lineage
				.iter()
				.any(|mime| mime.split_once('/').is_some_and(|(top, _)| top == group)),
			None => lineage.iter().any(|mime| mime == filter),
		},
	}
}

/// Every action in the service menus on this system, user menus first.
///
/// A file in the user's data directory hides a system file with the same name.
pub fn load_actions(dirs: &XdgDirs) -> Vec<ServiceAction> {
	let locales = desktop_entry::locale_keys();
	let mut seen = std::collections::HashSet::new();
	let mut actions = Vec::new();

	for (index, data_dir) in std::iter::once(&dirs.data_home)
		.chain(&dirs.data_dirs)
		.enumerate()
	{
		let is_user_dir = index == 0;
		for menu_dir in MENU_DIRS {
			let Ok(entries) = std::fs::read_dir(data_dir.join(menu_dir)) else {
				continue;
			};
			let mut files: Vec<PathBuf> = entries
				.flatten()
				.map(|entry| entry.path())
				.filter(|path| path.extension().is_some_and(|ext| ext == "desktop"))
				.collect();
			files.sort();
			for path in files {
				let Some(name) = path.file_name().map(|n| n.to_string_lossy().into_owned()) else {
					continue;
				};
				if !seen.insert(name.clone()) || (is_user_dir && !is_executable(&path)) {
					continue;
				}
				if let Ok(contents) = std::fs::read_to_string(&path) {
					actions.extend(parse(&name, &path, &contents, &locales));
				}
			}
		}
	}

	actions
}

/// Actions that apply to every one of `paths`, in menu order.
pub fn actions_for_paths(dirs: &XdgDirs, paths: &[PathBuf]) -> Vec<ServiceAction> {
	let actions = load_actions(dirs);
	if actions.is_empty() || paths.is_empty() {
		return Vec::new();
	}
	let lineages: Vec<Vec<String>> = paths
		.iter()
		.map(|path| {
			mime_apps::detect_mime(dirs, path)
				.map(|mime| mime_apps::mime_lineage(dirs, &mime))
				.unwrap_or_else(|| vec!["application/octet-stream".to_string()])
		})
		.collect();
	actions
		.into_iter()
		.filter(|action| action.matches(&lineages))
		.collect()
}

fn is_executable(path: &Path) -> bool {
	use std::os::unix::fs::PermissionsExt;
	path.metadata()
		.is_ok_and(|meta| meta.permissions().mode() & 0o111 != 0)
}

/// Reads the actions of one service menu file.
///
/// Only `Type=Service` files with `Actions` count; a `TryExec` that fails or
/// `Hidden=true` drops the whole file, and an action without `Exec` or a
/// name is skipped.
pub fn parse(
	file_name: &str,
	path: &Path,
	contents: &str,
	locales: &[String],
) -> Vec<ServiceAction> {
	let groups = groups(contents);
	let Some(main) = groups.get("Desktop Entry") else {
		return Vec::new();
	};
	let get = |key: &str| main.get(key).map(String::as_str);
	if get("Type") != Some("Service") || get("Hidden") == Some("true") {
		return Vec::new();
	}
	if get("TryExec").is_some_and(|program| !desktop_entry::program_exists(program)) {
		return Vec::new();
	}

	let list = |key: &str| -> Vec<String> {
		get(key)
			.unwrap_or_default()
			.split(';')
			.map(str::trim)
			.filter(|item| !item.is_empty())
			.map(str::to_string)
			.collect()
	};
	let mut mime_types = list("MimeType");
	mime_types.extend(
		list("X-KDE-ServiceTypes")
			.into_iter()
			.filter(|t| t.contains('/')),
	);
	if mime_types.is_empty() {
		return Vec::new();
	}
	let number = |key: &str| get(key).and_then(|v| v.trim().parse::<usize>().ok());
	let required = number("X-KDE-RequiredNumberOfUrls");
	let min_files = required
		.or_else(|| number("X-KDE-MinNumberOfUrls"))
		.unwrap_or(1)
		.max(1);
	let max_files = required.or_else(|| number("X-KDE-MaxNumberOfUrls"));
	let submenu = localized(main, "X-KDE-Submenu", locales);

	list("Actions")
		.into_iter()
		.filter_map(|action| {
			let group = groups.get(&format!("Desktop Action {action}"))?;
			Some(ServiceAction {
				id: format!("{file_name}#{action}"),
				name: localized(group, "Name", locales)?,
				icon: group.get("Icon").cloned().filter(|icon| !icon.is_empty()),
				submenu: submenu.clone(),
				exec: group.get("Exec").cloned().filter(|exec| !exec.is_empty())?,
				path: path.to_path_buf(),
				mime_types: mime_types.clone(),
				min_files,
				max_files,
			})
		})
		.collect()
}

fn localized(group: &HashMap<String, String>, key: &str, locales: &[String]) -> Option<String> {
	locales
		.iter()
		.find_map(|locale| group.get(&format!("{key}[{locale}]")))
		.or_else(|| group.get(key))
		.filter(|value| !value.is_empty())
		.cloned()
}

/// Key-value pairs of every `[group]`, first occurrence of a key winning.
fn groups(contents: &str) -> HashMap<String, HashMap<String, String>> {
	let mut groups: HashMap<String, HashMap<String, String>> = HashMap::new();
	let mut current: Option<String> = None;
	for line in contents.lines() {
		let line = line.trim();
		if let Some(name) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
			current = Some(name.to_string());
			continue;
		}
		let Some(group) = &current else { continue };
		if line.starts_with('#') {
			continue;
		}
		if let Some((key, value)) = line.split_once('=') {
			groups
				.entry(group.clone())
				.or_default()
				.entry(key.trim().to_string())
				.or_insert_with(|| desktop_entry::unescape_value(value.trim()));
		}
	}
	groups
}

#[cfg(test)]
mod tests {
	use super::*;

	const MENU: &str = "\
[Desktop Entry]
Type=Service
MimeType=image/*;application/pdf;
Actions=resize;share
X-KDE-Submenu=Images
X-KDE-Submenu[pt]=Imagens

[Desktop Action resize]
Name=Resize to 50%
Name[pt]=Reduzir para 50%
Icon=transform-scale
Exec=magick %F -resize 50%% out.png

[Desktop Action share]
Name=Upload
Exec=upload --file %f

[Desktop Action broken]
Name=Never listed
Exec=nothing
";

	fn lineage(mimes: &[&str]) -> Vec<String> {
		mimes.iter().map(|m| m.to_string()).collect()
	}

	#[test]
	fn reads_actions_with_localized_names_and_submenu() {
		let actions = parse(
			"images.desktop",
			Path::new("/menus/images.desktop"),
			MENU,
			&["pt".into()],
		);

		assert_eq!(
			actions
				.iter()
				.map(|a| (a.id.as_str(), a.name.as_str(), a.submenu.as_deref()))
				.collect::<Vec<_>>(),
			[
				("images.desktop#resize", "Reduzir para 50%", Some("Imagens")),
				("images.desktop#share", "Upload", Some("Imagens")),
			]
		);
		assert_eq!(actions[0].icon.as_deref(), Some("transform-scale"));
	}

	#[test]
	fn matches_wildcards_exact_types_and_the_whole_selection() {
		let action = &parse("m.desktop", Path::new("/m.desktop"), MENU, &[])[0];
		let png = lineage(&["image/png", "application/octet-stream"]);
		let pdf = lineage(&["application/pdf", "application/octet-stream"]);
		let text = lineage(&["text/plain", "application/octet-stream"]);

		assert!(action.matches(&[png.clone()]));
		assert!(action.matches(&[png.clone(), pdf]));
		assert!(!action.matches(&[png, text]));
		assert!(!action.matches(&[]));
	}

	#[test]
	fn all_files_excludes_folders_and_all_all_includes_them() {
		let menu = |mime: &str| {
			format!(
				"[Desktop Entry]\nType=Service\nMimeType={mime}\nActions=a\n[Desktop Action a]\nName=A\nExec=a %F\n"
			)
		};
		let folder = lineage(&["inode/directory"]);
		let file = lineage(&["text/plain", "application/octet-stream"]);

		let files_only = &parse("f.desktop", Path::new("/f"), &menu("all/allfiles"), &[])[0];
		assert!(files_only.matches(&[file.clone()]));
		assert!(!files_only.matches(&[folder.clone()]));

		let everything = &parse("e.desktop", Path::new("/e"), &menu("all/all"), &[])[0];
		assert!(everything.matches(&[folder.clone(), file]));

		let folders = &parse("d.desktop", Path::new("/d"), &menu("inode/directory"), &[])[0];
		assert!(folders.matches(&[folder]));
	}

	#[test]
	fn honours_the_number_of_selected_files() {
		let menu = "[Desktop Entry]\nType=Service\nMimeType=all/allfiles\nActions=diff\nX-KDE-RequiredNumberOfUrls=2\n[Desktop Action diff]\nName=Compare\nExec=meld %F\n";
		let action = &parse("diff.desktop", Path::new("/d"), menu, &[])[0];
		let file = lineage(&["text/plain"]);

		assert!(!action.matches(&[file.clone()]));
		assert!(action.matches(&[file.clone(), file.clone()]));
		assert!(!action.matches(&[file.clone(), file.clone(), file]));
	}

	#[test]
	fn skips_files_that_are_not_service_menus() {
		let application = MENU.replace("Type=Service", "Type=Application");
		let hidden = format!("{MENU}\n").replace("Type=Service", "Type=Service\nHidden=true");
		let missing_program =
			MENU.replace("Type=Service", "Type=Service\nTryExec=/nonexistent/tool");

		for contents in [application, hidden, missing_program] {
			assert!(parse("x.desktop", Path::new("/x"), &contents, &[]).is_empty());
		}
	}

	#[test]
	fn builds_commands_without_a_shell() {
		let actions = parse("m.desktop", Path::new("/m.desktop"), MENU, &[]);
		let files = [PathBuf::from("/a b.png"), PathBuf::from("/c.png")];

		assert_eq!(
			actions[0].command_lines(&files).unwrap(),
			[["magick", "/a b.png", "/c.png", "-resize", "50%", "out.png"]]
		);
		assert_eq!(
			actions[1].command_lines(&files).unwrap(),
			[
				["upload", "--file", "/a b.png"],
				["upload", "--file", "/c.png"]
			]
		);
	}

	#[test]
	fn user_menus_must_be_executable_and_hide_system_copies() {
		use std::os::unix::fs::PermissionsExt;

		let root = tempfile::tempdir().unwrap();
		let user = root.path().join("user");
		let system = root.path().join("system");
		let write = |dir: &Path, name: &str, label: &str, mode: u32| {
			let menus = dir.join("kio/servicemenus");
			std::fs::create_dir_all(&menus).unwrap();
			let path = menus.join(name);
			std::fs::write(
				&path,
				format!("[Desktop Entry]\nType=Service\nMimeType=all/all\nActions=a\n[Desktop Action a]\nName={label}\nExec=true\n"),
			)
			.unwrap();
			std::fs::set_permissions(&path, std::fs::Permissions::from_mode(mode)).unwrap();
		};
		write(&user, "shared.desktop", "user copy", 0o755);
		write(&user, "unmarked.desktop", "not executable", 0o644);
		write(&system, "shared.desktop", "system copy", 0o644);
		write(&system, "system.desktop", "system only", 0o644);

		let dirs = XdgDirs {
			config_home: root.path().join("config"),
			config_dirs: vec![],
			data_home: user,
			data_dirs: vec![system],
			desktops: vec![],
		};
		let names: Vec<_> = load_actions(&dirs).into_iter().map(|a| a.name).collect();

		assert_eq!(names, ["user copy", "system only"]);
	}

	#[test]
	fn offers_folder_actions_only_for_folders() {
		use std::os::unix::fs::PermissionsExt;

		let root = tempfile::tempdir().unwrap();
		let menus = root.path().join("data/kio/servicemenus");
		std::fs::create_dir_all(&menus).unwrap();
		let menu = menus.join("folder.desktop");
		std::fs::write(
			&menu,
			"[Desktop Entry]\nType=Service\nMimeType=inode/directory\nActions=count\n[Desktop Action count]\nName=Count files\nExec=count %f\n",
		)
		.unwrap();
		std::fs::set_permissions(&menu, std::fs::Permissions::from_mode(0o755)).unwrap();
		let file = root.path().join("notes.txt");
		std::fs::write(&file, "text").unwrap();

		let dirs = XdgDirs {
			config_home: root.path().join("config"),
			config_dirs: vec![],
			data_home: root.path().join("data"),
			data_dirs: vec![],
			desktops: vec![],
		};
		let names = |paths: &[PathBuf]| -> Vec<String> {
			actions_for_paths(&dirs, paths)
				.into_iter()
				.map(|a| a.name)
				.collect()
		};

		assert_eq!(names(&[root.path().to_path_buf()]), ["Count files"]);
		assert!(names(&[file]).is_empty());
		assert!(names(&[]).is_empty());
	}
}
