//! # Integrations
//!
//! An integration connects a third-party account to a library and exposes what
//! that account offers through WingDrive's own concepts: storage becomes a
//! cloud volume the Explorer, copy jobs and cloud sync already understand, and
//! services without a file tree (Google Photos) get their own jobs. Google is
//! the first provider.
//!
//! `IntegrationService` is node-wide state behind the `integrations.*`
//! operations: browser sign-in sessions in progress, one shared access-token
//! source per connected account, and local editing sessions for cloud files.
//! Account secrets themselves live encrypted in each library (`store`).
//!
//! The OAuth client comes from, in order, the client the user saved in the
//! library, the `WINGDRIVE_GOOGLE_CLIENT_ID` and `WINGDRIVE_GOOGLE_CLIENT_SECRET`
//! environment variables of the daemon, or the same variables at build time.
//! `WINGDRIVE_GOOGLE_ENDPOINTS_BASE` points every Google API at another server,
//! which the integration tests use to run against a local fake.

pub mod cloud_edit;
pub mod google;
pub mod store;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use specta::Type;
use tokio::sync::Mutex;
use tracing::{info, warn};
use uuid::Uuid;

use google::oauth::{self, GoogleService, OAuthClient};
use google::{
	GoogleApi, GoogleDriveBackend, GoogleEndpoints, GoogleError, GoogleResult, GoogleTokenSource,
};
use store::{IntegrationStore, StoredGoogleAccount};

use crate::domain::addressing::WingPath;
use crate::library::Library;
use crate::volume::backend::CloudServiceType;
use crate::volume::{Volume, VolumeFingerprint};

/// How long the browser sign-in may take before the session is dropped.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10 * 60);

/// Finished sign-in sessions stay readable this long so a slow UI poll still sees the result.
const FINISHED_SESSION_TTL: Duration = Duration::from_secs(10 * 60);

/// Providers WingDrive can connect.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum IntegrationProvider {
	Google,
}

/// Where the OAuth client in use comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum OAuthClientSource {
	/// Saved by the user in this library.
	Library,
	/// Set in the daemon's environment.
	Environment,
	/// Compiled into this build.
	Bundled,
}

/// The Drive volume of an account, addressable as `gdrive://<identifier>/`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct DriveMountInfo {
	pub fingerprint: String,
	pub identifier: String,
	/// Whether the volume is registered and reachable in this daemon.
	pub mounted: bool,
}

/// A connected account as clients see it, without secrets.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct IntegrationAccountInfo {
	pub id: String,
	pub provider: IntegrationProvider,
	pub email: String,
	pub name: Option<String>,
	pub picture: Option<String>,
	/// Services the account granted access to.
	pub services: Vec<GoogleService>,
	pub connected_at: DateTime<Utc>,
	pub drive: Option<DriveMountInfo>,
}

/// Progress of a browser sign-in.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum ConnectStatus {
	/// Waiting for the user to finish in the browser.
	Pending,
	Completed {
		account: IntegrationAccountInfo,
	},
	Failed {
		message: String,
	},
	Cancelled,
}

struct ConnectSession {
	library_id: Uuid,
	status: ConnectStatus,
	updated_at: Instant,
	task: Option<tokio::task::JoinHandle<()>>,
}

/// Errors from integration operations that are not Google API errors.
#[derive(Debug, thiserror::Error)]
pub enum IntegrationError {
	#[error(transparent)]
	Google(#[from] GoogleError),
	#[error("Could not read or write integration secrets: {0}")]
	Store(#[from] crate::crypto::cloud_credentials::CloudCredentialError),
	#[error("No connected account with ID {0}")]
	AccountNotFound(String),
	#[error("{0}")]
	Volume(String),
}

/// Node-wide integration state.
pub struct IntegrationService {
	http: reqwest::Client,
	endpoints: GoogleEndpoints,
	bundled_google_client: Option<OAuthClient>,
	sessions: Mutex<HashMap<Uuid, ConnectSession>>,
	token_sources: Mutex<HashMap<(Uuid, String), Arc<GoogleTokenSource>>>,
	pub cloud_edits: cloud_edit::CloudEditService,
}

impl std::fmt::Debug for IntegrationService {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("IntegrationService").finish_non_exhaustive()
	}
}

fn client_from(id: Option<String>, secret: Option<String>) -> Option<OAuthClient> {
	match (id, secret) {
		(Some(client_id), Some(client_secret))
			if !client_id.trim().is_empty() && !client_secret.trim().is_empty() =>
		{
			Some(OAuthClient {
				client_id: client_id.trim().to_string(),
				client_secret: client_secret.trim().to_string(),
			})
		}
		_ => None,
	}
}

impl IntegrationService {
	/// Service talking to the real Google endpoints.
	pub fn new(data_dir: PathBuf) -> Self {
		let bundled = client_from(
			option_env!("WINGDRIVE_GOOGLE_CLIENT_ID").map(str::to_string),
			option_env!("WINGDRIVE_GOOGLE_CLIENT_SECRET").map(str::to_string),
		);
		let endpoints = match std::env::var("WINGDRIVE_GOOGLE_ENDPOINTS_BASE") {
			Ok(base) if !base.trim().is_empty() => {
				warn!(%base, "Google APIs point at a custom server");
				GoogleEndpoints::for_base_url(&base)
			}
			_ => GoogleEndpoints::default(),
		};
		Self::with_endpoints(endpoints, bundled, data_dir)
	}

	/// Service with custom endpoints and bundled client, used by tests.
	pub fn with_endpoints(
		endpoints: GoogleEndpoints,
		bundled_google_client: Option<OAuthClient>,
		data_dir: PathBuf,
	) -> Self {
		let http = reqwest::Client::builder()
			.connect_timeout(Duration::from_secs(20))
			.read_timeout(Duration::from_secs(120))
			.user_agent(concat!("WingDrive/", env!("CARGO_PKG_VERSION")))
			.build()
			.expect("reqwest client builds with static settings");
		Self {
			http,
			endpoints,
			bundled_google_client,
			sessions: Mutex::new(HashMap::new()),
			token_sources: Mutex::new(HashMap::new()),
			cloud_edits: cloud_edit::CloudEditService::new(data_dir.join("cloud-edit")),
		}
	}

	pub fn endpoints(&self) -> &GoogleEndpoints {
		&self.endpoints
	}

	pub fn http(&self) -> &reqwest::Client {
		&self.http
	}

	/// The OAuth client to use for new sign-ins in `library`, and where it came from.
	pub async fn google_client(
		&self,
		store: &IntegrationStore,
	) -> Result<Option<(OAuthClient, OAuthClientSource)>, IntegrationError> {
		if let Some(client) = store.google_client().await? {
			return Ok(Some((client, OAuthClientSource::Library)));
		}
		if let Some(client) = client_from(
			std::env::var("WINGDRIVE_GOOGLE_CLIENT_ID").ok(),
			std::env::var("WINGDRIVE_GOOGLE_CLIENT_SECRET").ok(),
		) {
			return Ok(Some((client, OAuthClientSource::Environment)));
		}
		Ok(self
			.bundled_google_client
			.clone()
			.map(|client| (client, OAuthClientSource::Bundled)))
	}

	/// Starts a browser sign-in and returns the session ID and the consent URL.
	///
	/// The daemon listens for Google's redirect on a loopback port and finishes
	/// the sign-in in the background: it exchanges the code, reads the profile,
	/// saves the account (merging with an earlier connection of the same
	/// account) and mounts Drive when Drive was granted. Poll
	/// `connect_status` for the outcome.
	pub async fn start_google_connect(
		self: &Arc<Self>,
		library: Arc<Library>,
		services: Vec<GoogleService>,
		login_hint: Option<String>,
	) -> Result<(Uuid, String), IntegrationError> {
		let store = IntegrationStore::for_library(&library);
		let (client, _) = self
			.google_client(&store)
			.await?
			.ok_or(GoogleError::NotConfigured)?;
		let services = if services.is_empty() {
			vec![GoogleService::Drive, GoogleService::Photos]
		} else {
			services
		};
		let listener = oauth::LoopbackListener::bind()
			.await
			.map_err(GoogleError::from)?;
		let redirect_uri = listener.redirect_uri();
		let pkce = oauth::Pkce::generate();
		let state = oauth::random_token();
		let url = oauth::authorization_url(
			&self.endpoints,
			&client,
			&redirect_uri,
			&oauth::scopes_for(&services),
			&state,
			&pkce.challenge,
			login_hint.as_deref(),
		);
		let session_id = Uuid::new_v4();
		let library_id = library.id();

		let this = self.clone();
		let task = tokio::spawn(async move {
			let result = async {
				let code = listener.wait_for_code(&state, CONNECT_TIMEOUT).await?;
				let tokens = oauth::exchange_code(
					&this.http,
					&this.endpoints,
					&client,
					&code,
					&pkce.verifier,
					&redirect_uri,
				)
				.await?;
				this.finish_google_connect(&library, client, tokens).await
			}
			.await;
			let status = match result {
				Ok(account) => ConnectStatus::Completed { account },
				Err(error) => {
					warn!(%error, "Google sign-in failed");
					ConnectStatus::Failed {
						message: error.to_string(),
					}
				}
			};
			let mut sessions = this.sessions.lock().await;
			if let Some(session) = sessions.get_mut(&session_id) {
				if session.status == ConnectStatus::Pending {
					session.status = status;
					session.updated_at = Instant::now();
				}
				session.task = None;
			}
		});

		let mut sessions = self.sessions.lock().await;
		sessions.retain(|_, s| {
			s.status == ConnectStatus::Pending || s.updated_at.elapsed() < FINISHED_SESSION_TTL
		});
		sessions.insert(
			session_id,
			ConnectSession {
				library_id,
				status: ConnectStatus::Pending,
				updated_at: Instant::now(),
				task: Some(task),
			},
		);
		Ok((session_id, url))
	}

	/// Saves the account after a successful code exchange.
	async fn finish_google_connect(
		&self,
		library: &Arc<Library>,
		client: OAuthClient,
		tokens: oauth::TokenResponse,
	) -> Result<IntegrationAccountInfo, IntegrationError> {
		let refresh_token = tokens.refresh_token.clone().ok_or_else(|| {
			GoogleError::Authorization(
				"Google returned no refresh token; remove WingDrive from your Google account's third-party access and connect again".to_string(),
			)
		})?;
		let profile =
			oauth::fetch_user_info(&self.http, &self.endpoints, &tokens.access_token).await?;
		let email = profile
			.email
			.clone()
			.unwrap_or_else(|| profile.sub.clone())
			.to_lowercase();
		let store = IntegrationStore::for_library(library);
		let previous = store.google_account(&profile.sub).await?;

		// A reconnect may grant more scopes; `include_granted_scopes` returns the union.
		let mut account = StoredGoogleAccount {
			id: profile.sub.clone(),
			email,
			name: profile.name,
			picture: profile.picture,
			scopes: tokens.granted_scopes(),
			client,
			refresh_token,
			connected_at: previous
				.as_ref()
				.map(|p| p.connected_at)
				.unwrap_or_else(Utc::now),
			drive_fingerprint: previous.as_ref().and_then(|p| p.drive_fingerprint.clone()),
		};
		store.save_google_account(&account).await?;

		let source = Arc::new(GoogleTokenSource::new(
			self.http.clone(),
			self.endpoints.clone(),
			account.client.clone(),
			account.refresh_token.clone(),
		));
		source.seed(tokens.access_token, tokens.expires_in).await;
		self.token_sources
			.lock()
			.await
			.insert((library.id(), account.id.clone()), source);

		if GoogleService::Drive.is_granted(&account.scopes) {
			if let Err(error) = self.mount_drive(library, &mut account).await {
				warn!(%error, "Connected Google account but could not mount Drive");
			}
		}
		info!(account = %account.id, "Connected Google account");
		self.account_info(library, &account).await
	}

	/// Status of a sign-in started with `start_google_connect`.
	pub async fn connect_status(
		&self,
		library_id: Uuid,
		session_id: Uuid,
	) -> Option<ConnectStatus> {
		self.sessions
			.lock()
			.await
			.get(&session_id)
			.filter(|s| s.library_id == library_id)
			.map(|s| s.status.clone())
	}

	/// Stops waiting for a sign-in and closes its loopback listener.
	pub async fn cancel_connect(&self, library_id: Uuid, session_id: Uuid) -> bool {
		let mut sessions = self.sessions.lock().await;
		let Some(session) = sessions
			.get_mut(&session_id)
			.filter(|s| s.library_id == library_id)
		else {
			return false;
		};
		if let Some(task) = session.task.take() {
			task.abort();
		}
		if session.status == ConnectStatus::Pending {
			session.status = ConnectStatus::Cancelled;
			session.updated_at = Instant::now();
		}
		true
	}

	/// Shared token source for an account, created on first use.
	pub async fn token_source(
		&self,
		library_id: Uuid,
		account: &StoredGoogleAccount,
	) -> Arc<GoogleTokenSource> {
		let mut sources = self.token_sources.lock().await;
		sources
			.entry((library_id, account.id.clone()))
			.or_insert_with(|| {
				Arc::new(GoogleTokenSource::new(
					self.http.clone(),
					self.endpoints.clone(),
					account.client.clone(),
					account.refresh_token.clone(),
				))
			})
			.clone()
	}

	/// Authorized API access for an account of `library`.
	pub async fn google_api(
		&self,
		library: &Library,
		account_id: &str,
	) -> Result<(StoredGoogleAccount, GoogleApi), IntegrationError> {
		let account = IntegrationStore::for_library(library)
			.google_account(account_id)
			.await?
			.ok_or_else(|| IntegrationError::AccountNotFound(account_id.to_string()))?;
		let tokens = self.token_source(library.id(), &account).await;
		let api = GoogleApi::new(self.http.clone(), self.endpoints.clone(), tokens);
		Ok((account, api))
	}

	/// Photos client for an account, checking that Photos was granted.
	pub async fn photos_client(
		&self,
		library: &Library,
		account_id: &str,
	) -> Result<google::PhotosClient, IntegrationError> {
		let (account, api) = self.google_api(library, account_id).await?;
		if !GoogleService::Photos.is_granted(&account.scopes) {
			return Err(GoogleError::MissingScope("Google Photos".to_string()).into());
		}
		Ok(google::PhotosClient::new(api))
	}

	fn drive_fingerprint(account_id: &str) -> VolumeFingerprint {
		VolumeFingerprint::from_network_volume("GoogleDrive", &format!("google:{account_id}"))
	}

	fn drive_volume(
		&self,
		account: &StoredGoogleAccount,
		backend: Arc<GoogleDriveBackend>,
		device_id: Uuid,
		capacity: Option<(u64, u64)>,
	) -> Volume {
		let now = Utc::now();
		let mount_point = PathBuf::from(format!("gdrive://{}", account.email));
		let name = format!("Google Drive ({})", account.email);
		let (total, available) = capacity.unwrap_or((0, 0));
		Volume {
			id: Uuid::new_v5(
				&Uuid::NAMESPACE_URL,
				format!("gdrive:{}", account.id).as_bytes(),
			),
			fingerprint: Self::drive_fingerprint(&account.id),
			device_id,
			name: name.clone(),
			library_id: None,
			is_tracked: false,
			mount_point: mount_point.clone(),
			mount_points: vec![mount_point],
			volume_type: crate::volume::types::VolumeType::Network,
			mount_type: crate::volume::types::MountType::Network,
			disk_type: crate::volume::types::DiskType::Unknown,
			file_system: crate::volume::types::FileSystem::Other("GoogleDrive".to_string()),
			total_capacity: total,
			available_space: available,
			is_read_only: false,
			is_mounted: true,
			hardware_id: None,
			backend: Some(backend),
			cloud_identifier: Some(account.email.clone()),
			cloud_config: Some(serde_json::json!({
				"integration": "google",
				"account_id": account.id,
			})),
			apfs_container: None,
			container_volume_id: None,
			path_mappings: Vec::new(),
			is_user_visible: true,
			auto_track_eligible: false,
			read_speed_mbps: None,
			write_speed_mbps: None,
			created_at: now,
			updated_at: now,
			last_seen_at: now,
			total_files: None,
			total_directories: None,
			last_stats_update: None,
			display_name: Some(name),
			is_favorite: false,
			color: None,
			icon: None,
			error_message: None,
			supports_block_cloning: false,
		}
	}

	/// Registers the account's Drive as a cloud volume in this daemon.
	async fn register_drive(
		&self,
		library: &Library,
		account: &StoredGoogleAccount,
		query_capacity: bool,
	) -> Result<Volume, IntegrationError> {
		let context = library.core_context();
		let device_id = context
			.device_manager
			.device_id()
			.map_err(|e| IntegrationError::Volume(e.to_string()))?;
		let tokens = self.token_source(library.id(), account).await;
		let api = GoogleApi::new(self.http.clone(), self.endpoints.clone(), tokens);
		let backend = Arc::new(GoogleDriveBackend::new(api, None));
		let capacity = if query_capacity {
			use crate::volume::VolumeBackend;
			backend.capacity().await.ok().flatten()
		} else {
			None
		};
		let volume = self.drive_volume(account, backend, device_id, capacity);
		context
			.volume_manager
			.register_cloud_volume(volume.clone())
			.await;
		Ok(volume)
	}

	/// Mounts the account's Drive: registers the volume, tracks it in the
	/// library and adds a sidebar location that browses Drive live.
	///
	/// The location uses index mode "none" so connecting a large Drive does
	/// not start a crawl that downloads samples of every file. Users can
	/// still enable indexing on the location later.
	pub async fn mount_drive(
		&self,
		library: &Arc<Library>,
		account: &mut StoredGoogleAccount,
	) -> Result<DriveMountInfo, IntegrationError> {
		let volume = self.register_drive(library, account, true).await?;
		let context = library.core_context();
		match context
			.volume_manager
			.track_volume(library, &volume.fingerprint, volume.display_name.clone())
			.await
		{
			Ok(_) | Err(crate::volume::error::VolumeError::AlreadyTracked(_)) => {}
			Err(error) => return Err(IntegrationError::Volume(error.to_string())),
		}

		let first_mount = account.drive_fingerprint.is_none();
		account.drive_fingerprint = Some(volume.fingerprint.0.clone());
		IntegrationStore::for_library(library)
			.save_google_account(account)
			.await?;

		let location_path = WingPath::Cloud {
			service: CloudServiceType::GoogleDrive,
			identifier: account.email.clone(),
			path: String::new(),
		};
		if first_mount && self.drive_location(library, account).await.is_none() {
			let device_record = {
				use crate::infra::db::entities;
				use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
				entities::device::Entity::find()
					.filter(entities::device::Column::Uuid.eq(volume.device_id))
					.one(library.db().conn())
					.await
					.map_err(|e| IntegrationError::Volume(e.to_string()))?
			};
			if let Some(device) = device_record {
				let manager =
					crate::location::LocationManager::new(context.events.as_ref().clone());
				if let Err(error) = manager
					.add_location(
						library.clone(),
						location_path,
						Some(format!("Google Drive ({})", account.email)),
						device.id,
						crate::location::IndexMode::None,
						None,
						None,
						&context.volume_manager,
					)
					.await
				{
					warn!(%error, "Mounted Google Drive but could not add its location");
				}
			}
		}

		Ok(DriveMountInfo {
			fingerprint: volume.fingerprint.0,
			identifier: account.email.clone(),
			mounted: true,
		})
	}

	/// The location that points at the root of the account's Drive, if any.
	async fn drive_location(
		&self,
		library: &Library,
		account: &StoredGoogleAccount,
	) -> Option<Uuid> {
		let manager =
			crate::location::LocationManager::new(library.core_context().events.as_ref().clone());
		let root = format!("gdrive://{}/", account.email);
		manager
			.list_locations(library)
			.await
			.ok()?
			.into_iter()
			.find(|location| location.path.to_string_lossy() == root)
			.map(|location| location.id)
	}

	/// Re-registers the Drive volumes of every library after the daemon starts.
	pub async fn restore_drive_volumes(&self, libraries: &[Arc<Library>]) {
		for library in libraries {
			let accounts = match IntegrationStore::for_library(library)
				.google_accounts()
				.await
			{
				Ok(accounts) => accounts,
				Err(error) => {
					warn!(%error, library = %library.id(), "Could not read Google accounts");
					continue;
				}
			};
			for account in accounts.iter().filter(|a| a.drive_fingerprint.is_some()) {
				match self.register_drive(library, account, false).await {
					Ok(_) => info!(account = %account.id, "Restored Google Drive volume"),
					Err(error) => warn!(%error, "Could not restore Google Drive volume"),
				}
			}
		}
	}

	/// Unmounts Drive: removes the location, untracks and unregisters the volume.
	pub async fn unmount_drive(
		&self,
		library: &Arc<Library>,
		account: &mut StoredGoogleAccount,
	) -> Result<(), IntegrationError> {
		let context = library.core_context();
		if let Some(location_id) = self.drive_location(library, account).await {
			let manager = crate::location::LocationManager::new(context.events.as_ref().clone());
			if let Err(error) = manager.remove_location(library, location_id).await {
				warn!(%error, "Could not remove the Google Drive location");
			}
		}
		let fingerprint = Self::drive_fingerprint(&account.id);
		match context
			.volume_manager
			.untrack_volume(library, &fingerprint)
			.await
		{
			Ok(_) | Err(crate::volume::error::VolumeError::NotTracked(_)) => {}
			Err(error) => warn!(%error, "Could not untrack the Google Drive volume"),
		}
		context
			.volume_manager
			.unregister_cloud_volume(&fingerprint)
			.await;
		account.drive_fingerprint = None;
		Ok(())
	}

	/// Disconnects an account: unmounts Drive, revokes the grant at Google
	/// and deletes the stored secrets.
	///
	/// Revocation is best effort. If Google cannot be reached the local copy
	/// is still deleted, and the user can revoke from their Google account page.
	pub async fn remove_google_account(
		&self,
		library: &Arc<Library>,
		account_id: &str,
		revoke: bool,
	) -> Result<(), IntegrationError> {
		let store = IntegrationStore::for_library(library);
		let mut account = store
			.google_account(account_id)
			.await?
			.ok_or_else(|| IntegrationError::AccountNotFound(account_id.to_string()))?;
		self.unmount_drive(library, &mut account).await?;
		if revoke {
			if let Err(error) =
				oauth::revoke_token(&self.http, &self.endpoints, &account.refresh_token).await
			{
				warn!(%error, "Could not revoke the Google grant; it stays listed in the Google account");
			}
		}
		store.delete_google_account(account_id).await?;
		self.token_sources
			.lock()
			.await
			.remove(&(library.id(), account_id.to_string()));
		info!(account = %account_id, "Removed Google account");
		Ok(())
	}

	/// Client-facing view of an account.
	pub async fn account_info(
		&self,
		library: &Library,
		account: &StoredGoogleAccount,
	) -> Result<IntegrationAccountInfo, IntegrationError> {
		let drive = match &account.drive_fingerprint {
			Some(fingerprint) => {
				let mounted = library
					.core_context()
					.volume_manager
					.get_volume(&VolumeFingerprint(fingerprint.clone()))
					.await
					.is_some_and(|v| v.backend.is_some());
				Some(DriveMountInfo {
					fingerprint: fingerprint.clone(),
					identifier: account.email.clone(),
					mounted,
				})
			}
			None => None,
		};
		Ok(IntegrationAccountInfo {
			id: account.id.clone(),
			provider: IntegrationProvider::Google,
			email: account.email.clone(),
			name: account.name.clone(),
			picture: account.picture.clone(),
			services: account.granted_services(),
			connected_at: account.connected_at,
			drive,
		})
	}

	/// Every connected account of `library`.
	pub async fn list_accounts(
		&self,
		library: &Library,
	) -> Result<Vec<IntegrationAccountInfo>, IntegrationError> {
		let mut infos = Vec::new();
		for account in IntegrationStore::for_library(library)
			.google_accounts()
			.await?
		{
			infos.push(self.account_info(library, &account).await?);
		}
		Ok(infos)
	}
}

/// Result type for integration operations.
pub type IntegrationResult<T> = Result<T, IntegrationError>;

impl From<IntegrationError> for crate::infra::action::error::ActionError {
	fn from(error: IntegrationError) -> Self {
		match error {
			IntegrationError::AccountNotFound(_) => Self::InvalidInput(error.to_string()),
			IntegrationError::Google(GoogleError::NotConfigured) => {
				Self::InvalidInput(error.to_string())
			}
			other => Self::Internal(other.to_string()),
		}
	}
}

impl From<IntegrationError> for crate::infra::query::QueryError {
	fn from(error: IntegrationError) -> Self {
		Self::Internal(error.to_string())
	}
}
