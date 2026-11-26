//! Retry queue for failed sync messages
//!
//! Handles automatic retry of failed message sends with exponential backoff.

use crate::service::network::protocol::sync::messages::SyncMessage;
use chrono::{DateTime, Duration, Utc};
use std::collections::VecDeque;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Maximum number of retry attempts before giving up
///
/// After this many failed attempts, messages are dropped to prevent
/// unbounded queue growth for persistently unreachable devices.
const MAX_RETRIES: u32 = 5;

/// Initial retry delay in seconds
///
/// Uses exponential backoff: 5s, 10s, 20s, 40s, 80s
const INITIAL_DELAY_SECS: i64 = 5;

/// Entry in the retry queue
#[derive(Debug, Clone)]
struct RetryEntry {
	/// Target device to send to
	target_device: Uuid,

	/// Message to send
	message: SyncMessage,

	/// Number of attempts made
	attempts: u32,

	/// Next retry time
	next_retry: DateTime<Utc>,
}

/// A due message taken out of the queue by [`RetryQueue::get_ready`].
#[derive(Debug, Clone)]
pub struct RetryItem {
	/// Device the message is for
	pub target_device: Uuid,

	/// Message to send
	pub message: SyncMessage,

	/// Failed attempts so far
	attempts: u32,
}

/// Retry queue for failed message sends
pub struct RetryQueue {
	queue: RwLock<VecDeque<RetryEntry>>,
}

impl RetryQueue {
	/// Create a new empty retry queue
	pub fn new() -> Self {
		Self {
			queue: RwLock::new(VecDeque::new()),
		}
	}

	/// Enqueue a message for retry
	pub async fn enqueue(&self, target_device: Uuid, message: SyncMessage) {
		let entry = RetryEntry {
			target_device,
			message,
			attempts: 0,
			next_retry: Utc::now() + Duration::seconds(INITIAL_DELAY_SECS),
		};

		self.queue.write().await.push_back(entry);
	}

	/// Takes the messages that are due out of the queue.
	///
	/// The caller sends them and hands each failure back with [`RetryQueue::requeue`]. Taking a
	/// message out only on success would keep a delivered message in the queue: it used to be
	/// re-sent with backoff until it hit the retry limit.
	pub async fn get_ready(&self) -> Vec<RetryItem> {
		let now = Utc::now();
		let mut queue = self.queue.write().await;
		let mut ready = Vec::new();
		let mut i = 0;

		while i < queue.len() {
			if queue[i].next_retry <= now {
				let entry = queue.remove(i).expect("index checked against len");
				ready.push(RetryItem {
					target_device: entry.target_device,
					message: entry.message,
					attempts: entry.attempts,
				});
			} else {
				i += 1;
			}
		}

		ready
	}

	/// Puts back a message whose send failed, with exponential backoff, or drops it once it
	/// has used up its attempts. Catch-up delivers what a dropped message carried.
	pub async fn requeue(&self, item: RetryItem) {
		let attempts = item.attempts + 1;
		if attempts >= MAX_RETRIES {
			tracing::warn!(
				target_device = %item.target_device,
				attempts,
				"Max retries reached, dropping message"
			);
			return;
		}

		let delay_secs = INITIAL_DELAY_SECS * (2_i64.pow(attempts));
		self.queue.write().await.push_back(RetryEntry {
			target_device: item.target_device,
			message: item.message,
			attempts,
			next_retry: Utc::now() + Duration::seconds(delay_secs),
		});
	}

	/// Get current queue size
	pub async fn len(&self) -> usize {
		self.queue.read().await.len()
	}

	/// Check if queue is empty
	pub async fn is_empty(&self) -> bool {
		self.queue.read().await.is_empty()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn error_message() -> SyncMessage {
		SyncMessage::Error {
			library_id: Uuid::new_v4(),
			message: "test".to_string(),
		}
	}

	#[tokio::test]
	async fn enqueue_is_not_ready_until_backoff_elapses() {
		let queue = RetryQueue::new();
		let device_id = Uuid::new_v4();

		queue.enqueue(device_id, error_message()).await;
		assert_eq!(queue.len().await, 1);
		assert_eq!(queue.get_ready().await.len(), 0);
		assert_eq!(queue.len().await, 1);
	}

	#[tokio::test]
	async fn ready_entry_is_taken_out_and_requeue_backs_off() {
		let queue = RetryQueue::new();
		let device_id = Uuid::new_v4();
		let library_id = Uuid::new_v4();
		queue
			.enqueue(
				device_id,
				SyncMessage::Error {
					library_id,
					message: "retry me".to_string(),
				},
			)
			.await;

		{
			let mut q = queue.queue.write().await;
			q[0].next_retry = Utc::now() - Duration::seconds(1);
		}

		let mut ready = queue.get_ready().await;
		assert_eq!(ready.len(), 1);
		assert_eq!(ready[0].target_device, device_id);
		match &ready[0].message {
			SyncMessage::Error {
				library_id: id,
				message,
			} => {
				assert_eq!(*id, library_id);
				assert_eq!(message, "retry me");
			}
			other => panic!("unexpected message: {other:?}"),
		}
		// A delivered message is gone: nothing re-sends it.
		assert!(queue.is_empty().await);

		queue.requeue(ready.remove(0)).await;
		let q = queue.queue.read().await;
		assert_eq!(q.len(), 1);
		assert_eq!(q[0].attempts, 1);
		let delay = q[0].next_retry - Utc::now();
		assert!(delay.num_seconds() >= 8, "delay too small: {delay:?}");
		assert!(delay.num_seconds() <= 11, "delay too large: {delay:?}");
	}

	#[tokio::test]
	async fn max_retries_drops_message() {
		let queue = RetryQueue::new();
		let device_id = Uuid::new_v4();
		queue.enqueue(device_id, error_message()).await;

		for expected in 1..MAX_RETRIES {
			{
				let mut q = queue.queue.write().await;
				q[0].next_retry = Utc::now() - Duration::seconds(1);
			}
			let mut ready = queue.get_ready().await;
			assert_eq!(ready.len(), 1, "attempt {expected} should be ready");
			queue.requeue(ready.remove(0)).await;
			assert_eq!(queue.len().await, 1);
		}

		{
			let mut q = queue.queue.write().await;
			q[0].next_retry = Utc::now() - Duration::seconds(1);
		}
		let mut ready = queue.get_ready().await;
		assert_eq!(ready.len(), 1);
		queue.requeue(ready.remove(0)).await;
		assert!(
			queue.is_empty().await,
			"message should be dropped at max retries"
		);
	}

	#[tokio::test]
	async fn ready_returns_only_due_entries_across_devices() {
		let queue = RetryQueue::new();
		let due_device = Uuid::new_v4();
		let later_device = Uuid::new_v4();
		queue.enqueue(due_device, error_message()).await;
		queue.enqueue(later_device, error_message()).await;

		{
			let mut q = queue.queue.write().await;
			q[0].next_retry = Utc::now() - Duration::seconds(1);
			q[1].next_retry = Utc::now() + Duration::hours(1);
		}

		let ready = queue.get_ready().await;
		assert_eq!(ready.len(), 1);
		assert_eq!(ready[0].target_device, due_device);
		assert_eq!(queue.len().await, 1);
	}
}
