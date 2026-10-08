//! # Application icons
//!
//! Resolves a desktop entry's `Icon=` value to an image file the way the
//! XDG Icon Theme Specification describes: an absolute path is used as is,
//! a name is looked up in the current icon theme, the themes it inherits,
//! then `hicolor`, and finally the legacy `pixmaps` directories. Only menu
//! sizes (16 to 48 pixels) and scalable icons are considered; PNG is
//! preferred and SVG accepted. Results are returned as `data:` URLs and
//! cached for the life of the process, since themes rarely change while
//! the app runs.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Mutex, OnceLock};

use base64::Engine;

use crate::host_env::use_host_environment;
use crate::mime_apps::XdgDirs;

/// Icons larger than this are skipped so a stray huge file cannot bloat menus.
const MAX_ICON_BYTES: u64 = 512 * 1024;
const MIN_SIZE: u32 = 16;
const MAX_SIZE: u32 = 48;
/// The size menus look best at on HiDPI screens; closer sizes win.
const PREFERRED_SIZE: u32 = 32;

/// Icon directories of the resolved theme chain, in lookup order.
#[derive(Debug)]
pub struct IconLookup {
	/// Per theme in the chain, the directories holding menu-sized icons,
	/// best size first.
	themes: Vec<Vec<PathBuf>>,
	/// Legacy directories searched last, such as `/usr/share/pixmaps`.
	pixmaps: Vec<PathBuf>,
}

impl IconLookup {
	/// Builds the lookup for `theme` over the given icon base directories.
	pub fn new(bases: &[PathBuf], theme: Option<&str>, pixmaps: Vec<PathBuf>) -> Self {
		let mut chain = Vec::new();
		let mut seen = HashSet::new();
		let mut queue: Vec<String> = theme.into_iter().map(str::to_string).collect();
		while let Some(name) = queue.pop() {
			if name == "hicolor" || !seen.insert(name.clone()) {
				continue;
			}
			let index = bases.iter().find_map(|base| {
				std::fs::read_to_string(base.join(&name).join("index.theme")).ok()
			});
			if let Some(index) = &index {
				// Inherited themes are searched in order, after this one.
				let parents = theme_value(index, "Icon Theme", "Inherits").unwrap_or_default();
				queue.extend(parents.split(',').map(str::trim).rev().map(str::to_string));
			}
			chain.push(name);
		}
		chain.push("hicolor".to_string());

		let themes = chain
			.iter()
			.map(|theme| {
				bases
					.iter()
					.filter_map(|base| {
						let root = base.join(theme);
						let index = std::fs::read_to_string(root.join("index.theme")).ok()?;
						Some(menu_dirs(&index).into_iter().map(move |dir| root.join(dir)))
					})
					.flatten()
					.collect()
			})
			.collect();
		Self { themes, pixmaps }
	}

	/// Builds the lookup for the user's current theme.
	pub fn from_env() -> Self {
		let xdg = XdgDirs::from_env();
		let home = std::env::var_os("HOME").map(PathBuf::from);
		let bases: Vec<PathBuf> = home
			.map(|home| home.join(".icons"))
			.into_iter()
			.chain(std::iter::once(xdg.data_home.join("icons")))
			.chain(xdg.data_dirs.iter().map(|dir| dir.join("icons")))
			.collect();
		let pixmaps = xdg
			.data_dirs
			.iter()
			.map(|dir| dir.join("pixmaps"))
			.collect();
		Self::new(&bases, current_theme(&xdg).as_deref(), pixmaps)
	}

	/// Finds the file for an `Icon=` value.
	pub fn find(&self, icon: &str) -> Option<PathBuf> {
		let icon = icon.trim();
		if icon.is_empty() {
			return None;
		}
		let path = Path::new(icon);
		if path.is_absolute() {
			return (is_supported(path) && path.is_file()).then(|| path.to_path_buf());
		}
		// Some entries name the file, extension included.
		let name = icon
			.strip_suffix(".png")
			.or_else(|| icon.strip_suffix(".svg"))
			.unwrap_or(icon);
		if name.contains('/') {
			return None;
		}
		let png = format!("{name}.png");
		let svg = format!("{name}.svg");
		// A theme earlier in the chain wins; within a theme, PNG beats SVG.
		let in_dirs = |dirs: &[PathBuf]| {
			dirs.iter()
				.map(|dir| dir.join(&png))
				.chain(dirs.iter().map(|dir| dir.join(&svg)))
				.find(|candidate| candidate.is_file())
		};
		self.themes
			.iter()
			.find_map(|dirs| in_dirs(dirs))
			.or_else(|| in_dirs(&self.pixmaps))
	}
}

/// The icon for an `Icon=` value as a `data:` URL, cached per process.
pub fn icon_data_url(icon: &str) -> Option<String> {
	static LOOKUP: OnceLock<IconLookup> = OnceLock::new();
	static CACHE: OnceLock<Mutex<HashMap<String, Option<String>>>> = OnceLock::new();
	let cache = CACHE.get_or_init(Default::default);
	if let Some(hit) = cache.lock().ok()?.get(icon) {
		return hit.clone();
	}
	let url = LOOKUP
		.get_or_init(IconLookup::from_env)
		.find(icon)
		.and_then(|path| data_url(&path));
	cache.lock().ok()?.insert(icon.to_string(), url.clone());
	url
}

/// Encodes a PNG or SVG file as a `data:` URL.
pub fn data_url(path: &Path) -> Option<String> {
	let mime = match path.extension()?.to_str()? {
		"png" => "image/png",
		"svg" => "image/svg+xml",
		_ => return None,
	};
	if path.metadata().ok()?.len() > MAX_ICON_BYTES {
		return None;
	}
	let bytes = std::fs::read(path).ok()?;
	let encoded = base64::engine::general_purpose::STANDARD.encode(bytes);
	Some(format!("data:{mime};base64,{encoded}"))
}

fn is_supported(path: &Path) -> bool {
	path.extension()
		.is_some_and(|ext| ext == "png" || ext == "svg")
}

/// The icon theme name: GNOME's setting, then GTK's settings files, then
/// the theme part of `GTK_THEME`.
fn current_theme(xdg: &XdgDirs) -> Option<String> {
	let from_gsettings = || {
		let output = use_host_environment(&mut Command::new("gsettings"))
			.args(["get", "org.gnome.desktop.interface", "icon-theme"])
			.output()
			.ok()?;
		let value = String::from_utf8(output.stdout).ok()?;
		let value = value.trim().trim_matches('\'').to_string();
		(output.status.success() && !value.is_empty()).then_some(value)
	};
	let from_gtk_settings = || {
		["gtk-4.0", "gtk-3.0"].iter().find_map(|dir| {
			let text =
				std::fs::read_to_string(xdg.config_home.join(dir).join("settings.ini")).ok()?;
			theme_value(&text, "Settings", "gtk-icon-theme-name")
		})
	};
	let from_gtk_theme = || {
		let value = std::env::var("GTK_THEME").ok()?;
		let name = value.split(':').next()?.trim().to_string();
		(!name.is_empty()).then_some(name)
	};
	from_gsettings()
		.or_else(from_gtk_settings)
		.or_else(from_gtk_theme)
}

/// Reads `key` from `[group]` of an INI-style file.
fn theme_value(text: &str, group: &str, key: &str) -> Option<String> {
	let value = *parse_groups(text).get(group)?.get(key)?;
	Some(value.trim_matches('"').to_string())
}

/// Groups of an INI-style file: group name to its keys.
fn parse_groups(text: &str) -> HashMap<&str, HashMap<&str, &str>> {
	let mut groups: HashMap<&str, HashMap<&str, &str>> = HashMap::new();
	let mut current = "";
	for line in text.lines().map(str::trim) {
		if let Some(name) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
			current = name;
		} else if let Some((key, value)) = line.split_once('=') {
			groups
				.entry(current)
				.or_default()
				.entry(key.trim())
				.or_insert(value.trim());
		}
	}
	groups
}

/// The theme's subdirectories that hold menu-sized or scalable application
/// icons, ordered so the size closest to [`PREFERRED_SIZE`] comes first.
fn menu_dirs(index: &str) -> Vec<String> {
	let groups = parse_groups(index);
	let Some(theme) = groups.get("Icon Theme") else {
		return Vec::new();
	};
	let list = |key: &str| theme.get(key).copied().unwrap_or_default().split(',');
	let mut seen = HashSet::new();
	let mut dirs: Vec<(u32, String)> = list("Directories")
		.chain(list("ScaledDirectories"))
		.map(str::trim)
		.filter(|dir| !dir.is_empty() && seen.insert(*dir))
		.filter_map(|dir| {
			let keys = groups.get(dir)?;
			// Themes ship thousands of directories; app icons live in one context.
			if keys.get("Context").is_some_and(|c| *c != "Applications") {
				return None;
			}
			let size: u32 = keys.get("Size")?.parse().ok()?;
			let scale: u32 = keys.get("Scale").and_then(|s| s.parse().ok()).unwrap_or(1);
			let pixels = size * scale;
			// Scalable icons render at any size, so they rank like a perfect fit.
			let rank = if keys.get("Type") == Some(&"Scalable") {
				0
			} else if (MIN_SIZE..=MAX_SIZE).contains(&pixels) {
				pixels.abs_diff(PREFERRED_SIZE)
			} else {
				return None;
			};
			Some((rank, dir.to_string()))
		})
		.collect();
	dirs.sort_by_key(|(rank, _)| *rank);
	dirs.into_iter().map(|(_, dir)| dir).collect()
}

#[cfg(test)]
mod tests {
	use super::*;

	fn write(path: &Path, contents: &[u8]) {
		std::fs::create_dir_all(path.parent().unwrap()).unwrap();
		std::fs::write(path, contents).unwrap();
	}

	const HICOLOR: &str = "[Icon Theme]\nName=Hicolor\nDirectories=16x16/apps,32x32/apps,256x256/apps,scalable/apps\n\n[16x16/apps]\nSize=16\nType=Fixed\n\n[32x32/apps]\nSize=32\nType=Fixed\n\n[256x256/apps]\nSize=256\nType=Fixed\n\n[scalable/apps]\nSize=128\nType=Scalable\n";

	#[test]
	fn finds_theme_inherited_hicolor_pixmap_and_absolute_icons() {
		let tmp = tempfile::tempdir().unwrap();
		let base = tmp.path().join("icons");
		let pixmaps = tmp.path().join("pixmaps");
		write(&base.join("hicolor/index.theme"), HICOLOR.as_bytes());
		write(&base.join("hicolor/16x16/apps/viewer.png"), b"small");
		write(&base.join("hicolor/32x32/apps/viewer.png"), b"medium");
		write(&base.join("hicolor/256x256/apps/huge.png"), b"too big");
		write(&base.join("hicolor/scalable/apps/editor.svg"), b"<svg/>");
		write(
			&base.join("Parent/index.theme"),
			b"[Icon Theme]\nDirectories=apps/24\n\n[apps/24]\nSize=24\n",
		);
		write(
			&base.join("Parent/apps/24/editor.svg"),
			b"<svg>parent</svg>",
		);
		write(
			&base.join("Custom/index.theme"),
			b"[Icon Theme]\nInherits=Parent,hicolor\nDirectories=48x48/apps\n\n[48x48/apps]\nSize=48\n",
		);
		write(&base.join("Custom/48x48/apps/viewer.png"), b"themed");
		write(&pixmaps.join("legacy.png"), b"legacy");

		let lookup = IconLookup::new(
			std::slice::from_ref(&base),
			Some("Custom"),
			vec![pixmaps.clone()],
		);

		// The current theme wins over hicolor even at a less preferred size.
		assert_eq!(
			lookup.find("viewer"),
			Some(base.join("Custom/48x48/apps/viewer.png"))
		);
		// An inherited theme comes before hicolor.
		assert_eq!(
			lookup.find("editor"),
			Some(base.join("Parent/apps/24/editor.svg"))
		);
		// Sizes outside 16-48 are skipped; pixmaps are the last resort.
		assert_eq!(lookup.find("huge"), None);
		assert_eq!(lookup.find("legacy"), Some(pixmaps.join("legacy.png")));
		assert_eq!(lookup.find("legacy.png"), Some(pixmaps.join("legacy.png")));

		// Absolute paths are used directly when they are PNG or SVG.
		let absolute = base.join("hicolor/16x16/apps/viewer.png");
		assert_eq!(lookup.find(absolute.to_str().unwrap()), Some(absolute));
		assert_eq!(lookup.find("/nonexistent/icon.png"), None);
		assert_eq!(lookup.find("../escape"), None);

		// Without a theme, hicolor picks the size closest to 32 pixels.
		let plain = IconLookup::new(std::slice::from_ref(&base), None, vec![]);
		assert_eq!(
			plain.find("viewer"),
			Some(base.join("hicolor/32x32/apps/viewer.png"))
		);
	}

	#[test]
	fn encodes_png_and_svg_as_data_urls() {
		let tmp = tempfile::tempdir().unwrap();
		let png = tmp.path().join("a.png");
		let svg = tmp.path().join("a.svg");
		let xpm = tmp.path().join("a.xpm");
		write(&png, b"png");
		write(&svg, b"<svg/>");
		write(&xpm, b"xpm");
		assert_eq!(data_url(&png).unwrap(), "data:image/png;base64,cG5n");
		assert_eq!(
			data_url(&svg).unwrap(),
			"data:image/svg+xml;base64,PHN2Zy8+"
		);
		assert_eq!(data_url(&xpm), None);
	}
}
