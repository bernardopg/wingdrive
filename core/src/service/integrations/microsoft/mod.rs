//! # Microsoft integration
//!
//! Connects Microsoft accounts (personal, work or school) to a library and
//! mounts their OneDrive as a cloud volume. `oauth` runs the browser sign-in
//! against the Microsoft identity platform as a public client (PKCE, no client
//! secret), `tokens` keeps access tokens fresh and saves the refresh token
//! Microsoft rotates on every refresh, and `onedrive` adapts OpenDAL's OneDrive
//! service to those tokens. `MicrosoftEndpoints` holds every URL so tests can
//! point sign-in at a local fake server.

pub mod error;
pub mod oauth;
pub mod onedrive;
pub mod tokens;

pub use error::{MicrosoftError, MicrosoftResult};
pub use oauth::MicrosoftClient;
pub use onedrive::OneDriveBackend;
pub use tokens::MicrosoftTokenSource;

/// Base URLs for the Microsoft services the integration uses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MicrosoftEndpoints {
	/// Browser sign-in and consent page.
	pub authorize: String,
	/// Code exchange and token refresh.
	pub token: String,
	/// Microsoft Graph v1.0, without a trailing slash.
	pub graph: String,
}

impl Default for MicrosoftEndpoints {
	fn default() -> Self {
		// `common` accepts personal, work and school accounts.
		Self {
			authorize: "https://login.microsoftonline.com/common/oauth2/v2.0/authorize".to_string(),
			token: "https://login.microsoftonline.com/common/oauth2/v2.0/token".to_string(),
			graph: "https://graph.microsoft.com/v1.0".to_string(),
		}
	}
}

impl MicrosoftEndpoints {
	/// Points sign-in and Graph at one server, keeping Microsoft's path layout.
	///
	/// File operations go through OpenDAL, which always talks to the real Graph.
	pub fn for_base_url(base: &str) -> Self {
		let base = base.trim_end_matches('/');
		Self {
			authorize: format!("{base}/common/oauth2/v2.0/authorize"),
			token: format!("{base}/common/oauth2/v2.0/token"),
			graph: format!("{base}/v1.0"),
		}
	}
}
