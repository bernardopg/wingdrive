use super::*;
use crate::service::network::{device::registry::tests::test_registry, utils::SilentLogger};

async fn fixture() -> (PairingProtocolHandler, tempfile::TempDir) {
	let (registry, dir) = test_registry();
	let identity = NetworkIdentity::from_device_key(&[7; 32]).await.unwrap();
	let (commands, _) = tokio::sync::mpsc::unbounded_channel();
	let handler = PairingProtocolHandler::new(
		identity,
		Arc::new(RwLock::new(registry)),
		Arc::new(SilentLogger),
		commands,
		None,
		Arc::default(),
	);
	(handler, dir)
}

#[tokio::test]
async fn the_latest_pairing_code_follows_session_order_and_cancellation() {
	let (handler, _dir) = fixture().await;
	assert!(handler.get_current_pairing_code().await.is_none());

	let mut session_ids = Vec::new();
	for _ in 0..24 {
		let id = handler.start_pairing_session().await.unwrap();
		session_ids.push(id);
		assert_eq!(
			handler
				.get_current_pairing_code()
				.await
				.unwrap()
				.session_id(),
			id
		);
	}

	handler.cancel_session(session_ids.remove(8)).await.unwrap();

	while let Some(id) = session_ids.pop() {
		handler.cancel_session(id).await.unwrap();
		assert_eq!(
			handler
				.get_current_pairing_code()
				.await
				.map(|code| code.session_id()),
			session_ids.last().copied()
		);
	}
}

#[tokio::test]
async fn an_expired_session_and_duplicate_join_are_handled_without_networking() {
	let (handler, _dir) = fixture().await;
	let first = handler.start_pairing_session().await.unwrap();
	let joined = PairingCode::generate().unwrap();
	let joined_id = joined.session_id();

	handler
		.join_pairing_session(joined_id, joined.clone())
		.await
		.unwrap();
	assert!(handler
		.join_pairing_session(joined_id, joined)
		.await
		.is_err());
	assert_eq!(
		handler
			.get_current_pairing_code()
			.await
			.unwrap()
			.session_id(),
		joined_id
	);

	handler
		.active_sessions
		.write()
		.await
		.get_mut(&joined_id)
		.unwrap()
		.created_at = chrono::Utc::now() - chrono::Duration::minutes(11);
	assert_eq!(handler.cleanup_expired_sessions().await.unwrap(), 1);
	assert_eq!(handler.cleanup_expired_sessions().await.unwrap(), 0);
	assert_eq!(
		handler
			.get_current_pairing_code()
			.await
			.unwrap()
			.session_id(),
		first
	);
}
