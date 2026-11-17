//! # Background mode
//!
//! A file manager has to open instantly, and a cold WingDrive start pays for
//! the daemon and a WebKit process. With background mode on, closing the main
//! window only hides it and a tray icon keeps the app reachable, so later
//! launches reuse the running instance. "Start at login" launches WingDrive
//! hidden, through an XDG autostart entry on Linux and the current user's Run
//! registry key on Windows.
//!
//! ## Example
//! ```rust,ignore
//! let settings = background::DesktopSettingsState::load(&data_dir);
//! app.manage(settings);
//! background::create_tray(app.handle())?;
//! ```

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use tauri::menu::{Menu, MenuBuilder, MenuItem, MenuItemBuilder, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, Runtime};

use crate::launch;

const SETTINGS_FILE: &str = "desktop_settings.json";
#[cfg(target_os = "linux")]
const AUTOSTART_FILE: &str = "wingdrive.desktop";
const TRAY_LABELS_FILE: &str = "tray_labels.json";

/// Desktop shell preferences, stored next to the app's other per-device state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct DesktopSettings {
	/// Closing the main window hides it to the tray instead of quitting.
	pub keep_in_background: bool,
	/// Start WingDrive when the user logs in.
	pub start_at_login: bool,
	/// A login start keeps WingDrive in the tray instead of showing its window.
	pub start_minimized: bool,
	/// Look for a new signed release after startup (AppImage and Windows only).
	pub check_for_updates: bool,
	/// Terminal command for "Open Terminal Here"; empty means detect.
	pub terminal_command: Option<String>,
}

impl Default for DesktopSettings {
	fn default() -> Self {
		Self {
			keep_in_background: true,
			start_at_login: false,
			start_minimized: true,
			check_for_updates: true,
			terminal_command: None,
		}
	}
}

/// Loaded settings plus the file they persist to.
pub struct DesktopSettingsState {
	settings: Mutex<DesktopSettings>,
	path: PathBuf,
}

impl DesktopSettingsState {
	/// Reads settings from `data_dir`; a missing or unreadable file means defaults.
	pub fn load(data_dir: &Path) -> Self {
		let path = data_dir.join(SETTINGS_FILE);
		let settings = std::fs::read(&path)
			.ok()
			.and_then(|bytes| serde_json::from_slice(&bytes).ok())
			.unwrap_or_default();
		Self {
			settings: Mutex::new(settings),
			path,
		}
	}

	pub fn get(&self) -> DesktopSettings {
		self.settings.lock().unwrap().clone()
	}

	fn save(&self, settings: DesktopSettings) -> Result<(), String> {
		if let Some(parent) = self.path.parent() {
			std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
		}
		let json = serde_json::to_vec_pretty(&settings).map_err(|e| e.to_string())?;
		std::fs::write(&self.path, json).map_err(|e| e.to_string())?;
		*self.settings.lock().unwrap() = settings;
		Ok(())
	}
}

/// Tray menu items whose text follows the UI language.
struct TrayMenu<R: Runtime = tauri::Wry> {
	open: MenuItem<R>,
	new_tab: MenuItem<R>,
	logs: MenuItem<R>,
	quit: MenuItem<R>,
}

/// Tray menu labels in the current UI language, sent by the webview.
///
/// The last labels received are saved so the next launch builds the tray in
/// the user's language before the webview has loaded the daemon config.
///
/// Fields missing from an older saved file or webview keep their English default.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TrayLabels {
	open: String,
	new_tab: String,
	logs: String,
	quit: String,
}

impl Default for TrayLabels {
	fn default() -> Self {
		Self {
			open: "Open WingDrive".into(),
			new_tab: "New Tab".into(),
			logs: "Open Logs Folder".into(),
			quit: "Quit WingDrive".into(),
		}
	}
}

/// Where the tray labels persist: next to the desktop settings.
fn tray_labels_path<R: Runtime>(app: &AppHandle<R>) -> Option<PathBuf> {
	let state = app.try_state::<DesktopSettingsState>()?;
	Some(state.path.parent()?.join(TRAY_LABELS_FILE))
}

fn load_tray_labels<R: Runtime>(app: &AppHandle<R>) -> TrayLabels {
	tray_labels_path(app)
		.and_then(|path| std::fs::read(path).ok())
		.and_then(|bytes| serde_json::from_slice(&bytes).ok())
		.unwrap_or_default()
}

/// Whether closing the main window should hide it instead of quitting.
pub fn keeps_running<R: Runtime>(app: &AppHandle<R>) -> bool {
	app.try_state::<DesktopSettingsState>()
		.is_some_and(|state| state.get().keep_in_background)
}

/// Adds the tray icon with Open, New Tab, Open Logs Folder and Quit.
///
/// Linux trays built on AppIndicator deliver no click events, so every action
/// lives in the menu rather than on the icon itself.
pub fn create_tray<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
	let (menu, items) = build_tray_menu(app, load_tray_labels(app))?;
	let mut tray = TrayIconBuilder::with_id("main")
		.tooltip("WingDrive")
		.menu(&menu)
		.show_menu_on_left_click(false)
		.on_menu_event(|app, event| on_tray_menu(app, event.id().as_ref()))
		.on_tray_icon_event(|tray, event| on_tray_icon(tray.app_handle(), &event));
	if let Some(icon) = app.default_window_icon() {
		tray = tray.icon(icon.clone());
	}
	tray.build(app)?;
	app.manage(items);
	Ok(())
}

/// The tray menu, plus the items whose text follows the UI language.
fn build_tray_menu<R: Runtime>(
	app: &AppHandle<R>,
	labels: TrayLabels,
) -> tauri::Result<(Menu<R>, TrayMenu<R>)> {
	let open = MenuItemBuilder::with_id("tray-open", labels.open).build(app)?;
	let new_tab = MenuItemBuilder::with_id("tray-new-tab", labels.new_tab).build(app)?;
	let logs = MenuItemBuilder::with_id("tray-logs", labels.logs).build(app)?;
	let quit = MenuItemBuilder::with_id("tray-quit", labels.quit).build(app)?;
	let menu = MenuBuilder::new(app)
		.item(&open)
		.item(&new_tab)
		.item(&PredefinedMenuItem::separator(app)?)
		.item(&logs)
		.item(&quit)
		.build()?;
	Ok((
		menu,
		TrayMenu {
			open,
			new_tab,
			logs,
			quit,
		},
	))
}

/// Runs the tray menu item with `id`.
fn on_tray_menu<R: Runtime>(app: &AppHandle<R>, id: &str) {
	match id {
		"tray-open" => launch::show_main_window(app),
		"tray-new-tab" => {
			// $HOME is unset on Windows, where this opened the root of the current drive.
			let home = app.path().home_dir().unwrap_or_else(|_| "/".into());
			launch::deliver(
				app,
				vec![launch::OpenRequest {
					directory: home,
					select: None,
				}],
			);
		}
		"tray-logs" => open_logs_folder(app),
		"tray-quit" => app.exit(0),
		_ => {}
	}
}

/// The daemon's log folder inside its data directory, where `wing logs` reads too.
pub fn logs_folder_in(data_dir: &Path) -> PathBuf {
	data_dir.join("logs")
}

/// Opens the daemon's log folder in the system file manager.
///
/// A daemon started at login has no console on Windows (TAURI-019), so its log file is the only
/// place its output reaches, and users need a way to find it.
fn open_logs_folder<R: Runtime>(app: &AppHandle<R>) {
	let Some(data_dir) = app
		.try_state::<DesktopSettingsState>()
		.and_then(|state| state.path.parent().map(Path::to_path_buf))
	else {
		return;
	};
	let folder = logs_folder_in(&data_dir);
	tauri::async_runtime::spawn_blocking(move || {
		// The daemon creates it on its first start; an empty folder still opens.
		if let Err(error) = std::fs::create_dir_all(&folder) {
			tracing::warn!(%error, folder = %folder.display(), "Could not create the logs folder");
			return;
		}
		match crate::file_opening::open_default(&folder) {
			Ok(file_opening::OpenResult::Success) => {}
			result => {
				tracing::warn!(?result, folder = %folder.display(), "Could not open the logs folder")
			}
		}
	});
}

/// A left click on trays that report clicks opens the main window.
fn on_tray_icon<R: Runtime>(app: &AppHandle<R>, event: &TrayIconEvent) {
	if let TrayIconEvent::Click {
		button: MouseButton::Left,
		button_state: MouseButtonState::Up,
		..
	} = event
	{
		launch::show_main_window(app);
	}
}

/// Renames the tray menu items after the UI language changes and saves the
/// labels for the next launch.
#[tauri::command]
pub fn set_tray_labels<R: Runtime>(app: AppHandle<R>, labels: TrayLabels) -> Result<(), String> {
	if let Some(path) = tray_labels_path(&app) {
		let json = serde_json::to_vec_pretty(&labels).map_err(|e| e.to_string())?;
		if let Err(error) = std::fs::write(&path, json) {
			tracing::warn!(%error, path = %path.display(), "Could not save tray labels");
		}
	}
	let Some(menu) = app.try_state::<TrayMenu<R>>() else {
		return Ok(());
	};
	for (item, text) in [
		(&menu.open, labels.open),
		(&menu.new_tab, labels.new_tab),
		(&menu.logs, labels.logs),
		(&menu.quit, labels.quit),
	] {
		item.set_text(text).map_err(|e| e.to_string())?;
	}
	Ok(())
}

/// Returns the desktop shell settings.
#[tauri::command]
pub fn get_desktop_settings(state: tauri::State<'_, DesktopSettingsState>) -> DesktopSettings {
	state.get()
}

/// Saves the desktop shell settings and applies the autostart entry.
#[tauri::command]
pub fn set_desktop_settings(
	state: tauri::State<'_, DesktopSettingsState>,
	settings: DesktopSettings,
) -> Result<DesktopSettings, String> {
	apply_autostart(settings.start_at_login, settings.start_minimized)?;
	state.save(settings.clone())?;
	Ok(settings)
}

#[cfg(target_os = "linux")]
fn apply_autostart(enabled: bool, hidden: bool) -> Result<(), String> {
	apply_autostart_in(enabled, hidden, autostart_dir(), launcher_path)
}

/// Writes or removes the entry in `dir`; `launcher` is only asked when enabling.
#[cfg(target_os = "linux")]
fn apply_autostart_in(
	enabled: bool,
	hidden: bool,
	dir: Option<PathBuf>,
	launcher: impl FnOnce() -> Option<PathBuf>,
) -> Result<(), String> {
	let dir = dir.ok_or("Could not determine the autostart directory")?;
	let launcher = if enabled {
		Some(launcher().ok_or("Could not determine the WingDrive executable")?)
	} else {
		None
	};
	write_autostart(&dir, launcher.as_deref(), hidden)
}

#[cfg(target_os = "linux")]
fn write_autostart(dir: &Path, launcher: Option<&Path>, hidden: bool) -> Result<(), String> {
	let file = dir.join(AUTOSTART_FILE);
	let Some(launcher) = launcher else {
		return match std::fs::remove_file(&file) {
			Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(e.to_string()),
			_ => Ok(()),
		};
	};
	std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
	std::fs::write(&file, autostart_entry(launcher, hidden)).map_err(|e| e.to_string())
}

#[cfg(target_os = "windows")]
const WINDOWS_RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";

#[cfg(target_os = "windows")]
fn apply_autostart(enabled: bool, hidden: bool) -> Result<(), String> {
	let command = if enabled {
		let exe = std::env::current_exe().map_err(|e| e.to_string())?;
		Some(autostart_command(&exe, hidden))
	} else {
		None
	};
	write_run_value(WINDOWS_RUN_KEY, "WingDrive", command.as_deref())
}

/// The command line a login session runs, quoted because the install path may
/// contain spaces. Windows paths cannot contain double quotes.
#[cfg(target_os = "windows")]
fn autostart_command(exe: &Path, hidden: bool) -> String {
	format!("\"{}\"{}", exe.display(), hidden_arg(hidden))
}

/// Sets `name` under `subkey` of HKEY_CURRENT_USER, or removes it for `None`.
///
/// Writing to the current user's hive needs no elevation, and removing a value
/// that is already gone counts as success, like the Linux entry.
#[cfg(target_os = "windows")]
fn write_run_value(subkey: &str, name: &str, command: Option<&str>) -> Result<(), String> {
	use windows_sys::Win32::Foundation::{ERROR_FILE_NOT_FOUND, ERROR_SUCCESS};
	use windows_sys::Win32::System::Registry::{
		RegDeleteKeyValueW, RegSetKeyValueW, HKEY_CURRENT_USER, REG_SZ,
	};

	let subkey = wide_string(subkey);
	let name = wide_string(name);
	let status = match command {
		Some(command) => {
			let data = wide_string(command);
			unsafe {
				RegSetKeyValueW(
					HKEY_CURRENT_USER,
					subkey.as_ptr(),
					name.as_ptr(),
					REG_SZ,
					data.as_ptr().cast(),
					(data.len() * std::mem::size_of::<u16>()) as u32,
				)
			}
		}
		None => unsafe { RegDeleteKeyValueW(HKEY_CURRENT_USER, subkey.as_ptr(), name.as_ptr()) },
	};

	match status {
		ERROR_SUCCESS => Ok(()),
		ERROR_FILE_NOT_FOUND if command.is_none() => Ok(()),
		code => Err(format!("Could not update the login entry (error {code})")),
	}
}

#[cfg(target_os = "windows")]
fn wide_string(text: &str) -> Vec<u16> {
	text.encode_utf16().chain(std::iter::once(0)).collect()
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
fn apply_autostart(enabled: bool, _hidden: bool) -> Result<(), String> {
	if enabled {
		Err("Start at login is only available on Linux and Windows".to_string())
	} else {
		Ok(())
	}
}

#[cfg(target_os = "linux")]
fn autostart_dir() -> Option<PathBuf> {
	autostart_dir_from(
		std::env::var_os("XDG_CONFIG_HOME"),
		std::env::var_os("HOME"),
	)
}

/// `$XDG_CONFIG_HOME/autostart`, or `~/.config/autostart` when it is unset or
/// relative, as the base directory specification requires.
#[cfg(target_os = "linux")]
fn autostart_dir_from(
	config_home: Option<std::ffi::OsString>,
	home: Option<std::ffi::OsString>,
) -> Option<PathBuf> {
	config_home
		.map(PathBuf::from)
		.filter(|dir| dir.is_absolute())
		.or_else(|| home.map(|home| PathBuf::from(home).join(".config")))
		.map(|config| config.join("autostart"))
}

/// The path a login session should run.
///
/// Inside an AppImage the executable lives in a temporary mount, so the
/// AppImage file itself is the stable launcher.
#[cfg(target_os = "linux")]
fn launcher_path() -> Option<PathBuf> {
	launcher_from(std::env::var_os("APPIMAGE"))
}

#[cfg(target_os = "linux")]
fn launcher_from(appimage: Option<std::ffi::OsString>) -> Option<PathBuf> {
	appimage
		.map(PathBuf::from)
		.or_else(|| std::env::current_exe().ok())
}

/// The argument that keeps a login start in the tray.
#[cfg(any(target_os = "linux", target_os = "windows"))]
fn hidden_arg(hidden: bool) -> &'static str {
	if hidden {
		" --hidden"
	} else {
		""
	}
}

#[cfg(target_os = "linux")]
fn autostart_entry(launcher: &Path, hidden: bool) -> String {
	// Exec quoting escapes `"`, `` ` ``, `$` and `\` with a backslash, the string
	// escapes then double every backslash, and `%` starts a field code.
	let mut exec = String::new();
	for c in launcher.to_string_lossy().chars() {
		match c {
			'"' | '`' | '$' => exec.push_str("\\\\"),
			'\\' => exec.push_str("\\\\\\"),
			'%' => exec.push('%'),
			_ => {}
		}
		exec.push(c);
	}
	// GLib and systemd's autostart generator look up the program before they
	// turn `%%` back into `%`, so they reject a launcher path with `%` in it.
	// Starting it through `env` gives them a program they can find.
	let program = if exec.contains('%') {
		"/usr/bin/env "
	} else {
		""
	};
	format!(
		"[Desktop Entry]\nType=Application\nName=WingDrive\nComment=Keep WingDrive ready in the tray\nExec={program}\"{exec}\"{hidden}\nIcon=WingDrive\nTerminal=false\nX-GNOME-Autostart-enabled=true\n",
		hidden = hidden_arg(hidden)
	)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn tray_labels_saved_before_the_logs_item_still_load() {
		let labels: TrayLabels =
			serde_json::from_str(r#"{"open":"Abrir","new_tab":"Nova aba","quit":"Sair"}"#).unwrap();
		assert_eq!(labels.open, "Abrir");
		assert_eq!(labels.logs, "Open Logs Folder");
	}

	#[test]
	fn logs_folder_is_inside_the_data_directory() {
		assert_eq!(
			logs_folder_in(Path::new("/data/wingdrive")),
			PathBuf::from("/data/wingdrive/logs")
		);
	}

	#[test]
	fn settings_default_when_missing_and_round_trip() {
		let dir = tempfile::tempdir().unwrap();
		let state = DesktopSettingsState::load(dir.path());
		assert_eq!(state.get(), DesktopSettings::default());

		let changed = DesktopSettings {
			keep_in_background: false,
			start_at_login: true,
			start_minimized: false,
			check_for_updates: false,
			terminal_command: Some("kitty".into()),
		};
		state.save(changed.clone()).unwrap();
		assert_eq!(DesktopSettingsState::load(dir.path()).get(), changed);
	}

	fn app_with_settings(dir: &Path) -> tauri::App<tauri::test::MockRuntime> {
		let app = tauri::test::mock_app();
		app.manage(DesktopSettingsState::load(dir));
		app
	}

	// Building the tray itself needs an initialized GTK, so these tests stop
	// at the labels it is built from.
	#[test]
	fn tray_labels_come_from_the_saved_file_and_are_saved_again() {
		let dir = tempfile::tempdir().unwrap();
		std::fs::write(
			dir.path().join(TRAY_LABELS_FILE),
			r#"{"open":"Abrir","new_tab":"Nova aba","quit":"Sair"}"#,
		)
		.unwrap();
		let app = app_with_settings(dir.path());
		let labels = load_tray_labels(app.handle());
		assert_eq!(
			(
				labels.open.as_str(),
				labels.new_tab.as_str(),
				labels.quit.as_str()
			),
			("Abrir", "Nova aba", "Sair")
		);

		let labels = TrayLabels {
			open: "Open".into(),
			new_tab: "Tab".into(),
			logs: "Logs".into(),
			quit: "Quit".into(),
		};
		// Without a tray the labels are still saved for the next launch.
		set_tray_labels(app.handle().clone(), labels).unwrap();
		let saved: serde_json::Value =
			serde_json::from_slice(&std::fs::read(dir.path().join(TRAY_LABELS_FILE)).unwrap())
				.unwrap();
		assert_eq!(saved["new_tab"], "Tab");
		assert_eq!(load_tray_labels(app.handle()).quit, "Quit");
	}

	#[test]
	fn tray_falls_back_to_english_and_tolerates_missing_settings() {
		let dir = tempfile::tempdir().unwrap();
		std::fs::write(dir.path().join(TRAY_LABELS_FILE), "not json").unwrap();
		let app = app_with_settings(dir.path());
		assert_eq!(load_tray_labels(app.handle()).open, "Open WingDrive");

		// Without settings there is nowhere to save, and nothing fails.
		let app = tauri::test::mock_app();
		assert!(tray_labels_path(app.handle()).is_none());
		assert_eq!(load_tray_labels(app.handle()).new_tab, "New Tab");
		set_tray_labels(app.handle().clone(), TrayLabels::default()).unwrap();

		// A labels file that cannot be written is logged, not returned.
		let dir = tempfile::tempdir().unwrap();
		std::fs::create_dir(dir.path().join(TRAY_LABELS_FILE)).unwrap();
		let app = app_with_settings(dir.path());
		set_tray_labels(app.handle().clone(), TrayLabels::default()).unwrap();
	}

	#[test]
	fn tray_items_are_renamed_when_the_language_changes() {
		let dir = tempfile::tempdir().unwrap();
		let app = app_with_settings(dir.path());
		let (_menu, items) = build_tray_menu(app.handle(), TrayLabels::default()).unwrap();
		assert_eq!(items.open.text().unwrap(), "Open WingDrive");
		app.manage(items);

		let labels = TrayLabels {
			open: "Abrir o WingDrive".into(),
			new_tab: "Nova aba".into(),
			logs: "Abrir pasta de logs".into(),
			quit: "Sair do WingDrive".into(),
		};
		set_tray_labels(app.handle().clone(), labels).unwrap();
		let items = app.state::<TrayMenu<tauri::test::MockRuntime>>();
		assert_eq!(items.open.text().unwrap(), "Abrir o WingDrive");
		assert_eq!(items.new_tab.text().unwrap(), "Nova aba");
		assert_eq!(items.logs.text().unwrap(), "Abrir pasta de logs");
		assert_eq!(items.quit.text().unwrap(), "Sair do WingDrive");
	}

	fn app_with_open_queue() -> (
		tauri::App<tauri::test::MockRuntime>,
		std::sync::mpsc::Receiver<()>,
	) {
		use tauri::Listener;

		let app = tauri::test::mock_app();
		app.manage(launch::PendingOpenRequests::new(Vec::new()));
		tauri::WebviewWindowBuilder::new(&app, "main", tauri::WebviewUrl::default())
			.build()
			.unwrap();
		let (sender, delivered) = std::sync::mpsc::channel();
		app.listen_any(launch::OPEN_REQUESTS_EVENT, move |_| {
			let _ = sender.send(());
		});
		(app, delivered)
	}

	#[test]
	fn new_tab_from_the_tray_opens_the_home_folder() {
		let (app, delivered) = app_with_open_queue();
		on_tray_menu(app.handle(), "tray-new-tab");
		let home = std::env::var_os("HOME")
			.map(PathBuf::from)
			.unwrap_or_else(|| "/".into());
		assert_eq!(
			launch::take_open_requests(app.state()),
			[launch::OpenRequest {
				directory: home,
				select: None,
			}]
		);
		assert!(delivered.try_recv().is_ok());
	}

	#[test]
	fn other_tray_actions_only_bring_the_window_forward() {
		use tauri::tray::TrayIconId;
		use tauri::{PhysicalPosition, Rect};

		let (app, delivered) = app_with_open_queue();
		on_tray_menu(app.handle(), "tray-open");
		on_tray_menu(app.handle(), "unknown-item");
		let click = |button, button_state| TrayIconEvent::Click {
			id: TrayIconId::new("main"),
			position: PhysicalPosition::new(0.0, 0.0),
			rect: Rect::default(),
			button,
			button_state,
		};
		on_tray_icon(
			app.handle(),
			&click(MouseButton::Left, MouseButtonState::Up),
		);
		on_tray_icon(
			app.handle(),
			&click(MouseButton::Right, MouseButtonState::Up),
		);
		on_tray_icon(
			app.handle(),
			&click(MouseButton::Left, MouseButtonState::Down),
		);
		assert!(launch::take_open_requests(app.state()).is_empty());
		assert!(delivered.try_recv().is_err());
	}

	#[test]
	fn closing_hides_to_the_tray_only_when_background_mode_is_on() {
		let app = tauri::test::mock_app();
		assert!(!keeps_running(app.handle()));
		let dir = tempfile::tempdir().unwrap();
		let app = app_with_settings(dir.path());
		assert!(keeps_running(app.handle()));
		app.state::<DesktopSettingsState>()
			.save(DesktopSettings {
				keep_in_background: false,
				..DesktopSettings::default()
			})
			.unwrap();
		assert!(!keeps_running(app.handle()));
		assert!(!get_desktop_settings(app.state()).keep_in_background);
	}

	#[test]
	fn settings_cannot_be_saved_into_a_file_path() {
		let dir = tempfile::tempdir().unwrap();
		let blocker = dir.path().join("file");
		std::fs::write(&blocker, b"keep").unwrap();
		let state = DesktopSettingsState::load(&blocker);
		assert!(state.save(DesktopSettings::default()).is_err());
		assert_eq!(std::fs::read(&blocker).unwrap(), b"keep");
	}

	#[test]
	fn partial_settings_file_keeps_other_defaults() {
		let dir = tempfile::tempdir().unwrap();
		std::fs::write(dir.path().join(SETTINGS_FILE), r#"{"start_at_login":true}"#).unwrap();
		let settings = DesktopSettingsState::load(dir.path()).get();
		assert!(settings.keep_in_background);
		assert!(settings.start_at_login);
		// Settings saved before the options existed keep starting in the tray and checking.
		assert!(settings.start_minimized);
		assert!(settings.check_for_updates);
	}

	#[cfg(target_os = "windows")]
	#[test]
	fn autostart_command_quotes_the_executable_and_starts_hidden() {
		assert_eq!(
			autostart_command(Path::new(r"C:\Program Files\WingDrive\WingDrive.exe"), true),
			r#""C:\Program Files\WingDrive\WingDrive.exe" --hidden"#
		);
		assert_eq!(
			autostart_command(Path::new(r"C:\WingDrive\WingDrive.exe"), false),
			r#""C:\WingDrive\WingDrive.exe""#
		);
	}

	/// Uses a throwaway key so the test never touches the real Run entry.
	#[cfg(target_os = "windows")]
	#[test]
	fn login_entry_is_written_and_removed() {
		use windows_sys::Win32::Foundation::ERROR_SUCCESS;
		use windows_sys::Win32::System::Registry::{
			RegDeleteKeyW, RegGetValueW, HKEY_CURRENT_USER, RRF_RT_REG_SZ,
		};

		let subkey = format!(r"Software\WingDriveTest-{}", std::process::id());
		let read = |name: &str| -> Option<String> {
			let (key, name) = (wide_string(&subkey), wide_string(name));
			let mut buffer = [0u16; 512];
			let mut size = std::mem::size_of_val(&buffer) as u32;
			let status = unsafe {
				RegGetValueW(
					HKEY_CURRENT_USER,
					key.as_ptr(),
					name.as_ptr(),
					RRF_RT_REG_SZ,
					std::ptr::null_mut(),
					buffer.as_mut_ptr().cast(),
					&mut size,
				)
			};
			(status == ERROR_SUCCESS)
				.then(|| String::from_utf16_lossy(&buffer[..size as usize / 2 - 1]))
		};

		let command = r#""C:\app.exe" --hidden"#;
		write_run_value(&subkey, "WingDrive", Some(command)).unwrap();
		assert_eq!(read("WingDrive").as_deref(), Some(command));

		write_run_value(&subkey, "WingDrive", None).unwrap();
		assert_eq!(read("WingDrive"), None);
		// A second removal finds nothing and still succeeds.
		write_run_value(&subkey, "WingDrive", None).unwrap();

		let key = wide_string(&subkey);
		unsafe { RegDeleteKeyW(HKEY_CURRENT_USER, key.as_ptr()) };
	}

	#[cfg(target_os = "linux")]
	#[test]
	fn autostart_file_is_written_and_removed() {
		const CHILD_ENV: &str = "WINGDRIVE_TEST_AUTOSTART_DIR";
		if let Some(config) = std::env::var_os(CHILD_ENV) {
			let file = Path::new(&config).join("autostart").join(AUTOSTART_FILE);
			apply_autostart(true, true).unwrap();
			assert!(std::fs::read_to_string(&file).unwrap().contains("--hidden"));
			apply_autostart(true, false).unwrap();
			assert!(!std::fs::read_to_string(&file).unwrap().contains("--hidden"));
			apply_autostart(false, true).unwrap();
			assert!(!file.exists());
			apply_autostart(false, true).unwrap();

			// The settings command writes the entry and the settings together.
			let app = app_with_settings(Path::new(&config));
			let on = DesktopSettings {
				start_at_login: true,
				..DesktopSettings::default()
			};
			assert_eq!(set_desktop_settings(app.state(), on.clone()).unwrap(), on);
			assert!(file.exists());
			assert_eq!(DesktopSettingsState::load(Path::new(&config)).get(), on);
			set_desktop_settings(app.state(), DesktopSettings::default()).unwrap();
			assert!(!file.exists());
			return;
		}
		let config = tempfile::tempdir().unwrap();
		let output = std::process::Command::new(std::env::current_exe().unwrap())
			.args([
				"--exact",
				"background::tests::autostart_file_is_written_and_removed",
			])
			.env(CHILD_ENV, config.path())
			.env("HOME", config.path())
			.env("XDG_CONFIG_HOME", config.path())
			.env_remove("APPIMAGE")
			.output()
			.unwrap();
		assert!(
			output.status.success(),
			"child failed: {}\n{}",
			String::from_utf8_lossy(&output.stdout),
			String::from_utf8_lossy(&output.stderr)
		);
	}

	#[test]
	fn corrupt_settings_fall_back_to_defaults_without_rewriting_the_file() {
		let dir = tempfile::tempdir().unwrap();
		let path = dir.path().join(SETTINGS_FILE);
		for contents in ["{", r#"{"start_at_login":"yes"}"#, "[]"] {
			std::fs::write(&path, contents).unwrap();
			assert_eq!(
				DesktopSettingsState::load(dir.path()).get(),
				DesktopSettings::default()
			);
			assert_eq!(std::fs::read_to_string(&path).unwrap(), contents);
		}
	}

	#[test]
	fn failed_settings_write_keeps_the_last_in_memory_value() {
		let dir = tempfile::tempdir().unwrap();
		let state = DesktopSettingsState::load(dir.path());
		std::fs::create_dir(dir.path().join(SETTINGS_FILE)).unwrap();
		let changed = DesktopSettings {
			keep_in_background: false,
			..DesktopSettings::default()
		};
		assert!(state.save(changed).is_err());
		assert_eq!(state.get(), DesktopSettings::default());
	}

	#[test]
	fn settings_create_missing_parents_and_preserve_unicode_terminal_commands() {
		let dir = tempfile::tempdir().unwrap();
		let nested = dir.path().join("new/nested");
		let state = DesktopSettingsState::load(&nested);
		let changed = DesktopSettings {
			terminal_command: Some("/opt/Aplicações/terminal --title=José".into()),
			..DesktopSettings::default()
		};
		state.save(changed.clone()).unwrap();
		assert_eq!(state.get(), changed);
		assert_eq!(DesktopSettingsState::load(&nested).get(), changed);
	}

	#[cfg(target_os = "linux")]
	#[test]
	fn autostart_propagates_creation_and_removal_errors() {
		let root = tempfile::tempdir().unwrap();
		let blocked = root.path().join("blocked");
		std::fs::write(&blocked, b"keep").unwrap();
		assert!(write_autostart(&blocked, Some(Path::new("/opt/WingDrive")), true).is_err());
		assert_eq!(std::fs::read(&blocked).unwrap(), b"keep");

		let entry = root.path().join(AUTOSTART_FILE);
		std::fs::create_dir(&entry).unwrap();
		std::fs::write(entry.join("keep"), b"keep").unwrap();
		assert!(write_autostart(root.path(), None, true).is_err());
		assert_eq!(std::fs::read(entry.join("keep")).unwrap(), b"keep");
	}

	#[cfg(target_os = "linux")]
	#[test]
	fn autostart_folder_follows_the_base_directory_specification() {
		let dir = |config: Option<&str>, home: Option<&str>| {
			autostart_dir_from(config.map(Into::into), home.map(Into::into))
		};
		assert_eq!(
			dir(Some("/cfg"), Some("/home/u")),
			Some(PathBuf::from("/cfg/autostart"))
		);
		// A relative XDG_CONFIG_HOME is invalid and ignored.
		assert_eq!(
			dir(Some("cfg"), Some("/home/u")),
			Some(PathBuf::from("/home/u/.config/autostart"))
		);
		assert_eq!(dir(None, None), None);

		assert_eq!(
			launcher_from(Some("/apps/WingDrive.AppImage".into())),
			Some(PathBuf::from("/apps/WingDrive.AppImage"))
		);
		assert_eq!(launcher_from(None), std::env::current_exe().ok());
	}

	#[cfg(target_os = "linux")]
	#[test]
	fn autostart_reports_what_it_could_not_find() {
		let root = tempfile::tempdir().unwrap();
		assert_eq!(
			apply_autostart_in(false, true, None, || None).unwrap_err(),
			"Could not determine the autostart directory"
		);
		assert_eq!(
			apply_autostart_in(true, true, Some(root.path().into()), || None).unwrap_err(),
			"Could not determine the WingDrive executable"
		);
		assert!(!root.path().join(AUTOSTART_FILE).exists());
		// Turning it off never needs the executable.
		apply_autostart_in(false, true, Some(root.path().into()), || {
			panic!("the launcher is only needed to enable autostart")
		})
		.unwrap();
	}

	/// The Desktop Entry Specification applies the string escapes first, then
	/// the Exec quoting rules, then expands `%` field codes.
	#[cfg(target_os = "linux")]
	#[test]
	fn autostart_entry_escapes_launcher_paths_for_the_exec_key() {
		let entry = autostart_entry(Path::new(r#"/opt/100%d/a\b"c$d`e/WingDrive"#), true);
		assert!(
			entry.contains(
				r#"Exec=/usr/bin/env "/opt/100%%d/a\\\\b\\"c\\$d\\`e/WingDrive" --hidden"#
			),
			"{entry}"
		);
	}

	#[cfg(target_os = "linux")]
	#[test]
	fn autostart_entry_quotes_the_launcher_and_starts_hidden() {
		let entry = autostart_entry(Path::new("/opt/Wing Drive/WingDrive"), true);
		assert!(entry.contains("Exec=\"/opt/Wing Drive/WingDrive\" --hidden\n"));
		// With "Start minimized in the tray" off, the login shows the window.
		let entry = autostart_entry(Path::new("/opt/Wing Drive/WingDrive"), false);
		assert!(
			entry.contains("Exec=\"/opt/Wing Drive/WingDrive\"\n"),
			"{entry}"
		);
	}
}
