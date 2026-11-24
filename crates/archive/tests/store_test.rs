//! Store tests: signed index verification, safe installs, lockfile and updates.
//!
//! The fixtures in `tests/fixtures/store/` are signed with a throwaway test key (`test.pub`),
//! never with the registry keys.

use std::path::{Path, PathBuf};

use wing_archive::adapter::script::AdapterManifest;
use wing_archive::store::{self, Store, StoreAdapter};

fn fixture(name: &str) -> Vec<u8> {
	std::fs::read(
		Path::new(env!("CARGO_MANIFEST_DIR"))
			.join("tests/fixtures/store")
			.join(name),
	)
	.unwrap()
}

fn fixture_text(name: &str) -> String {
	String::from_utf8(fixture(name)).unwrap()
}

fn test_store(root: &Path) -> Store {
	Store::with_keys(root.join("adapters"), vec![fixture_text("test.pub")]).unwrap()
}

fn accepted(store: &Store, name: &str) -> store::StoreIndex {
	store
		.accept_index(
			&fixture(&format!("{name}.json")),
			&fixture_text(&format!("{name}.json.minisig")),
		)
		.unwrap()
}

/// A gzip tar with one entry per (path, bytes), built by hand so tests can write paths the
/// `tar` builder itself refuses.
fn raw_archive(entries: &[(&str, &[u8], u8)]) -> Vec<u8> {
	let mut tar = Vec::new();
	for (path, data, kind) in entries {
		let mut header = [0u8; 512];
		header[..path.len()].copy_from_slice(path.as_bytes());
		header[100..107].copy_from_slice(b"0000644");
		header[108..115].copy_from_slice(b"0000000");
		header[116..123].copy_from_slice(b"0000000");
		header[124..135].copy_from_slice(format!("{:011o}", data.len()).as_bytes());
		header[136..147].copy_from_slice(b"00000000000");
		header[156] = *kind;
		if *kind == b'2' {
			header[157..157 + 11].copy_from_slice(b"/etc/passwd");
		}
		header[257..263].copy_from_slice(b"ustar\0");
		header[263..265].copy_from_slice(b"00");
		header[148..156].copy_from_slice(b"        ");
		let sum: u32 = header.iter().map(|b| *b as u32).sum();
		header[148..155].copy_from_slice(format!("{sum:06o}\0").as_bytes());
		tar.extend_from_slice(&header);
		tar.extend_from_slice(data);
		tar.resize(tar.len().div_ceil(512) * 512, 0);
	}
	tar.extend_from_slice(&[0u8; 1024]);
	let mut gz = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
	std::io::Write::write_all(&mut gz, &tar).unwrap();
	gz.finish().unwrap()
}

/// The fixture adapter, pointing at `archive` by checksum.
fn adapter_for(index: &store::StoreIndex, archive: &[u8]) -> StoreAdapter {
	let mut adapter = index.adapter("sample").unwrap().clone();
	adapter.artifacts[0].sha256 = store::sha256_hex(archive);
	adapter
}

#[test]
fn accepts_an_index_signed_by_a_trusted_key_and_caches_it() {
	let tmp = tempfile::tempdir().unwrap();
	let store = test_store(tmp.path());
	let index = accepted(&store, "index");
	assert_eq!(index.sequence, 5);
	assert_eq!(index.adapter("sample").unwrap().version, "1.0.0");
	assert_eq!(store.cached_index().unwrap().unwrap().sequence, 5);
}

#[test]
fn the_registry_keys_do_not_trust_the_test_fixture() {
	let result = store::verify_index(
		&fixture("index.json"),
		&fixture_text("index.json.minisig"),
		&[store::ACTIVE_KEY, store::STANDBY_KEY],
	);
	assert!(result
		.unwrap_err()
		.to_string()
		.contains("does not match the registry keys"));
}

#[test]
fn refuses_a_tampered_index() {
	let mut index = fixture("index.json");
	let at = index.iter().position(|b| *b == b'5').unwrap();
	index[at] = b'9';
	let result = store::verify_index(
		&index,
		&fixture_text("index.json.minisig"),
		&[&fixture_text("test.pub")],
	);
	assert!(result.is_err());
}

#[test]
fn refuses_an_index_older_than_the_last_accepted() {
	let tmp = tempfile::tempdir().unwrap();
	let store = test_store(tmp.path());
	accepted(&store, "index");
	let error = store
		.accept_index(
			&fixture("index-older.json"),
			&fixture_text("index-older.json.minisig"),
		)
		.unwrap_err();
	assert!(
		error
			.to_string()
			.contains("older (4) than the last one accepted (5)"),
		"{error}"
	);
}

#[test]
fn installs_records_the_lockfile_and_removes() {
	let tmp = tempfile::tempdir().unwrap();
	let store = test_store(tmp.path());
	let index = accepted(&store, "index");
	let entry = store
		.install(
			index.adapter("sample").unwrap(),
			&fixture("sample-1.0.0.tar.gz"),
		)
		.unwrap();
	assert_eq!(entry.version, "1.0.0");
	assert_eq!(
		entry.commit.as_deref(),
		Some("0000000000000000000000000000000000000000")
	);
	let dir = store.installed_dir().join("sample");
	let manifest = AdapterManifest::from_file(&dir.join("adapter.toml")).unwrap();
	assert_eq!(
		manifest.adapter.permissions.read,
		vec!["~/Notes".to_string()]
	);
	assert_eq!(store.installed().unwrap()["sample"].version, "1.0.0");

	store.remove("sample").unwrap();
	assert!(!dir.exists());
	assert!(store.installed().unwrap().is_empty());
}

#[test]
fn refuses_an_archive_whose_checksum_differs_from_the_index() {
	let tmp = tempfile::tempdir().unwrap();
	let store = test_store(tmp.path());
	let index = accepted(&store, "index");
	let mut archive = fixture("sample-1.0.0.tar.gz");
	let last = archive.len() - 1;
	archive[last] ^= 0xff;
	let error = store
		.install(index.adapter("sample").unwrap(), &archive)
		.unwrap_err();
	assert!(error.to_string().contains("checksum"), "{error}");
	assert!(!store.installed_dir().join("sample").exists());
}

#[test]
fn refuses_archives_that_escape_the_adapter_folder() {
	let tmp = tempfile::tempdir().unwrap();
	let store = test_store(tmp.path());
	let index = accepted(&store, "index");
	for (path, kind, expected) in [
		("sample/../evil.txt", b'0', "leaves the adapter folder"),
		("/etc/evil.txt", b'0', "outside sample/"),
		("other/adapter.toml", b'0', "outside sample/"),
		("sample/link", b'2', "link or special file"),
	] {
		let archive = raw_archive(&[(path, b"x", kind)]);
		let error = store
			.install(&adapter_for(&index, &archive), &archive)
			.unwrap_err();
		assert!(error.to_string().contains(expected), "{path}: {error}");
	}
	assert!(!tmp.path().join("evil.txt").exists());
	assert!(!store.installed_dir().join("sample").exists());
}

#[test]
fn refuses_an_archive_whose_manifest_disagrees_with_the_index() {
	let tmp = tempfile::tempdir().unwrap();
	let store = test_store(tmp.path());
	let index = accepted(&store, "index");
	let manifest = String::from_utf8(fixture_manifest())
		.unwrap()
		.replace("1.0.0", "9.9.9");
	let archive = raw_archive(&[
		("sample/adapter.toml", manifest.as_bytes(), b'0'),
		("sample/sync.py", b"", b'0'),
	]);
	let error = store
		.install(&adapter_for(&index, &archive), &archive)
		.unwrap_err();
	assert!(error.to_string().contains("holds sample 9.9.9"), "{error}");
}

fn fixture_manifest() -> Vec<u8> {
	let tmp = tempfile::tempdir().unwrap();
	let bytes = fixture("sample-1.0.0.tar.gz");
	let mut archive = tar::Archive::new(flate2::read::GzDecoder::new(&bytes[..]));
	archive.unpack(tmp.path()).unwrap();
	std::fs::read(tmp.path().join("sample/adapter.toml")).unwrap()
}

#[test]
fn reports_updates_with_new_permissions_and_revocations() {
	let tmp = tempfile::tempdir().unwrap();
	let store = test_store(tmp.path());
	let index = accepted(&store, "index");
	store
		.install(
			index.adapter("sample").unwrap(),
			&fixture("sample-1.0.0.tar.gz"),
		)
		.unwrap();
	assert!(store.updates(&index).unwrap().is_empty());

	let newer = accepted(&store, "index-update");
	let [update] = store.updates(&newer).unwrap().try_into().unwrap();
	assert_eq!(
		(update.installed.as_str(), update.available.as_str()),
		("1.0.0", "1.1.0")
	);
	assert!(update.new_permissions);
	assert!(update.security);
	assert!(newer.revocation("sample", "1.0.0").is_some());
}

#[test]
fn manifests_must_name_a_supported_contract_and_a_safe_entry() {
	let manifest = String::from_utf8(fixture_manifest()).unwrap();
	assert!(AdapterManifest::parse(&manifest).is_ok());
	for (from, to, expected) in [
		("contract = 1", "contract = 2", "supports up to 1"),
		(
			"entry = \"sync.py\"",
			"entry = \"../outside.py\"",
			"inside the adapter folder",
		),
		(
			"entry = \"sync.py\"",
			"",
			"needs runtime.kind and runtime.entry",
		),
	] {
		let error = AdapterManifest::parse(&manifest.replace(from, to)).unwrap_err();
		assert!(error.to_string().contains(expected), "{to}: {error}");
	}
}

#[test]
fn picks_the_archive_for_this_platform() {
	let tmp = tempfile::tempdir().unwrap();
	let store = test_store(tmp.path());
	let mut adapter = accepted(&store, "index").adapter("sample").unwrap().clone();
	assert_eq!(adapter.artifact().unwrap().target, "any");
	let mut native = adapter.artifacts[0].clone();
	native.target = store::current_target();
	native.url = "native".into();
	adapter.artifacts.push(native);
	assert_eq!(adapter.artifact().unwrap().url, "native");
	assert!(adapter.supports_this_platform());
	let _: PathBuf = store.installed_dir();
}
