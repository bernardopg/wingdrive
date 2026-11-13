//! # Integrations
//!
//! An integration connects a third-party account to a library and exposes what
//! that account offers through WingDrive's own concepts: storage becomes a
//! cloud volume the Explorer, copy jobs and cloud sync already understand, and
//! services without a file tree (Google Photos) get their own jobs. Providers are
//! Google (Drive and Photos) and Microsoft (OneDrive).
//!
//! `IntegrationService` is node-wide state behind the `integrations.*`
//! operations: browser sign-in sessions in progress, one shared access-token
//! source per connected account, and local editing sessions for cloud files.
//! Account secrets themselves live encrypted in each library (`store`).
//!
//! ## Connection modes
//!
//! Every sign-in goes through an OAuth client registered with the provider.
//! The user picks whose client it is (`ConnectionMode`):
//!
//! - `App`: WingDrive's own client. The user only signs in. It comes from the
//!   daemon's environment (`WINGDRIVE_GOOGLE_CLIENT_ID` and
//!   `WINGDRIVE_GOOGLE_CLIENT_SECRET`, `WINGDRIVE_MICROSOFT_CLIENT_ID`) or the
//!   same variables at build time.
//! - `Custom`: a client the user registered in their own Google Cloud project
//!   or Microsoft Entra tenant and saved in the library.
//!
//! Each account remembers the client it signed in with, because refreshing a
//! token needs the same client. `WINGDRIVE_GOOGLE_ENDPOINTS_BASE` and
//! `WINGDRIVE_MICROSOFT_ENDPOINTS_BASE` point sign-in at another server, which
//! the integration tests use to run against local fakes.

pub mod cloud_edit;
pub mod google;
pub mod microsoft;
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
use microsoft::{MicrosoftClient, MicrosoftEndpoints, MicrosoftError, MicrosoftTokenSource};
use store::{IntegrationStore, StoredGoogleAccount, StoredMicrosoftAccount};

use crate::domain::addressing::WingPath;
use crate::library::Library;
use crate::volume::backend::{CloudServiceType, VolumeBackend};
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
	Microsoft,
}

/// Whose OAuth client an account signs in with.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionMode {
	/// WingDrive's own client: the user only signs in to their account.
	App,
	/// A client the user registered in their own Google Cloud project or
	/// Microsoft Entra tenant.
	Custom,
}

/// Where WingDrive's own OAuth client comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum OAuthClientSource {
	/// Set in the daemon's environment.
	Environment,
	/// Compiled into this build.
	Bundled,
}

/// WingDrive's own client for a provider, as clients see it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct AppClientInfo {
	pub source: OAuthClientSource,
	/// The client ID, which is public; a secret is never returned.
	pub client_id: String,
}

/// The OAuth clients a library can sign in with, for one provider.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct OAuthClientsInfo {
	/// WingDrive's own client; `None` when this build has none.
	pub app: Option<AppClientInfo>,
	/// The ID of the client the user saved in this library, if any.
	pub custom_client_id: Option<String>,
}

/// The storage volume of an account, addressable as `gdrive://<identifier>/`
/// for Google Drive or `onedrive://<identifier>/` for OneDrive.
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
	/// Google services the account granted access to; empty for Microsoft,
	/// whose only service is OneDrive (`drive`).
	pub services: Vec<GoogleService>,
	/// Whose OAuth client the account signed in with.
	pub mode: ConnectionMode,
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
	#[error(transparent)]
	Microsoft(#[from] MicrosoftError),
	/// The build has no WingDrive client for this provider.
	#[error("This build has no WingDrive sign-in for {0}; connect with your own client instead")]
	AppClientMissing(&'static str),
	/// The library has no client of the user's own for this provider.
	#[error("Save your own {0} client in Settings > Integrations first")]
	CustomClientMissing(&'static str),
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
	microsoft_endpoints: MicrosoftEndpoints,
	bundled_microsoft_client: Option<MicrosoftClient>,
	sessions: Mutex<HashMap<Uuid, ConnectSession>>,
	token_sources: Mutex<HashMap<(Uuid, String), Arc<GoogleTokenSource>>>,
	microsoft_tokens: Mutex<HashMap<(Uuid, String), Arc<MicrosoftTokenSource>>>,
	/// Serializes read-modify-write of stored Microsoft accounts, because a
	/// token rotation and a mount can update the same account at once.
	microsoft_writes: Arc<Mutex<()>>,
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

fn microsoft_client_from(id: Option<String>) -> Option<MicrosoftClient> {
	id.map(|id| id.trim().to_string())
		.filter(|id| !id.is_empty())
		.map(|client_id| MicrosoftClient { client_id })
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
		let microsoft_endpoints = match std::env::var("WINGDRIVE_MICROSOFT_ENDPOINTS_BASE") {
			Ok(base) if !base.trim().is_empty() => {
				warn!(%base, "Microsoft sign-in points at a custom server");
				MicrosoftEndpoints::for_base_url(&base)
			}
			_ => MicrosoftEndpoints::default(),
		};
		Self::with_endpoints(endpoints, bundled, data_dir).with_microsoft(
			microsoft_endpoints,
			microsoft_client_from(
				option_env!("WINGDRIVE_MICROSOFT_CLIENT_ID").map(str::to_string),
			),
		)
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
			microsoft_endpoints: MicrosoftEndpoints::default(),
			bundled_microsoft_client: None,
			sessions: Mutex::new(HashMap::new()),
			token_sources: Mutex::new(HashMap::new()),
			microsoft_tokens: Mutex::new(HashMap::new()),
			microsoft_writes: Arc::new(Mutex::new(())),
			cloud_edits: cloud_edit::CloudEditService::new(data_dir.join("cloud-edit")),
		}
	}

	/// Sets the Microsoft endpoints and WingDrive's own Microsoft client.
	pub fn with_microsoft(
		mut self,
		endpoints: MicrosoftEndpoints,
		bundled_client: Option<MicrosoftClient>,
	) -> Self {
		self.microsoft_endpoints = endpoints;
		self.bundled_microsoft_client = bundled_client;
		self
	}

	pub fn endpoints(&self) -> &GoogleEndpoints {
		&self.endpoints
	}

	pub fn http(&self) -> &reqwest::Client {
		&self.http
	}

	/// WingDrive's own Google client, from the environment or the build.
	pub fn google_app_client(&self) -> Option<(OAuthClient, OAuthClientSource)> {
		if let Some(client) = client_from(
			std::env::var("WINGDRIVE_GOOGLE_CLIENT_ID").ok(),
			std::env::var("WINGDRIVE_GOOGLE_CLIENT_SECRET").ok(),
		) {
			return Some((client, OAuthClientSource::Environment));
		}
		self.bundled_google_client
			.clone()
			.map(|client| (client, OAuthClientSource::Bundled))
	}

	/// The Google clients `library` can sign in with.
	pub async fn google_clients(
		&self,
		store: &IntegrationStore,
	) -> Result<OAuthClientsInfo, IntegrationError> {
		Ok(OAuthClientsInfo {
			app: self
				.google_app_client()
				.map(|(client, source)| AppClientInfo {
					source,
					client_id: client.client_id,
				}),
			custom_client_id: store.google_client().await?.map(|c| c.client_id),
		})
	}

	/// The Google client for a new sign-in in `mode`.
	async fn google_client_for(
		&self,
		store: &IntegrationStore,
		mode: ConnectionMode,
	) -> Result<OAuthClient, IntegrationError> {
		match mode {
			ConnectionMode::App => self
				.google_app_client()
				.map(|(client, _)| client)
				.ok_or(IntegrationError::AppClientMissing("Google")),
			ConnectionMode::Custom => store
				.google_client()
				.await?
				.ok_or(IntegrationError::CustomClientMissing("Google Cloud")),
		}
	}

	/// WingDrive's own Microsoft client, from the environment or the build.
	pub fn microsoft_app_client(&self) -> Option<(MicrosoftClient, OAuthClientSource)> {
		if let Some(client) =
			microsoft_client_from(std::env::var("WINGDRIVE_MICROSOFT_CLIENT_ID").ok())
		{
			return Some((client, OAuthClientSource::Environment));
		}
		self.bundled_microsoft_client
			.clone()
			.map(|client| (client, OAuthClientSource::Bundled))
	}

	/// The Microsoft clients `library` can sign in with.
	pub async fn microsoft_clients(
		&self,
		store: &IntegrationStore,
	) -> Result<OAuthClientsInfo, IntegrationError> {
		Ok(OAuthClientsInfo {
			app: self
				.microsoft_app_client()
				.map(|(client, source)| AppClientInfo {
					source,
					client_id: client.client_id,
				}),
			custom_client_id: store.microsoft_client().await?.map(|c| c.client_id),
		})
	}

	/// The Microsoft client for a new sign-in in `mode`.
	async fn microsoft_client_for(
		&self,
		store: &IntegrationStore,
		mode: ConnectionMode,
	) -> Result<MicrosoftClient, IntegrationError> {
		match mode {
			ConnectionMode::App => self
				.microsoft_app_client()
				.map(|(client, _)| client)
				.ok_or(IntegrationError::AppClientMissing("Microsoft")),
			ConnectionMode::Custom => store
				.microsoft_client()
				.await?
				.ok_or(IntegrationError::CustomClientMissing("Microsoft Entra")),
		}
	}

	/// Stores a finished sign-in's outcome on its session, unless it was cancelled.
	async fn finish_session(&self, session_id: Uuid, status: ConnectStatus) {
		let mut sessions = self.sessions.lock().await;
		if let Some(session) = sessions.get_mut(&session_id) {
			if session.status == ConnectStatus::Pending {
				session.status = status;
				session.updated_at = Instant::now();
			}
			session.task = None;
		}
	}

	/// Registers a sign-in session whose work runs in `task`.
	async fn add_session(
		&self,
		session_id: Uuid,
		library_id: Uuid,
		task: tokio::task::JoinHandle<()>,
	) {
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
		mode: ConnectionMode,
		services: Vec<GoogleService>,
		login_hint: Option<String>,
	) -> Result<(Uuid, String), IntegrationError> {
		let store = IntegrationStore::for_library(&library);
		let client = self.google_client_for(&store, mode).await?;
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
				this.finish_google_connect(&library, mode, client, tokens)
					.await
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
			this.finish_session(session_id, status).await;
		});
		self.add_session(session_id, library_id, task).await;
		Ok((session_id, url))
	}

	/// Saves the account after a successful code exchange.
	async fn finish_google_connect(
		&self,
		library: &Arc<Library>,
		mode: ConnectionMode,
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
			mode: Some(mode),
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

	/// Registers `mount` as a cloud volume in this daemon.
	async fn register_mount(
		&self,
		library: &Library,
		mount: &CloudMount,
		backend: Arc<dyn VolumeBackend>,
		query_capacity: bool,
	) -> Result<Volume, IntegrationError> {
		let context = library.core_context();
		let device_id = context
			.device_manager
			.device_id()
			.map_err(|e| IntegrationError::Volume(e.to_string()))?;
		let capacity = if query_capacity {
			backend.capacity().await.ok().flatten()
		} else {
			None
		};
		let volume = mount.volume(backend, device_id, capacity);
		context
			.volume_manager
			.register_cloud_volume(volume.clone())
			.await;
		Ok(volume)
	}

	/// Mounts an account's storage: registers the volume, tracks it in the
	/// library and, on the first mount, adds a sidebar location that browses
	/// it live.
	///
	/// The location uses index mode "none" so connecting a large drive does
	/// not start a crawl that downloads samples of every file. Users can
	/// still enable indexing on the location later.
	async fn mount_volume(
		&self,
		library: &Arc<Library>,
		mount: &CloudMount,
		backend: Arc<dyn VolumeBackend>,
		first_mount: bool,
	) -> Result<DriveMountInfo, IntegrationError> {
		let volume = self.register_mount(library, mount, backend, true).await?;
		let context = library.core_context();
		match context
			.volume_manager
			.track_volume(library, &volume.fingerprint, volume.display_name.clone())
			.await
		{
			Ok(_) | Err(crate::volume::error::VolumeError::AlreadyTracked(_)) => {}
			Err(error) => return Err(IntegrationError::Volume(error.to_string())),
		}

		if first_mount && self.mount_location(library, mount).await.is_none() {
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
						WingPath::Cloud {
							service: mount.service,
							identifier: mount.email.clone(),
							path: String::new(),
						},
						Some(mount.name()),
						device.id,
						crate::location::IndexMode::None,
						None,
						None,
						&context.volume_manager,
					)
					.await
				{
					warn!(%error, volume = %mount.label, "Mounted a cloud drive but could not add its location");
				}
			}
		}

		Ok(DriveMountInfo {
			fingerprint: volume.fingerprint.0,
			identifier: mount.email.clone(),
			mounted: true,
		})
	}

	/// The location that points at the root of `mount`, if any.
	async fn mount_location(&self, library: &Library, mount: &CloudMount) -> Option<Uuid> {
		let manager =
			crate::location::LocationManager::new(library.core_context().events.as_ref().clone());
		let root = mount.root();
		manager
			.list_locations(library)
			.await
			.ok()?
			.into_iter()
			.find(|location| location.path.to_string_lossy() == root)
			.map(|location| location.id)
	}

	/// Unmounts `mount`: removes its location, untracks and unregisters the volume.
	async fn unmount_volume(&self, library: &Arc<Library>, mount: &CloudMount) {
		let context = library.core_context();
		if let Some(location_id) = self.mount_location(library, mount).await {
			let manager = crate::location::LocationManager::new(context.events.as_ref().clone());
			if let Err(error) = manager.remove_location(library, location_id).await {
				warn!(%error, volume = %mount.label, "Could not remove the cloud drive location");
			}
		}
		let fingerprint = mount.fingerprint();
		match context
			.volume_manager
			.untrack_volume(library, &fingerprint)
			.await
		{
			Ok(_) | Err(crate::volume::error::VolumeError::NotTracked(_)) => {}
			Err(error) => warn!(%error, volume = %mount.label, "Could not untrack the cloud drive volume"),
		}
		context
			.volume_manager
			.unregister_cloud_volume(&fingerprint)
			.await;
	}

	/// Whether `fingerprint` is registered with a backend in this daemon.
	async fn mount_info(
		&self,
		library: &Library,
		fingerprint: &Option<String>,
		identifier: &str,
	) -> Option<DriveMountInfo> {
		let fingerprint = fingerprint.as_ref()?;
		let mounted = library
			.core_context()
			.volume_manager
			.get_volume(&VolumeFingerprint(fingerprint.clone()))
			.await
			.is_some_and(|v| v.backend.is_some());
		Some(DriveMountInfo {
			fingerprint: fingerprint.clone(),
			identifier: identifier.to_string(),
			mounted,
		})
	}

	fn google_drive_backend(&self, tokens: Arc<GoogleTokenSource>) -> Arc<dyn VolumeBackend> {
		let api = GoogleApi::new(self.http.clone(), self.endpoints.clone(), tokens);
		Arc::new(GoogleDriveBackend::new(api, None))
	}

	/// Mounts the account's Google Drive and remembers it on the account.
	pub async fn mount_drive(
		&self,
		library: &Arc<Library>,
		account: &mut StoredGoogleAccount,
	) -> Result<DriveMountInfo, IntegrationError> {
		let tokens = self.token_source(library.id(), account).await;
		let mount = CloudMount::google_drive(account);
		let info = self
			.mount_volume(
				library,
				&mount,
				self.google_drive_backend(tokens),
				account.drive_fingerprint.is_none(),
			)
			.await?;
		account.drive_fingerprint = Some(info.fingerprint.clone());
		IntegrationStore::for_library(library)
			.save_google_account(account)
			.await?;
		Ok(info)
	}

	/// Unmounts the account's Google Drive; the caller saves the account.
	pub async fn unmount_drive(
		&self,
		library: &Arc<Library>,
		account: &mut StoredGoogleAccount,
	) -> Result<(), IntegrationError> {
		self.unmount_volume(library, &CloudMount::google_drive(account))
			.await;
		account.drive_fingerprint = None;
		Ok(())
	}

	/// Re-registers the Google Drive and OneDrive volumes of every library
	/// after the daemon starts.
	pub async fn restore_drive_volumes(&self, libraries: &[Arc<Library>]) {
		for library in libraries {
			let store = IntegrationStore::for_library(library);
			match store.google_accounts().await {
				Ok(accounts) => {
					for account in accounts.iter().filter(|a| a.drive_fingerprint.is_some()) {
						let tokens = self.token_source(library.id(), account).await;
						let mount = CloudMount::google_drive(account);
						match self
							.register_mount(library, &mount, self.google_drive_backend(tokens), false)
							.await
						{
							Ok(_) => info!(account = %account.id, "Restored Google Drive volume"),
							Err(error) => warn!(%error, "Could not restore Google Drive volume"),
						}
					}
				}
				Err(error) => {
					warn!(%error, library = %library.id(), "Could not read Google accounts")
				}
			}
			match store.microsoft_accounts().await {
				Ok(accounts) => {
					for account in accounts.iter().filter(|a| a.drive_fingerprint.is_some()) {
						let backend = self.onedrive_backend(library, account).await;
						match self
							.register_mount(library, &CloudMount::onedrive(account), backend, false)
							.await
						{
							Ok(_) => info!(account = %account.id, "Restored OneDrive volume"),
							Err(error) => warn!(%error, "Could not restore OneDrive volume"),
						}
					}
				}
				Err(error) => {
					warn!(%error, library = %library.id(), "Could not read Microsoft accounts")
				}
			}
		}
	}

	/// Disconnects an account of either provider.
	///
	/// Google grants are revoked at Google when `revoke` is set. Microsoft has
	/// no revocation endpoint for a single app, so its grant stays listed at
	/// https://account.live.com/consent/Manage (or the tenant's My Apps page)
	/// until the user removes it there.
	pub async fn remove_account(
		&self,
		library: &Arc<Library>,
		account_id: &str,
		revoke: bool,
	) -> Result<(), IntegrationError> {
		let store = IntegrationStore::for_library(library);
		if store.microsoft_account(account_id).await?.is_some() {
			return self.remove_microsoft_account(library, account_id).await;
		}
		self.remove_google_account(library, account_id, revoke).await
	}

	/// Disconnects a Google account: unmounts Drive, revokes the grant at
	/// Google and deletes the stored secrets.
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

	/// Client-facing view of a Google account.
	pub async fn account_info(
		&self,
		library: &Library,
		account: &StoredGoogleAccount,
	) -> Result<IntegrationAccountInfo, IntegrationError> {
		// Accounts saved before modes existed signed in with the library's
		// own client when one was saved, and with WingDrive's otherwise.
		let mode = match account.mode {
			Some(mode) => mode,
			None => {
				let custom = IntegrationStore::for_library(library)
					.google_client()
					.await?;
				if custom.is_some_and(|c| c.client_id == account.client.client_id) {
					ConnectionMode::Custom
				} else {
					ConnectionMode::App
				}
			}
		};
		Ok(IntegrationAccountInfo {
			id: account.id.clone(),
			provider: IntegrationProvider::Google,
			email: account.email.clone(),
			name: account.name.clone(),
			picture: account.picture.clone(),
			services: account.granted_services(),
			mode,
			connected_at: account.connected_at,
			drive: self
				.mount_info(library, &account.drive_fingerprint, &account.email)
				.await,
		})
	}

	/// Client-facing view of a Microsoft account.
	pub async fn microsoft_account_info(
		&self,
		library: &Library,
		account: &StoredMicrosoftAccount,
	) -> IntegrationAccountInfo {
		IntegrationAccountInfo {
			id: account.id.clone(),
			provider: IntegrationProvider::Microsoft,
			email: account.email.clone(),
			name: account.name.clone(),
			picture: None,
			services: Vec::new(),
			mode: account.mode,
			connected_at: account.connected_at,
			drive: self
				.mount_info(library, &account.drive_fingerprint, &account.email)
				.await,
		}
	}

	/// Every connected account of `library`, Google first, then Microsoft.
	pub async fn list_accounts(
		&self,
		library: &Library,
	) -> Result<Vec<IntegrationAccountInfo>, IntegrationError> {
		let store = IntegrationStore::for_library(library);
		let mut infos = Vec::new();
		for account in store.google_accounts().await? {
			infos.push(self.account_info(library, &account).await?);
		}
		for account in store.microsoft_accounts().await? {
			infos.push(self.microsoft_account_info(library, &account).await);
		}
		Ok(infos)
	}

	/// Starts a Microsoft sign-in and returns the session ID and the sign-in URL.
	///
	/// Works like `start_google_connect`: the daemon listens for the redirect,
	/// then saves the account and mounts its OneDrive in the background.
	pub async fn start_microsoft_connect(
		self: &Arc<Self>,
		library: Arc<Library>,
		mode: ConnectionMode,
		login_hint: Option<String>,
	) -> Result<(Uuid, String), IntegrationError> {
		let store = IntegrationStore::for_library(&library);
		let client = self.microsoft_client_for(&store, mode).await?;
		let listener = oauth::LoopbackListener::bind_for("Microsoft", "localhost")
			.await
			.map_err(MicrosoftError::from)?;
		let redirect_uri = listener.redirect_uri();
		let pkce = oauth::Pkce::generate();
		let state = oauth::random_token();
		let url = microsoft::oauth::authorization_url(
			&self.microsoft_endpoints,
			&client,
			&redirect_uri,
			&state,
			&pkce.challenge,
			login_hint.as_deref(),
		);
		let session_id = Uuid::new_v4();
		let library_id = library.id();

		let this = self.clone();
		let task = tokio::spawn(async move {
			let result = async {
				let code = listener
					.wait_for_code(&state, CONNECT_TIMEOUT)
					.await
					.map_err(MicrosoftError::from)?;
				let tokens = microsoft::oauth::exchange_code(
					&this.http,
					&this.microsoft_endpoints,
					&client,
					&code,
					&pkce.verifier,
					&redirect_uri,
				)
				.await?;
				this.finish_microsoft_connect(&library, mode, client, tokens)
					.await
			}
			.await;
			let status = match result {
				Ok(account) => ConnectStatus::Completed { account },
				Err(error) => {
					warn!(%error, "Microsoft sign-in failed");
					ConnectStatus::Failed {
						message: error.to_string(),
					}
				}
			};
			this.finish_session(session_id, status).await;
		});
		self.add_session(session_id, library_id, task).await;
		Ok((session_id, url))
	}

	/// Saves the Microsoft account after a successful code exchange and mounts OneDrive.
	async fn finish_microsoft_connect(
		&self,
		library: &Arc<Library>,
		mode: ConnectionMode,
		client: MicrosoftClient,
		tokens: microsoft::oauth::TokenResponse,
	) -> Result<IntegrationAccountInfo, IntegrationError> {
		let refresh_token = tokens.refresh_token.clone().ok_or_else(|| {
			MicrosoftError::Authorization(
				"Microsoft returned no refresh token; the app registration must allow the offline_access permission".to_string(),
			)
		})?;
		let user = microsoft::oauth::fetch_user(
			&self.http,
			&self.microsoft_endpoints,
			&tokens.access_token,
		)
		.await?;
		let store = IntegrationStore::for_library(library);
		let previous = store.microsoft_account(&user.id).await?;
		let mut account = StoredMicrosoftAccount {
			id: user.id.clone(),
			email: user.email(),
			name: user.display_name.clone(),
			client,
			refresh_token,
			connected_at: previous
				.as_ref()
				.map(|p| p.connected_at)
				.unwrap_or_else(Utc::now),
			mode,
			drive_fingerprint: previous.as_ref().and_then(|p| p.drive_fingerprint.clone()),
		};
		{
			let _guard = self.microsoft_writes.lock().await;
			store.save_microsoft_account(&account).await?;
		}

		// A new grant replaces any token source built from the previous one.
		self.microsoft_tokens
			.lock()
			.await
			.remove(&(library.id(), account.id.clone()));
		let source = self.microsoft_token_source(library, &account).await;
		source.seed(tokens.access_token, tokens.expires_in).await;

		if let Err(error) = self.mount_onedrive(library, &mut account).await {
			warn!(%error, "Connected Microsoft account but could not mount OneDrive");
		}
		info!(account = %account.id, "Connected Microsoft account");
		Ok(self.microsoft_account_info(library, &account).await)
	}

	/// Shared token source for a Microsoft account, created on first use.
	///
	/// Rotated refresh tokens are written back to the account in the library,
	/// so the grant survives restarts past the 90-day lifetime of the first one.
	async fn microsoft_token_source(
		&self,
		library: &Arc<Library>,
		account: &StoredMicrosoftAccount,
	) -> Arc<MicrosoftTokenSource> {
		let mut sources = self.microsoft_tokens.lock().await;
		sources
			.entry((library.id(), account.id.clone()))
			.or_insert_with(|| {
				let weak = Arc::downgrade(library);
				let account_id = account.id.clone();
				let writes = self.microsoft_writes.clone();
				let on_rotate: microsoft::tokens::RotateHook = Box::new(move |token: String| {
					let Some(library) = weak.upgrade() else {
						return;
					};
					let account_id = account_id.clone();
					let writes = writes.clone();
					tokio::spawn(async move {
						let _guard = writes.lock().await;
						let store = IntegrationStore::for_library(&library);
						match store.microsoft_account(&account_id).await {
							Ok(Some(mut account)) => {
								account.refresh_token = token;
								if let Err(error) = store.save_microsoft_account(&account).await {
									warn!(%error, "Could not save the rotated Microsoft refresh token");
								}
							}
							// The account was removed meanwhile; nothing to keep.
							Ok(None) => {}
							Err(error) => {
								warn!(%error, "Could not read the Microsoft account to save its token")
							}
						}
					});
				});
				Arc::new(MicrosoftTokenSource::new(
					self.http.clone(),
					self.microsoft_endpoints.clone(),
					account.client.clone(),
					account.refresh_token.clone(),
					Some(on_rotate),
				))
			})
			.clone()
	}

	async fn onedrive_backend(
		&self,
		library: &Arc<Library>,
		account: &StoredMicrosoftAccount,
	) -> Arc<dyn VolumeBackend> {
		let tokens = self.microsoft_token_source(library, account).await;
		Arc::new(microsoft::OneDriveBackend::new(
			self.http.clone(),
			self.microsoft_endpoints.clone(),
			tokens,
		))
	}

	/// Mounts the account's OneDrive and remembers it on the account.
	pub async fn mount_onedrive(
		&self,
		library: &Arc<Library>,
		account: &mut StoredMicrosoftAccount,
	) -> Result<DriveMountInfo, IntegrationError> {
		let backend = self.onedrive_backend(library, account).await;
		let info = self
			.mount_volume(
				library,
				&CloudMount::onedrive(account),
				backend,
				account.drive_fingerprint.is_none(),
			)
			.await?;
		account.drive_fingerprint = Some(info.fingerprint.clone());
		self.update_microsoft_account(library, &account.id, |stored| {
			stored.drive_fingerprint = Some(info.fingerprint.clone());
		})
		.await?;
		Ok(info)
	}

	/// Changes a stored Microsoft account under the write lock, starting from
	/// what is stored so a concurrent token rotation is not lost.
	async fn update_microsoft_account(
		&self,
		library: &Library,
		account_id: &str,
		change: impl FnOnce(&mut StoredMicrosoftAccount),
	) -> Result<(), IntegrationError> {
		let _guard = self.microsoft_writes.lock().await;
		let store = IntegrationStore::for_library(library);
		if let Some(mut stored) = store.microsoft_account(account_id).await? {
			change(&mut stored);
			store.save_microsoft_account(&stored).await?;
		}
		Ok(())
	}

	/// Unmounts the account's OneDrive and saves the account.
	pub async fn unmount_onedrive(
		&self,
		library: &Arc<Library>,
		account: &mut StoredMicrosoftAccount,
	) -> Result<(), IntegrationError> {
		self.unmount_volume(library, &CloudMount::onedrive(account))
			.await;
		account.drive_fingerprint = None;
		self.update_microsoft_account(library, &account.id, |stored| {
			stored.drive_fingerprint = None;
		})
		.await
	}

	/// Disconnects a Microsoft account: unmounts OneDrive and deletes the stored secrets.
	pub async fn remove_microsoft_account(
		&self,
		library: &Arc<Library>,
		account_id: &str,
	) -> Result<(), IntegrationError> {
		let store = IntegrationStore::for_library(library);
		let account = store
			.microsoft_account(account_id)
			.await?
			.ok_or_else(|| IntegrationError::AccountNotFound(account_id.to_string()))?;
		self.unmount_volume(library, &CloudMount::onedrive(&account))
			.await;
		{
			let _guard = self.microsoft_writes.lock().await;
			store.delete_microsoft_account(account_id).await?;
		}
		self.microsoft_tokens
			.lock()
			.await
			.remove(&(library.id(), account_id.to_string()));
		info!(account = %account_id, "Removed Microsoft account");
		Ok(())
	}
}

/// The storage volume of one connected account.
struct CloudMount {
	service: CloudServiceType,
	/// File system name, also the fingerprint's volume kind.
	file_system: &'static str,
	/// Name shown in the sidebar, before the email.
	label: &'static str,
	/// `cloud_config.integration`, which keeps the volume out of the generic
	/// cloud volume loader.
	integration: &'static str,
	account_id: String,
	email: String,
}

impl CloudMount {
	fn google_drive(account: &StoredGoogleAccount) -> Self {
		Self {
			service: CloudServiceType::GoogleDrive,
			file_system: "GoogleDrive",
			label: "Google Drive",
			integration: "google",
			account_id: account.id.clone(),
			email: account.email.clone(),
		}
	}

	fn onedrive(account: &StoredMicrosoftAccount) -> Self {
		Self {
			service: CloudServiceType::OneDrive,
			file_system: "OneDrive",
			label: "OneDrive",
			integration: "microsoft",
			account_id: account.id.clone(),
			email: account.email.clone(),
		}
	}

	fn fingerprint(&self) -> VolumeFingerprint {
		VolumeFingerprint::from_network_volume(
			self.file_system,
			&format!("{}:{}", self.integration, self.account_id),
		)
	}

	fn name(&self) -> String {
		format!("{} ({})", self.label, self.email)
	}

	/// Path of the location at the drive's root.
	fn root(&self) -> String {
		format!("{}://{}/", self.service.scheme(), self.email)
	}

	fn volume(
		&self,
		backend: Arc<dyn VolumeBackend>,
		device_id: Uuid,
		capacity: Option<(u64, u64)>,
	) -> Volume {
		let now = Utc::now();
		let mount_point = PathBuf::from(format!("{}://{}", self.service.scheme(), self.email));
		let name = self.name();
		let (total, available) = capacity.unwrap_or((0, 0));
		Volume {
			id: Uuid::new_v5(
				&Uuid::NAMESPACE_URL,
				format!("{}:{}", self.service.scheme(), self.account_id).as_bytes(),
			),
			fingerprint: self.fingerprint(),
			device_id,
			name: name.clone(),
			library_id: None,
			is_tracked: false,
			mount_point: mount_point.clone(),
			mount_points: vec![mount_point],
			volume_type: crate::volume::types::VolumeType::Network,
			mount_type: crate::volume::types::MountType::Network,
			disk_type: crate::volume::types::DiskType::Unknown,
			file_system: crate::volume::types::FileSystem::Other(self.file_system.to_string()),
			total_capacity: total,
			available_space: available,
			is_read_only: false,
			is_mounted: true,
			hardware_id: None,
			backend: Some(backend),
			cloud_identifier: Some(self.email.clone()),
			cloud_config: Some(serde_json::json!({
				"integration": self.integration,
				"account_id": self.account_id,
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
}

/// Result type for integration operations.
pub type IntegrationResult<T> = Result<T, IntegrationError>;

impl From<IntegrationError> for crate::infra::action::error::ActionError {
	fn from(error: IntegrationError) -> Self {
		match error {
			IntegrationError::AccountNotFound(_) => Self::InvalidInput(error.to_string()),
			IntegrationError::Google(GoogleError::NotConfigured)
			| IntegrationError::AppClientMissing(_)
			| IntegrationError::CustomClientMissing(_) => Self::InvalidInput(error.to_string()),
			other => Self::Internal(other.to_string()),
		}
	}
}

impl From<IntegrationError> for crate::infra::query::QueryError {
	fn from(error: IntegrationError) -> Self {
		Self::Internal(error.to_string())
	}
}
