//! The job that sends a Wingdrop: offer, wait for the answer, push the files.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::context::CoreContext;
use crate::infra::job::{generic_progress::GenericProgress, prelude::*};
use crate::ops::files::copy::strategy::push_file_to_device;
use crate::service::wingdrop::{
	self, WingdropDecision, WingdropItem, WingdropMessage, WingdropOffer, OFFER_TTL,
};

/// Sends `sources` to the paired device `device_id` once it accepts them.
#[derive(Debug, Serialize, Deserialize, Job)]
pub struct WingdropSendJob {
	pub drop_id: Uuid,
	pub device_id: Uuid,
	pub sources: Vec<PathBuf>,
}

impl Job for WingdropSendJob {
	const NAME: &'static str = "wingdrop_send";
	// The receiver forgets an interrupted drop, so a rerun has to offer again.
	const RESUMABLE: bool = false;
	const DESCRIPTION: Option<&'static str> = Some("Send files to a paired device");
}

impl crate::infra::job::traits::DynJob for WingdropSendJob {
	fn job_name(&self) -> &'static str {
		Self::NAME
	}
}

/// A top-level item with the files to push and the folders that stay empty.
#[derive(Debug, PartialEq)]
struct PlannedItem {
	item: WingdropItem,
	/// (file on this device, path inside the item; empty for a single file)
	files: Vec<(PathBuf, PathBuf)>,
	empty_dirs: Vec<PathBuf>,
}

/// Walks the sources without following links; links and special files are
/// skipped, as other devices may not be able to recreate them.
fn plan(sources: &[PathBuf]) -> std::io::Result<Vec<PlannedItem>> {
	let mut items = Vec::new();
	for source in sources {
		let name = source
			.file_name()
			.ok_or_else(|| std::io::Error::other(format!("Cannot send {}", source.display())))?
			.to_string_lossy()
			.into_owned();
		let meta = std::fs::symlink_metadata(source)?;
		if meta.is_file() {
			items.push(PlannedItem {
				item: WingdropItem {
					name,
					is_dir: false,
					size: meta.len(),
					file_count: 1,
				},
				files: vec![(source.clone(), PathBuf::new())],
				empty_dirs: Vec::new(),
			});
		} else if meta.is_dir() {
			let mut files = Vec::new();
			let mut empty_dirs = Vec::new();
			let mut size = 0;
			let mut stack = vec![PathBuf::new()];
			while let Some(relative) = stack.pop() {
				let mut children: Vec<_> =
					std::fs::read_dir(source.join(&relative))?.collect::<Result<_, _>>()?;
				children.sort_by_key(|entry| entry.file_name());
				if children.is_empty() && !relative.as_os_str().is_empty() {
					empty_dirs.push(relative.clone());
				}
				for child in children {
					let child_relative = relative.join(child.file_name());
					let child_meta = std::fs::symlink_metadata(child.path())?;
					if child_meta.is_dir() {
						stack.push(child_relative);
					} else if child_meta.is_file() {
						size += child_meta.len();
						files.push((child.path(), child_relative));
					}
				}
			}
			items.push(PlannedItem {
				item: WingdropItem {
					name,
					is_dir: true,
					size,
					file_count: files.len() as u64,
				},
				files,
				empty_dirs,
			});
		}
	}
	Ok(items)
}

/// Tells the receiver the drop stopped unless it finished; runs on drop
/// because a cancelled job's future is dropped, not resumed.
struct CancelOnDrop {
	context: Arc<CoreContext>,
	device_id: Uuid,
	drop_id: Uuid,
	reason: Option<String>,
	armed: bool,
}

impl Drop for CancelOnDrop {
	fn drop(&mut self) {
		self.context.wingdrop.forget_outgoing(self.drop_id);
		if !self.armed {
			return;
		}
		let (context, device_id, drop_id) = (self.context.clone(), self.device_id, self.drop_id);
		let reason = self
			.reason
			.take()
			.or_else(|| Some("Cancelled by the sending device".to_string()));
		tokio::spawn(async move {
			let message = WingdropMessage::Cancelled { drop_id, reason };
			if let Err(error) = wingdrop::send_message(&context, device_id, message).await {
				tracing::warn!(%drop_id, %error, "Could not tell the receiver a Wingdrop stopped");
			}
		});
	}
}

fn human_bytes(bytes: u64) -> String {
	const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
	let mut value = bytes as f64;
	let mut unit = 0;
	while value >= 1024.0 && unit < UNITS.len() - 1 {
		value /= 1024.0;
		unit += 1;
	}
	if unit == 0 {
		format!("{bytes} B")
	} else {
		format!("{value:.1} {}", UNITS[unit])
	}
}

#[async_trait::async_trait]
impl JobHandler for WingdropSendJob {
	type Output = JobOutput;

	async fn run(&mut self, ctx: JobContext<'_>) -> JobResult<Self::Output> {
		let context = ctx.library().core_context().clone();
		let sources = self.sources.clone();
		let planned = tokio::task::spawn_blocking(move || plan(&sources))
			.await
			.map_err(|e| JobError::execution(e.to_string()))?
			.map_err(|e| JobError::execution(format!("Cannot read the files to send: {e}")))?;
		if planned.is_empty() {
			return Err(JobError::execution("Nothing to send"));
		}
		let receiver = wingdrop::device_name(&context, self.device_id).await;

		let (decision, receiver_cancelled) = context
			.wingdrop
			.register_outgoing(self.drop_id, self.device_id);
		let mut guard = CancelOnDrop {
			context: context.clone(),
			device_id: self.device_id,
			drop_id: self.drop_id,
			reason: None,
			armed: true,
		};

		let items: Vec<WingdropItem> = planned.iter().map(|p| p.item.clone()).collect();
		let total_bytes: u64 = items.iter().map(|item| item.size).sum();
		let offer = WingdropOffer {
			id: self.drop_id,
			// The receiver fills in who sent it from the authenticated connection.
			from_device_id: Uuid::nil(),
			from_device_name: String::new(),
			file_count: items.iter().map(|item| item.file_count).sum(),
			total_bytes,
			items,
			expires_at: chrono::Utc::now(),
		};
		ctx.log(format!(
			"Offering {} items ({}) to {receiver}",
			offer.items.len(),
			human_bytes(total_bytes)
		));
		if let Err(reason) =
			wingdrop::send_message(&context, self.device_id, WingdropMessage::Offer(offer)).await
		{
			guard.armed = false;
			return Err(JobError::execution(format!(
				"{receiver} refused the offer: {reason}"
			)));
		}

		ctx.progress(Progress::Generic(GenericProgress::new(
			0.0,
			"Waiting",
			format!("Waiting for {receiver} to accept"),
		)));
		let deadline = Instant::now() + OFFER_TTL + Duration::from_secs(5);
		tokio::pin!(decision);
		let decision = loop {
			tokio::select! {
				result = &mut decision => break result.ok(),
				_ = tokio::time::sleep(Duration::from_millis(250)) => {
					ctx.check_interrupt().await?;
					if Instant::now() >= deadline {
						guard.reason = Some("No answer in time".to_string());
						return Err(JobError::execution(format!("{receiver} did not answer in time")));
					}
				}
			}
		};

		let destinations = match decision {
			Some(WingdropDecision::Accepted(destinations)) => destinations,
			Some(WingdropDecision::Declined) => {
				guard.armed = false;
				return Err(JobError::execution(format!(
					"{receiver} declined the files"
				)));
			}
			Some(WingdropDecision::Cancelled(reason)) => {
				guard.armed = false;
				return Err(JobError::execution(
					reason.unwrap_or_else(|| format!("{receiver} cancelled the drop")),
				));
			}
			None => {
				guard.armed = false;
				return Err(JobError::execution(format!(
					"{receiver} cancelled the drop"
				)));
			}
		};
		if destinations.len() != planned.len() {
			guard.reason = Some("Destinations do not match the offer".to_string());
			return Err(JobError::execution(
				"The receiver answered with a different item list",
			));
		}

		let started = Instant::now();
		let done = Arc::new(AtomicU64::new(0));
		let mut sent_files = 0u64;
		for (planned_item, destination) in planned.iter().zip(&destinations) {
			for (local, relative) in &planned_item.files {
				if receiver_cancelled.load(Ordering::Relaxed) {
					guard.armed = false;
					return Err(JobError::execution(format!(
						"{receiver} cancelled the drop"
					)));
				}
				ctx.check_interrupt().await?;

				let target = if relative.as_os_str().is_empty() {
					PathBuf::from(destination)
				} else {
					Path::new(destination).join(relative)
				};
				let before = done.load(Ordering::Relaxed);
				let label = local
					.file_name()
					.map(|n| n.to_string_lossy().into_owned())
					.unwrap_or_default();
				let progress_ctx = &ctx;
				let counter = done.clone();
				let to = receiver.as_str();
				let report = move |file_bytes: u64, _file_size: u64| {
					let now = before + file_bytes;
					counter.store(now, Ordering::Relaxed);
					let fraction = if total_bytes > 0 {
						(now as f64 / total_bytes as f64).min(1.0)
					} else {
						1.0
					};
					progress_ctx.progress(Progress::Generic(
						GenericProgress::new(
							fraction as f32,
							"Sending",
							format!("{label} to {to}"),
						)
						.with_bytes(now.min(total_bytes), total_bytes)
						.with_performance(0.0, None, Some(started.elapsed())),
					));
				};
				let report: crate::ops::files::copy::strategy::ProgressCallback<'_> =
					Box::new(report);
				let push = push_file_to_device(
					&ctx,
					local,
					self.device_id,
					target.to_string_lossy().into_owned(),
					Some(&report),
				);
				// The receiver's cancel arrives as a message; stop mid-file then
				// instead of streaming the rest into refused writes.
				let stopped = async {
					while !receiver_cancelled.load(Ordering::Relaxed) {
						tokio::time::sleep(Duration::from_millis(200)).await;
					}
				};
				let result = tokio::select! {
					result = push => Some(result),
					_ = stopped => None,
				};
				match result {
					None => {
						guard.armed = false;
						return Err(JobError::execution(format!("{receiver} cancelled the drop")));
					}
					Some(Err(error)) => {
						guard.reason = Some(format!("Sending {} failed", local.display()));
						return Err(JobError::execution(format!(
							"Sending {} failed: {error}",
							local.display()
						)));
					}
					Some(Ok(_)) => {}
				}
				done.store(
					before + std::fs::metadata(local).map(|m| m.len()).unwrap_or(0),
					Ordering::Relaxed,
				);
				sent_files += 1;
			}
		}

		let empty_dirs = planned
			.iter()
			.enumerate()
			.flat_map(|(index, item)| {
				item.empty_dirs
					.iter()
					.map(move |dir| (index, dir.to_string_lossy().into_owned()))
			})
			.collect();
		match wingdrop::send_message(
			&context,
			self.device_id,
			WingdropMessage::Finished {
				drop_id: self.drop_id,
				empty_dirs,
			},
		)
		.await
		{
			Ok(_) => guard.armed = false,
			Err(error) => {
				guard.armed = false;
				return Err(JobError::execution(format!(
					"The files arrived, but {receiver} could not confirm: {error}"
				)));
			}
		}
		ctx.log(format!("Sent {sent_files} files to {receiver}"));
		Ok(JobOutput::Custom(serde_json::json!({
			"drop_id": self.drop_id,
			"device_id": self.device_id,
			"files": sent_files,
			"bytes": total_bytes,
		})))
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn plans_files_folders_and_empty_folders() {
		let dir = tempfile::tempdir().unwrap();
		let photos = dir.path().join("photos");
		std::fs::create_dir_all(photos.join("2024/empty")).unwrap();
		std::fs::write(photos.join("2024/a.jpg"), "aaa").unwrap();
		std::fs::write(photos.join("b.jpg"), "bb").unwrap();
		let note = dir.path().join("note.txt");
		std::fs::write(&note, "n").unwrap();
		#[cfg(unix)]
		std::os::unix::fs::symlink(&note, photos.join("link")).unwrap();

		let planned = plan(&[photos.clone(), note.clone()]).unwrap();
		assert_eq!(planned.len(), 2);
		let folder = &planned[0];
		assert_eq!(
			(folder.item.is_dir, folder.item.size, folder.item.file_count),
			(true, 5, 2)
		);
		let mut relatives: Vec<_> = folder.files.iter().map(|(_, r)| r.clone()).collect();
		relatives.sort();
		assert_eq!(
			relatives,
			[PathBuf::from("2024/a.jpg"), PathBuf::from("b.jpg")]
		);
		assert_eq!(folder.empty_dirs, [PathBuf::from("2024/empty")]);
		assert_eq!(
			planned[1].item,
			WingdropItem {
				name: "note.txt".into(),
				is_dir: false,
				size: 1,
				file_count: 1,
			}
		);
		assert_eq!(planned[1].files, [(note, PathBuf::new())]);
	}
}
