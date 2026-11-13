//! # Microsoft sign-in for desktop apps
//!
//! The flow the Microsoft identity platform documents for public clients: the
//! browser shows the sign-in page, Microsoft redirects to the daemon's loopback
//! listener (shared with Google), and the daemon trades the code for tokens.
//! Public clients have no secret, so PKCE (RFC 7636) is what binds the code to
//! this daemon. Every refresh returns a new refresh token, which the caller
//! must keep.

use serde::{Deserialize, Serialize};

use super::error::{MicrosoftError, MicrosoftResult};
use super::MicrosoftEndpoints;

/// Scopes WingDrive asks for: the profile to name the account, OneDrive
/// read and write, and `offline_access` for a refresh token.
///
/// `Files.ReadWrite` covers the user's own OneDrive and, unlike
/// `Files.ReadWrite.All`, needs no admin consent in work or school tenants.
pub const SCOPES: [&str; 6] = [
	"openid",
	"profile",
	"email",
	"offline_access",
	"User.Read",
	"Files.ReadWrite",
];

/// An app registered in Microsoft Entra as a public client
/// ("Mobile and desktop applications" platform).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MicrosoftClient {
	/// The "Application (client) ID", a GUID.
	pub client_id: String,
}

/// Builds the sign-in page URL.
pub fn authorization_url(
	endpoints: &MicrosoftEndpoints,
	client: &MicrosoftClient,
	redirect_uri: &str,
	state: &str,
	code_challenge: &str,
	login_hint: Option<&str>,
) -> String {
	let mut url =
		url::Url::parse(&endpoints.authorize).expect("authorize endpoint is a valid URL");
	{
		let mut query = url.query_pairs_mut();
		query
			.append_pair("client_id", &client.client_id)
			.append_pair("response_type", "code")
			.append_pair("response_mode", "query")
			.append_pair("redirect_uri", redirect_uri)
			.append_pair("scope", &SCOPES.join(" "))
			.append_pair("state", state)
			.append_pair("code_challenge", code_challenge)
			.append_pair("code_challenge_method", "S256")
			.append_pair("prompt", "select_account");
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
}

fn default_expires_in() -> u64 {
	3600
}

#[derive(Debug, Deserialize)]
struct OAuthErrorBody {
	error: String,
	#[serde(default)]
	error_description: Option<String>,
}

/// Profile of the signed-in account, from Graph `/me`.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MicrosoftUser {
	/// Stable account ID.
	pub id: String,
	#[serde(default)]
	pub display_name: Option<String>,
	#[serde(default)]
	pub mail: Option<String>,
	#[serde(default)]
	pub user_principal_name: Option<String>,
}

impl MicrosoftUser {
	/// The address to show: `mail`, or the sign-in name personal accounts use instead.
	pub fn email(&self) -> String {
		self.mail
			.clone()
			.filter(|m| !m.trim().is_empty())
			.or_else(|| self.user_principal_name.clone())
			.unwrap_or_else(|| self.id.clone())
			.to_lowercase()
	}
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
	endpoints: &MicrosoftEndpoints,
	pairs: &[(&str, &str)],
) -> MicrosoftResult<TokenResponse> {
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
		Ok(error) if error.error == "invalid_grant" => Err(MicrosoftError::InvalidGrant),
		Ok(error) => Err(MicrosoftError::Authorization(format!(
			"{}{}",
			error.error,
			error
				.error_description
				.map(|d| format!(": {d}"))
				.unwrap_or_default()
		))),
		Err(_) => Err(MicrosoftError::Api {
			status: status.as_u16(),
			message: String::from_utf8_lossy(&body).into_owned(),
		}),
	}
}

/// Trades an authorization code for tokens.
pub async fn exchange_code(
	http: &reqwest::Client,
	endpoints: &MicrosoftEndpoints,
	client: &MicrosoftClient,
	code: &str,
	code_verifier: &str,
	redirect_uri: &str,
) -> MicrosoftResult<TokenResponse> {
	let scope = SCOPES.join(" ");
	post_token_request(
		http,
		endpoints,
		&[
			("grant_type", "authorization_code"),
			("client_id", &client.client_id),
			("code", code),
			("code_verifier", code_verifier),
			("redirect_uri", redirect_uri),
			("scope", &scope),
		],
	)
	.await
}

/// Gets a fresh access token and a new refresh token.
pub async fn refresh_access_token(
	http: &reqwest::Client,
	endpoints: &MicrosoftEndpoints,
	client: &MicrosoftClient,
	refresh_token: &str,
) -> MicrosoftResult<TokenResponse> {
	let scope = SCOPES.join(" ");
	post_token_request(
		http,
		endpoints,
		&[
			("grant_type", "refresh_token"),
			("client_id", &client.client_id),
			("refresh_token", refresh_token),
			("scope", &scope),
		],
	)
	.await
}

/// Reads the profile of the account that owns `access_token`.
pub async fn fetch_user(
	http: &reqwest::Client,
	endpoints: &MicrosoftEndpoints,
	access_token: &str,
) -> MicrosoftResult<MicrosoftUser> {
	let response = http
		.get(format!(
			"{}/me?$select=id,displayName,mail,userPrincipalName",
			endpoints.graph
		))
		.bearer_auth(access_token)
		.send()
		.await?;
	let status = response.status();
	let body = response.bytes().await?;
	if !status.is_success() {
		return Err(MicrosoftError::Api {
			status: status.as_u16(),
			message: String::from_utf8_lossy(&body).into_owned(),
		});
	}
	Ok(serde_json::from_slice(&body)?)
}

/// Whether `id` looks like an Entra application ID (a GUID).
pub fn is_client_id(id: &str) -> bool {
	uuid::Uuid::parse_str(id.trim()).is_ok()
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn authorization_url_is_a_public_client_pkce_request() {
		let url = authorization_url(
			&MicrosoftEndpoints::default(),
			&MicrosoftClient {
				client_id: "00000000-0000-0000-0000-000000000001".into(),
			},
			"http://localhost:5555/oauth/callback",
			"state-1",
			"challenge-1",
			Some("ana@example.com"),
		);
		let parsed = url::Url::parse(&url).unwrap();
		let query: std::collections::HashMap<_, _> =
			parsed.query_pairs().into_owned().collect();
		assert_eq!(parsed.host_str(), Some("login.microsoftonline.com"));
		assert_eq!(query["code_challenge_method"], "S256");
		assert_eq!(query["redirect_uri"], "http://localhost:5555/oauth/callback");
		assert_eq!(query["login_hint"], "ana@example.com");
		assert!(query["scope"].contains("offline_access"));
		assert!(query["scope"].contains("Files.ReadWrite"));
		assert!(!query.contains_key("client_secret"));
	}

	#[test]
	fn email_falls_back_to_the_sign_in_name() {
		let user: MicrosoftUser = serde_json::from_str(
			r#"{"id":"abc","displayName":"Ana","mail":null,"userPrincipalName":"Ana@Outlook.com"}"#,
		)
		.unwrap();
		assert_eq!(user.email(), "ana@outlook.com");
	}

	#[test]
	fn client_ids_are_guids() {
		assert!(is_client_id("6731de76-14a6-49ae-97bc-6eba6914391e"));
		assert!(!is_client_id("1234-abc.apps.googleusercontent.com"));
	}
}
