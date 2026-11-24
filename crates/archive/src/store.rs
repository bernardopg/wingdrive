//! # Adapter store
//!
//! Adapters are published in the registry `github.com/bernardopg/wingdrive-adapters`, which signs
//! an `index.json` with minisign. This module verifies that index with the public keys compiled
//! into WingDrive, keeps the last accepted copy for offline use, and installs adapter archives
//! whose SHA-256 matches the index. It does no networking: the caller downloads the bytes, so the
//! same checks apply to the official registry, a local mirror and tests.
//!
//! Every installed adapter lives in `<data dir>/adapters/installed/<id>/`, shared by all libraries,
//! and `adapters.lock.json` records the exact version and checksum, like `dms plugins lock`.
//!
//! ```rust,no_run
//! use wing_archive::store::Store;
//! # fn demo(index: &[u8], signature: &str, archive: &[u8]) -> wing_archive::Result<()> {
//! let store = Store::open("/home/me/.wingdrive/adapters")?;
//! let index = store.accept_index(index, signature)?;
//! let obsidian = index.adapter("obsidian").expect("listed");
//! store.install(obsidian, archive)?;
//! # Ok(())
//! # }
//! ```

use std::collections::BTreeMap;
use std::io::Read;
use std::path::{Component, Path, PathBuf};

use minisign_verify::{PublicKey, Signature};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::adapter::script::{AdapterManifest, Permissions, RuntimeKind, SUPPORTED_CONTRACT};
use crate::error::{Error, Result};

/// The registry key that signs every published index.
pub const ACTIVE_KEY: &str = include_str!("../keys/active.pub");
/// The registry key that takes over if the active one is lost or leaked.
pub const STANDBY_KEY: &str = include_str!("../keys/standby.pub");
/// Where the official registry serves the newest signed index.
pub const INDEX_URL: &str =
	"https://github.com/bernardopg/wingdrive-adapters/releases/latest/download/index.json";
/// Where the official registry serves the signature of that index.
pub const SIGNATURE_URL: &str =
	"https://github.com/bernardopg/wingdrive-adapters/releases/latest/download/index.json.minisig";

/// Largest unpacked adapter accepted from an archive.
const MAX_UNPACKED_BYTES: u64 = 64 * 1024 * 1024;
/// Largest number of files and folders accepted from an archive.
const MAX_ENTRIES: usize = 5_000;

/// The signed list of published adapters (format 1).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreIndex {
	pub format: u32,
	pub sequence: u64,
	pub generated_at: String,
	pub registry: String,
	pub adapters: Vec<StoreAdapter>,
	#[serde(default)]
	pub revoked: Vec<Revocation>,
}

impl StoreIndex {
	/// The entry for one adapter id.
	pub fn adapter(&self, id: &str) -> Option<&StoreAdapter> {
		self.adapters.iter().find(|a| a.id == id)
	}

	/// The revocation that covers this version, if any.
	pub fn revocation(&self, id: &str, version: &str) -> Option<&Revocation> {
		self.revoked
			.iter()
			.find(|r| r.id == id && r.versions.iter().any(|v| v == version))
	}
}

/// One published adapter as the index describes it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreAdapter {
	pub id: String,
	pub name: String,
	pub summary: String,
	#[serde(default)]
	pub description: String,
	pub version: String,
	pub author: String,
	pub license: String,
	#[serde(default)]
	pub homepage: Option<String>,
	#[serde(default)]
	pub categories: Vec<String>,
	pub trust: Trust,
	pub contract: u32,
	pub runtime: StoreRuntime,
	pub platforms: Vec<String>,
	#[serde(default)]
	pub permissions: Permissions,
	pub source: serde_json::Value,
	pub icon: StoreIcon,
	pub artifacts: Vec<Artifact>,
}

impl StoreAdapter {
	/// The archive for the running platform: a per-target build for compiled adapters, or the
	/// single `any` archive of a Python adapter.
	pub fn artifact(&self) -> Option<&Artifact> {
		let target = current_target();
		self.artifacts
			.iter()
			.find(|a| a.target == target)
			.or_else(|| self.artifacts.iter().find(|a| a.target == "any"))
	}

	/// Whether the adapter says it runs on this platform.
	pub fn supports_this_platform(&self) -> bool {
		self.platforms.iter().any(|p| p == current_os())
	}
}

/// How far the registry trusts an adapter's author.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Trust {
	Official,
	Verified,
	Community,
}

/// Runtime of a published adapter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreRuntime {
	pub kind: RuntimeKind,
	pub entry: String,
}

/// The icon shown on the store card.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreIcon {
	pub url: String,
	pub sha256: String,
}

/// A downloadable archive of an adapter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
	pub target: String,
	pub url: String,
	pub sha256: String,
	pub size: u64,
}

/// A version the app must disable.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Revocation {
	pub id: String,
	pub versions: Vec<String>,
	pub reason: String,
	#[serde(default)]
	pub date: Option<String>,
}

/// What the lockfile records for one installed adapter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LockEntry {
	pub version: String,
	pub sha256: String,
	pub trust: Trust,
	pub permissions: Permissions,
	#[serde(default)]
	pub commit: Option<String>,
	pub installed_at: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct Lockfile {
	lockfile_version: u32,
	adapters: BTreeMap<String, LockEntry>,
	/// Highest index sequence accepted, so an older (replayed) index is refused.
	#[serde(default)]
	last_sequence: u64,
}

/// An installed adapter whose index entry is newer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AvailableUpdate {
	pub id: String,
	pub installed: String,
	pub available: String,
	/// The update asks for permissions the installed version did not have.
	pub new_permissions: bool,
	/// The installed version is revoked, so the update fixes a security problem.
	pub security: bool,
}

/// `linux`, `macos` or `windows`.
pub fn current_os() -> &'static str {
	if cfg!(target_os = "macos") {
		"macos"
	} else if cfg!(windows) {
		"windows"
	} else {
		"linux"
	}
}

/// `<os>-<arch>`, the target name the registry uses for compiled adapters.
pub fn current_target() -> String {
	format!("{}-{}", current_os(), std::env::consts::ARCH)
}

/// Verifies `index` against `signature` with any of `keys` and parses it.
///
/// A signature that matches none of the keys, a trusted comment that does not name this
/// registry, or an unknown format is refused before anything in the index is used.
pub fn verify_index(index: &[u8], signature: &str, keys: &[&str]) -> Result<StoreIndex> {
	let signature = Signature::decode(signature)
		.map_err(|e| Error::Store(format!("index signature is not a minisign signature: {e}")))?;
	let verified = keys.iter().any(|key| {
		PublicKey::decode(key)
			.map(|key| key.verify(index, &signature, false).is_ok())
			.unwrap_or(false)
	});
	if !verified {
		return Err(Error::Store(
			"the adapter index signature does not match the registry keys".into(),
		));
	}
	if !signature
		.trusted_comment()
		.starts_with("wingdrive-adapters index ")
	{
		return Err(Error::Store(
			"the adapter index has an unexpected trusted comment".into(),
		));
	}
	let index: StoreIndex = serde_json::from_slice(index)
		.map_err(|e| Error::Store(format!("the adapter index is not valid: {e}")))?;
	if index.format != 1 {
		return Err(Error::Store(format!(
			"adapter index format {} is newer than this WingDrive understands",
			index.format
		)));
	}
	Ok(index)
}

/// SHA-256 of `bytes` as lowercase hex.
pub fn sha256_hex(bytes: &[u8]) -> String {
	hex::encode(Sha256::digest(bytes))
}

/// The installed adapters and the cached index of one WingDrive data directory.
pub struct Store {
	root: PathBuf,
	keys: Vec<String>,
}

impl Store {
	/// Opens (and creates) the store under `root`, trusting the keys compiled into WingDrive.
	pub fn open(root: impl Into<PathBuf>) -> Result<Self> {
		Self::with_keys(root, vec![ACTIVE_KEY.to_string(), STANDBY_KEY.to_string()])
	}

	/// Opens the store trusting other keys, for tests and private registries.
	pub fn with_keys(root: impl Into<PathBuf>, keys: Vec<String>) -> Result<Self> {
		let root = root.into();
		std::fs::create_dir_all(root.join("installed"))?;
		std::fs::create_dir_all(root.join("cache"))?;
		Ok(Self { root, keys })
	}

	/// Folder holding one directory per installed adapter.
	pub fn installed_dir(&self) -> PathBuf {
		self.root.join("installed")
	}

	fn lockfile_path(&self) -> PathBuf {
		self.root.join("adapters.lock.json")
	}

	fn read_lockfile(&self) -> Result<Lockfile> {
		match std::fs::read(self.lockfile_path()) {
			Ok(bytes) => serde_json::from_slice(&bytes)
				.map_err(|e| Error::Store(format!("adapters.lock.json is damaged: {e}"))),
			Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Lockfile {
				lockfile_version: 1,
				..Default::default()
			}),
			Err(e) => Err(e.into()),
		}
	}

	fn write_lockfile(&self, lockfile: &Lockfile) -> Result<()> {
		let bytes = serde_json::to_vec_pretty(lockfile)
			.map_err(|e| Error::Store(format!("cannot write adapters.lock.json: {e}")))?;
		write_atomic(&self.lockfile_path(), &bytes)
	}

	/// Installed adapters with their locked versions.
	pub fn installed(&self) -> Result<BTreeMap<String, LockEntry>> {
		Ok(self.read_lockfile()?.adapters)
	}

	/// Verifies a freshly downloaded index, refuses one older than the last accepted, and caches
	/// it for offline use.
	pub fn accept_index(&self, index: &[u8], signature: &str) -> Result<StoreIndex> {
		let keys: Vec<&str> = self.keys.iter().map(String::as_str).collect();
		let parsed = verify_index(index, signature, &keys)?;
		let mut lockfile = self.read_lockfile()?;
		if parsed.sequence < lockfile.last_sequence {
			return Err(Error::Store(format!(
				"the adapter index is older ({}) than the last one accepted ({})",
				parsed.sequence, lockfile.last_sequence
			)));
		}
		write_atomic(&self.root.join("cache/index.json"), index)?;
		write_atomic(
			&self.root.join("cache/index.json.minisig"),
			signature.as_bytes(),
		)?;
		lockfile.lockfile_version = 1;
		lockfile.last_sequence = parsed.sequence;
		self.write_lockfile(&lockfile)?;
		Ok(parsed)
	}

	/// The last accepted index, verified again, for offline use.
	pub fn cached_index(&self) -> Result<Option<StoreIndex>> {
		let index = match std::fs::read(self.root.join("cache/index.json")) {
			Ok(index) => index,
			Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
			Err(e) => return Err(e.into()),
		};
		let signature = std::fs::read_to_string(self.root.join("cache/index.json.minisig"))?;
		let keys: Vec<&str> = self.keys.iter().map(String::as_str).collect();
		verify_index(&index, &signature, &keys).map(Some)
	}

	/// Installs `archive` for `adapter`, replacing an installed version atomically.
	///
	/// The archive must match the checksum in the signed index, unpack only inside one folder
	/// named after the adapter, and carry an `adapter.toml` with the same id and version.
	pub fn install(&self, adapter: &StoreAdapter, archive: &[u8]) -> Result<LockEntry> {
		let artifact = adapter.artifact().ok_or_else(|| {
			Error::Store(format!(
				"{} has no build for {}",
				adapter.id,
				current_target()
			))
		})?;
		let digest = sha256_hex(archive);
		if digest != artifact.sha256 {
			return Err(Error::Store(format!(
				"{} archive checksum is {digest}, the index says {}",
				adapter.id, artifact.sha256
			)));
		}
		if adapter.contract > SUPPORTED_CONTRACT {
			return Err(Error::Store(format!(
				"{} needs a newer WingDrive (adapter contract {})",
				adapter.id, adapter.contract
			)));
		}

		let staging = self
			.root
			.join(format!(".staging-{}-{}", adapter.id, std::process::id()));
		let _ = std::fs::remove_dir_all(&staging);
		let result = unpack(archive, &adapter.id, &staging).and_then(|()| {
			let manifest = AdapterManifest::from_file(&staging.join("adapter.toml"))?;
			if manifest.adapter.id != adapter.id || manifest.adapter.version != adapter.version {
				return Err(Error::Store(format!(
					"{} archive holds {} {}, the index says {} {}",
					adapter.id,
					manifest.adapter.id,
					manifest.adapter.version,
					adapter.id,
					adapter.version
				)));
			}
			Ok(())
		});
		if let Err(e) = result {
			let _ = std::fs::remove_dir_all(&staging);
			return Err(e);
		}

		let target = self.installed_dir().join(&adapter.id);
		let previous = self.root.join(format!(".previous-{}", adapter.id));
		let _ = std::fs::remove_dir_all(&previous);
		if target.exists() {
			std::fs::rename(&target, &previous)?;
		}
		if let Err(e) = std::fs::rename(&staging, &target) {
			if previous.exists() {
				let _ = std::fs::rename(&previous, &target);
			}
			return Err(e.into());
		}
		let _ = std::fs::remove_dir_all(&previous);

		let entry = LockEntry {
			version: adapter.version.clone(),
			sha256: digest,
			trust: adapter.trust,
			permissions: adapter.permissions.clone(),
			commit: adapter
				.source
				.get("commit")
				.and_then(|c| c.as_str())
				.map(str::to_owned),
			installed_at: chrono::Utc::now().to_rfc3339(),
		};
		let mut lockfile = self.read_lockfile()?;
		lockfile.adapters.insert(adapter.id.clone(), entry.clone());
		self.write_lockfile(&lockfile)?;
		Ok(entry)
	}

	/// Removes an installed adapter and its lockfile entry.
	pub fn remove(&self, id: &str) -> Result<()> {
		let mut lockfile = self.read_lockfile()?;
		if lockfile.adapters.remove(id).is_none() {
			return Err(Error::Store(format!("{id} is not installed")));
		}
		match std::fs::remove_dir_all(self.installed_dir().join(id)) {
			Ok(()) => {}
			Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
			Err(e) => return Err(e.into()),
		}
		self.write_lockfile(&lockfile)
	}

	/// Installed adapters that the index publishes in a newer version, or whose installed
	/// version is revoked.
	pub fn updates(&self, index: &StoreIndex) -> Result<Vec<AvailableUpdate>> {
		let mut updates = Vec::new();
		for (id, installed) in self.read_lockfile()?.adapters {
			let Some(published) = index.adapter(&id) else {
				continue;
			};
			let security = index.revocation(&id, &installed.version).is_some();
			if security || version_key(&published.version) > version_key(&installed.version) {
				let asks_more =
					|new: &[String], old: &[String]| new.iter().any(|p| !old.contains(p));
				let (new, old) = (&published.permissions, &installed.permissions);
				updates.push(AvailableUpdate {
					id,
					installed: installed.version,
					available: published.version.clone(),
					new_permissions: asks_more(&new.read, &old.read)
						|| asks_more(&new.network, &old.network)
						|| asks_more(&new.secrets, &old.secrets)
						|| asks_more(&new.tools, &old.tools),
					security,
				});
			}
		}
		Ok(updates)
	}
}

fn version_key(version: &str) -> Vec<u64> {
	version.split('.').map(|p| p.parse().unwrap_or(0)).collect()
}

fn write_atomic(path: &Path, bytes: &[u8]) -> Result<()> {
	let temp = path.with_extension("tmp");
	std::fs::write(&temp, bytes)?;
	std::fs::rename(&temp, path)?;
	Ok(())
}

/// Unpacks a `.tar.gz` whose entries all sit under `<id>/` into `dest`.
///
/// Refuses absolute paths, `..`, links and special files, and stops past the size and entry
/// limits, so a hostile archive cannot write outside `dest` or fill the disk.
fn unpack(archive: &[u8], id: &str, dest: &Path) -> Result<()> {
	std::fs::create_dir_all(dest)?;
	let mut tar = tar::Archive::new(flate2::read::GzDecoder::new(archive));
	let mut total = 0u64;
	for (count, entry) in tar
		.entries()
		.map_err(|e| Error::Store(format!("{id} archive is not a tar.gz: {e}")))?
		.enumerate()
	{
		if count >= MAX_ENTRIES {
			return Err(Error::Store(format!(
				"{id} archive has more than {MAX_ENTRIES} entries"
			)));
		}
		let mut entry = entry.map_err(|e| Error::Store(format!("{id} archive is damaged: {e}")))?;
		let path = entry
			.path()
			.map_err(|e| Error::Store(format!("{id} archive has an invalid path: {e}")))?
			.into_owned();
		let relative = path
			.strip_prefix(id)
			.map_err(|_| {
				Error::Store(format!(
					"{id} archive entry {} is outside {id}/",
					path.display()
				))
			})?
			.to_path_buf();
		if relative
			.components()
			.any(|c| !matches!(c, Component::Normal(_)))
		{
			return Err(Error::Store(format!(
				"{id} archive entry {} leaves the adapter folder",
				path.display()
			)));
		}
		let target = dest.join(&relative);
		let kind = entry.header().entry_type();
		if kind.is_dir() {
			std::fs::create_dir_all(&target)?;
			continue;
		}
		if !kind.is_file() {
			return Err(Error::Store(format!(
				"{id} archive entry {} is a link or special file",
				path.display()
			)));
		}
		total += entry.size();
		if total > MAX_UNPACKED_BYTES {
			return Err(Error::Store(format!(
				"{id} archive unpacks to more than 64 MiB"
			)));
		}
		if let Some(parent) = target.parent() {
			std::fs::create_dir_all(parent)?;
		}
		let mut bytes = Vec::with_capacity(entry.size() as usize);
		entry.read_to_end(&mut bytes)?;
		std::fs::write(&target, &bytes)?;
		#[cfg(unix)]
		{
			use std::os::unix::fs::PermissionsExt;
			let mode = if entry.header().mode().unwrap_or(0o644) & 0o111 != 0 {
				0o755
			} else {
				0o644
			};
			std::fs::set_permissions(&target, std::fs::Permissions::from_mode(mode))?;
		}
	}
	Ok(())
}
