//! Basic messaging protocol handler

use super::{library_messages::LibraryMessage, ProtocolEvent, ProtocolHandler};
use crate::service::network::{utils, NetworkingError, Result};
use async_trait::async_trait;
use iroh::{endpoint::Connection, Endpoint, EndpointAddr, EndpointId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Basic messaging protocol handler
pub struct MessagingProtocolHandler {
	/// Optional context for accessing libraries
	context: Option<Arc<crate::context::CoreContext>>,

	/// Device registry for node_id → device_id mapping
	device_registry: Arc<RwLock<crate::service::network::device::DeviceRegistry>>,

	/// Endpoint for creating and managing connections
	endpoint: Option<Endpoint>,

	/// Cached connections to remote nodes (keyed by EndpointId and ALPN)
	connections: Arc<RwLock<HashMap<(EndpointId, Vec<u8>), Connection>>>,
}

/// Basic message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
	/// Ping message for connection testing
	Ping {
		timestamp: chrono::DateTime<chrono::Utc>,
		payload: Option<Vec<u8>>,
	},
	/// Pong response
	Pong {
		timestamp: chrono::DateTime<chrono::Utc>,
		original_timestamp: chrono::DateTime<chrono::Utc>,
	},
	/// Generic data message
	Data {
		message_id: Uuid,
		content_type: String,
		payload: Vec<u8>,
	},
	/// Acknowledgment message
	Ack {
		message_id: Uuid,
		success: bool,
		error: Option<String>,
	},
	/// Graceful disconnect notification
	Goodbye {
		reason: String,
		timestamp: chrono::DateTime<chrono::Utc>,
	},
	/// Library-related message
	Library(LibraryMessage),
	/// Wingdrop offer, decision or progress
	Wingdrop(crate::service::wingdrop::WingdropMessage),
}

impl MessagingProtocolHandler {
	/// Create a new messaging protocol handler with shared connection cache
	///
	/// Uses the same connection cache as other protocols (pairing, sync, file transfer)
	/// to follow Iroh best practice of one persistent connection per device pair
	pub fn new(
		device_registry: Arc<RwLock<crate::service::network::device::DeviceRegistry>>,
		endpoint: Option<Endpoint>,
		active_connections: Arc<RwLock<HashMap<(EndpointId, Vec<u8>), Connection>>>,
	) -> Self {
		Self {
			context: None,
			device_registry,
			endpoint,
			connections: active_connections,
		}
	}

	/// Set context after creation
	pub fn set_context(&mut self, context: Arc<crate::context::CoreContext>) {
		self.context = Some(context);
	}

	async fn handle_ping(
		&self,
		_from_device: Uuid,
		timestamp: chrono::DateTime<chrono::Utc>,
		_payload: Option<Vec<u8>>,
	) -> Result<Vec<u8>> {
		let response = Message::Pong {
			timestamp: chrono::Utc::now(),
			original_timestamp: timestamp,
		};

		serde_json::to_vec(&response).map_err(|e| NetworkingError::Serialization(e))
	}

	async fn handle_pong(
		&self,
		_from_device: Uuid,
		_timestamp: chrono::DateTime<chrono::Utc>,
		original_timestamp: chrono::DateTime<chrono::Utc>,
	) -> Result<Vec<u8>> {
		let now = chrono::Utc::now();
		let rtt = now.signed_duration_since(original_timestamp);

		println!("Ping RTT: {}ms", rtt.num_milliseconds());

		// Return empty response for pong
		Ok(Vec::new())
	}

	async fn handle_data(
		&self,
		from_device: Uuid,
		message_id: Uuid,
		content_type: String,
		payload: Vec<u8>,
	) -> Result<Vec<u8>> {
		// Process the data message
		println!(
			"Received data message from {}: {} ({} bytes)",
			from_device,
			content_type,
			payload.len()
		);

		// Send acknowledgment
		let response = Message::Ack {
			message_id,
			success: true,
			error: None,
		};

		serde_json::to_vec(&response).map_err(|e| NetworkingError::Serialization(e))
	}

	async fn handle_ack(
		&self,
		_from_device: Uuid,
		message_id: Uuid,
		success: bool,
		error: Option<String>,
	) -> Result<Vec<u8>> {
		if success {
			println!("Message {} acknowledged successfully", message_id);
		} else {
			println!("Message {} failed: {:?}", message_id, error);
		}

		// Return empty response for ack
		Ok(Vec::new())
	}

	async fn handle_library_message(
		&self,
		_from_device: Uuid,
		library_msg: LibraryMessage,
	) -> Result<Vec<u8>> {
		use super::library_messages::{LibraryDiscoveryInfo, LibraryMessage};

		match library_msg {
			LibraryMessage::DiscoveryRequest { request_id } => {
				// Get context to access libraries
				let context = self.context.as_ref().ok_or_else(|| {
					NetworkingError::Protocol(
						"Context not available for library operations".to_string(),
					)
				})?;

				let library_manager = context.libraries().await;
				let libraries = library_manager.list().await;

				// Convert to discovery info
				let mut library_infos = Vec::new();
				for library in libraries {
					let library_id = library.id();
					let name = library.name().await;
					let config_guard = library.config().await;

					// Get library statistics from database
					let db = library.db();
					use crate::infra::db::entities;
					use sea_orm::{EntityTrait, PaginatorTrait};

					let entry_count = match entities::entry::Entity::find().count(db.conn()).await {
						Ok(count) => count,
						Err(_) => 0,
					};

					let location_count =
						match entities::location::Entity::find().count(db.conn()).await {
							Ok(count) => count,
							Err(_) => 0,
						};

					let device_count = match entities::device::Entity::find().count(db.conn()).await
					{
						Ok(count) => count,
						Err(_) => 0,
					};

					library_infos.push(LibraryDiscoveryInfo {
						id: library_id,
						name,
						description: config_guard.description.clone(),
						created_at: config_guard.created_at,
						total_entries: entry_count,
						total_locations: location_count,
						total_size_bytes: 0, // TODO: Calculate from entries
						device_count,
					});
				}

				let response = Message::Library(LibraryMessage::DiscoveryResponse {
					request_id,
					libraries: library_infos,
				});

				serde_json::to_vec(&response).map_err(|e| NetworkingError::Serialization(e))
			}

			LibraryMessage::RegisterDeviceRequest {
				request_id,
				library_id,
				device_id,
				device_name,
				device_slug,
				os_name,
				os_version,
				hardware_model,
				cpu_model,
				cpu_architecture,
				cpu_cores_physical,
				cpu_cores_logical,
				cpu_frequency_mhz,
				memory_total_bytes,
				form_factor,
				manufacturer,
				gpu_models,
				boot_disk_type,
				boot_disk_capacity_bytes,
				swap_total_bytes,
			} => {
				// Get context
				let context = self.context.as_ref().ok_or_else(|| {
					NetworkingError::Protocol(
						"Context not available for library operations".to_string(),
					)
				})?;

				let library_manager = context.libraries().await;

				// Determine which library to register device in
				let libraries = if let Some(lib_id) = library_id {
					// Specific library
					vec![library_manager.get_library(lib_id).await.ok_or_else(|| {
						NetworkingError::Protocol(format!("Library not found: {}", lib_id))
					})?]
				} else {
					// All libraries
					library_manager.list().await
				};

				// Register device in each library
				use crate::infra::db::entities;
				use chrono::Utc;
				use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};

				let mut success = true;
				let mut error_msg = None;

				for library in libraries {
					let db = library.db();

					// Check if device already exists
					let existing = entities::device::Entity::find()
						.filter(entities::device::Column::Uuid.eq(device_id))
						.one(db.conn())
						.await;

					match existing {
						Ok(Some(existing_device)) => {
							// Device exists (from pre-registration) - update with full hardware
							let mut device_model: entities::device::ActiveModel =
								existing_device.into();

							// Update all fields with data from message
							device_model.name = Set(device_name.clone());
							device_model.slug = Set(device_slug.clone());
							device_model.os = Set(os_name.clone());
							device_model.os_version = Set(os_version.clone());
							device_model.hardware_model = Set(hardware_model.clone());
							device_model.cpu_model = Set(cpu_model.clone());
							device_model.cpu_architecture = Set(cpu_architecture.clone());
							device_model.cpu_cores_physical = Set(cpu_cores_physical);
							device_model.cpu_cores_logical = Set(cpu_cores_logical);
							device_model.cpu_frequency_mhz = Set(cpu_frequency_mhz);
							device_model.memory_total_bytes = Set(memory_total_bytes);
							device_model.form_factor = Set(form_factor.clone());
							device_model.manufacturer = Set(manufacturer.clone());
							device_model.gpu_models =
								Set(gpu_models.clone().map(|g| serde_json::json!(g)));
							device_model.boot_disk_type = Set(boot_disk_type.clone());
							device_model.boot_disk_capacity_bytes = Set(boot_disk_capacity_bytes);
							device_model.swap_total_bytes = Set(swap_total_bytes);
							device_model.is_online = Set(false);
							device_model.last_seen_at = Set(Utc::now());
							device_model.updated_at = Set(Utc::now());

							if let Err(e) = device_model.update(db.conn()).await {
								success = false;
								error_msg = Some(format!("Failed to update device: {}", e));
								break;
							}

							tracing::info!(
								"Updated existing device {} in library {} with full hardware specs",
								device_id,
								library.id()
							);
						}
						Ok(None) => {
							// Get existing slugs for collision detection
							let existing_slugs: Vec<String> =
								match entities::device::Entity::find().all(db.conn()).await {
									Ok(devices) => devices.iter().map(|d| d.slug.clone()).collect(),
									Err(e) => {
										success = false;
										error_msg = Some(format!("Database error: {}", e));
										break;
									}
								};

							// Check if the device's slug conflicts and rename if needed
							let unique_slug = crate::library::Library::ensure_unique_slug(
								&device_slug,
								&existing_slugs,
							);

							if unique_slug != device_slug {
								tracing::info!(
									"Device slug collision in library {}. Registering device as '{}' instead of '{}'",
									library.id(),
									unique_slug,
									device_slug
								);
							}

							// Register remote device with full hardware specs
							let device_model = entities::device::ActiveModel {
								id: sea_orm::ActiveValue::NotSet,
								uuid: Set(device_id),
								name: Set(device_name.clone()),
								slug: Set(unique_slug),
								os: Set(os_name.clone()),
								os_version: Set(os_version.clone()),
								hardware_model: Set(hardware_model.clone()),
								cpu_model: Set(cpu_model.clone()),
								cpu_architecture: Set(cpu_architecture.clone()),
								cpu_cores_physical: Set(cpu_cores_physical),
								cpu_cores_logical: Set(cpu_cores_logical),
								cpu_frequency_mhz: Set(cpu_frequency_mhz),
								memory_total_bytes: Set(memory_total_bytes),
								form_factor: Set(form_factor.clone()),
								manufacturer: Set(manufacturer.clone()),
								gpu_models: Set(gpu_models.clone().map(|g| serde_json::json!(g))),
								boot_disk_type: Set(boot_disk_type.clone()),
								boot_disk_capacity_bytes: Set(boot_disk_capacity_bytes),
								swap_total_bytes: Set(swap_total_bytes),
								network_addresses: Set(serde_json::json!([])),
								is_online: Set(false),
								last_seen_at: Set(Utc::now()),
								capabilities: Set(serde_json::json!({
									"indexing": true,
									"p2p": true,
									"volume_detection": true
								})),
								created_at: Set(Utc::now()),
								sync_enabled: Set(true),
								updated_at: Set(Utc::now()),
							};

							if let Err(e) = device_model.insert(db.conn()).await {
								success = false;
								error_msg = Some(format!("Failed to register device: {}", e));
								break;
							}

							tracing::info!(
								"Registered device {} in library {} with full hardware specs",
								device_id,
								library.id()
							);

							// Send RegisterDeviceRequest back to register ourselves with the new device
							let context_clone = context.clone();
							let sender_device_id = device_id;
							tokio::spawn(async move {
								// Get our device info
								if let Ok(our_device) = context_clone.device_manager.to_device() {
									// Get our slug for this library
									if let Some(lib_id) = library_id {
										if let Ok(our_slug) =
											context_clone.device_manager.slug_for_library(lib_id)
										{
											// Get networking
											if let Some(networking) =
												context_clone.get_networking().await
											{
												let our_register_request =
													LibraryMessage::RegisterDeviceRequest {
														request_id: Uuid::new_v4(),
														library_id,
														device_id: our_device.id,
														device_name: our_device.name,
														device_slug: our_slug,
														os_name: our_device.os.to_string(),
														os_version: our_device.os_version,
														hardware_model: our_device.hardware_model,
														cpu_model: our_device.cpu_model,
														cpu_architecture: our_device
															.cpu_architecture,
														cpu_cores_physical: our_device
															.cpu_cores_physical,
														cpu_cores_logical: our_device
															.cpu_cores_logical,
														cpu_frequency_mhz: our_device
															.cpu_frequency_mhz,
														memory_total_bytes: our_device
															.memory_total_bytes,
														form_factor: our_device
															.form_factor
															.map(|f| f.to_string()),
														manufacturer: our_device.manufacturer,
														gpu_models: our_device.gpu_models,
														boot_disk_type: our_device.boot_disk_type,
														boot_disk_capacity_bytes: our_device
															.boot_disk_capacity_bytes,
														swap_total_bytes: our_device
															.swap_total_bytes,
													};

												// Send to the device that just registered with us
												if let Err(e) = networking
													.send_library_request(
														sender_device_id,
														our_register_request,
													)
													.await
												{
													tracing::warn!(
														"Failed to send bidirectional RegisterDeviceRequest: {}",
														e
													);
												} else {
													tracing::info!(
														"Sent RegisterDeviceRequest back to {} for bidirectional registration",
														sender_device_id
													);
												}
											}
										}
									}
								}
							});
						}
						Err(e) => {
							success = false;
							error_msg = Some(format!("Database error: {}", e));
							break;
						}
					}
				}

				// Send response confirming registration
				let response = Message::Library(LibraryMessage::RegisterDeviceResponse {
					request_id,
					success,
					message: error_msg.clone(),
				});

				serde_json::to_vec(&response).map_err(|e| NetworkingError::Serialization(e))
			}

			LibraryMessage::DiscoveryResponse { .. }
			| LibraryMessage::RegisterDeviceResponse { .. } => {
				// These are responses, not requests
				Ok(Vec::new())
			}

			LibraryMessage::CreateSharedLibraryRequest {
				request_id,
				library_id,
				library_name,
				description,
				requesting_device_id,
				requesting_device_name,
				requesting_device_slug,
				requesting_device_os,
				requesting_device_os_version,
				requesting_device_hardware_model,
				requesting_device_cpu_model,
				requesting_device_cpu_architecture,
				requesting_device_cpu_cores_physical,
				requesting_device_cpu_cores_logical,
				requesting_device_cpu_frequency_mhz,
				requesting_device_memory_total_bytes,
				requesting_device_form_factor,
				requesting_device_manufacturer,
				requesting_device_gpu_models,
				requesting_device_boot_disk_type,
				requesting_device_boot_disk_capacity_bytes,
				requesting_device_swap_total_bytes,
			} => {
				tracing::info!(
					"Received CreateSharedLibraryRequest: {} ({}) from device {} (slug: {})",
					library_name,
					library_id,
					requesting_device_id,
					requesting_device_slug
				);

				let context = self.context.as_ref().ok_or_else(|| {
					NetworkingError::Protocol("Context not available".to_string())
				})?;

				let library_manager = context.libraries().await;

				// Check if library already exists
				if let Some(existing_library) = library_manager.get_library(library_id).await {
					tracing::info!("Library {} already exists, returning success", library_id);

					// Get this device's slug in the library
					let device_slug = context.device_manager.slug_for_library(library_id).ok();

					let response = Message::Library(LibraryMessage::CreateSharedLibraryResponse {
						request_id,
						success: true,
						message: Some("Library already exists".to_string()),
						device_slug,
					});
					return serde_json::to_vec(&response)
						.map_err(|e| NetworkingError::Serialization(e));
				}

				// Create library with specific UUID
				// Pre-register the requesting device with full hardware specs
				match library_manager
					.create_library_with_id_and_initial_device(
						library_id,
						library_name.clone(),
						description,
						requesting_device_id,
						requesting_device_name,
						requesting_device_slug,
						requesting_device_os,
						requesting_device_os_version,
						requesting_device_hardware_model,
						requesting_device_cpu_model,
						requesting_device_cpu_architecture,
						requesting_device_cpu_cores_physical,
						requesting_device_cpu_cores_logical,
						requesting_device_cpu_frequency_mhz,
						requesting_device_memory_total_bytes,
						requesting_device_form_factor,
						requesting_device_manufacturer,
						requesting_device_gpu_models,
						requesting_device_boot_disk_type,
						requesting_device_boot_disk_capacity_bytes,
						requesting_device_swap_total_bytes,
						context.clone(),
					)
					.await
				{
					Ok(_) => {
						tracing::info!("Successfully created shared library: {}", library_name);

						// Get this device's resolved slug in the new library
						// After ensure_device_registered, this will return the collision-resolved slug
						let device_slug = context.device_manager.slug_for_library(library_id).ok();

						let response =
							Message::Library(LibraryMessage::CreateSharedLibraryResponse {
								request_id,
								success: true,
								message: None,
								device_slug,
							});
						serde_json::to_vec(&response).map_err(|e| NetworkingError::Serialization(e))
					}
					Err(e) => {
						tracing::error!("Failed to create library: {}", e);
						let response =
							Message::Library(LibraryMessage::CreateSharedLibraryResponse {
								request_id,
								success: false,
								message: Some(e.to_string()),
								device_slug: None,
							});
						serde_json::to_vec(&response).map_err(|e| NetworkingError::Serialization(e))
					}
				}
			}

			LibraryMessage::CreateSharedLibraryResponse { .. } => {
				// This is a response, not a request
				Ok(Vec::new())
			}

			LibraryMessage::LibraryStateRequest {
				request_id,
				library_id,
			} => {
				tracing::info!("Received LibraryStateRequest for library {}", library_id);

				let context = self.context.as_ref().ok_or_else(|| {
					NetworkingError::Protocol("Context not available".to_string())
				})?;

				let library_manager = context.libraries().await;
				let library = library_manager
					.get_library(library_id)
					.await
					.ok_or_else(|| {
						NetworkingError::Protocol(format!("Library {} not found", library_id))
					})?;

				let db = library.db();

				// Query all device slugs from this library
				use crate::infra::db::entities;
				use sea_orm::EntityTrait;

				let devices = entities::device::Entity::find()
					.all(db.conn())
					.await
					.map_err(|e| NetworkingError::Protocol(format!("Database error: {}", e)))?;

				let device_slugs: Vec<String> = devices.iter().map(|d| d.slug.clone()).collect();

				tracing::info!(
					"Returning {} device slugs for library {}",
					device_slugs.len(),
					library_id
				);

				let response = Message::Library(LibraryMessage::LibraryStateResponse {
					request_id,
					library_id,
					library_name: library.name().await,
					device_slugs,
					device_count: devices.len(),
				});

				serde_json::to_vec(&response).map_err(|e| NetworkingError::Serialization(e))
			}

			LibraryMessage::LibraryStateResponse { .. } => {
				// This is a response, not a request
				Ok(Vec::new())
			}
		}
	}

	/// Send a library message to a remote node and wait for response
	/// Uses cached connections and creates new streams (Iroh best practice)
	pub async fn send_library_message(
		&self,
		node_id: EndpointId,
		message: LibraryMessage,
	) -> Result<LibraryMessage> {
		tracing::info!("Sending library message to node {}: {:?}", node_id, message);
		match self
			.send_request(node_id, Message::Library(message))
			.await?
		{
			Message::Library(lib_msg) => {
				tracing::debug!("Received library message response: {:?}", lib_msg);
				Ok(lib_msg)
			}
			_ => Err(NetworkingError::Protocol(
				"Expected Library message in response".to_string(),
			)),
		}
	}

	/// Sends a Wingdrop message to a node and returns its reply.
	pub async fn send_wingdrop_message(
		&self,
		node_id: EndpointId,
		message: crate::service::wingdrop::WingdropMessage,
	) -> Result<crate::service::wingdrop::WingdropMessage> {
		match self
			.send_request(node_id, Message::Wingdrop(message))
			.await?
		{
			Message::Wingdrop(reply) => Ok(reply),
			_ => Err(NetworkingError::Protocol(
				"Expected Wingdrop message in response".to_string(),
			)),
		}
	}

	/// Sends one message and waits for the reply. Either side may close an
	/// idle or duplicate connection at any moment (reconnection keeps only
	/// one), so a request that dies on a closed connection before any reply
	/// is sent once more on a fresh one. Handlers treat repeats as no-ops.
	async fn send_request(&self, node_id: EndpointId, envelope: Message) -> Result<Message> {
		match self.send_request_once(node_id, &envelope).await {
			Err(NetworkingError::Transport(reason) | NetworkingError::ConnectionFailed(reason))
				if !reason.contains("timed out") =>
			{
				tracing::debug!(%node_id, %reason, "Request lost with its connection, retrying");
				self.connections.write().await.remove(&(
					node_id,
					crate::service::network::core::MESSAGING_ALPN.to_vec(),
				));
				self.send_request_once(node_id, &envelope).await
			}
			result => result,
		}
	}

	async fn send_request_once(&self, node_id: EndpointId, envelope: &Message) -> Result<Message> {
		use tokio::io::{AsyncReadExt, AsyncWriteExt};
		use tokio::time::{timeout, Duration};

		// Get or create cached connection
		let endpoint = self.endpoint.as_ref().ok_or_else(|| {
			NetworkingError::ConnectionFailed("No endpoint available".to_string())
		})?;

		let logger: Arc<dyn utils::NetworkLogger> = Arc::new(utils::SilentLogger);
		let conn = utils::get_or_create_connection(
			self.connections.clone(),
			endpoint,
			node_id,
			crate::service::network::core::MESSAGING_ALPN,
			&logger,
		)
		.await?;

		// Create new stream on existing connection
		let (mut send, mut recv) = conn.open_bi().await.map_err(|e| {
			NetworkingError::ConnectionFailed(format!("Failed to open stream: {}", e))
		})?;

		let msg_data =
			serde_json::to_vec(envelope).map_err(|e| NetworkingError::Serialization(e))?;

		// Send with length prefix
		let len = msg_data.len() as u32;
		send.write_all(&len.to_be_bytes())
			.await
			.map_err(|e| NetworkingError::Transport(format!("Failed to send length: {}", e)))?;
		send.write_all(&msg_data)
			.await
			.map_err(|e| NetworkingError::Transport(format!("Failed to send data: {}", e)))?;

		// Properly close stream (Iroh best practice)
		send.finish()
			.map_err(|e| NetworkingError::Transport(format!("Failed to finish stream: {}", e)))?;

		tracing::debug!("Message sent, waiting for response...");

		// Read response with timeout
		let result = timeout(Duration::from_secs(30), async {
			let mut len_buf = [0u8; 4];
			recv.read_exact(&mut len_buf).await.map_err(|e| {
				NetworkingError::Transport(format!("Failed to read response length: {}", e))
			})?;
			let resp_len = u32::from_be_bytes(len_buf) as usize;

			tracing::debug!("Receiving response of {} bytes", resp_len);

			let mut resp_buf = vec![0u8; resp_len];
			recv.read_exact(&mut resp_buf).await.map_err(|e| {
				NetworkingError::Transport(format!("Failed to read response: {}", e))
			})?;
			Ok::<_, NetworkingError>(resp_buf)
		})
		.await;

		let resp_buf = match result {
			Ok(Ok(buf)) => buf,
			Ok(Err(e)) => return Err(e),
			Err(_) => {
				return Err(NetworkingError::Transport(
					"Request timed out after 30s".to_string(),
				))
			}
		};

		serde_json::from_slice(&resp_buf).map_err(NetworkingError::Serialization)
	}

	/// Handles a Wingdrop message from an authenticated device and returns
	/// the serialized reply.
	async fn handle_wingdrop_message(
		&self,
		from_device: Uuid,
		message: crate::service::wingdrop::WingdropMessage,
	) -> Vec<u8> {
		use crate::service::wingdrop::{self, WingdropMessage};

		let reply = match self.context.as_ref() {
			None => WingdropMessage::Rejected {
				drop_id: wingdrop_id(&message),
				reason: "This device is not ready to receive files".to_string(),
			},
			Some(context) => {
				let config = crate::config::AppConfig::load_from(&context.data_dir)
					.map(|c| c.wingdrop)
					.unwrap_or_default();
				if matches!(message, WingdropMessage::Offer(_)) && !config.enabled {
					WingdropMessage::Rejected {
						drop_id: wingdrop_id(&message),
						reason: "This device is not accepting Wingdrop files".to_string(),
					}
				} else {
					let is_offer = matches!(message, WingdropMessage::Offer(_));
					let from_name = {
						let registry = self.device_registry.read().await;
						device_name(&registry, from_device)
					};
					let auto_accept = config
						.auto_accept
						.then(|| wingdrop::destination_root(config.destination.as_deref()));
					let (reply, auto) = context.wingdrop.handle_message(
						from_device,
						from_name,
						message,
						&context.events,
						auto_accept,
					);
					if is_offer && matches!(reply, WingdropMessage::OfferReceived { .. }) {
						spawn_offer_expiry(context.clone());
					}
					if let Some(auto) = auto {
						spawn_send_decision(context.clone(), auto);
					}
					reply
				}
			}
		};
		serde_json::to_vec(&Message::Wingdrop(reply)).unwrap_or_default()
	}
}

#[async_trait]
impl ProtocolHandler for MessagingProtocolHandler {
	fn protocol_name(&self) -> &str {
		"messaging"
	}

	fn as_any(&self) -> &dyn std::any::Any {
		self
	}

	async fn handle_stream(
		&self,
		mut send: Box<dyn tokio::io::AsyncWrite + Send + Unpin>,
		mut recv: Box<dyn tokio::io::AsyncRead + Send + Unpin>,
		remote_node_id: EndpointId,
	) {
		use tokio::io::{AsyncReadExt, AsyncWriteExt};

		// Simple request-response messaging over streams
		loop {
			// Read message length (4 bytes)
			let mut len_buf = [0u8; 4];
			match recv.read_exact(&mut len_buf).await {
				Ok(_) => {}
				Err(_) => break, // Connection closed
			}
			let msg_len = u32::from_be_bytes(len_buf) as usize;

			// Read message
			let mut msg_buf = vec![0u8; msg_len];
			if let Err(e) = recv.read_exact(&mut msg_buf).await {
				eprintln!("Failed to read message: {}", e);
				break;
			}

			// Deserialize and handle
			match serde_json::from_slice::<Message>(&msg_buf) {
				Ok(message) => {
					// Process message based on type
					let response = match &message {
						Message::Ping { timestamp, payload } => {
							let pong = Message::Pong {
								timestamp: chrono::Utc::now(),
								original_timestamp: *timestamp,
							};
							serde_json::to_vec(&pong).unwrap_or_default()
						}
						Message::Data { message_id, .. } => {
							let ack = Message::Ack {
								message_id: *message_id,
								success: true,
								error: None,
							};
							serde_json::to_vec(&ack).unwrap_or_default()
						}
						Message::Library(lib_msg) => {
							// Handle library message - need to derive device_id from node_id
							// For now, use a placeholder (TODO: proper mapping)
							// Map node_id to device_id using registry
							let device_id_opt = {
								let registry = self.device_registry.read().await;
								registry.get_device_by_node(remote_node_id)
							};

							let resp = match device_id_opt {
								Some(device_id) => {
									match self
										.handle_library_message(device_id, lib_msg.clone())
										.await
									{
										Ok(resp) => resp,
										Err(e) => {
											tracing::error!(
												"Failed to handle library message: {}",
												e
											);
											Vec::new()
										}
									}
								}
								None => {
									tracing::warn!(
										"Received library message from unknown node {}",
										remote_node_id
									);
									Vec::new()
								}
							};
							resp
						}
						Message::Wingdrop(drop_msg) => {
							let device_id_opt = {
								let registry = self.device_registry.read().await;
								registry.get_device_by_node(remote_node_id)
							};
							match device_id_opt {
								Some(device_id) => {
									self.handle_wingdrop_message(device_id, drop_msg.clone())
										.await
								}
								None => {
									tracing::warn!(
										"Received Wingdrop message from unknown node {}",
										remote_node_id
									);
									serde_json::to_vec(&Message::Wingdrop(
										crate::service::wingdrop::WingdropMessage::Rejected {
											drop_id: wingdrop_id(drop_msg),
											reason: "Unknown device".to_string(),
										},
									))
									.unwrap_or_default()
								}
							}
						}
						Message::Goodbye { reason, .. } => {
							// Received graceful disconnect from remote device
							eprintln!("Remote device disconnecting gracefully: {}", reason);
							// Close the stream by breaking the loop
							break;
						}
						_ => Vec::new(), // No response for Pong/Ack
					};

					// Send response if any
					if !response.is_empty() {
						let len = response.len() as u32;
						if send.write_all(&len.to_be_bytes()).await.is_err() {
							break;
						}
						if send.write_all(&response).await.is_err() {
							break;
						}
						let _ = send.flush().await;
					}
				}
				Err(e) => {
					eprintln!("Failed to deserialize message: {}", e);
					break;
				}
			}
		}
	}

	async fn handle_request(&self, from_device: Uuid, request_data: Vec<u8>) -> Result<Vec<u8>> {
		let message: Message =
			serde_json::from_slice(&request_data).map_err(|e| NetworkingError::Serialization(e))?;

		match message {
			Message::Ping { timestamp, payload } => {
				self.handle_ping(from_device, timestamp, payload).await
			}
			Message::Pong {
				timestamp,
				original_timestamp,
			} => {
				self.handle_pong(from_device, timestamp, original_timestamp)
					.await
			}
			Message::Data {
				message_id,
				content_type,
				payload,
			} => {
				self.handle_data(from_device, message_id, content_type, payload)
					.await
			}
			Message::Ack {
				message_id,
				success,
				error,
			} => {
				self.handle_ack(from_device, message_id, success, error)
					.await
			}
			Message::Library(lib_msg) => self.handle_library_message(from_device, lib_msg).await,
			Message::Wingdrop(drop_msg) => {
				Ok(self.handle_wingdrop_message(from_device, drop_msg).await)
			}
			Message::Goodbye { reason, .. } => {
				println!(
					"Device {} disconnecting gracefully: {}",
					from_device, reason
				);
				// Return empty response, connection will be closed by the sender
				Ok(Vec::new())
			}
		}
	}

	async fn handle_response(
		&self,
		_from_device: Uuid,
		_from_node: EndpointId,
		_response_data: Vec<u8>,
	) -> Result<()> {
		// Messaging protocol handles responses in handle_request
		Ok(())
	}

	async fn handle_event(&self, _event: ProtocolEvent) -> Result<()> {
		// Basic messaging doesn't need special event handling
		Ok(())
	}
}

fn wingdrop_id(message: &crate::service::wingdrop::WingdropMessage) -> Uuid {
	use crate::service::wingdrop::WingdropMessage::*;
	match message {
		Offer(offer) => offer.id,
		OfferReceived { drop_id }
		| Decision { drop_id, .. }
		| Finished { drop_id, .. }
		| Cancelled { drop_id, .. }
		| Ack { drop_id }
		| Rejected { drop_id, .. } => *drop_id,
	}
}

/// The name a paired device announced, for showing who sends an offer.
fn device_name(
	registry: &crate::service::network::device::DeviceRegistry,
	device_id: Uuid,
) -> String {
	use crate::service::network::device::DeviceState;
	match registry.get_device_state(device_id) {
		Some(
			DeviceState::Paired { info, .. }
			| DeviceState::Connected { info, .. }
			| DeviceState::Disconnected { info, .. },
		) => info.device_name.clone(),
		_ => device_id.to_string(),
	}
}

/// Expires the offer if nobody answers in time, so its prompt closes.
fn spawn_offer_expiry(context: Arc<crate::context::CoreContext>) {
	use crate::service::wingdrop;
	tokio::spawn(async move {
		tokio::time::sleep(wingdrop::OFFER_TTL + std::time::Duration::from_secs(1)).await;
		for offer in context.wingdrop.expire_pending() {
			wingdrop::emit_status(&context.events, offer.id, wingdrop::WingdropStatus::Expired);
		}
	});
}

/// Tells the sender an offer was accepted on arrival.
fn spawn_send_decision(
	context: Arc<crate::context::CoreContext>,
	auto: crate::service::wingdrop::AutoAccepted,
) {
	tokio::spawn(async move {
		let destinations = auto
			.destinations
			.iter()
			.map(|p| p.to_string_lossy().into_owned())
			.collect();
		if let Err(error) = crate::service::wingdrop::send_decision(
			&context,
			auto.device_id,
			auto.drop_id,
			true,
			destinations,
		)
		.await
		{
			tracing::warn!(drop_id = %auto.drop_id, %error, "Could not send an automatic Wingdrop decision");
			context.wingdrop.cancel_incoming(auto.drop_id, None);
		}
	});
}
