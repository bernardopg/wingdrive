//! # Wingdrop
//!
//! Send files to a paired device the way AirDrop does: the receiver sees an
//! offer, accepts or declines it, and accepted files land in its Wingdrop
//! folder (Downloads by default).
//!
//! The exchange runs over the messaging protocol, which authenticates the
//! peer by its node, so an offer's sender is the device that sent it, not
//! the one it claims. The files then travel through the file transfer
//! protocol, which only writes inside library locations; an accepted drop
//! grants its sender write access to exactly the paths the receiver chose
//! for it, and only until the drop ends.
//!
//! 1. Sender → `Offer` (names, sizes) → receiver replies `OfferReceived`.
//! 2. Receiver → `Decision`: declined, or accepted with one destination per
//!    item, reserved on disk under free names so nothing is overwritten.
//! 3. Sender pushes every file to its destination, then sends `Finished`
//!    with the empty folders to create; `Cancelled` from either side stops
//!    the drop and the receiver removes what it reserved.

use std::collections::HashMap;
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use specta::Type;
use tokio::sync::oneshot;
use uuid::Uuid;

use crate::infra::event::{Event, EventBus};

/// How long an offer waits for an answer before it expires on both sides.
pub const OFFER_TTL: Duration = Duration::from_secs(5 * 60);

/// An accepted drop with no transfer activity for this long is abandoned.
const ACCEPTED_IDLE_TTL: Duration = Duration::from_secs(30 * 60);

/// Most items one offer may list, so a peer cannot fill memory or the screen.
const MAX_ITEMS: usize = 10_000;

/// A top-level file or folder in an offer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct WingdropItem {
	/// Name only, never a path
	pub name: String,
	pub is_dir: bool,
	/// Bytes of the file, or of every file in the folder
	pub size: u64,
	/// 1 for a file, the number of files for a folder
	pub file_count: u64,
}

/// Files a device wants to send to this one.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct WingdropOffer {
	pub id: Uuid,
	pub from_device_id: Uuid,
	pub from_device_name: String,
	pub items: Vec<WingdropItem>,
	pub total_bytes: u64,
	pub file_count: u64,
	pub expires_at: DateTime<Utc>,
}

/// What became of a drop, as clients see it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case", tag = "status")]
pub enum WingdropStatus {
	Accepted,
	Declined,
	Expired,
	Cancelled { reason: Option<String> },
	Completed { paths: Vec<String> },
}

/// Messages exchanged inside `messaging::Message::Wingdrop`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WingdropMessage {
	Offer(WingdropOffer),
	OfferReceived {
		drop_id: Uuid,
	},
	/// One destination per offered item when accepted
	Decision {
		drop_id: Uuid,
		accepted: bool,
		destinations: Vec<String>,
	},
	/// Folders with nothing in them, as (item index, path inside the item)
	Finished {
		drop_id: Uuid,
		empty_dirs: Vec<(usize, String)>,
	},
	Cancelled {
		drop_id: Uuid,
		reason: Option<String>,
	},
	Ack {
		drop_id: Uuid,
	},
	Rejected {
		drop_id: Uuid,
		reason: String,
	},
}

/// The receiver's answer, delivered to the sending job.
#[derive(Debug, Clone)]
pub enum WingdropDecision {
	Accepted(Vec<String>),
	Declined,
	Cancelled(Option<String>),
}

#[derive(Debug)]
enum IncomingStatus {
	Pending,
	Accepted {
		destinations: Vec<PathBuf>,
		last_activity: Instant,
	},
}

#[derive(Debug)]
struct Incoming {
	offer: WingdropOffer,
	status: IncomingStatus,
}

struct Outgoing {
	device_id: Uuid,
	decision: Option<oneshot::Sender<WingdropDecision>>,
	/// Set when the receiver cancels mid-transfer
	cancelled: Arc<AtomicBool>,
}

/// Drops this device is sending or receiving.
#[derive(Default)]
pub struct WingdropState {
	incoming: Mutex<HashMap<Uuid, Incoming>>,
	outgoing: Mutex<HashMap<Uuid, Outgoing>>,
	/// Destinations of cancelled drops, so chunks still on the way are
	/// refused instead of recreating what the cancel removed
	revoked: Mutex<Vec<(PathBuf, Instant)>>,
}

/// How long refused destinations are remembered after a cancel.
const REVOKED_TTL: Duration = Duration::from_secs(60 * 60);

/// Why an offer or a decision was refused.
#[derive(Debug, thiserror::Error, PartialEq)]
pub enum WingdropError {
	#[error("This drop no longer exists")]
	UnknownDrop,
	#[error("This drop was already answered")]
	AlreadyAnswered,
	#[error("Invalid offer: {0}")]
	InvalidOffer(String),
	#[error("Could not prepare the destination: {0}")]
	Destination(String),
}

/// A name that is one plain path component, safe to join to a folder.
fn safe_name(name: &str) -> bool {
	let mut components = Path::new(name).components();
	matches!(components.next(), Some(Component::Normal(_)))
		&& components.next().is_none()
		&& !name.contains(['/', '\\'])
}

/// A relative path made only of plain components.
pub fn safe_relative(path: &str) -> Option<PathBuf> {
	let path = Path::new(path);
	let mut out = PathBuf::new();
	for component in path.components() {
		match component {
			Component::Normal(part) => out.push(part),
			Component::CurDir => {}
			_ => return None,
		}
	}
	(!out.as_os_str().is_empty()).then_some(out)
}

impl WingdropState {
	pub fn new() -> Self {
		Self::default()
	}

	// ---- receiving ----

	/// Records an offer from an authenticated peer. The peer's identity and
	/// the expiry are set here, never taken from the message.
	pub fn receive_offer(
		&self,
		mut offer: WingdropOffer,
		from_device_id: Uuid,
		from_device_name: String,
	) -> Result<WingdropOffer, WingdropError> {
		if offer.items.is_empty() {
			return Err(WingdropError::InvalidOffer("no items".into()));
		}
		if offer.items.len() > MAX_ITEMS {
			return Err(WingdropError::InvalidOffer("too many items".into()));
		}
		if let Some(item) = offer.items.iter().find(|item| !safe_name(&item.name)) {
			return Err(WingdropError::InvalidOffer(format!(
				"unsafe name {:?}",
				item.name
			)));
		}
		offer.from_device_id = from_device_id;
		offer.from_device_name = from_device_name;
		offer.total_bytes = offer.items.iter().map(|item| item.size).sum();
		offer.file_count = offer.items.iter().map(|item| item.file_count).sum();
		offer.expires_at = Utc::now() + chrono::Duration::from_std(OFFER_TTL).unwrap_or_default();

		let mut incoming = self.incoming.lock().unwrap();
		if let Some(existing) = incoming.get(&offer.id) {
			// A retried request: the same sender may repeat its offer.
			return if existing.offer.from_device_id == from_device_id {
				Ok(existing.offer.clone())
			} else {
				Err(WingdropError::InvalidOffer("duplicate drop id".into()))
			};
		}
		incoming.insert(
			offer.id,
			Incoming {
				offer: offer.clone(),
				status: IncomingStatus::Pending,
			},
		);
		Ok(offer)
	}

	/// Offers waiting for an answer, oldest first.
	pub fn pending_offers(&self) -> Vec<WingdropOffer> {
		let now = Utc::now();
		let mut offers: Vec<_> = self
			.incoming
			.lock()
			.unwrap()
			.values()
			.filter(|drop| matches!(drop.status, IncomingStatus::Pending))
			.filter(|drop| drop.offer.expires_at > now)
			.map(|drop| drop.offer.clone())
			.collect();
		offers.sort_by_key(|offer| offer.expires_at);
		offers
	}

	/// Accepts a pending offer: reserves a free name under `root` for every
	/// item (an empty file or folder, so two drops never pick the same name)
	/// and grants the sender write access there. Returns the sender and the
	/// reserved paths, in item order.
	pub fn accept(
		&self,
		drop_id: Uuid,
		root: &Path,
	) -> Result<(Uuid, Vec<PathBuf>), WingdropError> {
		let mut incoming = self.incoming.lock().unwrap();
		let drop = incoming
			.get_mut(&drop_id)
			.ok_or(WingdropError::UnknownDrop)?;
		if !matches!(drop.status, IncomingStatus::Pending) {
			return Err(WingdropError::AlreadyAnswered);
		}
		if drop.offer.expires_at <= Utc::now() {
			return Err(WingdropError::UnknownDrop);
		}
		std::fs::create_dir_all(root).map_err(|e| WingdropError::Destination(e.to_string()))?;

		let mut destinations = Vec::with_capacity(drop.offer.items.len());
		for item in &drop.offer.items {
			let reserved = reserve(root, &item.name, item.is_dir);
			match reserved {
				Ok(path) => destinations.push(path),
				Err(e) => {
					remove_reserved(&destinations);
					return Err(WingdropError::Destination(e.to_string()));
				}
			}
		}
		drop.status = IncomingStatus::Accepted {
			destinations: destinations.clone(),
			last_activity: Instant::now(),
		};
		Ok((drop.offer.from_device_id, destinations))
	}

	/// Declines a pending offer. Returns its sender.
	pub fn decline(&self, drop_id: Uuid) -> Result<Uuid, WingdropError> {
		let mut incoming = self.incoming.lock().unwrap();
		match incoming.get(&drop_id) {
			Some(drop) if matches!(drop.status, IncomingStatus::Pending) => {
				let from = drop.offer.from_device_id;
				incoming.remove(&drop_id);
				Ok(from)
			}
			Some(_) => Err(WingdropError::AlreadyAnswered),
			None => Err(WingdropError::UnknownDrop),
		}
	}

	/// Whether `device_id` may write `path` as part of an accepted drop.
	/// A write counts as activity and keeps the drop alive.
	pub fn allows_write(&self, device_id: Uuid, path: &Path) -> bool {
		let mut incoming = self.incoming.lock().unwrap();
		for drop in incoming.values_mut() {
			if drop.offer.from_device_id != device_id {
				continue;
			}
			if let IncomingStatus::Accepted {
				destinations,
				last_activity,
			} = &mut drop.status
			{
				let inside = destinations
					.iter()
					.zip(&drop.offer.items)
					.any(|(dest, item)| {
						if item.is_dir {
							path.starts_with(dest) && path != dest.as_path()
						} else {
							path == dest.as_path()
						}
					});
				if inside && last_activity.elapsed() < ACCEPTED_IDLE_TTL {
					*last_activity = Instant::now();
					return true;
				}
			}
		}
		false
	}

	/// Ends an accepted drop sent by `device_id`, creating its empty folders.
	/// Returns the paths that now hold the drop.
	pub fn finish(
		&self,
		drop_id: Uuid,
		device_id: Uuid,
		empty_dirs: &[(usize, String)],
	) -> Result<Vec<PathBuf>, WingdropError> {
		let mut incoming = self.incoming.lock().unwrap();
		let destinations = match incoming.get(&drop_id) {
			Some(Incoming {
				offer,
				status: IncomingStatus::Accepted { destinations, .. },
			}) if offer.from_device_id == device_id => destinations.clone(),
			_ => return Err(WingdropError::UnknownDrop),
		};
		incoming.remove(&drop_id);
		drop(incoming);

		for (index, relative) in empty_dirs {
			if let (Some(root), Some(relative)) =
				(destinations.get(*index), safe_relative(relative))
			{
				if root.is_dir() {
					let _ = std::fs::create_dir_all(root.join(relative));
				}
			}
		}
		Ok(destinations)
	}

	/// Drops an offer or an accepted drop of `device_id` and removes what was
	/// reserved for it. Returns whether there was one.
	pub fn cancel_incoming(&self, drop_id: Uuid, device_id: Option<Uuid>) -> bool {
		let mut incoming = self.incoming.lock().unwrap();
		let owned = incoming
			.get(&drop_id)
			.is_some_and(|drop| device_id.is_none_or(|id| drop.offer.from_device_id == id));
		if !owned {
			return false;
		}
		if let Some(Incoming {
			status: IncomingStatus::Accepted { destinations, .. },
			..
		}) = incoming.remove(&drop_id)
		{
			// Refuse further writes before removing, so nothing comes back.
			let mut revoked = self.revoked.lock().unwrap();
			revoked.retain(|(_, at)| at.elapsed() < REVOKED_TTL);
			revoked.extend(destinations.iter().map(|d| (d.clone(), Instant::now())));
			drop(revoked);
			remove_reserved(&destinations);
		}
		true
	}

	/// Whether `path` belongs to a drop that was cancelled; its transfers
	/// must stop writing.
	pub fn is_revoked(&self, path: &Path) -> bool {
		self.revoked
			.lock()
			.unwrap()
			.iter()
			.any(|(dest, at)| at.elapsed() < REVOKED_TTL && path.starts_with(dest))
	}

	/// The sender of an incoming drop.
	pub fn incoming_sender(&self, drop_id: Uuid) -> Option<Uuid> {
		self.incoming
			.lock()
			.unwrap()
			.get(&drop_id)
			.map(|drop| drop.offer.from_device_id)
	}

	/// Removes offers nobody answered in time. Returns them.
	pub fn expire_pending(&self) -> Vec<WingdropOffer> {
		let now = Utc::now();
		let mut incoming = self.incoming.lock().unwrap();
		let expired: Vec<Uuid> = incoming
			.iter()
			.filter(|(_, drop)| {
				matches!(drop.status, IncomingStatus::Pending) && drop.offer.expires_at <= now
			})
			.map(|(id, _)| *id)
			.collect();
		expired
			.into_iter()
			.filter_map(|id| incoming.remove(&id).map(|drop| drop.offer))
			.collect()
	}

	// ---- sending ----

	/// Starts tracking a drop this device sends. The receiver resolves the
	/// returned channel; the flag is set when the receiver cancels later.
	pub fn register_outgoing(
		&self,
		drop_id: Uuid,
		device_id: Uuid,
	) -> (oneshot::Receiver<WingdropDecision>, Arc<AtomicBool>) {
		let (tx, rx) = oneshot::channel();
		let cancelled = Arc::new(AtomicBool::new(false));
		self.outgoing.lock().unwrap().insert(
			drop_id,
			Outgoing {
				device_id,
				decision: Some(tx),
				cancelled: cancelled.clone(),
			},
		);
		(rx, cancelled)
	}

	pub fn forget_outgoing(&self, drop_id: Uuid) {
		self.outgoing.lock().unwrap().remove(&drop_id);
	}

	/// Delivers a decision from `device_id`; ignored unless it is the device
	/// the drop was sent to.
	pub fn resolve_outgoing(
		&self,
		drop_id: Uuid,
		device_id: Uuid,
		decision: WingdropDecision,
	) -> bool {
		let mut outgoing = self.outgoing.lock().unwrap();
		let Some(drop) = outgoing.get_mut(&drop_id) else {
			return false;
		};
		if drop.device_id != device_id {
			return false;
		}
		if matches!(decision, WingdropDecision::Cancelled(_)) {
			drop.cancelled.store(true, Ordering::Relaxed);
		}
		match drop.decision.take() {
			Some(tx) => tx.send(decision).is_ok(),
			// Already accepted: a cancel only raises the flag.
			None => true,
		}
	}

	// ---- messages ----

	/// Handles a message from the authenticated peer `from_device`; the
	/// result is the reply to send back.
	pub fn handle_message(
		self: &Arc<Self>,
		from_device: Uuid,
		from_name: String,
		message: WingdropMessage,
		events: &EventBus,
		auto_accept: Option<PathBuf>,
	) -> (WingdropMessage, Option<AutoAccepted>) {
		match message {
			WingdropMessage::Offer(offer) => {
				let drop_id = offer.id;
				let repeated = self.incoming_sender(drop_id) == Some(from_device);
				match self.receive_offer(offer, from_device, from_name) {
					Ok(_) if repeated => (WingdropMessage::OfferReceived { drop_id }, None),
					Ok(offer) => {
						events.emit(Event::WingdropOffered {
							offer: offer.clone(),
						});
						let auto = auto_accept.and_then(|root| match self.accept(drop_id, &root) {
							Ok((device_id, destinations)) => {
								emit_status(events, drop_id, WingdropStatus::Accepted);
								Some(AutoAccepted {
									device_id,
									drop_id,
									destinations,
								})
							}
							Err(error) => {
								tracing::warn!(%drop_id, %error, "Could not auto-accept a Wingdrop offer");
								None
							}
						});
						(WingdropMessage::OfferReceived { drop_id }, auto)
					}
					Err(error) => (
						WingdropMessage::Rejected {
							drop_id,
							reason: error.to_string(),
						},
						None,
					),
				}
			}
			WingdropMessage::Decision {
				drop_id,
				accepted,
				destinations,
			} => {
				let decision = if accepted {
					WingdropDecision::Accepted(destinations)
				} else {
					WingdropDecision::Declined
				};
				self.resolve_outgoing(drop_id, from_device, decision);
				(WingdropMessage::Ack { drop_id }, None)
			}
			WingdropMessage::Finished {
				drop_id,
				empty_dirs,
			} => match self.finish(drop_id, from_device, &empty_dirs) {
				Ok(paths) => {
					emit_status(
						events,
						drop_id,
						WingdropStatus::Completed {
							paths: paths
								.iter()
								.map(|p| p.to_string_lossy().into_owned())
								.collect(),
						},
					);
					(WingdropMessage::Ack { drop_id }, None)
				}
				// Already finished (a retried request) or never accepted: nothing to do.
				Err(_) => (WingdropMessage::Ack { drop_id }, None),
			},
			WingdropMessage::Cancelled { drop_id, reason } => {
				// Either an incoming drop its sender stopped, or an outgoing one
				// the receiver stopped.
				if self.cancel_incoming(drop_id, Some(from_device)) {
					emit_status(
						events,
						drop_id,
						WingdropStatus::Cancelled {
							reason: reason.clone(),
						},
					);
				}
				self.resolve_outgoing(drop_id, from_device, WingdropDecision::Cancelled(reason));
				(WingdropMessage::Ack { drop_id }, None)
			}
			WingdropMessage::OfferReceived { drop_id }
			| WingdropMessage::Ack { drop_id }
			| WingdropMessage::Rejected { drop_id, .. } => (WingdropMessage::Ack { drop_id }, None),
		}
	}
}

/// An offer accepted on arrival; its decision still has to reach the sender.
#[derive(Debug)]
pub struct AutoAccepted {
	pub device_id: Uuid,
	pub drop_id: Uuid,
	pub destinations: Vec<PathBuf>,
}

/// Sends `message` to the paired device `device_id` and returns its reply.
/// A `Rejected` reply becomes an error carrying the peer's reason.
pub async fn send_message(
	context: &crate::context::CoreContext,
	device_id: Uuid,
	message: WingdropMessage,
) -> Result<WingdropMessage, String> {
	let networking = context
		.get_networking()
		.await
		.ok_or_else(|| "Networking is not running".to_string())?;
	match networking.send_wingdrop(device_id, message).await {
		Ok(WingdropMessage::Rejected { reason, .. }) => Err(reason),
		Ok(reply) => Ok(reply),
		Err(error) => Err(error.to_string()),
	}
}

/// The name a paired device announced, or its UUID.
pub async fn device_name(context: &crate::context::CoreContext, device_id: Uuid) -> String {
	use crate::service::network::device::DeviceState;
	if let Some(networking) = context.get_networking().await {
		let registry = networking.device_registry();
		let registry = registry.read().await;
		if let Some(
			DeviceState::Paired { info, .. }
			| DeviceState::Connected { info, .. }
			| DeviceState::Disconnected { info, .. },
		) = registry.get_device_state(device_id)
		{
			return info.device_name.clone();
		}
	}
	device_id.to_string()
}

/// Tells the sender of `drop_id` whether it was accepted, and where to.
pub async fn send_decision(
	context: &crate::context::CoreContext,
	device_id: Uuid,
	drop_id: Uuid,
	accepted: bool,
	destinations: Vec<String>,
) -> Result<(), String> {
	send_message(
		context,
		device_id,
		WingdropMessage::Decision {
			drop_id,
			accepted,
			destinations,
		},
	)
	.await
	.map(|_| ())
}

pub fn emit_status(events: &EventBus, drop_id: Uuid, status: WingdropStatus) {
	events.emit(Event::WingdropUpdated { drop_id, status });
}

/// Folder that receives drops: the configured one, else Downloads, else home.
pub fn destination_root(configured: Option<&Path>) -> PathBuf {
	configured
		.map(Path::to_path_buf)
		.or_else(dirs::download_dir)
		.or_else(dirs::home_dir)
		.unwrap_or_else(std::env::temp_dir)
}

/// Creates an empty file or folder under a free variant of `name` and returns
/// it. Creating it is what claims the name against other drops.
fn reserve(root: &Path, name: &str, is_dir: bool) -> std::io::Result<PathBuf> {
	for _ in 0..64 {
		let path = crate::ops::archive::job::free_path(root, name);
		let created = if is_dir {
			std::fs::create_dir(&path)
		} else {
			std::fs::OpenOptions::new()
				.write(true)
				.create_new(true)
				.open(&path)
				.map(|_| ())
		};
		match created {
			Ok(()) => return Ok(path),
			Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
			Err(e) => return Err(e),
		}
	}
	Err(std::io::ErrorKind::AlreadyExists.into())
}

fn remove_reserved(paths: &[PathBuf]) {
	for path in paths {
		let result = if path.is_dir() {
			std::fs::remove_dir_all(path)
		} else {
			std::fs::remove_file(path)
		};
		if let Err(error) = result {
			if error.kind() != std::io::ErrorKind::NotFound {
				tracing::warn!(path = %path.display(), %error, "Could not remove a cancelled Wingdrop item");
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn offer(items: Vec<WingdropItem>) -> WingdropOffer {
		WingdropOffer {
			id: Uuid::new_v4(),
			from_device_id: Uuid::nil(),
			from_device_name: "spoofed".into(),
			items,
			total_bytes: 0,
			file_count: 0,
			expires_at: Utc::now(),
		}
	}

	fn item(name: &str, is_dir: bool) -> WingdropItem {
		WingdropItem {
			name: name.into(),
			is_dir,
			size: 3,
			file_count: 1,
		}
	}

	#[test]
	fn offers_take_the_authenticated_sender_and_reject_paths() {
		let state = WingdropState::new();
		let sender = Uuid::new_v4();
		let received = state
			.receive_offer(offer(vec![item("a.txt", false)]), sender, "laptop".into())
			.unwrap();
		assert_eq!(received.from_device_id, sender);
		assert_eq!(received.from_device_name, "laptop");
		assert!(received.expires_at > Utc::now());
		assert_eq!(state.pending_offers().len(), 1);

		for bad in ["../x", "a/b", "/etc/passwd", "..", ".", ""] {
			assert!(
				state
					.receive_offer(offer(vec![item(bad, false)]), sender, "x".into())
					.is_err(),
				"{bad:?}"
			);
		}
	}

	#[test]
	fn accepting_reserves_free_names_and_grants_only_them() {
		let dir = tempfile::tempdir().unwrap();
		std::fs::write(dir.path().join("a.txt"), "old").unwrap();
		let state = WingdropState::new();
		let (sender, other) = (Uuid::new_v4(), Uuid::new_v4());
		let o = state
			.receive_offer(
				offer(vec![item("a.txt", false), item("photos", true)]),
				sender,
				"laptop".into(),
			)
			.unwrap();

		let (from, dests) = state.accept(o.id, dir.path()).unwrap();
		assert_eq!(from, sender);
		assert_eq!(dests[0], dir.path().join("a 2.txt"));
		assert_eq!(dests[1], dir.path().join("photos"));
		assert_eq!(
			std::fs::read_to_string(dir.path().join("a.txt")).unwrap(),
			"old"
		);
		assert_eq!(
			state.accept(o.id, dir.path()),
			Err(WingdropError::AlreadyAnswered)
		);

		assert!(state.allows_write(sender, &dests[0]));
		assert!(state.allows_write(sender, &dests[1].join("2024/x.jpg")));
		assert!(!state.allows_write(sender, &dests[1]));
		assert!(!state.allows_write(sender, &dir.path().join("a.txt")));
		assert!(!state.allows_write(other, &dests[0]));

		let paths = state
			.finish(
				o.id,
				sender,
				&[(1, "empty/inside".into()), (1, "../escape".into())],
			)
			.unwrap();
		assert_eq!(paths, dests);
		assert!(dests[1].join("empty/inside").is_dir());
		assert!(!dir.path().join("escape").exists());
		assert!(!state.allows_write(sender, &dests[0]));
	}

	#[test]
	fn cancelling_removes_reserved_items_only_for_the_sender() {
		let dir = tempfile::tempdir().unwrap();
		let state = WingdropState::new();
		let sender = Uuid::new_v4();
		let o = state
			.receive_offer(offer(vec![item("photos", true)]), sender, "laptop".into())
			.unwrap();
		let (_, dests) = state.accept(o.id, dir.path()).unwrap();
		assert!(!state.cancel_incoming(o.id, Some(Uuid::new_v4())));
		assert!(dests[0].exists());
		assert!(state.cancel_incoming(o.id, Some(sender)));
		assert!(!dests[0].exists());
		assert!(state.is_revoked(&dests[0].join("late.jpg")));
		assert!(!state.is_revoked(&dir.path().join("other")));
	}

	#[test]
	fn decisions_reach_only_the_drop_they_belong_to() {
		let state = WingdropState::new();
		let (receiver, stranger) = (Uuid::new_v4(), Uuid::new_v4());
		let drop_id = Uuid::new_v4();
		let (mut rx, cancelled) = state.register_outgoing(drop_id, receiver);
		assert!(!state.resolve_outgoing(drop_id, stranger, WingdropDecision::Declined));
		assert!(rx.try_recv().is_err());
		assert!(state.resolve_outgoing(
			drop_id,
			receiver,
			WingdropDecision::Accepted(vec!["/x".into()])
		));
		assert!(matches!(rx.try_recv(), Ok(WingdropDecision::Accepted(_))));
		state.resolve_outgoing(drop_id, receiver, WingdropDecision::Cancelled(None));
		assert!(cancelled.load(Ordering::Relaxed));
	}

	#[test]
	fn unanswered_offers_expire() {
		let state = WingdropState::new();
		let o = state
			.receive_offer(offer(vec![item("a", false)]), Uuid::new_v4(), "x".into())
			.unwrap();
		state
			.incoming
			.lock()
			.unwrap()
			.get_mut(&o.id)
			.unwrap()
			.offer
			.expires_at = Utc::now();
		assert!(state.pending_offers().is_empty());
		assert_eq!(state.expire_pending().len(), 1);
		assert_eq!(state.decline(o.id), Err(WingdropError::UnknownDrop));
	}
}
