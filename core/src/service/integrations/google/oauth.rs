//! # Google OAuth for installed apps
//!
//! WingDrive signs in to Google with the flow Google recommends for desktop
//! apps: the browser shows the consent page, Google redirects to a one-shot
//! HTTP listener on `127.0.0.1`, and the daemon trades the code for tokens.
//! PKCE (RFC 7636) binds the code to this daemon, so another local process that
//! sees the redirect cannot redeem it. A desktop client secret is not
//! confidential, Google still requires it in the token request.
//!
//! ## Example
//! ```rust,no_run
//! # async fn demo(http: reqwest::Client) -> Result<(), Box<dyn std::error::Error>> {
//! use wingdrive_core::service::integrations::google::oauth::*;
//! use wingdrive_core::service::integrations::google::{GoogleEndpoints, GoogleService};
//!
//! let endpoints = GoogleEndpoints::default();
//! let client = OAuthClient { client_id: "id".into(), client_secret: "secret".into() };
//! let listener = LoopbackListener::bind().await?;
//! let pkce = Pkce::generate();
//! let state = random_token();
//! let url = authorization_url(&endpoints, &client, &listener.redirect_uri(), &scopes_for(&[GoogleService::Drive]), &state, &pkce.challenge, None);
//! // Open `url` in the browser, then:
//! let code = listener.wait_for_code(&state, std::time::Duration::from_secs(600)).await?;
//! let tokens = exchange_code(&http, &endpoints, &client, &code, &pkce.verifier, &listener.redirect_uri()).await?;
//! # Ok(()) }
//! ```

use std::time::Duration;

use base64::Engine;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use specta::Type;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tracing::{debug, warn};

use super::endpoints::GoogleEndpoints;
use super::error::{GoogleError, GoogleResult};

/// Scopes every connection asks for, so the account can be named in the UI.
pub const IDENTITY_SCOPES: [&str; 3] = ["openid", "email", "profile"];

/// Full Drive access: browse, download, upload, edit and delete any file.
pub const DRIVE_SCOPE: &str = "https://www.googleapis.com/auth/drive";

/// Upload to Google Photos. Since March 2025 the Library API only lists and
/// edits media this app created, so the picker covers everything else.
pub const PHOTOS_SCOPES: [&str; 4] = [
	"https://www.googleapis.com/auth/photoslibrary.appendonly",
	"https://www.googleapis.com/auth/photoslibrary.readonly.appcreateddata",
	"https://www.googleapis.com/auth/photoslibrary.edit.appcreateddata",
	"https://www.googleapis.com/auth/photospicker.mediaitems.readonly",
];

/// Scope the Photos picker needs to read what the user selected.
pub const PHOTOS_PICKER_SCOPE: &str =
	"https://www.googleapis.com/auth/photospicker.mediaitems.readonly";

/// Scope needed to upload to Google Photos.
pub const PHOTOS_APPEND_SCOPE: &str = "https://www.googleapis.com/auth/photoslibrary.appendonly";

/// Path of the loopback redirect. Google accepts any path and port for
/// loopback redirects of desktop clients.
const CALLBACK_PATH: &str = "/oauth/callback";

/// Google services an account can enable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum GoogleService {
	/// Google Drive as a browsable cloud volume.
	Drive,
	/// Google Photos upload, picker import and app albums.
	Photos,
}

impl GoogleService {
	/// The scopes this service needs.
	pub fn scopes(self) -> Vec<&'static str> {
		match self {
			Self::Drive => vec![DRIVE_SCOPE],
			Self::Photos => PHOTOS_SCOPES.to_vec(),
		}
	}

	/// Whether `granted` covers this service.
	pub fn is_granted(self, granted: &[String]) -> bool {
		self.scopes()
			.iter()
			.all(|scope| granted.iter().any(|g| g == scope))
	}
}

/// OAuth client registered in a Google Cloud project as a "Desktop app".
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OAuthClient {
	pub client_id: String,
	pub client_secret: String,
}

/// PKCE verifier and its S256 challenge.
#[derive(Debug, Clone)]
pub struct Pkce {
	pub verifier: String,
	pub challenge: String,
}

impl Pkce {
	/// Generates a 43-character verifier from 256 random bits.
	pub fn generate() -> Self {
		let verifier = random_token();
		let challenge = base64::engine::general_purpose::URL_SAFE_NO_PAD
			.encode(Sha256::digest(verifier.as_bytes()));
		Self {
			verifier,
			challenge,
		}
	}
}

/// URL-safe token with 244 random bits from the OS RNG, used for `state` and PKCE.
pub fn random_token() -> String {
	let mut bytes = Vec::with_capacity(32);
	bytes.extend_from_slice(uuid::Uuid::new_v4().as_bytes());
	bytes.extend_from_slice(uuid::Uuid::new_v4().as_bytes());
	base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

/// All scopes to request for `services`, identity first.
pub fn scopes_for(services: &[GoogleService]) -> Vec<String> {
	let mut scopes: Vec<String> = IDENTITY_SCOPES.iter().map(|s| s.to_string()).collect();
	for service in services {
		for scope in service.scopes() {
			if !scopes.iter().any(|s| s == scope) {
				scopes.push(scope.to_string());
			}
		}
	}
	scopes
}

/// Builds the consent page URL.
///
/// `access_type=offline` with `prompt=consent` makes Google return a refresh
/// token every time, including when the user connects an account again to
/// grant another service.
pub fn authorization_url(
	endpoints: &GoogleEndpoints,
	client: &OAuthClient,
	redirect_uri: &str,
	scopes: &[String],
	state: &str,
	code_challenge: &str,
	login_hint: Option<&str>,
) -> String {
	let mut url = url::Url::parse(&endpoints.authorize).expect("authorize endpoint is a valid URL");
	{
		let mut query = url.query_pairs_mut();
		query
			.append_pair("client_id", &client.client_id)
			.append_pair("redirect_uri", redirect_uri)
			.append_pair("response_type", "code")
			.append_pair("scope", &scopes.join(" "))
			.append_pair("state", state)
			.append_pair("code_challenge", code_challenge)
			.append_pair("code_challenge_method", "S256")
			.append_pair("access_type", "offline")
			.append_pair("prompt", "consent select_account")
			.append_pair("include_granted_scopes", "true");
		if let Some(hint) = login_hint {
			query.append_pair("login_hint", hint);
		}
	}
	url.to_string()
}

/// Token endpoint response.
#[derive(Debug, Clone, Deserialize)]
pub struct TokenResponse {
	pub access_token: String,
	#[serde(default = "default_expires_in")]
	pub expires_in: u64,
	#[serde(default)]
	pub refresh_token: Option<String>,
	#[serde(default)]
	pub scope: Option<String>,
}

fn default_expires_in() -> u64 {
	3600
}

impl TokenResponse {
	/// Scopes Google actually granted; users can untick services on the consent page.
	pub fn granted_scopes(&self) -> Vec<String> {
		self.scope
			.as_deref()
			.unwrap_or_default()
			.split_whitespace()
			.map(str::to_string)
			.collect()
	}
}

#[derive(Debug, Deserialize)]
struct OAuthErrorBody {
	error: String,
	#[serde(default)]
	error_description: Option<String>,
}

/// Profile of the signed-in account.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleUserInfo {
	/// Stable account ID; emails can change, this cannot.
	pub sub: String,
	#[serde(default)]
	pub email: Option<String>,
	#[serde(default)]
	pub name: Option<String>,
	#[serde(default)]
	pub picture: Option<String>,
}

fn form_body(pairs: &[(&str, &str)]) -> String {
	let mut serializer = url::form_urlencoded::Serializer::new(String::new());
	for (key, value) in pairs {
		serializer.append_pair(key, value);
	}
	serializer.finish()
}

async fn post_token_request(
	http: &reqwest::Client,
	endpoints: &GoogleEndpoints,
	pairs: &[(&str, &str)],
) -> GoogleResult<TokenResponse> {
	let response = http
		.post(&endpoints.token)
		.header(
			reqwest::header::CONTENT_TYPE,
			"application/x-www-form-urlencoded",
		)
		.body(form_body(pairs))
		.send()
		.await?;
	let status = response.status();
	let body = response.bytes().await?;
	if status.is_success() {
		return Ok(serde_json::from_slice(&body)?);
	}
	match serde_json::from_slice::<OAuthErrorBody>(&body) {
		Ok(error) if error.error == "invalid_grant" => Err(GoogleError::InvalidGrant),
		Ok(error) => Err(GoogleError::Authorization(format!(
			"{}{}",
			error.error,
			error
				.error_description
				.map(|d| format!(": {d}"))
				.unwrap_or_default()
		))),
		Err(_) => Err(GoogleError::Api {
			status: status.as_u16(),
			message: String::from_utf8_lossy(&body).into_owned(),
		}),
	}
}

/// Trades an authorization code for tokens.
pub async fn exchange_code(
	http: &reqwest::Client,
	endpoints: &GoogleEndpoints,
	client: &OAuthClient,
	code: &str,
	code_verifier: &str,
	redirect_uri: &str,
) -> GoogleResult<TokenResponse> {
	post_token_request(
		http,
		endpoints,
		&[
			("grant_type", "authorization_code"),
			("code", code),
			("code_verifier", code_verifier),
			("redirect_uri", redirect_uri),
			("client_id", &client.client_id),
			("client_secret", &client.client_secret),
		],
	)
	.await
}

/// Gets a fresh access token from a refresh token.
///
/// Returns `GoogleError::InvalidGrant` when the user revoked access or the
/// grant expired (seven days for apps in testing mode), so the UI can ask to
/// reconnect instead of retrying.
pub async fn refresh_access_token(
	http: &reqwest::Client,
	endpoints: &GoogleEndpoints,
	client: &OAuthClient,
	refresh_token: &str,
) -> GoogleResult<TokenResponse> {
	post_token_request(
		http,
		endpoints,
		&[
			("grant_type", "refresh_token"),
			("refresh_token", refresh_token),
			("client_id", &client.client_id),
			("client_secret", &client.client_secret),
		],
	)
	.await
}

/// Revokes a token at Google, which also removes the app from the account's
/// third-party access list.
pub async fn revoke_token(
	http: &reqwest::Client,
	endpoints: &GoogleEndpoints,
	token: &str,
) -> GoogleResult<()> {
	let response = http
		.post(&endpoints.revoke)
		.header(
			reqwest::header::CONTENT_TYPE,
			"application/x-www-form-urlencoded",
		)
		.body(form_body(&[("token", token)]))
		.send()
		.await?;
	let status = response.status();
	// 400 means the token is already invalid, which is the state revocation wants.
	if status.is_success() || status.as_u16() == 400 {
		Ok(())
	} else {
		Err(GoogleError::Api {
			status: status.as_u16(),
			message: response.text().await.unwrap_or_default(),
		})
	}
}

/// Reads the profile of the account that owns `access_token`.
pub async fn fetch_user_info(
	http: &reqwest::Client,
	endpoints: &GoogleEndpoints,
	access_token: &str,
) -> GoogleResult<GoogleUserInfo> {
	let response = http
		.get(&endpoints.userinfo)
		.bearer_auth(access_token)
		.send()
		.await?;
	let status = response.status();
	let body = response.bytes().await?;
	if !status.is_success() {
		return Err(GoogleError::Api {
			status: status.as_u16(),
			message: String::from_utf8_lossy(&body).into_owned(),
		});
	}
	Ok(serde_json::from_slice(&body)?)
}

/// Parameters Google appends to the redirect.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct CallbackParams {
	pub code: Option<String>,
	pub state: Option<String>,
	pub error: Option<String>,
}

/// Parses the request line of the redirect, such as
/// `GET /oauth/callback?code=x&state=y HTTP/1.1`.
///
/// Returns `None` for requests to other paths, like the browser's favicon probe.
pub fn parse_callback_request(request_line: &str) -> Option<CallbackParams> {
	let mut parts = request_line.split_whitespace();
	let method = parts.next()?;
	let target = parts.next()?;
	if method != "GET" {
		return None;
	}
	let url = url::Url::parse(&format!("http://127.0.0.1{target}")).ok()?;
	if url.path() != CALLBACK_PATH {
		return None;
	}
	let mut params = CallbackParams::default();
	for (key, value) in url.query_pairs() {
		match key.as_ref() {
			"code" => params.code = Some(value.into_owned()),
			"state" => params.state = Some(value.into_owned()),
			"error" => params.error = Some(value.into_owned()),
			_ => {}
		}
	}
	Some(params)
}

/// One-shot HTTP listener on a random loopback port that receives the redirect.
#[derive(Debug)]
pub struct LoopbackListener {
	listener: TcpListener,
	port: u16,
}

impl LoopbackListener {
	/// Binds `127.0.0.1` on a port the OS picks.
	pub async fn bind() -> std::io::Result<Self> {
		let listener = TcpListener::bind(("127.0.0.1", 0)).await?;
		let port = listener.local_addr()?.port();
		Ok(Self { listener, port })
	}

	/// The redirect URI registered in the authorization request.
	pub fn redirect_uri(&self) -> String {
		format!("http://127.0.0.1:{}{CALLBACK_PATH}", self.port)
	}

	/// Waits for the redirect and returns the authorization code.
	///
	/// A redirect with the wrong `state` is answered and ignored rather than
	/// failing the sign-in, so a stale browser tab cannot cancel a new attempt.
	pub async fn wait_for_code(
		self,
		expected_state: &str,
		timeout: Duration,
	) -> GoogleResult<String> {
		let wait = async {
			loop {
				let (mut stream, _) = self.listener.accept().await?;
				let mut buffer = vec![0u8; 8192];
				let mut read = 0;
				// Only the request line matters; stop at the end of the headers.
				while read < buffer.len() {
					let n = match tokio::time::timeout(
						Duration::from_secs(5),
						stream.read(&mut buffer[read..]),
					)
					.await
					{
						Ok(Ok(n)) => n,
						_ => 0,
					};
					if n == 0 {
						break;
					}
					read += n;
					if buffer[..read].windows(4).any(|w| w == b"\r\n\r\n") {
						break;
					}
				}
				let request = String::from_utf8_lossy(&buffer[..read]);
				let request_line = request.lines().next().unwrap_or_default();
				let Some(params) = parse_callback_request(request_line) else {
					let _ = respond(&mut stream, 404, "Not found").await;
					continue;
				};
				if params.state.as_deref() != Some(expected_state) {
					warn!("Ignoring OAuth redirect with an unexpected state");
					let _ = respond(&mut stream, 400, &callback_page(false)).await;
					continue;
				}
				if let Some(error) = params.error {
					let _ = respond(&mut stream, 200, &callback_page(false)).await;
					return Err(GoogleError::Authorization(error));
				}
				let Some(code) = params.code else {
					let _ = respond(&mut stream, 400, &callback_page(false)).await;
					return Err(GoogleError::Authorization(
						"redirect carried no authorization code".to_string(),
					));
				};
				let _ = respond(&mut stream, 200, &callback_page(true)).await;
				debug!("Received OAuth authorization code on the loopback listener");
				return Ok(code);
			}
		};
		match tokio::time::timeout(timeout, wait).await {
			Ok(result) => result,
			Err(_) => Err(GoogleError::Expired),
		}
	}
}

async fn respond(
	stream: &mut tokio::net::TcpStream,
	status: u16,
	body: &str,
) -> std::io::Result<()> {
	let reason = match status {
		200 => "OK",
		400 => "Bad Request",
		_ => "Not Found",
	};
	let response = format!(
		"HTTP/1.1 {status} {reason}\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\nCache-Control: no-store\r\n\r\n{body}",
		body.len()
	);
	stream.write_all(response.as_bytes()).await?;
	stream.flush().await
}

/// Page the browser shows after the redirect. It carries both languages
/// because the daemon does not know the UI language.
fn callback_page(success: bool) -> String {
	let (title, en, pt) = if success {
		(
			"WingDrive",
			"Your Google account is connected. You can close this tab and go back to WingDrive.",
			"Sua conta Google foi conectada. Você pode fechar esta aba e voltar ao WingDrive.",
		)
	} else {
		(
			"WingDrive",
			"Google sign-in did not finish. Go back to WingDrive and try again.",
			"O login no Google não terminou. Volte ao WingDrive e tente de novo.",
		)
	};
	format!(
		"<!doctype html><html><head><meta charset=\"utf-8\"><title>{title}</title>\
		<style>body{{font-family:system-ui,sans-serif;max-width:32rem;margin:4rem auto;padding:0 1rem;line-height:1.5}}</style>\
		</head><body><h1>{title}</h1><p>{en}</p><p lang=\"pt-BR\">{pt}</p></body></html>"
	)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn pkce_challenge_is_s256_of_the_verifier() {
		let pkce = Pkce::generate();
		assert!(pkce.verifier.len() >= 43, "RFC 7636 needs 43 to 128 chars");
		let expected = base64::engine::general_purpose::URL_SAFE_NO_PAD
			.encode(Sha256::digest(pkce.verifier.as_bytes()));
		assert_eq!(pkce.challenge, expected);
		assert_ne!(Pkce::generate().verifier, pkce.verifier);
	}

	#[test]
	fn rfc7636_appendix_b_vector() {
		let challenge = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(Sha256::digest(
			b"dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk",
		));
		assert_eq!(challenge, "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM");
	}

	#[test]
	fn scopes_include_identity_and_deduplicate() {
		let scopes = scopes_for(&[
			GoogleService::Drive,
			GoogleService::Photos,
			GoogleService::Drive,
		]);
		assert_eq!(&scopes[..3], &["openid", "email", "profile"]);
		assert_eq!(scopes.iter().filter(|s| *s == DRIVE_SCOPE).count(), 1);
		assert!(scopes.iter().any(|s| s == PHOTOS_PICKER_SCOPE));
	}

	#[test]
	fn service_grant_check_needs_every_scope() {
		let mut granted = vec![DRIVE_SCOPE.to_string()];
		assert!(GoogleService::Drive.is_granted(&granted));
		assert!(!GoogleService::Photos.is_granted(&granted));
		granted.extend(PHOTOS_SCOPES.iter().map(|s| s.to_string()));
		assert!(GoogleService::Photos.is_granted(&granted));
	}

	#[test]
	fn authorization_url_carries_pkce_and_offline_access() {
		let client = OAuthClient {
			client_id: "client-1".into(),
			client_secret: "s3cret".into(),
		};
		let url = authorization_url(
			&GoogleEndpoints::default(),
			&client,
			"http://127.0.0.1:4000/oauth/callback",
			&scopes_for(&[GoogleService::Drive]),
			"state-1",
			"challenge-1",
			Some("me@example.com"),
		);
		let parsed = url::Url::parse(&url).unwrap();
		let pairs: std::collections::HashMap<_, _> = parsed.query_pairs().into_owned().collect();
		assert_eq!(pairs["client_id"], "client-1");
		assert_eq!(pairs["code_challenge_method"], "S256");
		assert_eq!(pairs["code_challenge"], "challenge-1");
		assert_eq!(pairs["access_type"], "offline");
		assert_eq!(pairs["state"], "state-1");
		assert_eq!(pairs["login_hint"], "me@example.com");
		assert!(pairs["scope"].contains(DRIVE_SCOPE));
		assert!(!url.contains("s3cret"));
	}

	#[test]
	fn callback_parsing_ignores_other_paths() {
		assert_eq!(parse_callback_request("GET /favicon.ico HTTP/1.1"), None);
		assert_eq!(
			parse_callback_request("POST /oauth/callback HTTP/1.1"),
			None
		);
		let params =
			parse_callback_request("GET /oauth/callback?code=4%2F0Ab&state=xyz HTTP/1.1").unwrap();
		assert_eq!(params.code.as_deref(), Some("4/0Ab"));
		assert_eq!(params.state.as_deref(), Some("xyz"));
		let denied =
			parse_callback_request("GET /oauth/callback?error=access_denied&state=xyz HTTP/1.1")
				.unwrap();
		assert_eq!(denied.error.as_deref(), Some("access_denied"));
	}

	async fn send(port: u16, path: &str) -> String {
		let mut stream = tokio::net::TcpStream::connect(("127.0.0.1", port))
			.await
			.unwrap();
		stream
			.write_all(format!("GET {path} HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n").as_bytes())
			.await
			.unwrap();
		let mut response = String::new();
		stream.read_to_string(&mut response).await.unwrap();
		response
	}

	#[tokio::test]
	async fn loopback_listener_returns_the_code_and_skips_wrong_state() {
		let listener = LoopbackListener::bind().await.unwrap();
		let port = listener.port;
		assert!(listener.redirect_uri().starts_with("http://127.0.0.1:"));
		let waiter = tokio::spawn(async move {
			listener
				.wait_for_code("good", Duration::from_secs(10))
				.await
		});
		assert!(send(port, "/favicon.ico").await.starts_with("HTTP/1.1 404"));
		assert!(send(port, "/oauth/callback?code=x&state=bad")
			.await
			.starts_with("HTTP/1.1 400"));
		let ok = send(port, "/oauth/callback?code=the-code&state=good").await;
		assert!(ok.starts_with("HTTP/1.1 200"));
		assert!(ok.contains("Você pode fechar esta aba"));
		assert_eq!(waiter.await.unwrap().unwrap(), "the-code");
	}

	#[tokio::test]
	async fn loopback_listener_reports_denial_and_timeout() {
		let listener = LoopbackListener::bind().await.unwrap();
		let port = listener.port;
		let waiter =
			tokio::spawn(async move { listener.wait_for_code("s", Duration::from_secs(10)).await });
		send(port, "/oauth/callback?error=access_denied&state=s").await;
		assert!(matches!(
			waiter.await.unwrap(),
			Err(GoogleError::Authorization(e)) if e == "access_denied"
		));

		let listener = LoopbackListener::bind().await.unwrap();
		let result = listener.wait_for_code("s", Duration::from_millis(50)).await;
		assert!(matches!(result, Err(GoogleError::Expired)));
	}
}
