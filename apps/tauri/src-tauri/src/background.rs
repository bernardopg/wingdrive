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
use tauri::menu::{MenuBuilder, MenuItem, MenuItemBuilder, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Manager};

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

/// Tray menu items whose text follows the UI language.
struct TrayMenu {
	open: MenuItem<tauri::Wry>,
	new_tab: MenuItem<tauri::Wry>,
	quit: MenuItem<tauri::Wry>,
}

/// Tray menu labels in the current UI language, sent by the webview.
///
/// The last labels received are saved so the next launch builds the tray in
/// the user's language before the webview has loaded the daemon config.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrayLabels {
	open: String,
	new_tab: String,
	quit: String,
}

impl Default for TrayLabels {
	fn default() -> Self {
		Self {
			open: "Open WingDrive".into(),
			new_tab: "New Tab".into(),
			quit: "Quit WingDrive".into(),
		}
	}
}

/// Where the tray labels persist: next to the desktop settings.
fn tray_labels_path(app: &AppHandle) -> Option<PathBuf> {
	let state = app.try_state::<DesktopSettingsState>()?;
	Some(state.path.parent()?.join(TRAY_LABELS_FILE))
}

fn load_tray_labels(app: &AppHandle) -> TrayLabels {
	tray_labels_path(app)
		.and_then(|path| std::fs::read(path).ok())
		.and_then(|bytes| serde_json::from_slice(&bytes).ok())
		.unwrap_or_default()
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
	let labels = load_tray_labels(app);
	let open = MenuItemBuilder::with_id("tray-open", labels.open).build(app)?;
	let new_tab = MenuItemBuilder::with_id("tray-new-tab", labels.new_tab).build(app)?;
	let quit = MenuItemBuilder::with_id("tray-quit", labels.quit).build(app)?;
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
	app.manage(TrayMenu {
		open,
		new_tab,
		quit,
	});
	Ok(())
}

/// Renames the tray menu items after the UI language changes and saves the
/// labels for the next launch.
#[tauri::command]
pub fn set_tray_labels(app: AppHandle, labels: TrayLabels) -> Result<(), String> {
	if let Some(path) = tray_labels_path(&app) {
		let json = serde_json::to_vec_pretty(&labels).map_err(|e| e.to_string())?;
		if let Err(error) = std::fs::write(&path, json) {
			tracing::warn!(%error, path = %path.display(), "Could not save tray labels");
		}
	}
	let Some(menu) = app.try_state::<TrayMenu>() else {
		return Ok(());
	};
	for (item, text) in [
		(&menu.open, labels.open),
		(&menu.new_tab, labels.new_tab),
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

#[cfg(target_os = "windows")]
const WINDOWS_RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";

#[cfg(target_os = "windows")]
fn apply_autostart(enabled: bool) -> Result<(), String> {
	let command = if enabled {
		let exe = std::env::current_exe().map_err(|e| e.to_string())?;
		Some(autostart_command(&exe))
	} else {
		None
	};
	write_run_value(WINDOWS_RUN_KEY, "WingDrive", command.as_deref())
}

/// The command line a login session runs, quoted because the install path may
/// contain spaces. Windows paths cannot contain double quotes.
#[cfg(target_os = "windows")]
fn autostart_command(exe: &Path) -> String {
	format!("\"{}\" --hidden", exe.display())
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
fn apply_autostart(enabled: bool) -> Result<(), String> {
	if enabled {
		Err("Start at login is only available on Linux and Windows".to_string())
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
		"[Desktop Entry]\nType=Application\nName=WingDrive\nComment=Keep WingDrive ready in the tray\nExec={program}\"{exec}\" --hidden\nIcon=WingDrive\nTerminal=false\nX-GNOME-Autostart-enabled=true\n"
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

	#[cfg(target_os = "windows")]
	#[test]
	fn autostart_command_quotes_the_executable_and_starts_hidden() {
		assert_eq!(
			autostart_command(Path::new(r"C:\Program Files\WingDrive\WingDrive.exe")),
			r#""C:\Program Files\WingDrive\WingDrive.exe" --hidden"#
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

	/// The Desktop Entry Specification applies the string escapes first, then
	/// the Exec quoting rules, then expands `%` field codes.
	#[cfg(target_os = "linux")]
	#[test]
	fn autostart_entry_escapes_launcher_paths_for_the_exec_key() {
		let entry = autostart_entry(Path::new(r#"/opt/100%d/a\b"c$d`e/WingDrive"#));
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
		let entry = autostart_entry(Path::new("/opt/Wing Drive/WingDrive"));
		assert!(entry.contains("Exec=\"/opt/Wing Drive/WingDrive\" --hidden\n"));
	}
}
