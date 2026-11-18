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

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn default_endpoints_are_google_over_https() {
		let endpoints = GoogleEndpoints::default();
		for url in [
			&endpoints.authorize,
			&endpoints.token,
			&endpoints.revoke,
			&endpoints.userinfo,
			&endpoints.drive,
			&endpoints.drive_upload,
			&endpoints.photos,
			&endpoints.picker,
		] {
			assert!(url.starts_with("https://"), "{url}");
			assert!(!url.ends_with('/'), "{url}");
		}
		assert!(endpoints
			.authorize
			.starts_with("https://accounts.google.com/"));
	}

	#[test]
	fn a_base_url_keeps_google_paths_and_drops_the_trailing_slash() {
		let endpoints = GoogleEndpoints::for_base_url("http://127.0.0.1:9000/");
		assert_eq!(
			endpoints.authorize,
			"http://127.0.0.1:9000/o/oauth2/v2/auth"
		);
		assert_eq!(endpoints.token, "http://127.0.0.1:9000/token");
		assert_eq!(endpoints.revoke, "http://127.0.0.1:9000/revoke");
		assert_eq!(endpoints.userinfo, "http://127.0.0.1:9000/v1/userinfo");
		assert_eq!(endpoints.drive, "http://127.0.0.1:9000/drive/v3");
		assert_eq!(
			endpoints.drive_upload,
			"http://127.0.0.1:9000/upload/drive/v3"
		);
		assert_eq!(endpoints.photos, "http://127.0.0.1:9000/photos");
		assert_eq!(endpoints.picker, "http://127.0.0.1:9000/picker");
		assert_eq!(
			GoogleEndpoints::for_base_url("http://127.0.0.1:9000"),
			endpoints
		);
	}
}
