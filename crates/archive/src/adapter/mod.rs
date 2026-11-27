//! Adapter system: trait definition, registry, sync reporting.

pub mod sandbox;
pub mod script;

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, RwLock};

use serde::{Deserialize, Serialize};

use crate::db::SourceDb;
use crate::error::Result;
use crate::safety::TrustTier;

/// Report returned after an adapter sync completes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncReport {
	pub records_upserted: u64,
	pub records_deleted: u64,
	pub links_created: u64,
	pub links_removed: u64,
	pub duration_ms: u64,
	pub error: Option<String>,
}

/// Info about a registered adapter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterInfo {
	pub id: String,
	pub name: String,
	pub description: String,
	pub version: String,
	pub author: String,
	pub data_type: String,
	pub kind: AdapterKind,
	pub trust_tier: TrustTier,
	pub icon_svg: Option<String>,
	pub update_available: bool,
}

/// Whether an adapter is compiled-in or script-based.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AdapterKind {
	Native,
	Script,
}

/// Trait that all adapters implement.
pub trait Adapter: Send + Sync + 'static {
	fn id(&self) -> &str;
	fn name(&self) -> &str;
	fn data_type(&self) -> &str;
	fn description(&self) -> &str {
		""
	}
	fn version(&self) -> &str {
		"0.0.0"
	}
	fn author(&self) -> &str {
		""
	}
	fn icon_svg(&self) -> Option<&str> {
		None
	}
	fn trust_tier(&self) -> TrustTier {
		TrustTier::External
	}
	/// The record schema sources of this adapter use, wherever the adapter was loaded from.
	fn schema(&self) -> Option<&crate::schema::DataTypeSchema> {
		None
	}
	/// The fields a user fills when adding a source of this adapter.
	fn config_fields(&self) -> &[script::ConfigField] {
		&[]
	}
	/// Texts in other languages, from `[adapter.i18n.<lang>]`.
	fn translations(
		&self,
	) -> Option<&std::collections::BTreeMap<String, script::AdapterTranslation>> {
		None
	}
	/// Config values the adapter finds on this machine, keyed by config field.
	fn detect<'a>(&'a self) -> Pin<Box<dyn Future<Output = script::Suggestions> + Send + 'a>> {
		Box::pin(async { script::Suggestions::new() })
	}
	fn sync<'a>(
		&'a self,
		db: &'a SourceDb,
		config: &'a serde_json::Value,
	) -> Pin<Box<dyn Future<Output = Result<SyncReport>> + Send + 'a>>;
}

/// Registry of available adapters.
pub struct AdapterRegistry {
	adapters: RwLock<HashMap<String, Arc<dyn Adapter>>>,
}

impl AdapterRegistry {
	pub fn new() -> Self {
		Self {
			adapters: RwLock::new(HashMap::new()),
		}
	}

	pub fn register(&self, adapter: Arc<dyn Adapter>) {
		self.adapters
			.write()
			.expect("adapter registry poisoned")
			.insert(adapter.id().to_string(), adapter);
	}

	/// Removes an adapter, for example after the store uninstalled it.
	pub fn unregister(&self, id: &str) -> Option<Arc<dyn Adapter>> {
		self.adapters
			.write()
			.expect("adapter registry poisoned")
			.remove(id)
	}

	pub fn get(&self, id: &str) -> Option<Arc<dyn Adapter>> {
		self.adapters
			.read()
			.expect("adapter registry poisoned")
			.get(id)
			.cloned()
	}

	pub fn list(&self) -> Vec<AdapterInfo> {
		self.adapters
			.read()
			.expect("adapter registry poisoned")
			.values()
			.map(|a| AdapterInfo {
				id: a.id().to_string(),
				name: a.name().to_string(),
				description: a.description().to_string(),
				version: a.version().to_string(),
				author: a.author().to_string(),
				data_type: a.data_type().to_string(),
				kind: AdapterKind::Native,
				trust_tier: a.trust_tier(),
				icon_svg: a.icon_svg().map(|s| s.to_string()),
				update_available: false,
			})
			.collect()
	}
}

impl Default for AdapterRegistry {
	fn default() -> Self {
		Self::new()
	}
}

/// Result of an adapter update operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterUpdateResult {
	pub adapter_id: String,
	pub old_version: String,
	pub new_version: String,
	pub schema_changed: bool,
	pub backup_path: String,
}

#[cfg(test)]
mod tests {
	use super::*;

	/// An adapter that implements only the required methods.
	struct Minimal;

	impl Adapter for Minimal {
		fn id(&self) -> &str {
			"minimal"
		}
		fn name(&self) -> &str {
			"Minimal"
		}
		fn data_type(&self) -> &str {
			"note"
		}
		fn sync<'a>(
			&'a self,
			_db: &'a SourceDb,
			_config: &'a serde_json::Value,
		) -> Pin<Box<dyn Future<Output = Result<SyncReport>> + Send + 'a>> {
			Box::pin(async { Err(crate::error::Error::AdapterSync("not used".into())) })
		}
	}

	#[tokio::test]
	async fn defaults_describe_an_external_adapter_without_extras() {
		let adapter = Minimal;
		assert_eq!(adapter.description(), "");
		assert_eq!(adapter.version(), "0.0.0");
		assert_eq!(adapter.author(), "");
		assert!(adapter.icon_svg().is_none());
		assert_eq!(adapter.trust_tier(), TrustTier::External);
		assert!(adapter.schema().is_none());
		assert!(adapter.config_fields().is_empty());
		assert!(adapter.translations().is_none());
		assert!(adapter.detect().await.is_empty());
	}

	#[test]
	fn the_registry_lists_and_unregisters_adapters() {
		let registry = AdapterRegistry::default();
		registry.register(Arc::new(Minimal));
		let listed = registry.list();
		assert_eq!(listed.len(), 1);
		assert_eq!(listed[0].id, "minimal");
		assert_eq!(listed[0].kind, AdapterKind::Native);
		assert!(!listed[0].update_available);
		let removed = registry.unregister("minimal").expect("registered");
		assert_eq!(removed.id(), "minimal");
		assert!(registry.get("minimal").is_none());
		assert!(registry.unregister("minimal").is_none());
		assert!(registry.list().is_empty());
	}
}
