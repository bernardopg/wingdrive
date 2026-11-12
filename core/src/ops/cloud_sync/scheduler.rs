//! Starts sync pairs that have an interval when their next run is due.

use std::sync::Arc;
use std::time::Duration;

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use tracing::{debug, warn};

use super::job::CloudSyncJob;
use crate::context::CoreContext;
use crate::infra::db::entities::cloud_sync_pair;

/// How often the scheduler looks for due pairs.
const TICK: Duration = Duration::from_secs(60);

/// Whether a pair with `interval_minutes` that last ran at `last_run` is due.
pub fn is_due(
	interval_minutes: Option<i32>,
	last_run: Option<chrono::DateTime<chrono::Utc>>,
	now: chrono::DateTime<chrono::Utc>,
) -> bool {
	let Some(minutes) = interval_minutes.filter(|m| *m > 0) else {
		return false;
	};
	match last_run {
		None => true,
		Some(last) => now - last >= chrono::Duration::minutes(minutes as i64),
	}
}

/// Spawns the scheduler loop for the life of the daemon.
pub fn spawn(context: Arc<CoreContext>) {
	tokio::spawn(async move {
		// Give cloud volumes time to be restored after startup.
		tokio::time::sleep(TICK).await;
		let mut interval = tokio::time::interval(TICK);
		loop {
			interval.tick().await;
			let Some(libraries) = context.library_manager.read().await.clone() else {
				continue;
			};
			for library in libraries.list().await {
				let pairs = match cloud_sync_pair::Entity::find()
					.filter(cloud_sync_pair::Column::Enabled.eq(true))
					.filter(cloud_sync_pair::Column::IntervalMinutes.is_not_null())
					.all(library.db().conn())
					.await
				{
					Ok(pairs) => pairs,
					Err(error) => {
						warn!(%error, "Could not read cloud sync pairs");
						continue;
					}
				};
				let now = chrono::Utc::now();
				for pair in pairs {
					if super::is_running(pair.uuid)
						|| !is_due(pair.interval_minutes, pair.last_run_at, now)
					{
						continue;
					}
					debug!(pair = %pair.name, "Starting scheduled cloud sync");
					if let Err(error) = library.jobs().dispatch(CloudSyncJob::new(pair.uuid)).await
					{
						warn!(%error, pair = %pair.name, "Could not start scheduled cloud sync");
					}
				}
			}
		}
	});
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn due_only_with_an_interval_that_elapsed() {
		let now = chrono::Utc::now();
		assert!(!is_due(None, None, now));
		assert!(!is_due(Some(0), None, now));
		assert!(is_due(Some(15), None, now));
		assert!(!is_due(
			Some(15),
			Some(now - chrono::Duration::minutes(10)),
			now
		));
		assert!(is_due(
			Some(15),
			Some(now - chrono::Duration::minutes(15)),
			now
		));
	}
}
