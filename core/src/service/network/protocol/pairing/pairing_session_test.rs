use super::*;
use crate::service::network::{device::registry::tests::test_registry, utils::SilentLogger};

async fn fixture(identity_byte: u8) -> (PairingProtocolHandler, tempfile::TempDir) {
	let (registry, dir) = test_registry();
	let identity = NetworkIdentity::from_device_key(&[identity_byte; 32])
		.await
		.unwrap();
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
	let (handler, _dir) = fixture(7).await;
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
	let (handler, _dir) = fixture(7).await;
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

#[tokio::test]
async fn signed_pairing_registers_both_peers_and_persists_matching_keys() {
	let (initiator, _initiator_dir) = fixture(11).await;
	let (joiner, _joiner_dir) = fixture(23).await;
	let initiator_info = initiator.get_device_info().await.unwrap();
	let joiner_info = joiner.get_device_info().await.unwrap();
	assert_ne!(initiator_info.device_id, joiner_info.device_id);

	let id = initiator.start_pairing_session().await.unwrap();
	let code = initiator.get_current_pairing_code().await.unwrap();
	assert_eq!(id, code.session_id());
	joiner.join_pairing_session(id, code).await.unwrap();
	let request = PairingMessage::PairingRequest {
		session_id: id,
		device_info: joiner_info.clone(),
		public_key: joiner.identity.public_key_bytes(),
	};

	let response = initiator
		.handle_request(joiner_info.device_id, serde_json::to_vec(&request).unwrap())
		.await
		.unwrap();
	let PairingMessage::Challenge {
		session_id,
		challenge,
		device_info,
	} = serde_json::from_slice(&response).unwrap()
	else {
		panic!("expected the initiator's challenge")
	};
	assert_eq!(session_id, id);
	assert_eq!(device_info.device_id, initiator_info.device_id);

	let response = joiner
		.handle_pairing_challenge(session_id, challenge, device_info)
		.await
		.unwrap();
	let complete = initiator
		.handle_request(joiner_info.device_id, response)
		.await
		.unwrap();
	let PairingMessage::Complete {
		session_id,
		success,
		reason,
	} = serde_json::from_slice(&complete).unwrap()
	else {
		panic!("expected the signed challenge to complete the pairing")
	};
	assert_eq!(session_id, id);
	assert!(success);
	assert!(reason.is_none());

	joiner
		.handle_completion(
			session_id,
			success,
			reason,
			initiator_info.device_id,
			initiator_info.network_fingerprint.node_id.parse().unwrap(),
		)
		.await
		.unwrap();

	let initiator_registry = initiator.device_registry.read().await;
	let joiner_registry = joiner.device_registry.read().await;
	let initiator_credentials = initiator_registry
		.get_persisted_device(joiner_info.device_id)
		.await
		.unwrap();
	let joiner_credentials = joiner_registry
		.get_persisted_device(initiator_info.device_id)
		.await
		.unwrap();
	assert!(initiator_credentials.is_some() && joiner_credentials.is_some());
	let initiator_keys = initiator_registry
		.get_session_keys(joiner_info.device_id)
		.unwrap();
	let joiner_keys = joiner_registry
		.get_session_keys(initiator_info.device_id)
		.unwrap();
	assert!(initiator_keys.send_key == joiner_keys.receive_key);
	assert!(initiator_keys.receive_key == joiner_keys.send_key);

	for handler in [&initiator, &joiner] {
		let sessions = handler.get_active_sessions().await;
		assert_eq!(sessions.len(), 1);
		assert!(matches!(sessions[0].state, PairingState::Completed));
	}
}
