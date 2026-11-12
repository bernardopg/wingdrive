//! # Google endpoints
//!
//! Every Google URL the integration calls lives in one struct so tests can point
//! the whole client at a local fake server with `GoogleEndpoints::for_base_url`.

/// Base URLs for the Google services the integration uses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GoogleEndpoints {
	/// Browser consent page.
	pub authorize: String,
	/// Code exchange and token refresh.
	pub token: String,
	/// Token revocation.
	pub revoke: String,
	/// OpenID Connect user profile.
	pub userinfo: String,
	/// Drive v3 metadata API, without a trailing slash.
	pub drive: String,
	/// Drive v3 upload API, without a trailing slash.
	pub drive_upload: String,
	/// Photos Library API, without a trailing slash.
	pub photos: String,
	/// Photos Picker API, without a trailing slash.
	pub picker: String,
}

impl Default for GoogleEndpoints {
	fn default() -> Self {
		Self {
			authorize: "https://accounts.google.com/o/oauth2/v2/auth".to_string(),
			token: "https://oauth2.googleapis.com/token".to_string(),
			revoke: "https://oauth2.googleapis.com/revoke".to_string(),
			userinfo: "https://openidconnect.googleapis.com/v1/userinfo".to_string(),
			drive: "https://www.googleapis.com/drive/v3".to_string(),
			drive_upload: "https://www.googleapis.com/upload/drive/v3".to_string(),
			photos: "https://photoslibrary.googleapis.com".to_string(),
			picker: "https://photospicker.googleapis.com".to_string(),
		}
	}
}

impl GoogleEndpoints {
	/// Points every service at one server, keeping Google's path layout.
	pub fn for_base_url(base: &str) -> Self {
		let base = base.trim_end_matches('/');
		Self {
			authorize: format!("{base}/o/oauth2/v2/auth"),
			token: format!("{base}/token"),
			revoke: format!("{base}/revoke"),
			userinfo: format!("{base}/v1/userinfo"),
			drive: format!("{base}/drive/v3"),
			drive_upload: format!("{base}/upload/drive/v3"),
			photos: format!("{base}/photos"),
			picker: format!("{base}/picker"),
		}
	}
}
