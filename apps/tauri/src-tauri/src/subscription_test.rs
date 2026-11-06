use super::{DaemonConnectionPool, SubscriptionManager};
use serde_json::json;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::oneshot;

#[tokio::test]
async fn cancelling_a_window_only_cancels_its_own_subscriptions() {
	let manager = SubscriptionManager::new();
	let first = manager.next_id();
	let second = manager.next_id();
	let other = manager.next_id();
	assert_ne!(first, second);
	assert_ne!(second, other);
	let (first_tx, first_rx) = oneshot::channel();
	let (second_tx, second_rx) = oneshot::channel();
	let (other_tx, mut other_rx) = oneshot::channel();
	manager.register(first, "one".into(), first_tx).await;
	manager.register(second, "one".into(), second_tx).await;
	manager.register(other, "two".into(), other_tx).await;
	assert_eq!(manager.get_active_count().await, 3);
	assert_eq!(manager.cancel_window("one").await, 2);
	assert_eq!(first_rx.await, Ok(()));
	assert_eq!(second_rx.await, Ok(()));
	assert!(other_rx.try_recv().is_err());
	assert_eq!(manager.get_active_count().await, 1);
	assert!(!manager.cancel(first).await);
	assert!(manager.cancel(other).await);
	assert_eq!(other_rx.await, Ok(()));
	assert_eq!(manager.cancel_window("two").await, 0);
}

#[tokio::test]
async fn connection_pool_writes_subscription_requests_and_resets_state() {
	let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
	let address = listener.local_addr().unwrap();
	let client = tokio::net::TcpStream::connect(address).await.unwrap();
	let (server, _) = listener.accept().await.unwrap();
	let (_, writer) = client.into_split();
	let pool = DaemonConnectionPool::new(address.to_string());
	*pool.writer.lock().await = Some(writer);
	let mut received = BufReader::new(server);
	let mut line = String::new();
	let id = pool.next_id();
	assert_ne!(id, pool.next_id());
	pool.subscribe(
		id,
		vec!["ResourceChanged".into()],
		Some(json!({"library": "test"})),
	)
	.await
	.unwrap();
	received.read_line(&mut line).await.unwrap();
	let request: serde_json::Value = serde_json::from_str(&line).unwrap();
	assert_eq!(
		request["Subscribe"]["event_types"],
		json!(["ResourceChanged"])
	);
	assert_eq!(request["Subscribe"]["filter"], json!({"library": "test"}));
	line.clear();
	pool.unsubscribe(id).await.unwrap();
	received.read_line(&mut line).await.unwrap();
	assert_eq!(
		serde_json::from_str::<serde_json::Value>(&line).unwrap(),
		json!("Unsubscribe")
	);
	assert!(pool.unsubscribe(id).await.is_err());
	pool.reset().await;
	assert!(pool.writer.lock().await.is_none());
	assert!(pool.subscriptions.read().await.is_empty());
	assert!(pool.subscribe(99, vec![], None).await.is_err());
}
