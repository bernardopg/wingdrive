use super::super::*;
use crate::service::network::{protocol::ProtocolHandler, utils::SilentLogger};
use tokio::io::{AsyncReadExt, AsyncWriteExt, DuplexStream};

async fn send(stream: &mut DuplexStream, message: &FileTransferMessage) {
	let bytes = rmp_serde::to_vec(message).unwrap();
	stream.write_u8(0).await.unwrap();
	stream
		.write_u32(u32::try_from(bytes.len()).unwrap())
		.await
		.unwrap();
	stream.write_all(&bytes).await.unwrap();
}

async fn receive(stream: &mut DuplexStream) -> FileTransferMessage {
	assert_eq!(stream.read_u8().await.unwrap(), 0);
	let size = stream.read_u32().await.unwrap();
	assert!(size < 64 * 1024);
	let mut bytes = vec![0u8; size as usize];
	stream.read_exact(&mut bytes).await.unwrap();
	rmp_serde::from_slice(&bytes).unwrap()
}

#[tokio::test]
async fn push_stream_writes_the_file_and_returns_the_final_acknowledgment() {
	let root = tempfile::tempdir().unwrap();
	let allowed = root.path().canonicalize().unwrap();
	let destination = allowed.join("received.bin");
	let handler = FileTransferProtocolHandler::new_default(Arc::new(SilentLogger));
	handler.set_allowed_paths(vec![allowed]);
	let (mut client, server) = tokio::io::duplex(8192);
	let (server_read, server_write) = tokio::io::split(server);
	let node = iroh::SecretKey::generate().public();
	let stream = handler.handle_stream(Box::new(server_write), Box::new(server_read), node);
	let id = Uuid::new_v4();
	let data = b"isolated file stream fixture";
	let hash = blake3::hash(data);

	let exchange = async {
		send(
			&mut client,
			&FileTransferMessage::TransferRequest {
				transfer_id: id,
				file_metadata: FileMetadata {
					name: "received.bin".into(),
					size: data.len() as u64,
					modified: None,
					is_directory: false,
					checksum: Some(hash.to_hex().to_string()),
					mime_type: None,
				},
				transfer_mode: TransferMode::TrustedCopy,
				chunk_size: 64 * 1024,
				total_chunks: 1,
				destination_path: destination.to_string_lossy().into_owned(),
			},
		)
		.await;
		send(
			&mut client,
			&FileTransferMessage::FileChunk {
				transfer_id: id,
				chunk_index: 0,
				data: data.to_vec(),
				nonce: [0; 12],
				chunk_checksum: *hash.as_bytes(),
			},
		)
		.await;
		send(
			&mut client,
			&FileTransferMessage::TransferComplete {
				transfer_id: id,
				final_checksum: hash.to_hex().to_string(),
				total_bytes: data.len() as u64,
			},
		)
		.await;

		assert!(matches!(
			receive(&mut client).await,
			FileTransferMessage::TransferFinalAck {transfer_id} if transfer_id == id
		));
		client.shutdown().await.unwrap();
	};
	tokio::time::timeout(std::time::Duration::from_secs(5), async {
		tokio::join!(stream, exchange)
	})
	.await
	.unwrap();
	assert_eq!(tokio::fs::read(destination).await.unwrap(), data);
	assert_eq!(
		handler.get_session(&id).unwrap().state,
		TransferState::Completed
	);
}
