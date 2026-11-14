//! End-to-end test of the Microsoft integration against a local fake of the
//! Microsoft identity platform and the Graph endpoints WingDrive calls itself
//! (`/me` and `/me/drive`). File operations go through OpenDAL to the real
//! Graph and are not exercised here.
//!
//! The test runs the product flow through `Core`: sign-in with WingDrive's
//! app registration, OneDrive mounted as a volume with a sidebar location,
//! the rotated refresh token written back to the library, the custom mode
//! with the user's own app registration, and removing the account.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use axum::body::Bytes;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde_json::json;
use tempfile::TempDir;
use wing_core::service::integrations::microsoft::MicrosoftClient;
use wing_core::service::integrations::store::IntegrationStore;
use wing_core::service::integrations::{
	ConnectStatus, ConnectionMode, IntegrationError, IntegrationProvider,
};
use wing_core::volume::VolumeFingerprint;
use wing_core::Core;

type TestResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

const APP_CLIENT: &str = "11111111-1111-1111-1111-111111111111";
const OWN_CLIENT: &str = "22222222-2222-2222-2222-222222222222";

#[derive(Default)]
struct FakeState {
	/// Valid refresh token and how many times it was rotated.
	refresh_token: String,
	rotations: u32,
	access_tokens: Vec<String>,
	/// Every token request carried a client secret, which public clients must not send.
	secrets_sent: u32,
	client_ids: Vec<String>,
}

type Shared = Arc<Mutex<FakeState>>;

fn form(body: &[u8]) -> HashMap<String, String> {
	url::form_urlencoded::parse(body).into_owned().collect()
}

async fn token(State(state): State<Shared>, body: Bytes) -> Response {
	let form = form(&body);
	let mut s = state.lock().unwrap();
	if form.contains_key("client_secret") {
		s.secrets_sent += 1;
	}
	s.client_ids
		.push(form.get("client_id").cloned().unwrap_or_default());
	match form.get("grant_type").map(String::as_str) {
		Some("authorization_code")
			if form.get("code").map(String::as_str) == Some("code-carla") =>
		{
			if form.get("code_verifier").is_none_or(|v| v.len() < 43) {
				return (
					StatusCode::BAD_REQUEST,
					Json(json!({"error": "invalid_request"})),
				)
					.into_response();
			}
			s.refresh_token = "rt-0".to_string();
			s.access_tokens.push("at-0".to_string());
			// Expires at once, so the next request refreshes and rotates.
			Json(json!({"access_token": "at-0", "expires_in": 1, "refresh_token": "rt-0", "token_type": "Bearer"}))
				.into_response()
		}
		Some("refresh_token") if form.get("refresh_token") == Some(&s.refresh_token) => {
			s.rotations += 1;
			let access = format!("at-{}", s.rotations);
			let refresh = format!("rt-{}", s.rotations);
			s.refresh_token = refresh.clone();
			s.access_tokens.push(access.clone());
			Json(json!({"access_token": access, "expires_in": 3600, "refresh_token": refresh, "token_type": "Bearer"}))
				.into_response()
		}
		_ => (
			StatusCode::BAD_REQUEST,
			Json(json!({"error": "invalid_grant", "error_description": "AADSTS70000"})),
		)
			.into_response(),
	}
}

fn authorized(state: &Shared, headers: &HeaderMap) -> bool {
	let token = headers
		.get("authorization")
		.and_then(|v| v.to_str().ok())
		.and_then(|v| v.strip_prefix("Bearer "))
		.unwrap_or_default();
	state
		.lock()
		.unwrap()
		.access_tokens
		.iter()
		.any(|t| t == token)
}

async fn me(State(state): State<Shared>, headers: HeaderMap) -> Response {
	if !authorized(&state, &headers) {
		return StatusCode::UNAUTHORIZED.into_response();
	}
	// Personal accounts often have no `mail`; the sign-in name stands in.
	Json(json!({"id": "ms-carla", "displayName": "Carla", "mail": null, "userPrincipalName": "Carla@Outlook.com"}))
		.into_response()
}

async fn drive(State(state): State<Shared>, headers: HeaderMap) -> Response {
	if !authorized(&state, &headers) {
		return StatusCode::UNAUTHORIZED.into_response();
	}
	Json(json!({"quota": {"total": 5368709120u64, "remaining": 4294967296u64}})).into_response()
}

async fn start_fake() -> (Shared, String) {
	let state: Shared = Arc::default();
	let app = Router::new()
		.route("/common/oauth2/v2.0/token", post(token))
		.route("/v1.0/me", get(me))
		.route("/v1.0/me/drive", get(drive))
		.with_state(state.clone());
	let listener = tokio::net::TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], 0)))
		.await
		.unwrap();
	let base_url = format!("http://{}", listener.local_addr().unwrap());
	tokio::spawn(async move {
		axum::serve(listener, app).await.unwrap();
	});
	(state, base_url)
}

async fn wait_for(
	core: &Core,
	library_id: uuid::Uuid,
	session_id: uuid::Uuid,
) -> Result<ConnectStatus, Box<dyn std::error::Error + Send + Sync>> {
	for _ in 0..100 {
		let status = core
			.context
			.integrations
			.connect_status(library_id, session_id)
			.await
			.unwrap();
		if status != ConnectStatus::Pending {
			return Ok(status);
		}
		tokio::time::sleep(Duration::from_millis(50)).await;
	}
	Err("sign-in did not finish".into())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn onedrive_sign_in_rotation_custom_client_and_removal() -> TestResult {
	let (fake, base_url) = start_fake().await;
	// Set before Core starts: the integration service reads the endpoints once.
	std::env::set_var("WINGDRIVE_MICROSOFT_ENDPOINTS_BASE", &base_url);
	std::env::set_var("WINGDRIVE_MICROSOFT_CLIENT_ID", APP_CLIENT);

	let temp = TempDir::new()?;
	let core = Core::new(temp.path().join("data")).await?;
	let library = core
		.libraries
		.create_library("Microsoft", None, core.context.clone())
		.await?;
	let integrations = core.context.integrations.clone();
	let store = IntegrationStore::for_library(&library);

	let clients = integrations.microsoft_clients(&store).await?;
	assert_eq!(
		clients.app.map(|a| a.client_id).as_deref(),
		Some(APP_CLIENT)
	);
	assert_eq!(clients.custom_client_id, None);

	// Sign in with WingDrive's app registration.
	let (session_id, auth_url) = integrations
		.start_microsoft_connect(library.clone(), ConnectionMode::App, None)
		.await?;
	let url = url::Url::parse(&auth_url)?;
	let query: HashMap<_, _> = url.query_pairs().into_owned().collect();
	assert_eq!(query["client_id"], APP_CLIENT);
	assert_eq!(query["code_challenge_method"], "S256");
	assert!(query["redirect_uri"].starts_with("http://localhost:"));
	assert!(query["redirect_uri"].ends_with("/oauth/callback"));
	let callback = query["redirect_uri"].replace("localhost", "127.0.0.1");
	let page = reqwest::get(format!(
		"{callback}?code=code-carla&state={}",
		query["state"]
	))
	.await?
	.text()
	.await?;
	assert!(page.contains("Microsoft"));

	let ConnectStatus::Completed { account } = wait_for(&core, library.id(), session_id).await?
	else {
		panic!("sign-in failed");
	};
	assert_eq!(account.provider, IntegrationProvider::Microsoft);
	assert_eq!(account.email, "carla@outlook.com");
	assert_eq!(account.name.as_deref(), Some("Carla"));
	assert_eq!(account.mode, ConnectionMode::App);
	let drive = account
		.drive
		.clone()
		.expect("OneDrive is mounted on connect");
	assert!(drive.mounted);
	assert_eq!(drive.identifier, "carla@outlook.com");

	// The volume and its sidebar location point at onedrive://.
	let volume = core
		.context
		.volume_manager
		.get_volume(&VolumeFingerprint(drive.fingerprint.clone()))
		.await
		.expect("volume registered");
	assert_eq!(
		volume.mount_point.to_string_lossy(),
		"onedrive://carla@outlook.com"
	);
	assert_eq!(volume.total_capacity, 5368709120);
	let locations = wing_core::location::LocationManager::new(core.context.events.as_ref().clone())
		.list_locations(&library)
		.await?;
	assert!(locations
		.iter()
		.any(|l| l.path.to_string_lossy() == "onedrive://carla@outlook.com/"));

	// Reading the quota refreshed the expired token; the rotated refresh token is stored.
	let mut stored = String::new();
	for _ in 0..100 {
		stored = store
			.microsoft_account("ms-carla")
			.await?
			.expect("account saved")
			.refresh_token;
		if stored != "rt-0" {
			break;
		}
		tokio::time::sleep(Duration::from_millis(50)).await;
	}
	assert_eq!(stored, fake.lock().unwrap().refresh_token);
	assert!(fake.lock().unwrap().rotations >= 1);
	assert_eq!(
		fake.lock().unwrap().secrets_sent,
		0,
		"public clients send no secret"
	);

	// The custom mode needs the user's own app registration.
	assert!(matches!(
		integrations
			.start_microsoft_connect(library.clone(), ConnectionMode::Custom, None)
			.await,
		Err(IntegrationError::CustomClientMissing(_))
	));
	store
		.set_microsoft_client(Some(&MicrosoftClient {
			client_id: OWN_CLIENT.into(),
		}))
		.await?;
	let (custom_session, custom_url) = integrations
		.start_microsoft_connect(
			library.clone(),
			ConnectionMode::Custom,
			Some("carla@outlook.com".into()),
		)
		.await?;
	let custom_query: HashMap<_, _> = url::Url::parse(&custom_url)?
		.query_pairs()
		.into_owned()
		.collect();
	assert_eq!(custom_query["client_id"], OWN_CLIENT);
	assert_eq!(custom_query["login_hint"], "carla@outlook.com");
	assert!(
		integrations
			.cancel_connect(library.id(), custom_session)
			.await
	);

	// Removing the account unmounts OneDrive and forgets the grant.
	integrations
		.remove_account(&library, "ms-carla", true)
		.await?;
	assert!(integrations.list_accounts(&library).await?.is_empty());
	assert!(core
		.context
		.volume_manager
		.get_volume(&VolumeFingerprint(drive.fingerprint))
		.await
		.is_none());
	Ok(())
}
