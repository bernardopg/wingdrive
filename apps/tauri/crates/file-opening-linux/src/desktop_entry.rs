//! # Desktop entries
//!
//! Linux applications describe themselves with XDG desktop entries. This
//! module reads the fields "Open With" needs and builds the command line from
//! the `Exec` key, following the Desktop Entry Specification's quoting and
//! field codes. Commands run without a shell, so file names are never
//! interpreted by one.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// The parts of a desktop entry that matter for opening files.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopEntry {
	/// Desktop file ID, such as `org.gnome.Evince.desktop`.
	pub id: String,
	pub path: PathBuf,
	pub name: String,
	pub exec: String,
	pub icon: Option<String>,
	pub mime_types: Vec<String>,
	pub terminal: bool,
	/// Hidden from menus; still a valid handler when it is a user's default.
	pub no_display: bool,
}

/// Parses `[Desktop Entry]`, returning None for entries that cannot launch.
///
/// `Hidden=true` deletes the entry, a failing `TryExec` means the program is
/// not installed, and only `Type=Application` entries with `Exec` launch.
pub fn parse(id: &str, path: &Path, contents: &str, locales: &[String]) -> Option<DesktopEntry> {
	let fields = main_group(contents);
	let get = |key: &str| fields.get(key).map(String::as_str);
	if get("Type") != Some("Application") || get("Hidden") == Some("true") {
		return None;
	}
	if let Some(try_exec) = get("TryExec") {
		if !program_exists(try_exec) {
			return None;
		}
	}
	let name = locales
		.iter()
		.find_map(|locale| get(&format!("Name[{locale}]")))
		.or_else(|| get("Name"))?;
	Some(DesktopEntry {
		id: id.to_string(),
		path: path.to_path_buf(),
		name: name.to_string(),
		exec: get("Exec")?.to_string(),
		icon: get("Icon").map(str::to_string),
		mime_types: split_list(get("MimeType").unwrap_or_default()),
		terminal: get("Terminal") == Some("true"),
		no_display: get("NoDisplay") == Some("true"),
	})
}

/// Locale keys to try for localized values, most specific first: `pt_BR`, `pt`.
pub fn locale_keys() -> Vec<String> {
	let raw = ["LC_ALL", "LC_MESSAGES", "LANG"]
		.iter()
		.find_map(|var| std::env::var(var).ok().filter(|v| !v.is_empty()))
		.unwrap_or_default();
	let base = raw.split(['.', '@']).next().unwrap_or_default();
	let mut keys = Vec::new();
	if !base.is_empty() && base != "C" && base != "POSIX" {
		keys.push(base.to_string());
		if let Some((lang, _)) = base.split_once('_') {
			keys.push(lang.to_string());
		}
	}
	keys
}

fn main_group(contents: &str) -> HashMap<String, String> {
	let mut fields = HashMap::new();
	let mut in_main = false;
	for line in contents.lines() {
		let line = line.trim();
		if line.starts_with('[') {
			in_main = line == "[Desktop Entry]";
			continue;
		}
		if !in_main || line.starts_with('#') {
			continue;
		}
		if let Some((key, value)) = line.split_once('=') {
			fields
				.entry(key.trim().to_string())
				.or_insert_with(|| unescape_value(value.trim()));
		}
	}
	fields
}

/// Applies the string escapes defined for desktop entry values.
fn unescape_value(value: &str) -> String {
	let mut out = String::with_capacity(value.len());
	let mut chars = value.chars();
	while let Some(c) = chars.next() {
		if c != '\\' {
			out.push(c);
			continue;
		}
		match chars.next() {
			Some('s') => out.push(' '),
			Some('n') => out.push('\n'),
			Some('t') => out.push('\t'),
			Some('r') => out.push('\r'),
			Some('\\') => out.push('\\'),
			Some(other) => {
				out.push('\\');
				out.push(other);
			}
			None => out.push('\\'),
		}
	}
	out
}

fn split_list(value: &str) -> Vec<String> {
	value
		.split(';')
		.map(str::trim)
		.filter(|item| !item.is_empty())
		.map(str::to_string)
		.collect()
}

/// Whether `program` is an executable path or found on `PATH`.
pub fn program_exists(program: &str) -> bool {
	let is_executable = |path: &Path| {
		use std::os::unix::fs::PermissionsExt;
		path.metadata()
			.is_ok_and(|meta| meta.is_file() && meta.permissions().mode() & 0o111 != 0)
	};
	if program.contains('/') {
		return is_executable(Path::new(program));
	}
	std::env::var_os("PATH").is_some_and(|paths| {
		std::env::split_paths(&paths).any(|dir| is_executable(&dir.join(program)))
	})
}

/// Splits an `Exec` value into arguments using the spec's double-quote rules.
fn tokenize(exec: &str) -> Option<Vec<String>> {
	let mut args = Vec::new();
	let mut current = String::new();
	let mut in_quotes = false;
	let mut has_token = false;
	let mut chars = exec.chars();
	while let Some(c) = chars.next() {
		match c {
			'"' => {
				in_quotes = !in_quotes;
				has_token = true;
			}
			'\\' if in_quotes => current.push(chars.next()?),
			' ' | '\t' if !in_quotes => {
				if has_token {
					args.push(std::mem::take(&mut current));
					has_token = false;
				}
			}
			_ => {
				current.push(c);
				has_token = true;
			}
		}
	}
	if in_quotes {
		return None;
	}
	if has_token {
		args.push(current);
	}
	Some(args)
}

/// Builds the command lines that open `paths` with this entry.
///
/// Entries with `%f` or `%u` take one file per invocation, so several files
/// produce several commands; `%F` and `%U` take them all at once. An entry
/// with no file code gets the files appended, which is what launchers do.
pub fn command_lines(entry: &DesktopEntry, paths: &[PathBuf]) -> Result<Vec<Vec<String>>, String> {
	let args = tokenize(&entry.exec).ok_or_else(|| format!("Malformed Exec in {}", entry.id))?;
	let single = args
		.iter()
		.any(|arg| arg.contains("%f") || arg.contains("%u"));
	let multiple = args.iter().any(|arg| arg == "%F" || arg == "%U");
	let groups: Vec<&[PathBuf]> = if single && !multiple {
		paths.chunks(1).collect()
	} else {
		vec![paths]
	};
	Ok(groups
		.into_iter()
		.map(|files| expand(entry, &args, files, single || multiple))
		.collect())
}

fn expand(
	entry: &DesktopEntry,
	args: &[String],
	files: &[PathBuf],
	has_file_code: bool,
) -> Vec<String> {
	let uri = |path: &PathBuf| file_uri(path);
	let path_string = |path: &PathBuf| path.to_string_lossy().into_owned();
	let mut out = Vec::new();
	for arg in args {
		match arg.as_str() {
			"%F" => out.extend(files.iter().map(path_string)),
			"%U" => out.extend(files.iter().map(uri)),
			"%i" => {
				if let Some(icon) = &entry.icon {
					out.push("--icon".to_string());
					out.push(icon.clone());
				}
			}
			_ => {
				let mut expanded = String::new();
				let mut chars = arg.chars();
				while let Some(c) = chars.next() {
					if c != '%' {
						expanded.push(c);
						continue;
					}
					match chars.next() {
						Some('%') => expanded.push('%'),
						Some('f') => {
							expanded.push_str(&files.first().map(path_string).unwrap_or_default())
						}
						Some('u') => expanded.push_str(&files.first().map(uri).unwrap_or_default()),
						Some('c') => expanded.push_str(&entry.name),
						Some('k') => expanded.push_str(&entry.path.to_string_lossy()),
						// Deprecated or unknown codes expand to nothing.
						_ => {}
					}
				}
				if !expanded.is_empty() || !arg.starts_with('%') {
					out.push(expanded);
				}
			}
		}
	}
	if !has_file_code {
		out.extend(files.iter().map(path_string));
	}
	out
}

/// Percent-encodes a path as a `file://` URI.
pub fn file_uri(path: &Path) -> String {
	use std::os::unix::ffi::OsStrExt;
	let mut uri = String::from("file://");
	for &byte in path.as_os_str().as_bytes() {
		if byte.is_ascii_alphanumeric() || b"/-_.~".contains(&byte) {
			uri.push(byte as char);
		} else {
			uri.push_str(&format!("%{byte:02X}"));
		}
	}
	uri
}

#[cfg(test)]
mod tests {
	use super::*;

	fn entry(exec: &str) -> DesktopEntry {
		DesktopEntry {
			id: "viewer.desktop".into(),
			path: "/usr/share/applications/viewer.desktop".into(),
			name: "Viewer".into(),
			exec: exec.into(),
			icon: Some("viewer".into()),
			mime_types: vec![],
			terminal: false,
			no_display: false,
		}
	}

	#[test]
	fn parses_localized_name_and_skips_unlaunchable_entries() {
		let text = "[Desktop Entry]\nType=Application\nName=Viewer\nName[pt_BR]=Visualizador\nExec=viewer %U\nMimeType=image/png;image/jpeg;\n[Desktop Action new]\nName=Other\n";
		let parsed = parse(
			"v.desktop",
			Path::new("/v.desktop"),
			text,
			&["pt_BR".into(), "pt".into()],
		)
		.unwrap();
		assert_eq!(parsed.name, "Visualizador");
		assert_eq!(parsed.mime_types, vec!["image/png", "image/jpeg"]);

		let hidden = "[Desktop Entry]\nType=Application\nName=X\nExec=x\nHidden=true\n";
		assert!(parse("x.desktop", Path::new("/x"), hidden, &[]).is_none());
		let missing = "[Desktop Entry]\nType=Application\nName=X\nExec=x\nTryExec=/nonexistent/x\n";
		assert!(parse("x.desktop", Path::new("/x"), missing, &[]).is_none());
		let link = "[Desktop Entry]\nType=Link\nName=X\nURL=https://x\n";
		assert!(parse("x.desktop", Path::new("/x"), link, &[]).is_none());
	}

	#[test]
	fn single_file_codes_run_once_per_file() {
		let files = vec![PathBuf::from("/a b.txt"), PathBuf::from("/c.txt")];
		let lines = command_lines(&entry("viewer --open %f"), &files).unwrap();
		assert_eq!(
			lines,
			vec![
				vec!["viewer", "--open", "/a b.txt"],
				vec!["viewer", "--open", "/c.txt"]
			]
		);
	}

	#[test]
	fn list_codes_quotes_and_icons_expand() {
		let files = vec![PathBuf::from("/a b.txt"), PathBuf::from("/c.txt")];
		let lines = command_lines(&entry(r#""/opt/My App/run" %i --title=%c %U"#), &files).unwrap();
		assert_eq!(
			lines,
			vec![vec![
				"/opt/My App/run",
				"--icon",
				"viewer",
				"--title=Viewer",
				"file:///a%20b.txt",
				"file:///c.txt"
			]]
		);
	}

	#[test]
	fn entries_without_file_codes_get_files_appended() {
		let files = vec![PathBuf::from("/c.txt")];
		let lines = command_lines(&entry("viewer"), &files).unwrap();
		assert_eq!(lines, vec![vec!["viewer", "/c.txt"]]);
		assert!(command_lines(&entry("viewer \"unterminated"), &files).is_err());
	}
}
