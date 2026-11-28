//! SourceManager: library-scoped wrapper around wing-archive Engine.

use std::{path::PathBuf, sync::Arc};

use tracing::{info, warn};
use uuid::Uuid;
use wing_archive::{Engine, EngineConfig};

use crate::crypto::key_manager::{KeyManager, KeyManagerError};

/// Manages archive data sources for a single library.
///
/// Config values of fields the adapter marks `secret` (tokens, passwords) never reach the
/// archive registry, which stores config as plain JSON: they live in the encrypted secret
/// store under [`SourceManager::secret_key`] and join the config only while a sync runs.
pub struct SourceManager {
	engine: Engine,
	installed_dir: PathBuf,
	secrets: Arc<KeyManager>,
	library_id: Uuid,
}

/// Points at a folder of adapters under development, loaded next to the installed ones.
pub const DEV_ADAPTERS_ENV: &str = "WINGDRIVE_DEV_ADAPTERS";

impl SourceManager {
	/// Create a new source manager rooted at the library's archive directory.
	///
	/// `installed_dir` is where the adapter store installs adapters for every library
	/// (`<data dir>/adapters/installed`); they are registered here so any library can use them.
	pub async fn new(
		library_path: PathBuf,
		installed_dir: PathBuf,
		secrets: Arc<KeyManager>,
		library_id: Uuid,
	) -> Result<Self, String> {
		let data_dir = library_path.join("archive");

		let config = EngineConfig {
			data_dir: data_dir.clone(),
		};
		let engine = Engine::new(config)
			.await
			.map_err(|e| format!("Failed to initialize archive engine: {e}"))?;

		let manager = Self {
			engine,
			installed_dir,
			secrets,
			library_id,
		};
		manager.reload_installed();
		info!("Source manager initialized at {}", library_path.display());
		Ok(manager)
	}

	/// Registers the adapters the store installed, and the ones in `WINGDRIVE_DEV_ADAPTERS` when
	/// that variable points at a folder of adapters under development.
	///
	/// Called at start and after every store install or update, so a new version takes effect
	/// without restarting the daemon.
	pub fn reload_installed(&self) {
		let dev_dir = std::env::var_os(DEV_ADAPTERS_ENV).map(PathBuf::from);
		for dir in std::iter::once(self.installed_dir.clone()).chain(dev_dir) {
			if !dir.is_dir() {
				continue;
			}
			if let Err(e) = Engine::load_script_adapters(&dir, self.engine.adapters()) {
				warn!(path = %dir.display(), error = %e, "failed to load adapters");
			}
		}
		self.disable_revoked();
	}

	/// Unloads installed adapters whose version the signed index revokes, so a revoked version
	/// stops running at once instead of waiting for the user to update it.
	fn disable_revoked(&self) {
		let Some(root) = self.installed_dir.parent() else {
			return;
		};
		let Ok(store) = wing_archive::store::Store::open(root.to_path_buf()) else {
			return;
		};
		let (Ok(Some(index)), Ok(installed)) = (store.cached_index(), store.installed()) else {
			return;
		};
		for (id, entry) in installed {
			if let Some(revocation) = index.revocation(&id, &entry.version) {
				warn!(
					adapter = %id,
					version = %entry.version,
					reason = %revocation.reason,
					"adapter version revoked by the store; disabled"
				);
				self.engine.adapters().unregister(&id);
			}
		}
	}

	/// Drops an adapter the store removed, so it no longer appears in this library.
	pub fn unregister_adapter(&self, adapter_id: &str) {
		self.engine.adapters().unregister(adapter_id);
	}

	/// List all sources.
	pub async fn list_sources(&self) -> Result<Vec<wing_archive::SourceInfo>, String> {
		self.engine
			.list_sources()
			.await
			.map_err(|e| format!("Failed to list sources: {e}"))
	}

	/// Create a new source. Secret values go to the secret store, the rest to the registry.
	pub async fn create_source(
		&self,
		name: &str,
		adapter_id: &str,
		mut config: serde_json::Value,
	) -> Result<wing_archive::SourceInfo, String> {
		let secrets = self.take_secrets(adapter_id, &mut config)?;
		let source = self
			.engine
			.create_source(name, adapter_id, config)
			.await
			.map_err(|e| format!("Failed to create source: {e}"))?;
		if let Err(e) = self.store_secrets(&source.id, secrets).await {
			let _ = self.engine.delete_source(&source.id).await;
			return Err(e);
		}
		Ok(source)
	}

	/// Rename a source or change its config.
	///
	/// `config` holds the fields to change; `null` removes a field. A secret field that is
	/// empty or missing keeps its stored value, so a form never has to show a token to save
	/// other changes.
	pub async fn update_source(
		&self,
		source_id: &str,
		name: Option<&str>,
		config: Option<serde_json::Value>,
	) -> Result<wing_archive::SourceInfo, String> {
		let source = self.get_source(source_id).await?;
		let new_config = match config {
			None => None,
			Some(mut changes) => {
				let secrets = self.take_secrets(&source.adapter_id, &mut changes)?;
				self.store_secrets(source_id, secrets).await?;
				let mut merged = source.config.clone();
				if let (Some(merged), Some(changes)) = (merged.as_object_mut(), changes.as_object())
				{
					for (key, value) in changes {
						if value.is_null() {
							merged.remove(key);
						} else {
							merged.insert(key.clone(), value.clone());
						}
					}
				}
				Some(merged)
			}
		};
		self.engine
			.update_source(source_id, name, new_config.as_ref())
			.await
			.map_err(|e| format!("Failed to update source: {e}"))
	}

	/// Delete a source and its stored secrets.
	pub async fn delete_source(&self, source_id: &str) -> Result<(), String> {
		let adapter_id = self.get_source(source_id).await.ok().map(|s| s.adapter_id);
		self.engine
			.delete_source(source_id)
			.await
			.map_err(|e| format!("Failed to delete source: {e}"))?;
		if let Some(adapter_id) = adapter_id {
			for key in self.secret_fields(&adapter_id) {
				let _ = self
					.secrets
					.delete_secret(&self.secret_key(source_id, &key))
					.await;
			}
		}
		Ok(())
	}

	/// Sync a source, with its secrets read from the secret store for this run.
	pub async fn sync_source(&self, source_id: &str) -> Result<wing_archive::SyncReport, String> {
		let source = self.get_source(source_id).await?;

		// Sources created before secrets left the registry still hold them in plain text.
		let mut config = source.config.clone();
		let legacy = self.take_secrets(&source.adapter_id, &mut config)?;
		if !legacy.is_empty() {
			self.store_secrets(source_id, legacy).await?;
			self.engine
				.update_source(source_id, None, Some(&config))
				.await
				.map_err(|e| format!("Failed to move secrets out of the registry: {e}"))?;
			info!(source_id, "Moved source secrets to the secret store");
		}

		let mut secrets = serde_json::Map::new();
		for key in self.secret_fields(&source.adapter_id) {
			match self
				.secrets
				.get_secret(&self.secret_key(source_id, &key))
				.await
			{
				Ok(bytes) => {
					let value = String::from_utf8(bytes)
						.map_err(|_| format!("Stored secret {key} is not text"))?;
					secrets.insert(key, serde_json::Value::String(value));
				}
				Err(KeyManagerError::KeyNotFound(_)) => {}
				Err(e) => return Err(format!("Failed to read secret {key}: {e}")),
			}
		}

		self.engine
			.sync_with(source_id, secrets)
			.await
			.map_err(|e| format!("Failed to sync source: {e}"))
	}

	/// A source's adapter, its config without secrets (legacy plain-text ones are hidden too)
	/// and the secret fields that have a stored value.
	pub async fn source_config(
		&self,
		source_id: &str,
	) -> Result<(String, serde_json::Value, Vec<String>), String> {
		let source = self.get_source(source_id).await?;
		let mut config = source.config.clone();
		let legacy = self.take_secrets(&source.adapter_id, &mut config)?;
		let mut stored = self.stored_secrets(source_id).await?;
		for (key, _) in legacy {
			if !stored.contains(&key) {
				stored.push(key);
			}
		}
		Ok((source.adapter_id, config, stored))
	}

	/// Which secret fields of a source have a stored value.
	pub async fn stored_secrets(&self, source_id: &str) -> Result<Vec<String>, String> {
		let source = self.get_source(source_id).await?;
		let mut stored = Vec::new();
		for key in self.secret_fields(&source.adapter_id) {
			if self
				.secrets
				.get_secret(&self.secret_key(source_id, &key))
				.await
				.is_ok()
			{
				stored.push(key);
			}
		}
		Ok(stored)
	}

	/// Secret store key of one secret config field of a source.
	fn secret_key(&self, source_id: &str, field: &str) -> String {
		format!("source-secret:{}:{source_id}:{field}", self.library_id)
	}

	/// Config fields the adapter marks secret.
	fn secret_fields(&self, adapter_id: &str) -> Vec<String> {
		self.engine
			.adapter_config_fields(adapter_id)
			.map(|fields| {
				fields
					.into_iter()
					.filter(|field| field.secret)
					.map(|field| field.key)
					.collect()
			})
			.unwrap_or_default()
	}

	/// Remove the secret fields from `config` and return the non-empty ones.
	fn take_secrets(
		&self,
		adapter_id: &str,
		config: &mut serde_json::Value,
	) -> Result<Vec<(String, String)>, String> {
		let Some(object) = config.as_object_mut() else {
			return Ok(Vec::new());
		};
		let mut taken = Vec::new();
		for key in self.secret_fields(adapter_id) {
			match object.remove(&key) {
				Some(serde_json::Value::String(value)) if !value.is_empty() => {
					taken.push((key, value))
				}
				Some(serde_json::Value::String(_)) | Some(serde_json::Value::Null) | None => {}
				Some(other) => return Err(format!("Secret field {key} must be text, got {other}")),
			}
		}
		Ok(taken)
	}

	async fn store_secrets(
		&self,
		source_id: &str,
		secrets: Vec<(String, String)>,
	) -> Result<(), String> {
		for (key, value) in secrets {
			self.secrets
				.set_secret(&self.secret_key(source_id, &key), value.as_bytes())
				.await
				.map_err(|e| format!("Failed to store secret {key}: {e}"))?;
		}
		Ok(())
	}

	async fn get_source(&self, source_id: &str) -> Result<wing_archive::SourceInfo, String> {
		self.engine
			.get_source(source_id)
			.await
			.map_err(|e| format!("Failed to read source: {e}"))
	}

	/// List items from a source.
	pub async fn list_items(
		&self,
		source_id: &str,
		limit: usize,
		offset: usize,
	) -> Result<Vec<wing_archive::db::ItemRow>, String> {
		self.engine
			.list_items(source_id, limit, offset)
			.await
			.map_err(|e| format!("Failed to list items: {e}"))
	}

	/// Adapters registered in this library. Updates come from the adapter store.
	pub fn list_adapters(&self) -> Vec<wing_archive::AdapterInfo> {
		self.engine.list_adapters_with_updates(None)
	}

	/// Adapters are updated through the store, which installs the new version for every library.
	pub fn update_adapter(
		&self,
		adapter_id: &str,
	) -> Result<wing_archive::AdapterUpdateResult, String> {
		Err(format!(
			"Update {adapter_id} from the adapter store (adapters.store.install)"
		))
	}

	/// Get config fields for an adapter.
	pub fn adapter_config_fields(
		&self,
		adapter_id: &str,
	) -> Result<Vec<wing_archive::adapter::script::ConfigField>, String> {
		self.engine
			.adapter_config_fields(adapter_id)
			.map_err(|e| format!("Failed to get adapter config: {e}"))
	}

	/// Config field labels in other languages: language, then field key.
	pub fn config_translations(
		&self,
		adapter_id: &str,
	) -> std::collections::BTreeMap<
		String,
		std::collections::BTreeMap<String, wing_archive::adapter::script::ConfigFieldTranslation>,
	> {
		self.engine
			.config_translations(adapter_id)
			.unwrap_or_default()
	}

	/// Config values the adapter suggests for this machine, keyed by config field.
	pub async fn detect_config(
		&self,
		adapter_id: &str,
	) -> Result<wing_archive::adapter::script::Suggestions, String> {
		self.engine
			.detect_config(adapter_id)
			.await
			.map_err(|e| format!("Failed to detect adapter config: {e}"))
	}

	/// Get the underlying engine.
	pub fn engine(&self) -> &Engine {
		&self.engine
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use serde_json::json;

	const MANIFEST: &str = r#"
[adapter]
id = "fixture"
name = "Fixture"
summary = "Local test adapter"
version = "1.0.0"
author = "WingDrive"
license = "MIT"
contract = 1
platforms = ["linux", "macos", "windows"]

[adapter.runtime]
kind = "python3"
entry = "sync.py"

[[adapter.config]]
key = "token"
name = "Token"
type = "string"
secret = true

[[adapter.config]]
key = "folder"
name = "Folder"
type = "string"

[data_type]
id = "fixture"
name = "Fixture"

[models.note]
fields.title = "string"
fields.body = "text"

[search]
primary_model = "note"
title = "title"
preview = "body"
search_fields = ["title", "body"]
"#;

	/// Log lines written while a test runs, at every level.
	#[derive(Clone, Default)]
	struct CapturedLogs(Arc<std::sync::Mutex<Vec<u8>>>);

	impl std::io::Write for CapturedLogs {
		fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
			self.0.lock().unwrap().extend_from_slice(buf);
			Ok(buf.len())
		}

		fn flush(&mut self) -> std::io::Result<()> {
			Ok(())
		}
	}

	impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for CapturedLogs {
		type Writer = Self;

		fn make_writer(&'a self) -> Self::Writer {
			self.clone()
		}
	}

	/// Files below `dir` whose bytes contain `needle`.
	fn files_containing(dir: &std::path::Path, needle: &[u8]) -> Vec<std::path::PathBuf> {
		let mut found = Vec::new();
		let mut pending = vec![dir.to_path_buf()];
		while let Some(dir) = pending.pop() {
			for entry in std::fs::read_dir(&dir).unwrap() {
				let path = entry.unwrap().path();
				if path.is_dir() {
					pending.push(path);
				} else if std::fs::read(&path)
					.unwrap()
					.windows(needle.len())
					.any(|window| window == needle)
				{
					found.push(path);
				}
			}
		}
		found
	}

	/// Installs the fixture adapter with a sync script that checks the token it receives
	/// and stores only whether it matched, so the source database never holds the token.
	async fn manager_with_checking_adapter(
		root: &std::path::Path,
		expected_token: &str,
	) -> (SourceManager, Arc<KeyManager>) {
		let installed = root.join("installed");
		let adapter = installed.join("fixture");
		std::fs::create_dir_all(&adapter).unwrap();
		std::fs::write(adapter.join("adapter.toml"), MANIFEST).unwrap();
		std::fs::write(
			adapter.join("sync.py"),
			format!(
				"import json, sys\ndata = json.load(sys.stdin)\nok = data['config'].get('token') == {expected_token:?}\nprint(json.dumps({{'upsert': 'note', 'external_id': 'token', 'fields': {{'title': 'token', 'body': 'received' if ok else 'missing'}}}}))\n"
			),
		)
		.unwrap();
		let keys = Arc::new(
			KeyManager::new_with_fallback(root.to_path_buf(), Some(root.join("device.key")))
				.unwrap(),
		);
		let manager = SourceManager::new(
			root.join("library"),
			installed,
			keys.clone(),
			Uuid::new_v4(),
		)
		.await
		.unwrap();
		(manager, keys)
	}

	/// Runs alone in a child process with a global subscriber. A scoped subscriber
	/// (`set_default`) misses events while other tests run in parallel: tracing caches each
	/// callsite's interest for the whole process, and a callsite first hit on a thread without
	/// a subscriber can stay silent, so the check saw no logs at all on CI.
	#[test]
	fn secrets_reach_the_adapter_but_not_the_source_database_or_logs() {
		const CHILD: &str = "WING_TEST_SECRETS_LOG_CHECK";
		const TOKEN: &str = "synthetic-token-7f3a9c";
		if std::env::var_os(CHILD).is_none() {
			let output = std::process::Command::new(std::env::current_exe().unwrap())
				.args([
					"data::manager::tests::secrets_reach_the_adapter_but_not_the_source_database_or_logs",
					"--exact",
					"--test-threads=1",
				])
				.env(CHILD, "1")
				.output()
				.unwrap();
			assert!(
				output.status.success(),
				"the check failed in its own process:\n{}",
				String::from_utf8_lossy(&output.stdout)
			);
			return;
		}

		let logs = CapturedLogs::default();
		let subscriber = tracing_subscriber::fmt()
			.with_max_level(tracing::Level::TRACE)
			.with_writer(logs.clone())
			.with_ansi(false)
			.finish();
		tracing::subscriber::set_global_default(subscriber).unwrap();

		tokio::runtime::Builder::new_current_thread()
			.enable_all()
			.build()
			.unwrap()
			.block_on(async {
				let temp = tempfile::tempdir().unwrap();
				let (manager, _keys) = manager_with_checking_adapter(temp.path(), TOKEN).await;
				let source = manager
					.create_source(
						"Checked",
						"fixture",
						json!({"token": TOKEN, "folder": "one"}),
					)
					.await
					.unwrap();
				manager
					.update_source(&source.id, None, Some(json!({"token": TOKEN})))
					.await
					.unwrap();
				let report = manager.sync_source(&source.id).await.unwrap();
				assert_eq!(report.error, None);

				let items = manager.list_items(&source.id, 10, 0).await.unwrap();
				assert_eq!(items[0].preview.as_deref(), Some("received"));
				assert_eq!(
					files_containing(&temp.path().join("library"), TOKEN.as_bytes()),
					Vec::<std::path::PathBuf>::new(),
					"the token was written to the library's files"
				);
				let logs = String::from_utf8(logs.0.lock().unwrap().clone()).unwrap();
				assert!(!logs.is_empty(), "the subscriber captured nothing");
				assert!(!logs.contains(TOKEN), "the token reached the logs:\n{logs}");
			});
	}

	#[tokio::test]
	async fn sync_moves_plain_text_secrets_out_of_the_registry() {
		const TOKEN: &str = "synthetic-legacy-token";
		let temp = tempfile::tempdir().unwrap();
		let (manager, keys) = manager_with_checking_adapter(temp.path(), TOKEN).await;
		// A source created before secrets left the registry.
		let source = manager
			.engine()
			.create_source(
				"Legacy",
				"fixture",
				json!({"token": TOKEN, "folder": "one"}),
			)
			.await
			.unwrap();
		assert_eq!(source.config["token"], TOKEN);
		let (_, config, stored) = manager.source_config(&source.id).await.unwrap();
		assert_eq!(config, json!({"folder": "one"}));
		assert_eq!(stored, vec!["token"]);

		let report = manager.sync_source(&source.id).await.unwrap();
		assert_eq!(report.error, None);
		let items = manager.list_items(&source.id, 10, 0).await.unwrap();
		assert_eq!(items[0].preview.as_deref(), Some("received"));
		let migrated = manager.get_source(&source.id).await.unwrap();
		assert_eq!(migrated.config, json!({"folder": "one"}));
		assert_eq!(
			keys.get_secret(&manager.secret_key(&source.id, "token"))
				.await
				.unwrap(),
			TOKEN.as_bytes()
		);
	}

	#[tokio::test]
	async fn source_lifecycle_keeps_secrets_out_of_registry() {
		let temp = tempfile::tempdir().unwrap();
		let installed = temp.path().join("installed");
		let adapter = installed.join("fixture");
		std::fs::create_dir_all(&adapter).unwrap();
		std::fs::write(adapter.join("adapter.toml"), MANIFEST).unwrap();
		std::fs::write(adapter.join("sync.py"), "import json, sys\ndata = json.load(sys.stdin)\nprint(json.dumps({'upsert': 'note', 'external_id': 'input', 'fields': {'title': 'input', 'body': json.dumps(data['config'])}}))\n").unwrap();
		let keys = Arc::new(
			KeyManager::new_with_fallback(
				temp.path().to_path_buf(),
				Some(temp.path().join("device.key")),
			)
			.unwrap(),
		);
		let manager = SourceManager::new(
			temp.path().join("library"),
			installed,
			keys.clone(),
			Uuid::new_v4(),
		)
		.await
		.unwrap();
		assert!(manager
			.list_adapters()
			.iter()
			.any(|adapter| adapter.id == "fixture"));
		assert_eq!(manager.adapter_config_fields("fixture").unwrap().len(), 2);
		let source = manager
			.create_source(
				"First",
				"fixture",
				json!({"token":"synthetic-token", "folder":"one", "remove":"yes"}),
			)
			.await
			.unwrap();
		assert_eq!(source.config, json!({"folder":"one", "remove":"yes"}));
		assert_eq!(manager.list_sources().await.unwrap().len(), 1);
		let (adapter_id, config, stored) = manager.source_config(&source.id).await.unwrap();
		assert_eq!(adapter_id, "fixture");
		assert_eq!(config, source.config);
		assert_eq!(stored, vec!["token"]);
		let updated = manager
			.update_source(
				&source.id,
				Some("Renamed"),
				Some(json!({"token":"", "folder":"two", "remove":null})),
			)
			.await
			.unwrap();
		assert_eq!(updated.name, "Renamed");
		assert_eq!(updated.config, json!({"folder":"two"}));
		assert_eq!(
			keys.get_secret(&manager.secret_key(&source.id, "token"))
				.await
				.unwrap(),
			b"synthetic-token"
		);
		assert!(manager
			.update_source(&source.id, None, Some(json!({"token":42})))
			.await
			.unwrap_err()
			.contains("must be text"));
		assert_eq!(
			manager.list_items(&source.id, 10, 0).await.unwrap().len(),
			0
		);
		let report = manager.sync_source(&source.id).await.unwrap();
		assert_eq!(report.error, None);
		let items = manager.list_items(&source.id, 10, 0).await.unwrap();
		assert_eq!(items.len(), 1);
		let received: serde_json::Value =
			serde_json::from_str(items[0].preview.as_deref().unwrap()).unwrap();
		assert_eq!(
			received,
			json!({"token": "synthetic-token", "folder": "two"})
		);
		manager.delete_source(&source.id).await.unwrap();
		assert!(manager.list_sources().await.unwrap().is_empty());
		assert!(keys
			.get_secret(&manager.secret_key(&source.id, "token"))
			.await
			.is_err());
		assert!(manager.source_config(&source.id).await.is_err());
		manager.unregister_adapter("fixture");
		assert!(!manager
			.list_adapters()
			.iter()
			.any(|adapter| adapter.id == "fixture"));
		assert!(manager
			.update_adapter("fixture")
			.unwrap_err()
			.contains("adapter store"));
	}
}
