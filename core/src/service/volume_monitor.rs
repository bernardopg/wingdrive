//! Volume monitoring service
//!
//! Periodically refreshes volume information and updates tracked volumes in the database.

use crate::{
	context::CoreContext, infra::event::EventBus, library::LibraryManager, service::Service,
	volume::VolumeManager,
};
use anyhow::Result;
use std::sync::{Arc, Weak};
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Configuration for volume monitoring
#[derive(Debug, Clone)]
pub struct VolumeMonitorConfig {
	/// How often to refresh volume information (in seconds)
	pub refresh_interval_secs: u64,
	/// Whether to update tracked volumes in the database
	pub update_tracked_volumes: bool,
}

impl Default for VolumeMonitorConfig {
	fn default() -> Self {
		Self {
			refresh_interval_secs: 30,
			update_tracked_volumes: true,
		}
	}
}

/// Background service that monitors volume state changes
pub struct VolumeMonitorService {
	volume_manager: Arc<VolumeManager>,
	library_manager: Weak<LibraryManager>,
	config: VolumeMonitorConfig,
	running: RwLock<bool>,
	handle: RwLock<Option<tokio::task::JoinHandle<()>>>,
}

impl VolumeMonitorService {
	/// Create a new volume monitor service
	pub fn new(
		volume_manager: Arc<VolumeManager>,
		library_manager: Weak<LibraryManager>,
		config: VolumeMonitorConfig,
	) -> Self {
		Self {
			volume_manager,
			library_manager,
			config,
			running: RwLock::new(false),
			handle: RwLock::new(None),
		}
	}

	/// Monitor volumes and update tracked volumes in libraries
	async fn monitor_loop(
		volume_manager: Arc<VolumeManager>,
		library_manager: Weak<LibraryManager>,
		config: VolumeMonitorConfig,
		running: Arc<RwLock<bool>>,
	) {
		let mut interval = tokio::time::interval(Duration::from_secs(config.refresh_interval_secs));

		while *running.read().await {
			interval.tick().await;

			// Refresh all volumes
			if let Err(e) = volume_manager.refresh_volumes().await {
				error!("Failed to refresh volumes: {}", e);
				continue;
			}

			// Update tracked volumes if enabled and library manager is available
			if config.update_tracked_volumes {
				if let Some(lib_manager) = library_manager.upgrade() {
					debug!("Updating tracked volumes across libraries");

					// Get all open libraries
					let libraries = lib_manager.get_open_libraries().await;

					for library in &libraries {
						// Get tracked volumes for this library
						match volume_manager.get_tracked_volumes(&library).await {
							Ok(tracked_volumes) => {
								for tracked in tracked_volumes {
									// Check if volume is still present
									if let Some(current_volume) =
										volume_manager.get_volume(&tracked.fingerprint).await
									{
										// Update volume state if changed
										if tracked.is_online != current_volume.is_mounted {
											if let Err(e) = volume_manager
												.update_tracked_volume_state(
													&library,
													&tracked.fingerprint,
													&current_volume,
												)
												.await
											{
												error!(
                                                    "Failed to update tracked volume {} in library {}: {}",
                                                    tracked.fingerprint,
                                                    library.id(),
                                                    e
                                                );
											} else {
												debug!(
                                                    "Updated tracked volume {} in library {} (online: {} -> {})",
                                                    tracked.fingerprint,
                                                    library.id(),
                                                    tracked.is_online,
                                                    current_volume.is_mounted
                                                );
											}
										}
									} else if vanished_volume_goes_offline(
										&tracked,
										volume_manager.device_id,
									) {
										// Detection no longer returns it: an unplugged drive,
										// or one that left while the daemon was down.
										match volume_manager
											.mark_tracked_volume_offline(
												&library,
												&tracked.fingerprint,
											)
											.await
										{
											Ok(()) => info!(
												"Tracked volume {} vanished; marked offline in library {}",
												tracked.fingerprint,
												library.id()
											),
											Err(e) => error!(
												"Failed to mark vanished volume {} offline in library {}: {}",
												tracked.fingerprint,
												library.id(),
												e
											),
										}
									}
								}
							}
							Err(e) => {
								error!(
									"Failed to get tracked volumes for library {}: {}",
									library.id(),
									e
								);
							}
						}
					}

					// Check for new external volumes to auto-track
					let all_volumes = volume_manager.get_all_volumes().await;
					for volume in all_volumes {
						// Only consider external volumes
						if matches!(volume.mount_type, crate::volume::types::MountType::External) {
							for library in &libraries {
								// Check if auto-tracking is enabled
								let config = library.config().await;
								if config.settings.auto_track_external_volumes {
									// Check if not already tracked
									if !volume_manager
										.is_volume_tracked(&library, &volume.fingerprint)
										.await
										.unwrap_or(false)
									{
										// Auto-track the external volume
										match volume_manager
											.track_volume(&library, &volume.fingerprint, None)
											.await
										{
											Ok(_) => {
												info!(
                                                    "Auto-tracked external volume '{}' in library '{}'",
                                                    volume.name,
                                                    library.name().await
                                                );
											}
											Err(e) => {
												debug!(
													"Failed to auto-track external volume '{}': {}",
													volume.name, e
												);
											}
										}
									}
								}
							}
						}
					}
				} else {
					debug!("Library manager not available, skipping tracked volume updates");
				}
			}
		}

		info!("Volume monitoring stopped");
	}
}

/// Whether a tracked volume that detection no longer returns should go offline.
///
/// Only this device's rows are touched: the table holds every paired device's
/// volumes, and local detection says nothing about a peer's drives. Network and
/// cloud volumes are skipped because they are restored from the database, not
/// detected, and stay out of memory while their credentials are unavailable.
fn vanished_volume_goes_offline(
	tracked: &crate::volume::types::TrackedVolume,
	device_id: uuid::Uuid,
) -> bool {
	tracked.is_online && tracked.device_id == device_id && tracked.is_network_drive != Some(true)
}

#[async_trait::async_trait]
impl Service for VolumeMonitorService {
	async fn start(&self) -> Result<()> {
		let mut running = self.running.write().await;
		if *running {
			warn!("Volume monitor service already running");
			return Ok(());
		}

		*running = true;

		let volume_manager = self.volume_manager.clone();
		let library_manager = self.library_manager.clone();
		let config = self.config.clone();
		let running_flag = Arc::new(RwLock::new(*running));

		let handle = tokio::spawn(Self::monitor_loop(
			volume_manager,
			library_manager,
			config,
			running_flag,
		));

		*self.handle.write().await = Some(handle);

		info!(
			"Volume monitor service started (refresh every {}s)",
			self.config.refresh_interval_secs
		);

		Ok(())
	}

	async fn stop(&self) -> Result<()> {
		*self.running.write().await = false;

		if let Some(handle) = self.handle.write().await.take() {
			handle.abort();
		}

		info!("Volume monitor service stopped");
		Ok(())
	}

	fn is_running(&self) -> bool {
		// Use blocking read since this is a sync method
		*self.running.blocking_read()
	}

	fn name(&self) -> &'static str {
		"volume_monitor"
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::volume::{types::TrackedVolume, VolumeFingerprint};

	fn tracked(device_id: uuid::Uuid, is_online: bool, is_network_drive: bool) -> TrackedVolume {
		TrackedVolume {
			id: 1,
			uuid: uuid::Uuid::new_v4(),
			device_id,
			fingerprint: VolumeFingerprint("usb".into()),
			display_name: None,
			tracked_at: chrono::Utc::now(),
			last_seen_at: chrono::Utc::now(),
			is_online,
			total_capacity: None,
			available_capacity: None,
			read_speed_mbps: None,
			write_speed_mbps: None,
			last_speed_test_at: None,
			file_system: None,
			mount_point: None,
			is_removable: Some(true),
			is_network_drive: is_network_drive.then_some(true),
			device_model: None,
			volume_type: "External".into(),
			is_user_visible: None,
			auto_track_eligible: None,
			total_files: None,
			total_directories: None,
			last_stats_update: None,
		}
	}

	#[test]
	fn only_this_devices_online_local_volumes_go_offline() {
		let this = uuid::Uuid::new_v4();
		let peer = uuid::Uuid::new_v4();

		assert!(vanished_volume_goes_offline(
			&tracked(this, true, false),
			this
		));
		assert!(!vanished_volume_goes_offline(
			&tracked(this, false, false),
			this
		));
		assert!(!vanished_volume_goes_offline(
			&tracked(peer, true, false),
			this
		));
		assert!(!vanished_volume_goes_offline(
			&tracked(this, true, true),
			this
		));
	}
}
