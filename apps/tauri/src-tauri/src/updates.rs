//! # Updates
//!
//! WingDrive updates itself only where nothing else owns its files: the AppImage
//! the user downloaded and the Windows installers. A package manager (pacman
//! with the AUR packages, apt with the `.deb`) installs into system folders and
//! records every file it owns, so an update written there behind its back would
//! leave the package database lying about the installed version. Those installs
//! report the package manager as their update channel and never download
//! anything.
//!
//! Releases publish a `latest.json` signed with the WingDrive updater key
//! (REL-001); the updater plugin checks its signature before installing.
//!
//! ## Example
//! ```rust,ignore
//! let info = updates::updater_info(app.handle().clone());
//! if info.channel == updates::UpdateChannel::SelfUpdate { /* offer the check */ }
//! ```

use std::sync::atomic::{AtomicBool, Ordering};

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, Runtime};

use crate::notifications::{NotificationKind, NotificationRequest, Notifier};

/// Event carrying download progress while an update installs.
pub const PROGRESS_EVENT: &str = "updater://progress";

/// Who installs new versions of this copy of WingDrive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UpdateChannel {
	/// WingDrive downloads, verifies and installs signed releases itself.
	SelfUpdate,
	/// pacman, yay, apt or another package manager owns the files.
	PackageManager,
	/// No signed releases exist for this platform yet (macOS).
	Unsupported,
}

/// What the settings page needs to show the update section.
#[derive(Debug, Clone, Serialize)]
pub struct UpdaterInfo {
	pub channel: UpdateChannel,
	pub current_version: String,
}

/// A newer release that passed the check.
#[derive(Debug, Clone, Serialize)]
pub struct AvailableUpdate {
	pub version: String,
	pub date: Option<String>,
	pub notes: Option<String>,
}

/// Download progress, in bytes.
#[derive(Debug, Clone, Serialize)]
struct Progress {
	downloaded: u64,
	total: Option<u64>,
}

/// The update channel of the running copy.
pub fn channel() -> UpdateChannel {
	channel_for(std::env::var_os("APPIMAGE").is_some())
}

/// The channel for a Linux run with or without `$APPIMAGE` (set only by the
/// AppImage runtime, never by the extracted copy the AUR package installs).
fn channel_for(running_from_appimage: bool) -> UpdateChannel {
	if cfg!(target_os = "windows") {
		UpdateChannel::SelfUpdate
	} else if cfg!(target_os = "linux") {
		if running_from_appimage {
			UpdateChannel::SelfUpdate
		} else {
			UpdateChannel::PackageManager
		}
	} else {
		UpdateChannel::Unsupported
	}
}

#[tauri::command]
pub fn updater_info<R: Runtime>(app: AppHandle<R>) -> UpdaterInfo {
	UpdaterInfo {
		channel: channel(),
		current_version: app.package_info().version.to_string(),
	}
}

/// Asks the release server for a newer signed version.
///
/// Every channel may check; only `SelfUpdate` installs. A package-manager install learns that a
/// release exists and is told to update through its package manager.
#[tauri::command]
pub async fn check_for_update<R: Runtime>(
	app: AppHandle<R>,
) -> Result<Option<AvailableUpdate>, String> {
	find_update(&app).await.map(|update| {
		update.map(|update| AvailableUpdate {
			version: update.version.clone(),
			date: update.date.map(|date| date.to_string()),
			notes: update.body.clone(),
		})
	})
}

/// Downloads, verifies and installs the newest release, then restarts.
///
/// On Windows the installer closes WingDrive itself and opens the new version.
#[tauri::command]
pub async fn install_update<R: Runtime>(app: AppHandle<R>) -> Result<(), String> {
	if channel() != UpdateChannel::SelfUpdate {
		return Err("Updates come from your package manager".to_string());
	}
	let Some(update) = find_update(&app).await? else {
		return Err("WingDrive is up to date".to_string());
	};
	tracing::info!(version = %update.version, "Installing update");
	let mut downloaded = 0u64;
	let progress_app = app.clone();
	update
		.download_and_install(
			move |chunk, total| {
				downloaded += chunk as u64;
				let _ = progress_app.emit(PROGRESS_EVENT, Progress { downloaded, total });
			},
			|| tracing::info!("Update downloaded and verified"),
		)
		.await
		.map_err(|error| error.to_string())?;
	app.restart()
}

async fn find_update<R: Runtime>(
	app: &AppHandle<R>,
) -> Result<Option<tauri_plugin_updater::Update>, String> {
	use tauri_plugin_updater::UpdaterExt;
	app.updater()
		.map_err(|error| error.to_string())?
		.check()
		.await
		.map_err(|error| error.to_string())
}

/// Set once per run so the startup check never notifies twice.
static STARTUP_CHECK_DONE: AtomicBool = AtomicBool::new(false);

/// File in the app data folder holding the last version the user was told about.
const NOTIFIED_FILE: &str = "update-notified";

/// What the notification tells the user to do on each channel.
fn notification_body(channel: UpdateChannel) -> &'static str {
	match channel {
		UpdateChannel::SelfUpdate => "Install it from Settings > General > Updates.",
		UpdateChannel::PackageManager => {
			"Update it with the package manager you installed WingDrive with, for example yay -Syu."
		}
		UpdateChannel::Unsupported => "Download it from wingdrive.org.",
	}
}

/// Checks for a new release once after startup and tells the user about it with a desktop
/// notification, once per version: a restart does not repeat it.
pub fn spawn_startup_check<R: Runtime>(app: AppHandle<R>, enabled: bool) {
	if !enabled || STARTUP_CHECK_DONE.swap(true, Ordering::SeqCst) {
		return;
	}
	tauri::async_runtime::spawn(async move {
		// Leave the first seconds to the window and the daemon.
		tokio::time::sleep(std::time::Duration::from_secs(30)).await;
		match find_update(&app).await {
			Ok(Some(update)) => {
				tracing::info!(version = %update.version, "Update available");
				let notified = app
					.path()
					.app_data_dir()
					.ok()
					.map(|dir| dir.join(NOTIFIED_FILE));
				if let Some(path) = &notified {
					if std::fs::read_to_string(path).ok().as_deref() == Some(update.version.as_str()) {
						return;
					}
				}
				let request = NotificationRequest {
					title: format!("WingDrive {} is available", update.version),
					body: notification_body(channel()).to_string(),
					kind: NotificationKind::Info,
					open_label: None,
				};
				if let Some(notifier) = app.try_state::<Notifier>() {
					match notifier.send(&app, &request).await {
						Ok(_) => {
							if let Some(path) = &notified {
								let _ = std::fs::create_dir_all(path.parent().unwrap_or(path));
								let _ = std::fs::write(path, &update.version);
							}
						}
						Err(error) => tracing::debug!(%error, "Update notification not shown"),
					}
				}
			}
			Ok(None) => tracing::debug!("WingDrive is up to date"),
			// Offline or the release server is unreachable: try again next start.
			Err(error) => tracing::debug!(%error, "Update check failed"),
		}
	});
}

#[cfg(test)]
mod tests {
	mod http;

	use super::*;

	#[test]
	fn only_the_appimage_and_windows_update_themselves() {
		if cfg!(target_os = "linux") {
			// The AUR package runs the extracted AppImage from /opt without $APPIMAGE.
			assert_eq!(channel_for(false), UpdateChannel::PackageManager);
			assert_eq!(channel_for(true), UpdateChannel::SelfUpdate);
		}
		if cfg!(target_os = "windows") {
			assert_eq!(channel_for(false), UpdateChannel::SelfUpdate);
		}
		if cfg!(target_os = "macos") {
			assert_eq!(channel_for(false), UpdateChannel::Unsupported);
		}
	}

	#[test]
	fn package_manager_installs_are_told_how_to_update() {
		assert!(notification_body(UpdateChannel::PackageManager).contains("package manager"));
		assert!(notification_body(UpdateChannel::SelfUpdate).contains("Settings"));
	}

	#[test]
	fn channel_serializes_for_the_interface() {
		assert_eq!(
			serde_json::to_string(&UpdateChannel::PackageManager).unwrap(),
			"\"package_manager\""
		);
		assert_eq!(
			serde_json::to_string(&UpdateChannel::SelfUpdate).unwrap(),
			"\"self_update\""
		);
	}
}
