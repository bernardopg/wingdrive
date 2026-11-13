//! Editing `portals.conf` in temporary folders; the user's files stay untouched.

use std::path::{Path, PathBuf};

use super::super::portals_conf::*;

fn desktops(value: &str) -> Vec<String> {
	current_desktops(Some(value))
}

#[test]
fn desktops_are_lowercased_and_unsafe_names_skipped() {
	assert_eq!(desktops("Hyprland"), ["hyprland"]);
	assert_eq!(desktops("ubuntu:GNOME"), ["ubuntu", "gnome"]);
	assert_eq!(desktops("../etc::KDE"), ["kde"]);
	assert!(current_desktops(None).is_empty());
}

#[test]
fn the_per_desktop_file_wins_only_when_it_has_preferred_keys() {
	let dir = tempfile::tempdir().unwrap();
	let list = desktops("ubuntu:Hyprland");
	assert_eq!(
		config_file(dir.path(), &list),
		dir.path().join("portals.conf")
	);

	std::fs::write(dir.path().join("hyprland-portals.conf"), "[preferred]\n").unwrap();
	assert_eq!(
		config_file(dir.path(), &list),
		dir.path().join("portals.conf"),
		"the portal ignores a file without keys"
	);

	std::fs::write(
		dir.path().join("hyprland-portals.conf"),
		"[preferred]\ndefault=hyprland;gtk\n",
	)
	.unwrap();
	assert_eq!(
		config_file(dir.path(), &list),
		dir.path().join("hyprland-portals.conf")
	);

	std::fs::write(
		dir.path().join("ubuntu-portals.conf"),
		"[preferred]\ndefault=gnome\n",
	)
	.unwrap();
	assert_eq!(
		config_file(dir.path(), &list),
		dir.path().join("ubuntu-portals.conf")
	);
}

#[test]
fn the_value_is_read_from_the_preferred_section_only() {
	let text = "\
[other]
org.freedesktop.impl.portal.FileChooser=kde
[preferred]
default=hyprland;gtk
# org.freedesktop.impl.portal.FileChooser=commented
org.freedesktop.impl.portal.FileChooser = gtk
";
	assert_eq!(file_chooser_value(text).as_deref(), Some("gtk"));
	assert_eq!(file_chooser_value("[preferred]\ndefault=gtk\n"), None);
	assert_eq!(file_chooser_value(""), None);
}

#[test]
fn setting_the_value_keeps_every_other_line() {
	let text = "\
# managed by me
[preferred]
default=hyprland;gtk
org.freedesktop.impl.portal.FileChooser=gtk
org.freedesktop.impl.portal.Settings=darkman

[other]
key=value
";
	let enabled = set_file_chooser(text, Some("wingdrive;gtk"));
	assert_eq!(
		enabled,
		text.replace("FileChooser=gtk", "FileChooser=wingdrive;gtk")
	);
	let removed = set_file_chooser(text, None);
	assert!(!removed.contains("FileChooser"));
	assert!(removed.contains("# managed by me\n[preferred]\ndefault=hyprland;gtk\norg.freedesktop.impl.portal.Settings=darkman\n"));
}

#[test]
fn a_missing_key_or_section_is_added_where_the_portal_reads_it() {
	let section_last = "[preferred]\ndefault=gtk\n\n";
	assert_eq!(
		set_file_chooser(section_last, Some("wingdrive")),
		"[preferred]\ndefault=gtk\norg.freedesktop.impl.portal.FileChooser=wingdrive\n\n"
	);

	let section_first = "[preferred]\ndefault=gtk\n\n[other]\nx=1\n";
	let written = set_file_chooser(section_first, Some("wingdrive"));
	assert_eq!(
		written,
		"[preferred]\ndefault=gtk\norg.freedesktop.impl.portal.FileChooser=wingdrive\n\n[other]\nx=1\n"
	);
	assert_eq!(file_chooser_value(&written).as_deref(), Some("wingdrive"));

	let no_section = "# just a comment";
	let written = set_file_chooser(no_section, Some("wingdrive"));
	assert_eq!(
		written,
		"# just a comment\n\n[preferred]\norg.freedesktop.impl.portal.FileChooser=wingdrive\n"
	);
	assert_eq!(
		set_file_chooser("", Some("wingdrive")),
		"[preferred]\norg.freedesktop.impl.portal.FileChooser=wingdrive\n"
	);
	assert!(has_preferred_keys(&set_file_chooser("", Some("wingdrive"))));
}

#[test]
fn duplicate_keys_collapse_into_one() {
	let text = "[preferred]\norg.freedesktop.impl.portal.FileChooser=kde\norg.freedesktop.impl.portal.FileChooser=gtk\n";
	assert_eq!(
		file_chooser_value(text).as_deref(),
		Some("gtk"),
		"the last one counts"
	);
	assert_eq!(
		set_file_chooser(text, Some("wingdrive;gtk")),
		"[preferred]\norg.freedesktop.impl.portal.FileChooser=wingdrive;gtk\n"
	);
}

#[test]
fn lists_put_wingdrive_first_and_restore_without_it() {
	assert_eq!(enabled_value(Some("gtk")), "wingdrive;gtk");
	assert_eq!(enabled_value(Some("kde; gtk;")), "wingdrive;kde;gtk");
	assert_eq!(enabled_value(Some("gtk;wingdrive")), "wingdrive;gtk");
	assert_eq!(enabled_value(None), "wingdrive");
	assert!(is_enabled(Some("wingdrive;gtk")));
	assert!(is_enabled(Some(" wingdrive ")));
	assert!(!is_enabled(Some("gtk;wingdrive")));
	assert!(!is_enabled(None));

	assert_eq!(
		restored_value(Some("wingdrive;gtk"), None).as_deref(),
		Some("gtk")
	);
	assert_eq!(restored_value(Some("wingdrive"), None), None);
	let stored = StoredPrevious {
		file: "/x".into(),
		previous: Some("kde".into()),
	};
	assert_eq!(
		restored_value(Some("wingdrive;gtk"), Some(&stored)).as_deref(),
		Some("kde"),
		"the stored value wins"
	);
}

struct Fixture {
	_dir: tempfile::TempDir,
	paths: PortalPaths,
}

impl Fixture {
	fn new(desktop: &str) -> Self {
		let dir = tempfile::tempdir().unwrap();
		let portal_dir = dir.path().join("share/xdg-desktop-portal/portals");
		std::fs::create_dir_all(&portal_dir).unwrap();
		std::fs::write(
			portal_dir.join(PORTAL_FILE),
			include_str!("../../../portal/wingdrive.portal"),
		)
		.unwrap();
		let paths = PortalPaths {
			config_dir: dir.path().join("config/xdg-desktop-portal"),
			desktops: desktops(desktop),
			state_file: dir.path().join("data/file-chooser-portal.json"),
			portal_dirs: vec![dir.path().join("missing"), portal_dir],
		};
		Self { _dir: dir, paths }
	}

	fn write(&self, name: &str, text: &str) -> PathBuf {
		std::fs::create_dir_all(&self.paths.config_dir).unwrap();
		let path = self.paths.config_dir.join(name);
		std::fs::write(&path, text).unwrap();
		path
	}
}

fn read(path: &Path) -> String {
	std::fs::read_to_string(path).unwrap()
}

#[test]
fn the_switch_edits_the_hyprland_file_and_restores_the_previous_value() {
	let fixture = Fixture::new("Hyprland");
	let generic = fixture.write("portals.conf", "[preferred]\ndefault=gtk\n");
	let hyprland = fixture.write(
		"hyprland-portals.conf",
		"[preferred]\ndefault=hyprland;gtk\norg.freedesktop.impl.portal.FileChooser=gtk\n",
	);
	let before = read(&hyprland);

	let status = fixture.paths.status();
	assert!(!status.enabled);
	assert!(status.backend_installed);
	assert_eq!(status.config_file, hyprland);

	let status = fixture.paths.enable().unwrap();
	assert!(status.enabled);
	assert_eq!(status.value.as_deref(), Some("wingdrive;gtk"));
	assert_eq!(
		read(&generic),
		"[preferred]\ndefault=gtk\n",
		"portals.conf is left alone"
	);

	// A second enable keeps the stored value instead of storing the list.
	fixture.paths.enable().unwrap();

	let status = fixture.paths.disable().unwrap();
	assert!(!status.enabled);
	assert_eq!(read(&hyprland), before);
	assert!(!fixture.paths.state_file.exists());
}

#[test]
fn without_a_previous_line_switching_off_removes_the_line() {
	let fixture = Fixture::new("Hyprland");
	let status = fixture.paths.enable().unwrap();
	let file = fixture.paths.config_dir.join("portals.conf");
	assert_eq!(
		status.config_file, file,
		"portals.conf when no per-desktop file exists"
	);
	assert_eq!(
		read(&file),
		"[preferred]\norg.freedesktop.impl.portal.FileChooser=wingdrive\n"
	);
	let status = fixture.paths.disable().unwrap();
	assert!(!status.enabled);
	assert_eq!(status.value, None);
	assert_eq!(read(&file), "[preferred]\n");
}

#[test]
fn switching_off_without_stored_state_drops_wingdrive_from_the_list() {
	let fixture = Fixture::new("KDE");
	let file = fixture.write(
		"portals.conf",
		"[preferred]\norg.freedesktop.impl.portal.FileChooser=wingdrive;kde\n",
	);
	assert!(fixture.paths.status().enabled);
	fixture.paths.disable().unwrap();
	assert_eq!(
		read(&file),
		"[preferred]\norg.freedesktop.impl.portal.FileChooser=kde\n"
	);
}

#[test]
fn switching_off_leaves_a_value_the_user_changed() {
	let fixture = Fixture::new("Hyprland");
	let file = fixture.write(
		"portals.conf",
		"[preferred]\norg.freedesktop.impl.portal.FileChooser=gtk\n",
	);
	fixture.paths.enable().unwrap();
	std::fs::write(
		&file,
		"[preferred]\norg.freedesktop.impl.portal.FileChooser=kde\n",
	)
	.unwrap();
	let status = fixture.paths.disable().unwrap();
	assert_eq!(status.value.as_deref(), Some("kde"));
	assert!(!fixture.paths.state_file.exists());
}

#[test]
fn enabling_is_refused_while_the_backend_is_not_installed() {
	let mut fixture = Fixture::new("Hyprland");
	fixture.paths.portal_dirs.truncate(1);
	assert!(!fixture.paths.status().backend_installed);
	let error = fixture.paths.enable().unwrap_err();
	assert!(error.contains(PORTAL_FILE), "{error}");
	assert!(!fixture.paths.config_dir.join("portals.conf").exists());
}

#[test]
fn a_symlinked_config_keeps_its_link() {
	let fixture = Fixture::new("Hyprland");
	let target = fixture
		.paths
		.state_file
		.parent()
		.unwrap()
		.join("dotfiles/portals.conf");
	std::fs::create_dir_all(target.parent().unwrap()).unwrap();
	std::fs::write(
		&target,
		"[preferred]\norg.freedesktop.impl.portal.FileChooser=gtk\n",
	)
	.unwrap();
	std::fs::create_dir_all(&fixture.paths.config_dir).unwrap();
	let link = fixture.paths.config_dir.join("portals.conf");
	std::os::unix::fs::symlink(&target, &link).unwrap();

	fixture.paths.enable().unwrap();
	assert!(link.symlink_metadata().unwrap().file_type().is_symlink());
	assert!(read(&target).contains("FileChooser=wingdrive;gtk"));
	fixture.paths.disable().unwrap();
	assert!(read(&target).contains("FileChooser=gtk\n"));
}

#[test]
fn the_shipped_portal_file_names_the_backend() {
	let text = include_str!("../../../portal/wingdrive.portal");
	assert!(text.contains("DBusName=org.freedesktop.impl.portal.desktop.wingdrive"));
	assert!(text.contains("Interfaces=org.freedesktop.impl.portal.FileChooser;"));
	// UseIn would select WingDrive on a desktop with no portals.conf, without opting in.
	assert!(!text.contains("UseIn"));
	let service =
		include_str!("../../../portal/org.freedesktop.impl.portal.desktop.wingdrive.service");
	assert!(service.contains("Name=org.freedesktop.impl.portal.desktop.wingdrive"));
	assert!(!service.contains("FileManager1"));
}

fn vars(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<std::ffi::OsString> {
	let map: std::collections::HashMap<String, String> = pairs
		.iter()
		.map(|(k, v)| (k.to_string(), v.to_string()))
		.collect();
	move |name| map.get(name).map(std::ffi::OsString::from)
}

#[test]
fn paths_follow_xdg_variables_and_ignore_relative_ones() {
	let state = PathBuf::from("/state.json");
	let paths = PortalPaths::from_vars(
		state.clone(),
		vars(&[
			("HOME", "/home/me"),
			("XDG_CONFIG_HOME", "/cfg"),
			("XDG_DATA_HOME", "relative/data"),
			("XDG_DATA_DIRS", "/a:/b"),
			("XDG_CURRENT_DESKTOP", "Hyprland:KDE"),
		]),
	);
	assert_eq!(paths.config_dir, PathBuf::from("/cfg/xdg-desktop-portal"));
	assert_eq!(paths.state_file, state);
	assert_eq!(paths.desktops, ["hyprland", "kde"]);
	// A relative XDG_DATA_HOME is invalid, so the home default applies.
	assert_eq!(
		paths.portal_dirs,
		[
			PathBuf::from("/home/me/.local/share/xdg-desktop-portal/portals"),
			PathBuf::from("/a/xdg-desktop-portal/portals"),
			PathBuf::from("/b/xdg-desktop-portal/portals"),
		]
	);
}

#[test]
fn defaults_apply_without_xdg_variables_and_the_portal_dir_override_wins() {
	let defaults = PortalPaths::from_vars(PathBuf::new(), vars(&[("HOME", "/h")]));
	assert_eq!(
		defaults.config_dir,
		PathBuf::from("/h/.config/xdg-desktop-portal")
	);
	assert!(defaults.desktops.is_empty());
	assert_eq!(
		defaults.portal_dirs.last(),
		Some(&PathBuf::from("/usr/share/xdg-desktop-portal/portals"))
	);
	let overridden = PortalPaths::from_vars(
		PathBuf::new(),
		vars(&[
			("HOME", "/h"),
			("XDG_DESKTOP_PORTAL_DIR", "/portals"),
			("XDG_DATA_DIRS", ""),
		]),
	);
	assert_eq!(overridden.portal_dirs, [PathBuf::from("/portals")]);
}

#[test]
fn status_reports_an_unwritten_config_and_a_missing_backend() {
	let dir = tempfile::tempdir().unwrap();
	let paths = PortalPaths {
		config_dir: dir.path().join("config"),
		desktops: vec![],
		state_file: dir.path().join("state.json"),
		portal_dirs: vec![dir.path().join("none")],
	};
	let status = paths.status();
	assert!(!status.enabled);
	assert!(!status.backend_installed);
	assert_eq!(status.value, None);
	assert_eq!(status.config_file, dir.path().join("config/portals.conf"));
	// Switching off with nothing to restore and no state file still succeeds.
	assert!(!paths.disable().unwrap().enabled);
}

#[test]
fn an_unreadable_config_is_an_error_not_an_empty_file() {
	let fixture = Fixture::new("");
	// A folder where the config file should be cannot be read as text.
	std::fs::create_dir_all(fixture.paths.config_dir.join("portals.conf")).unwrap();
	let error = fixture.paths.enable().unwrap_err();
	assert!(error.contains("portals.conf"), "{error}");
}

#[test]
fn a_corrupt_state_file_is_ignored() {
	let fixture = Fixture::new("");
	let config = fixture.write("portals.conf", "[preferred]\ndefault=gtk\n");
	fixture.paths.enable().unwrap();
	std::fs::write(&fixture.paths.state_file, "not json").unwrap();
	// Without a readable state, switching off drops wingdrive and keeps the rest.
	let status = fixture.paths.disable().unwrap();
	assert!(!status.enabled);
	assert!(!read(&config).contains("wingdrive"));
	assert!(!fixture.paths.state_file.exists());
}
