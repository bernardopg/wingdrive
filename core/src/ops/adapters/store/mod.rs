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
	/// `summary` and `description` in other languages, keyed by language code (`pt`).
	pub translations: std::collections::BTreeMap<String, StoreCardText>,
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

/// An adapter's store texts in one language.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct StoreCardText {
	pub summary: String,
	pub description: Option<String>,
}

/// The store: every published adapter and the state of the installed ones.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct StoreListing {
	/// Sequence of the index shown, or `None` before the first refresh.
	pub sequence: Option<u32>,
	/// How adapters are confined here: `bubblewrap` (only their declared paths and, without
	/// declared hosts, no network), `flatpak` (the app's own sandbox) or `none`.
	pub isolation: String,
	pub generated_at: Option<String>,
	pub adapters: Vec<StoreCard>,
	/// Installed adapters with an update or a revoked version.
	pub updates: u32,
}

fn isolation() -> String {
	match wing_archive::adapter::sandbox::isolation() {
		Ok(kind) => kind.as_str().to_string(),
		// bubblewrap is missing or blocked: adapters will refuse to run until that is fixed.
		Err(_) => "unavailable".to_string(),
	}
}

fn listing(store: &Store, index: Option<&StoreIndex>) -> Result<StoreListing, String> {
	let installed = store.installed().map_err(|e| e.to_string())?;
	let Some(index) = index else {
		return Ok(StoreListing {
			isolation: isolation(),
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
				translations: a
					.i18n
					.iter()
					.map(|(lang, text)| {
						(
							lang.clone(),
							StoreCardText {
								summary: text.summary.clone(),
								description: text.description.clone(),
							},
						)
					})
					.collect(),
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
		isolation: isolation(),
		sequence: Some(index.sequence.min(u32::MAX as u64) as u32),
		generated_at: Some(index.generated_at.clone()),
		adapters,
		updates: updates.len() as u32,
	})
}

#[cfg(test)]
mod tests {
	use std::io::{Read, Write};
	use std::net::TcpListener;

	use super::*;
	use crate::infra::action::CoreAction;

	fn fixture(name: &str) -> Vec<u8> {
		std::fs::read(
			Path::new(env!("CARGO_MANIFEST_DIR"))
				.join("../crates/archive/tests/fixtures/store")
				.join(name),
		)
		.unwrap()
	}

	#[test]
	fn install_and_remove_reject_blank_ids() {
		let install = InstallAdapterAction::from_input(InstallAdapterInput {
			id: " \t".into(),
			accept_permissions: false,
		});
		assert!(install.unwrap_err().contains("Adapter id cannot be empty"));
		let remove = RemoveAdapterAction::from_input(RemoveAdapterInput { id: " ".into() });
		assert!(remove.unwrap_err().contains("Adapter id cannot be empty"));
	}

	#[test]
	fn signed_catalog_and_archive_reject_tampering() {
		let tmp = tempfile::tempdir().unwrap();
		let store = fixture_store(tmp.path());
		let signature = String::from_utf8(fixture("index.json.minisig")).unwrap();
		let mut catalog = fixture("index.json");
		catalog.push(b' ');
		assert!(store.accept_index(&catalog, &signature).is_err());
		assert!(store.cached_index().unwrap().is_none());
		let index = accepted(&store, "index");
		let mut archive = fixture("sample-1.0.0.tar.gz");
		archive[0] ^= 1;
		assert!(store
			.install(&index.adapters[0], &archive)
			.unwrap_err()
			.to_string()
			.contains("checksum"));
		assert!(store.installed().unwrap().is_empty());
	}

	fn fixture_store(root: &Path) -> Store {
		Store::with_keys(root, vec![String::from_utf8(fixture("test.pub")).unwrap()]).unwrap()
	}

	fn accepted(store: &Store, name: &str) -> StoreIndex {
		store
			.accept_index(
				&fixture(&format!("{name}.json")),
				&String::from_utf8(fixture(&format!("{name}.json.minisig"))).unwrap(),
			)
			.unwrap()
	}

	fn serve(response: Vec<u8>) -> String {
		let listener = TcpListener::bind("127.0.0.1:0").unwrap();
		let url = format!("http://{}", listener.local_addr().unwrap());
		std::thread::spawn(move || {
			let (mut stream, _) = listener.accept().unwrap();
			// One read takes the whole request line; the body is never needed.
			let mut request = [0; 2048];
			let _ = stream.read(&mut request);
			stream.write_all(&response).unwrap();
		});
		url
	}

	#[tokio::test]
	async fn download_enforces_status_and_both_size_limits() {
		let client = http().unwrap();
		let url = serve(b"HTTP/1.1 200 OK\r\nContent-Length: 4\r\n\r\ntest".to_vec());
		assert_eq!(download(&client, &url, 4).await.unwrap(), b"test");
		let url = serve(b"HTTP/1.1 200 OK\r\nContent-Length: 4\r\n\r\ntest".to_vec());
		assert!(download(&client, &url, 3)
			.await
			.unwrap_err()
			.contains("larger than 3"));
		let url = serve(
			b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n4\r\ntest\r\n0\r\n\r\n".to_vec(),
		);
		assert!(download(&client, &url, 3)
			.await
			.unwrap_err()
			.contains("larger than 3"));
		let url = serve(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n".to_vec());
		assert!(download(&client, &url, 10)
			.await
			.unwrap_err()
			.contains("404"));
	}

	#[tokio::test]
	async fn archive_download_checks_target_and_size() {
		let tmp = tempfile::tempdir().unwrap();
		let store = fixture_store(tmp.path());
		let index = accepted(&store, "index");
		let mut adapter = index.adapters[0].clone();
		adapter.artifacts.clear();
		assert!(download_archive(&adapter)
			.await
			.unwrap_err()
			.contains("no build"));
		adapter.artifacts = index.adapters[0].artifacts.clone();
		adapter.artifacts[0].size = MAX_ARCHIVE_BYTES + 1;
		assert!(download_archive(&adapter)
			.await
			.unwrap_err()
			.contains("larger than 64 MiB"));
		adapter.artifacts[0].size = 5;
		adapter.artifacts[0].url =
			serve(b"HTTP/1.1 200 OK\r\nContent-Length: 4\r\n\r\ntest".to_vec());
		assert!(download_archive(&adapter)
			.await
			.unwrap_err()
			.contains("archive is 4 bytes"));
		adapter.artifacts[0].size = 4;
		adapter.artifacts[0].url =
			serve(b"HTTP/1.1 200 OK\r\nContent-Length: 4\r\n\r\ntest".to_vec());
		assert_eq!(download_archive(&adapter).await.unwrap(), b"test");
	}

	#[test]
	fn listing_tracks_installs_updates_revocations_and_icons() {
		let tmp = tempfile::tempdir().unwrap();
		let store = fixture_store(tmp.path());
		let empty = listing(&store, None).unwrap();
		assert!(empty.adapters.is_empty());
		assert_eq!(empty.sequence, None);
		let index = accepted(&store, "index");
		let card = &listing(&store, Some(&index)).unwrap().adapters[0];
		assert_eq!(card.id, "sample");
		assert_eq!(card.permissions.read, ["~/Notes"]);
		assert!(card.installed_version.is_none());
		assert!(card.icon_svg.is_none());
		std::fs::create_dir_all(icons_dir(&store)).unwrap();
		std::fs::write(icon_path(&store, &index.adapters[0]), "<svg/>").unwrap();
		store
			.install(&index.adapters[0], &fixture("sample-1.0.0.tar.gz"))
			.unwrap();
		let installed = listing(&store, Some(&index)).unwrap();
		assert_eq!(
			installed.adapters[0].installed_version.as_deref(),
			Some("1.0.0")
		);
		assert_eq!(installed.adapters[0].icon_svg.as_deref(), Some("<svg/>"));
		let update = accepted(&store, "index-update");
		let listed = listing(&store, Some(&update)).unwrap();
		assert_eq!(listed.updates, 1);
		assert!(listed.adapters[0].update_available);
		assert_eq!(
			listed.adapters[0].revoked_reason.as_deref(),
			Some("fixture revocation")
		);
		assert!(listed.adapters[0].update_needs_consent);
		store.remove("sample").unwrap();
		assert!(listing(&store, Some(&update)).unwrap().adapters[0]
			.installed_version
			.is_none());
	}
}
