//! Strategy router for selecting the optimal delete method

use super::strategy::{DeleteStrategy, LocalDeleteStrategy, RemoteDeleteStrategy};
use crate::{domain::addressing::WingPath, volume::VolumeManager};

pub struct DeleteStrategyRouter;

impl DeleteStrategyRouter {
	/// Select optimal delete strategy based on path locations
	pub async fn select_strategy(
		paths: &[WingPath],
		_volume_manager: Option<&VolumeManager>,
	) -> Box<dyn DeleteStrategy> {
		// Cloud paths go through their volume backend on this device, so the
		// local strategy handles them; only other devices need the remote one.
		let all_here = paths.iter().all(|p| p.is_local() || p.is_cloud());

		if all_here {
			Box::new(LocalDeleteStrategy)
		} else {
			// At least one remote path - use remote strategy
			Box::new(RemoteDeleteStrategy)
		}
	}

	/// Describe the strategy that will be used
	pub async fn describe_strategy(paths: &[WingPath]) -> String {
		let local_count = paths
			.iter()
			.filter(|p| p.is_local() || p.is_cloud())
			.count();
		let remote_count = paths.len() - local_count;

		if remote_count == 0 {
			"Local deletion".to_string()
		} else if local_count == 0 {
			// Count unique remote devices
			let mut devices = std::collections::HashSet::new();
			for path in paths {
				if let Some(device_id) = path.device_id() {
					devices.insert(device_id);
				}
			}
			format!("Remote deletion ({} devices)", devices.len())
		} else {
			format!(
				"Mixed deletion ({} local, {} remote)",
				local_count, remote_count
			)
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[tokio::test]
	async fn cloud_paths_are_deleted_on_this_device() {
		let cloud = WingPath::Cloud {
			service: crate::volume::backend::CloudServiceType::GoogleDrive,
			identifier: "me@example.com".into(),
			path: "photo.jpg".into(),
		};
		assert_eq!(
			DeleteStrategyRouter::describe_strategy(&[cloud.clone(), WingPath::local("/tmp/a")])
				.await,
			"Local deletion"
		);
	}
}
