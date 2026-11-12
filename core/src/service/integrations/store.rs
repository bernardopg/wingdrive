//! # Integration account store
//!
//! Connected accounts and custom OAuth clients live in the library's
//! `cloud_credentials` table, encrypted with the library key like cloud volume
//! credentials. Each library keeps its own accounts, so a shared or synced
//! library never carries another library's Google grants. Profile fields are
//! encrypted together with the tokens because an email address is personal data.

use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::google::oauth::{GoogleService, OAuthClient};
use crate::crypto::cloud_credentials::{CloudCredentialError, CloudCredentialManager};
use crate::crypto::key_manager::KeyManager;
use crate::infra::db::Database;

const GOOGLE_ACCOUNT_PREFIX: &str = "integration:google:account:";
const GOOGLE_CLIENT_KEY: &str = "integration:google:client";

/// A Google account connected to a library, with its grant.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StoredGoogleAccount {
	/// Google's stable account ID (`sub`).
	pub id: String,
	pub email: String,
	#[serde(default)]
	pub name: Option<String>,
	#[serde(default)]
	pub picture: Option<String>,
	/// Scopes Google granted, which can be fewer than requested.
	pub scopes: Vec<String>,
	/// The OAuth client that issued the refresh token; refreshing needs the same one.
	pub client: OAuthClient,
	pub refresh_token: String,
	pub connected_at: DateTime<Utc>,
	/// Fingerprint of the Drive volume, when Drive is mounted.
	#[serde(default)]
	pub drive_fingerprint: Option<String>,
}

impl StoredGoogleAccount {
	/// Services whose scopes the account granted.
	pub fn granted_services(&self) -> Vec<GoogleService> {
		[GoogleService::Drive, GoogleService::Photos]
			.into_iter()
			.filter(|service| service.is_granted(&self.scopes))
			.collect()
	}
}

/// Encrypted storage for one library's integration secrets.
pub struct IntegrationStore {
	credentials: CloudCredentialManager,
}

impl IntegrationStore {
	pub fn new(key_manager: Arc<KeyManager>, db: Arc<Database>, library_id: uuid::Uuid) -> Self {
		Self {
			credentials: CloudCredentialManager::new(key_manager, db, library_id),
		}
	}

	/// Store for `library`.
	pub fn for_library(library: &crate::library::Library) -> Self {
		Self::new(
			library.core_context().key_manager.clone(),
			library.db().clone(),
			library.id(),
		)
	}

	pub async fn save_google_account(
		&self,
		account: &StoredGoogleAccount,
	) -> Result<(), CloudCredentialError> {
		self.credentials
			.store_secret(
				&format!("{GOOGLE_ACCOUNT_PREFIX}{}", account.id),
				"GoogleAccount",
				account,
			)
			.await
	}

	pub async fn google_account(
		&self,
		id: &str,
	) -> Result<Option<StoredGoogleAccount>, CloudCredentialError> {
		self.credentials
			.get_secret(&format!("{GOOGLE_ACCOUNT_PREFIX}{id}"))
			.await
	}

	/// Every connected Google account, ordered by email.
	pub async fn google_accounts(&self) -> Result<Vec<StoredGoogleAccount>, CloudCredentialError> {
		let mut accounts = Vec::new();
		for key in self
			.credentials
			.list_secret_keys(GOOGLE_ACCOUNT_PREFIX)
			.await?
		{
			if let Some(account) = self.credentials.get_secret(&key).await? {
				accounts.push(account);
			}
		}
		accounts.sort_by(|a: &StoredGoogleAccount, b| a.email.cmp(&b.email));
		Ok(accounts)
	}

	pub async fn delete_google_account(&self, id: &str) -> Result<(), CloudCredentialError> {
		self.credentials
			.delete_secret(&format!("{GOOGLE_ACCOUNT_PREFIX}{id}"))
			.await
	}

	/// The OAuth client the user configured for this library, if any.
	pub async fn google_client(&self) -> Result<Option<OAuthClient>, CloudCredentialError> {
		self.credentials.get_secret(GOOGLE_CLIENT_KEY).await
	}

	pub async fn set_google_client(
		&self,
		client: Option<&OAuthClient>,
	) -> Result<(), CloudCredentialError> {
		match client {
			Some(client) => {
				self.credentials
					.store_secret(GOOGLE_CLIENT_KEY, "GoogleOAuthClient", client)
					.await
			}
			None => self.credentials.delete_secret(GOOGLE_CLIENT_KEY).await,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	async fn store() -> (tempfile::TempDir, IntegrationStore) {
		let dir = tempfile::tempdir().unwrap();
		let key_manager = Arc::new(
			KeyManager::new_with_fallback(
				dir.path().to_path_buf(),
				Some(dir.path().join("device_key")),
			)
			.unwrap(),
		);
		let db = Arc::new(
			Database::create(&dir.path().join("library.db"))
				.await
				.unwrap(),
		);
		db.migrate().await.unwrap();
		let store = IntegrationStore::new(key_manager, db, uuid::Uuid::new_v4());
		(dir, store)
	}

	fn account(id: &str, email: &str) -> StoredGoogleAccount {
		StoredGoogleAccount {
			id: id.into(),
			email: email.into(),
			name: Some("Test".into()),
			picture: None,
			scopes: vec![super::super::google::oauth::DRIVE_SCOPE.into()],
			client: OAuthClient {
				client_id: "cid".into(),
				client_secret: "secret".into(),
			},
			refresh_token: "refresh-token-value".into(),
			connected_at: Utc::now(),
			drive_fingerprint: None,
		}
	}

	#[tokio::test(flavor = "multi_thread")]
	async fn accounts_round_trip_and_are_encrypted_at_rest() {
		let (dir, store) = store().await;
		store
			.save_google_account(&account("2", "b@example.com"))
			.await
			.unwrap();
		store
			.save_google_account(&account("1", "a@example.com"))
			.await
			.unwrap();

		let listed = store.google_accounts().await.unwrap();
		assert_eq!(
			listed.iter().map(|a| a.email.as_str()).collect::<Vec<_>>(),
			vec!["a@example.com", "b@example.com"]
		);
		assert_eq!(listed[0].granted_services(), vec![GoogleService::Drive]);

		let mut updated = account("1", "a@example.com");
		updated.drive_fingerprint = Some("fp".into());
		store.save_google_account(&updated).await.unwrap();
		assert_eq!(
			store
				.google_account("1")
				.await
				.unwrap()
				.unwrap()
				.drive_fingerprint
				.as_deref(),
			Some("fp")
		);

		// Neither the token nor the email may appear in the database file.
		let mut raw = std::fs::read(dir.path().join("library.db")).unwrap();
		raw.extend(std::fs::read(dir.path().join("library.db-wal")).unwrap_or_default());
		let haystack = String::from_utf8_lossy(&raw);
		assert!(!haystack.contains("refresh-token-value"));
		assert!(!haystack.contains("a@example.com"));

		store.delete_google_account("1").await.unwrap();
		assert!(store.google_account("1").await.unwrap().is_none());
		assert_eq!(store.google_accounts().await.unwrap().len(), 1);
	}

	#[tokio::test(flavor = "multi_thread")]
	async fn custom_client_can_be_set_and_cleared() {
		let (_dir, store) = store().await;
		assert!(store.google_client().await.unwrap().is_none());
		let client = OAuthClient {
			client_id: "id".into(),
			client_secret: "sec".into(),
		};
		store.set_google_client(Some(&client)).await.unwrap();
		assert_eq!(store.google_client().await.unwrap(), Some(client));
		store.set_google_client(None).await.unwrap();
		assert!(store.google_client().await.unwrap().is_none());
	}
}
