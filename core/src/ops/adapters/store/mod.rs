//! # Adapter store operations
//!
//! The daemon side of the adapter store (`docs/design/adapter-store.md`). It downloads the
//! registry's signed index and the adapter archives, hands the bytes to
//! [`wing_archive::store::Store`] for every check, keeps installed adapters in
//! `<data dir>/adapters/installed` for all libraries, and reloads the libraries that are open after
//! an install or a removal.
//!
//! `WINGDRIVE_STORE_INDEX_URL` points the daemon at a mirror of the index (its signature is read
//! from the same URL plus `.minisig`). The signature is still checked against the keys built into
//! WingDrive, so a mirror can only serve what the registry signed.

pub mod install;
pub mod list;
pub mod refresh;
pub mod remove;

use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use specta::Type;
use tracing::warn;
use wing_archive::adapter::script::{python_interpreter, Permissions, RuntimeKind};
use wing_archive::store::{self, Store, StoreAdapter, StoreIndex};

use crate::context::CoreContext;

pub use install::*;
pub use list::*;
pub use refresh::*;
pub use remove::*;

/// Overrides the index URL, for mirrors and tests.
pub const INDEX_URL_ENV: &str = "WINGDRIVE_STORE_INDEX_URL";

const MAX_INDEX_BYTES: usize = 5 * 1024 * 1024;
const MAX_ICON_BYTES: usize = 64 * 1024;
const MAX_ARCHIVE_BYTES: u64 = 64 * 1024 * 1024;

/// `<data dir>/adapters`: installed adapters, the cached index and the lockfile.
pub fn store_root(data_dir: &Path) -> PathBuf {
	data_dir.join("adapters")
}

/// `<data dir>/adapters/installed`, the folder every library loads store adapters from.
pub fn installed_dir(data_dir: &Path) -> PathBuf {
	store_root(data_dir).join("installed")
}

/// How often the daemon fetches the signed index to notice adapter updates and revocations.
const CHECK_EVERY: std::time::Duration = std::time::Duration::from_secs(6 * 60 * 60);

/// Fetches the signed index shortly after start and every six hours, so the update badges and
/// revocations stay current without the user opening the store. `WINGDRIVE_STORE_CHECK=0`
/// turns it off.
pub fn spawn_update_checks(context: Arc<CoreContext>) {
	if std::env::var("WINGDRIVE_STORE_CHECK").as_deref() == Ok("0") {
		return;
	}
	tokio::spawn(async move {
		tokio::time::sleep(std::time::Duration::from_secs(30)).await;
		let mut interval = tokio::time::interval(CHECK_EVERY);
		loop {
			interval.tick().await;
			let result = async {
				let store = open(&context)?;
				let index = fetch_index(&store).await?;
				listing(&store, Some(&index))
			}
			.await;
			match result {
				Ok(listing) => {
					if listing.updates > 0 {
						tracing::info!(updates = listing.updates, "adapter updates available");
					}
					// Revocations in the new index take effect now.
					reload_libraries(&context, None).await;
				}
				Err(error) => tracing::debug!(%error, "adapter store check failed"),
			}
		}
	});
}

fn open(context: &CoreContext) -> Result<Store, String> {
	Store::open(store_root(&context.data_dir)).map_err(|e| e.to_string())
}

fn http() -> Result<reqwest::Client, String> {
	reqwest::Client::builder()
		.user_agent(concat!("WingDrive/", env!("CARGO_PKG_VERSION")))
		.timeout(Duration::from_secs(60))
		.build()
		.map_err(|e| format!("Cannot start the HTTP client: {e}"))
}

/// Downloads `url`, refusing responses larger than `limit` bytes.
async fn download(client: &reqwest::Client, url: &str, limit: usize) -> Result<Vec<u8>, String> {
	let response = client
		.get(url)
		.send()
		.await
		.and_then(|r| r.error_for_status())
		.map_err(|e| format!("Download of {url} failed: {e}"))?;
	if response
		.content_length()
		.is_some_and(|len| len > limit as u64)
	{
		return Err(format!("{url} is larger than {limit} bytes"));
	}
	let bytes = response
		.bytes()
		.await
		.map_err(|e| format!("Download of {url} failed: {e}"))?;
	if bytes.len() > limit {
		return Err(format!("{url} is larger than {limit} bytes"));
	}
	Ok(bytes.to_vec())
}

/// Fetches, verifies and caches the newest index, then caches the icons it lists.
async fn fetch_index(store: &Store) -> Result<StoreIndex, String> {
	let client = http()?;
	let index_url = std::env::var(INDEX_URL_ENV).unwrap_or_else(|_| store::INDEX_URL.to_string());
	let signature_url = if std::env::var(INDEX_URL_ENV).is_ok() {
		format!("{index_url}.minisig")
	} else {
		store::SIGNATURE_URL.to_string()
	};
	let index = download(&client, &index_url, MAX_INDEX_BYTES).await?;
	let signature = download(&client, &signature_url, 4096).await?;
	let signature =
		String::from_utf8(signature).map_err(|_| "The index signature is not text".to_string())?;
	let index = store
		.accept_index(&index, &signature)
		.map_err(|e| e.to_string())?;

	let icons = icons_dir(store);
	std::fs::create_dir_all(&icons).map_err(|e| e.to_string())?;
	for adapter in &index.adapters {
		let path = icon_path(store, adapter);
		if path.exists() {
			continue;
		}
		match download(&client, &adapter.icon.url, MAX_ICON_BYTES).await {
			Ok(bytes) if store::sha256_hex(&bytes) == adapter.icon.sha256 => {
				if let Err(e) = std::fs::write(&path, bytes) {
					warn!(adapter = %adapter.id, error = %e, "cannot cache adapter icon");
				}
			}
			Ok(_) => warn!(adapter = %adapter.id, "adapter icon checksum does not match the index"),
			Err(e) => warn!(adapter = %adapter.id, error = %e, "cannot download adapter icon"),
		}
	}
	Ok(index)
}

fn icons_dir(store: &Store) -> PathBuf {
	store.installed_dir().with_file_name("cache").join("icons")
}

fn icon_path(store: &Store, adapter: &StoreAdapter) -> PathBuf {
	icons_dir(store).join(format!("{}-{}.svg", adapter.id, adapter.version))
}

/// Downloads the archive for this platform and checks its size against the index.
async fn download_archive(adapter: &StoreAdapter) -> Result<Vec<u8>, String> {
	let artifact = adapter.artifact().ok_or_else(|| {
		format!(
			"{} has no build for {}",
			adapter.id,
			store::current_target()
		)
	})?;
	if artifact.size > MAX_ARCHIVE_BYTES {
		return Err(format!("{} archive is larger than 64 MiB", adapter.id));
	}
	let bytes = download(&http()?, &artifact.url, artifact.size as usize).await?;
	if bytes.len() as u64 != artifact.size {
		return Err(format!(
			"{} archive is {} bytes, the index says {}",
			adapter.id,
			bytes.len(),
			artifact.size
		));
	}
	Ok(bytes)
}

/// Whether the interpreter or program an adapter needs is present on this machine.
fn runtime_available(kind: RuntimeKind) -> bool {
	static PYTHON: OnceLock<bool> = OnceLock::new();
	match kind {
		RuntimeKind::Binary => true,
		RuntimeKind::Python3 => *PYTHON.get_or_init(|| {
			std::process::Command::new(python_interpreter())
				.arg("--version")
				.output()
				.map(|o| o.status.success())
				.unwrap_or(false)
		}),
	}
}

/// Registers new or updated store adapters in every open library, and drops `removed`.
async fn reload_libraries(context: &Arc<CoreContext>, removed: Option<&str>) {
	for library in context.libraries().await.list().await {
		if let Some(manager) = library.source_manager() {
			if let Some(id) = removed {
				manager.unregister_adapter(id);
			}
			manager.reload_installed();
		}
	}
}

/// What an adapter may touch, as the store card shows it.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Type)]
pub struct StorePermissions {
	pub read: Vec<String>,
	pub network: Vec<String>,
	pub secrets: Vec<String>,
	pub tools: Vec<String>,
}

impl From<&Permissions> for StorePermissions {
	fn from(p: &Permissions) -> Self {
		Self {
			read: p.read.clone(),
			network: p.network.clone(),
			secrets: p.secrets.clone(),
			tools: p.tools.clone(),
		}
	}
}

impl StorePermissions {
	fn is_empty(&self) -> bool {
		self.read.is_empty()
			&& self.network.is_empty()
			&& self.secrets.is_empty()
			&& self.tools.is_empty()
	}
}

/// One adapter as the store shows it.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct StoreCard {
	pub id: String,
	pub name: String,
	/// Up to two lines for the card.
	pub summary: String,
	pub description: String,
	pub version: String,
	pub author: String,
	pub license: String,
	pub homepage: Option<String>,
	pub categories: Vec<String>,
	/// `official`, `verified` or `community`.
	pub trust: String,
	pub permissions: StorePermissions,
	pub platforms: Vec<String>,
	/// The adapter says it runs on this operating system.
	pub supported_here: bool,
	/// The interpreter it needs (Python) is installed.
	pub runtime_available: bool,
	pub installed_version: Option<String>,
	pub update_available: bool,
	/// The update asks for permissions the installed version did not have.
	pub update_needs_consent: bool,
	/// Set when the installed version is revoked; the reason says why.
	pub revoked_reason: Option<String>,
	pub icon_svg: Option<String>,
}

/// The store: every published adapter and the state of the installed ones.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct StoreListing {
	/// Sequence of the index shown, or `None` before the first refresh.
	pub sequence: Option<u32>,
	pub generated_at: Option<String>,
	pub adapters: Vec<StoreCard>,
	/// Installed adapters with an update or a revoked version.
	pub updates: u32,
}

fn listing(store: &Store, index: Option<&StoreIndex>) -> Result<StoreListing, String> {
	let installed = store.installed().map_err(|e| e.to_string())?;
	let Some(index) = index else {
		return Ok(StoreListing {
			sequence: None,
			generated_at: None,
			adapters: Vec::new(),
			updates: 0,
		});
	};
	let updates = store.updates(index).map_err(|e| e.to_string())?;
	let adapters = index
		.adapters
		.iter()
		.map(|a| {
			let update = updates.iter().find(|u| u.id == a.id);
			let installed_version = installed.get(&a.id).map(|e| e.version.clone());
			StoreCard {
				id: a.id.clone(),
				name: a.name.clone(),
				summary: a.summary.clone(),
				description: a.description.clone(),
				version: a.version.clone(),
				author: a.author.clone(),
				license: a.license.clone(),
				homepage: a.homepage.clone(),
				categories: a.categories.clone(),
				trust: format!("{:?}", a.trust).to_lowercase(),
				permissions: StorePermissions::from(&a.permissions),
				platforms: a.platforms.clone(),
				supported_here: a.supports_this_platform(),
				runtime_available: runtime_available(a.runtime.kind),
				revoked_reason: installed_version
					.as_deref()
					.and_then(|v| index.revocation(&a.id, v))
					.map(|r| r.reason.clone()),
				installed_version,
				update_available: update.is_some(),
				update_needs_consent: update.is_some_and(|u| u.new_permissions),
				icon_svg: std::fs::read_to_string(icon_path(store, a)).ok(),
			}
		})
		.collect();
	Ok(StoreListing {
		sequence: Some(index.sequence.min(u32::MAX as u64) as u32),
		generated_at: Some(index.generated_at.clone()),
		adapters,
		updates: updates.len() as u32,
	})
}
