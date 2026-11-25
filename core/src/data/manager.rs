//! SourceManager: library-scoped wrapper around wing-archive Engine.

use std::path::PathBuf;

use tracing::{info, warn};
use wing_archive::{Engine, EngineConfig};

/// Manages archive data sources for a single library.
pub struct SourceManager {
	engine: Engine,
	installed_dir: PathBuf,
}

/// Points at a folder of adapters under development, loaded next to the installed ones.
pub const DEV_ADAPTERS_ENV: &str = "WINGDRIVE_DEV_ADAPTERS";

impl SourceManager {
	/// Create a new source manager rooted at the library's archive directory.
	///
	/// `installed_dir` is where the adapter store installs adapters for every library
	/// (`<data dir>/adapters/installed`); they are registered here so any library can use them.
	pub async fn new(library_path: PathBuf, installed_dir: PathBuf) -> Result<Self, String> {
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

	/// Create a new source.
	pub async fn create_source(
		&self,
		name: &str,
		adapter_id: &str,
		config: serde_json::Value,
	) -> Result<wing_archive::SourceInfo, String> {
		self.engine
			.create_source(name, adapter_id, config)
			.await
			.map_err(|e| format!("Failed to create source: {e}"))
	}

	/// Delete a source.
	pub async fn delete_source(&self, source_id: &str) -> Result<(), String> {
		self.engine
			.delete_source(source_id)
			.await
			.map_err(|e| format!("Failed to delete source: {e}"))
	}

	/// Sync a source.
	pub async fn sync_source(&self, source_id: &str) -> Result<wing_archive::SyncReport, String> {
		self.engine
			.sync(source_id)
			.await
			.map_err(|e| format!("Failed to sync source: {e}"))
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
