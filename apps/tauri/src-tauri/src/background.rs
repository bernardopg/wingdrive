//! # Background mode
//!
//! A file manager has to open instantly, and a cold WingDrive start pays for
//! the daemon and a WebKit process. With background mode on, closing the main
//! window only hides it and a tray icon keeps the app reachable, so later
//! launches reuse the running instance. "Start at login" writes an XDG
//! autostart entry that launches WingDrive hidden.
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
use tauri::menu::{MenuBuilder, MenuItemBuilder, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Manager};

use crate::launch;

const SETTINGS_FILE: &str = "desktop_settings.json";
const AUTOSTART_FILE: &str = "wingdrive.desktop";

/// Desktop shell preferences, stored next to the app's other per-device state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct DesktopSettings {
	/// Closing the main window hides it to the tray instead of quitting.
	pub keep_in_background: bool,
	/// Start WingDrive hidden when the user logs in.
	pub start_at_login: bool,
	/// Terminal command for "Open Terminal Here"; empty means detect.
	pub terminal_command: Option<String>,
}

impl Default for DesktopSettings {
	fn default() -> Self {
		Self {
			keep_in_background: true,
			start_at_login: false,
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

/// Whether closing the main window should hide it instead of quitting.
pub fn keeps_running(app: &AppHandle) -> bool {
	app.try_state::<DesktopSettingsState>()
		.is_some_and(|state| state.get().keep_in_background)
}

/// Adds the tray icon with Open, New Tab and Quit.
///
/// Linux trays built on AppIndicator deliver no click events, so every action
/// lives in the menu rather than on the icon itself.
pub fn create_tray(app: &AppHandle) -> tauri::Result<()> {
	let open = MenuItemBuilder::with_id("tray-open", "Open WingDrive").build(app)?;
	let new_tab = MenuItemBuilder::with_id("tray-new-tab", "New Tab").build(app)?;
	let quit = MenuItemBuilder::with_id("tray-quit", "Quit WingDrive").build(app)?;
	let menu = MenuBuilder::new(app)
		.item(&open)
		.item(&new_tab)
		.item(&PredefinedMenuItem::separator(app)?)
		.item(&quit)
		.build()?;

	let mut tray = TrayIconBuilder::with_id("main")
		.tooltip("WingDrive")
		.menu(&menu)
		.show_menu_on_left_click(false)
		.on_menu_event(|app, event| match event.id().as_ref() {
			"tray-open" => launch::show_main_window(app),
			"tray-new-tab" => {
				let home = std::env::var_os("HOME")
					.map(PathBuf::from)
					.unwrap_or_else(|| "/".into());
				launch::deliver(
					app,
					vec![launch::OpenRequest {
						directory: home,
						select: None,
					}],
				);
			}
			"tray-quit" => app.exit(0),
			_ => {}
		})
		.on_tray_icon_event(|tray, event| {
			if let tauri::tray::TrayIconEvent::Click {
				button: tauri::tray::MouseButton::Left,
				button_state: tauri::tray::MouseButtonState::Up,
				..
			} = event
			{
				launch::show_main_window(tray.app_handle());
			}
		});
	if let Some(icon) = app.default_window_icon() {
		tray = tray.icon(icon.clone());
	}
	tray.build(app)?;
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
	apply_autostart(settings.start_at_login)?;
	state.save(settings.clone())?;
	Ok(settings)
}

#[cfg(target_os = "linux")]
fn apply_autostart(enabled: bool) -> Result<(), String> {
	let dir = autostart_dir().ok_or("Could not determine the autostart directory")?;
	let file = dir.join(AUTOSTART_FILE);
	if !enabled {
		return match std::fs::remove_file(&file) {
			Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(e.to_string()),
			_ => Ok(()),
		};
	}
	let launcher = launcher_path().ok_or("Could not determine the WingDrive executable")?;
	std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
	std::fs::write(&file, autostart_entry(&launcher)).map_err(|e| e.to_string())
}

#[cfg(not(target_os = "linux"))]
fn apply_autostart(enabled: bool) -> Result<(), String> {
	if enabled {
		Err("Start at login is only available on Linux".to_string())
	} else {
		Ok(())
	}
}

#[cfg(target_os = "linux")]
fn autostart_dir() -> Option<PathBuf> {
	std::env::var_os("XDG_CONFIG_HOME")
		.map(PathBuf::from)
		.filter(|dir| dir.is_absolute())
		.or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
		.map(|config| config.join("autostart"))
}

/// The path a login session should run.
///
/// Inside an AppImage the executable lives in a temporary mount, so the
/// AppImage file itself is the stable launcher.
#[cfg(target_os = "linux")]
fn launcher_path() -> Option<PathBuf> {
	std::env::var_os("APPIMAGE")
		.map(PathBuf::from)
		.or_else(|| std::env::current_exe().ok())
}

#[cfg(target_os = "linux")]
fn autostart_entry(launcher: &Path) -> String {
	// Desktop entry Exec values quote arguments with double quotes.
	let exec = launcher
		.to_string_lossy()
		.replace('\\', "\\\\")
		.replace('"', "\\\"");
	format!(
		"[Desktop Entry]\nType=Application\nName=WingDrive\nComment=Keep WingDrive ready in the tray\nExec=\"{exec}\" --hidden\nIcon=WingDrive\nTerminal=false\nX-GNOME-Autostart-enabled=true\n"
	)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn settings_default_when_missing_and_round_trip() {
		let dir = tempfile::tempdir().unwrap();
		let state = DesktopSettingsState::load(dir.path());
		assert_eq!(state.get(), DesktopSettings::default());

		let changed = DesktopSettings {
			keep_in_background: false,
			start_at_login: true,
			terminal_command: Some("kitty".into()),
		};
		state.save(changed.clone()).unwrap();
		assert_eq!(DesktopSettingsState::load(dir.path()).get(), changed);
	}

	#[test]
	fn partial_settings_file_keeps_other_defaults() {
		let dir = tempfile::tempdir().unwrap();
		std::fs::write(dir.path().join(SETTINGS_FILE), r#"{"start_at_login":true}"#).unwrap();
		let settings = DesktopSettingsState::load(dir.path()).get();
		assert!(settings.keep_in_background);
		assert!(settings.start_at_login);
	}

	#[cfg(target_os = "linux")]
	#[test]
	fn autostart_file_is_written_and_removed() {
		let config = tempfile::tempdir().unwrap();
		// Only this test reads XDG_CONFIG_HOME in this binary.
		std::env::set_var("XDG_CONFIG_HOME", config.path());
		let file = config.path().join("autostart").join(AUTOSTART_FILE);

		apply_autostart(true).unwrap();
		assert!(std::fs::read_to_string(&file).unwrap().contains("--hidden"));
		apply_autostart(false).unwrap();
		assert!(!file.exists());
		apply_autostart(false).unwrap();
	}

	#[cfg(target_os = "linux")]
	#[test]
	fn autostart_entry_quotes_the_launcher_and_starts_hidden() {
		let entry = autostart_entry(Path::new("/opt/Wing Drive/WingDrive"));
		assert!(entry.contains("Exec=\"/opt/Wing Drive/WingDrive\" --hidden\n"));
	}
}
