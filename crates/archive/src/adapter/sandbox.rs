//! Confinement for contract 1 adapters.
//!
//! On Linux every adapter runs inside bubblewrap (`bwrap`): a new mount, PID, IPC and UTS
//! namespace whose filesystem holds only the system (`/usr`, read-only), the adapter's own
//! folder (read-only), the paths it declares in `[adapter.permissions] read` (read-only), its
//! state folder (read-write) and an empty `/tmp`. The network namespace is shared only when the
//! adapter declares at least one host; otherwise the adapter has no network at all. Host names in
//! `network` are reviewed in the registry, not enforced here.
//!
//! Inside a Flatpak the app's own sandbox applies instead, because bubblewrap cannot create
//! namespaces there. On macOS and Windows adapters are not confined yet.
//!
//! `WINGDRIVE_ADAPTER_SANDBOX=off` turns confinement off on purpose (for containers without user
//! namespaces); without it a Linux system lacking `bwrap` refuses to run adapters.

use std::path::{Path, PathBuf};

use crate::error::{Error, Result};

/// Environment variable that turns confinement off.
pub const SANDBOX_ENV: &str = "WINGDRIVE_ADAPTER_SANDBOX";

/// How adapters run on this system.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Isolation {
	/// bubblewrap confines the adapter to its declared paths.
	Bubblewrap,
	/// The Flatpak sandbox around WingDrive applies; adapters are not confined further.
	Flatpak,
	/// No confinement: macOS, Windows, or turned off with `WINGDRIVE_ADAPTER_SANDBOX=off`.
	None,
}

impl Isolation {
	/// A short name for logs and the interface.
	pub fn as_str(self) -> &'static str {
		match self {
			Self::Bubblewrap => "bubblewrap",
			Self::Flatpak => "flatpak",
			Self::None => "none",
		}
	}
}

/// What confines adapters here, or an error when Linux has no way to confine them.
pub fn isolation() -> Result<Isolation> {
	if std::env::var(SANDBOX_ENV).as_deref() == Ok("off") {
		return Ok(Isolation::None);
	}
	if !cfg!(target_os = "linux") {
		return Ok(Isolation::None);
	}
	if Path::new("/.flatpak-info").exists() {
		return Ok(Isolation::Flatpak);
	}
	match probe() {
		Probe::Works => Ok(Isolation::Bubblewrap),
		Probe::Missing => Err(Error::AdapterSync(format!(
			"adapters run inside bubblewrap on Linux, and bwrap was not found: install the \
			 bubblewrap package, or set {SANDBOX_ENV}=off to run adapters without confinement"
		))),
		Probe::Fails(reason) => Err(Error::AdapterSync(format!(
			"adapters run inside bubblewrap on Linux, and bwrap cannot start here ({reason}). \
			 Containers need user namespaces; otherwise set {SANDBOX_ENV}=off to run adapters \
			 without confinement"
		))),
	}
}

#[derive(Debug, Clone)]
enum Probe {
	Works,
	Missing,
	Fails(String),
}

/// Starts bubblewrap once per process with the same namespaces adapters get. Finding the binary
/// is not enough: Docker's default seccomp profile and Ubuntu's AppArmor rules can forbid user
/// namespaces.
fn probe() -> Probe {
	static PROBE: std::sync::OnceLock<Probe> = std::sync::OnceLock::new();
	PROBE
		.get_or_init(|| {
			let Some(bwrap) = bwrap() else {
				return Probe::Missing;
			};
			let grant = Grant {
				adapter_dir: PathBuf::from("/usr"),
				program: PathBuf::from("/usr/bin/true"),
				..Default::default()
			};
			let output = std::process::Command::new(bwrap)
				.args(bwrap_args(&grant))
				.arg("/usr/bin/true")
				.stdin(std::process::Stdio::null())
				.output();
			match output {
				Ok(output) if output.status.success() => Probe::Works,
				Ok(output) => Probe::Fails(
					String::from_utf8_lossy(&output.stderr)
						.lines()
						.last()
						.unwrap_or("unknown error")
						.trim()
						.to_string(),
				),
				Err(error) => Probe::Fails(error.to_string()),
			}
		})
		.clone()
}

/// The bubblewrap binary, if installed.
fn bwrap() -> Option<PathBuf> {
	find_in_path("bwrap")
}

fn find_in_path(name: &str) -> Option<PathBuf> {
	let path = std::env::var_os("PATH")?;
	std::env::split_paths(&path)
		.map(|dir| dir.join(name))
		.find(|candidate| candidate.is_file())
}

/// What one adapter run may touch.
#[derive(Debug, Clone, Default)]
pub struct Grant {
	/// The adapter's installed folder, mounted read-only.
	pub adapter_dir: PathBuf,
	/// The source's state folder, mounted read-write.
	pub state_dir: Option<PathBuf>,
	/// Declared read paths, already expanded.
	pub read: Vec<PathBuf>,
	/// The adapter declared hosts, so it gets the network.
	pub network: bool,
	/// The interpreter or binary that starts the adapter.
	pub program: PathBuf,
}

/// Expands `~`, `~/…` and `{config.<key>}` in a declared read path.
///
/// Returns `None` for paths that stay relative or name a config field the source did not set,
/// so they are never mounted.
pub fn expand(declared: &str, home: Option<&Path>, config: &serde_json::Value) -> Option<PathBuf> {
	let mut text = declared.trim().to_string();
	while let Some(start) = text.find("{config.") {
		let end = start + text[start..].find('}')?;
		let key = &text[start + "{config.".len()..end];
		let value = config.get(key)?.as_str()?.trim();
		if value.is_empty() {
			return None;
		}
		text.replace_range(start..=end, value);
	}
	let path = if text == "~" {
		home?.to_path_buf()
	} else if let Some(rest) = text.strip_prefix("~/") {
		home?.join(rest)
	} else {
		PathBuf::from(text)
	};
	let path = expand_user_home(path, home);
	path.is_absolute().then_some(path)
}

/// `~` inside a config value (a user typing `~/Notes`) means the home folder too.
fn expand_user_home(path: PathBuf, home: Option<&Path>) -> PathBuf {
	match (path.strip_prefix("~"), home) {
		(Ok(rest), Some(home)) => home.join(rest),
		_ => path,
	}
}

/// The bubblewrap arguments that confine `grant`, ending with `--` before the program.
pub fn bwrap_args(grant: &Grant) -> Vec<String> {
	let mut args: Vec<String> = [
		"--unshare-all",
		"--die-with-parent",
		"--new-session",
		"--cap-drop",
		"ALL",
		"--proc",
		"/proc",
		"--dev",
		"/dev",
		"--tmpfs",
		"/tmp",
		"--ro-bind",
		"/usr",
		"/usr",
	]
	.iter()
	.map(|s| s.to_string())
	.collect();
	if grant.network {
		args.push("--share-net".into());
	}

	// Merged-/usr systems link these into /usr; others have real folders.
	for top in ["/bin", "/sbin", "/lib", "/lib64", "/lib32"] {
		let path = Path::new(top);
		match std::fs::read_link(path) {
			Ok(target) => push(&mut args, "--symlink", &target, path),
			Err(_) if path.is_dir() => push(&mut args, "--ro-bind", path, path),
			Err(_) => {}
		}
	}

	// What the dynamic loader, TLS, name resolution and time zones read.
	for system in [
		"/etc/ld.so.cache",
		"/etc/ld.so.conf",
		"/etc/ld.so.conf.d",
		"/etc/ssl",
		"/etc/pki",
		"/etc/ca-certificates",
		"/etc/crypto-policies",
		"/etc/localtime",
		"/etc/nsswitch.conf",
		"/etc/hosts",
		"/etc/host.conf",
		"/etc/gai.conf",
		"/etc/resolv.conf",
		"/etc/services",
		"/etc/protocols",
		"/etc/python3",
		"/run/systemd/resolve",
		"/nix/store",
	] {
		let path = Path::new(system);
		if path.exists() {
			push(&mut args, "--ro-bind", path, path);
		}
	}

	// An interpreter outside /usr (pyenv, /opt, Homebrew on Linux) brings its own prefix.
	if let Some(prefix) = program_prefix(&grant.program) {
		push(&mut args, "--ro-bind", &prefix, &prefix);
	}

	push(
		&mut args,
		"--ro-bind",
		&grant.adapter_dir,
		&grant.adapter_dir,
	);

	for path in &grant.read {
		if let Ok(real) = path.canonicalize() {
			// Mount the real file at the path the adapter asks for, so links still resolve.
			push(&mut args, "--ro-bind", &real, path);
		}
	}

	if let Some(state) = &grant.state_dir {
		push(&mut args, "--bind", state, state);
	}

	args.push("--chdir".into());
	args.push(grant.adapter_dir.to_string_lossy().into_owned());
	args.push("--".into());
	args
}

fn push(args: &mut Vec<String>, flag: &str, source: &Path, dest: &Path) {
	args.push(flag.to_string());
	args.push(source.to_string_lossy().into_owned());
	args.push(dest.to_string_lossy().into_owned());
}

/// The install prefix of a program that lives outside `/usr`, such as `~/.pyenv/versions/3.12`.
fn program_prefix(program: &Path) -> Option<PathBuf> {
	let resolved = if program.is_absolute() {
		program.to_path_buf()
	} else {
		find_in_path(program.to_str()?)?
	};
	let real = resolved.canonicalize().ok()?;
	if real.starts_with("/usr") || real.starts_with("/nix/store") {
		return None;
	}
	// <prefix>/bin/python3
	let prefix = real.parent()?.parent()?.to_path_buf();
	(prefix != Path::new("/")).then_some(prefix)
}

/// Address space an adapter process may use.
pub const MEMORY_LIMIT: u64 = 2 * 1024 * 1024 * 1024;

/// Caps the adapter's memory at [`MEMORY_LIMIT`] and its CPU time at `cpu_seconds`, so a
/// runaway adapter cannot take the machine down. The wall-clock timeout is enforced by the
/// caller; CPU time can only be lower.
pub fn limit_resources(command: &mut tokio::process::Command, cpu_seconds: u64) {
	#[cfg(unix)]
	{
		// SAFETY: the closure runs in the forked child before exec and calls only setrlimit,
		// which is async-signal-safe.
		unsafe {
			command.pre_exec(move || {
				let memory = libc::rlimit {
					rlim_cur: MEMORY_LIMIT as libc::rlim_t,
					rlim_max: MEMORY_LIMIT as libc::rlim_t,
				};
				let cpu = libc::rlimit {
					rlim_cur: cpu_seconds as libc::rlim_t,
					rlim_max: cpu_seconds.saturating_add(5) as libc::rlim_t,
				};
				if libc::setrlimit(libc::RLIMIT_AS, &memory) != 0
					|| libc::setrlimit(libc::RLIMIT_CPU, &cpu) != 0
				{
					return Err(std::io::Error::last_os_error());
				}
				Ok(())
			});
		}
	}
	#[cfg(not(unix))]
	{
		let _ = (command, cpu_seconds);
	}
}

/// Wraps `program` in bubblewrap for `grant`.
pub fn wrap(grant: &Grant) -> Result<tokio::process::Command> {
	let bwrap = bwrap().ok_or_else(|| Error::AdapterSync("bwrap was not found".into()))?;
	let mut command = tokio::process::Command::new(bwrap);
	command.args(bwrap_args(grant));
	command.arg(&grant.program);
	Ok(command)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn expands_home_and_config_fields() {
		let home = Path::new("/home/ana");
		let config =
			serde_json::json!({ "vault_path": "/data/Notes", "tilde": "~/Mail", "empty": "" });
		assert_eq!(
			expand("~/.config/gh/hosts.yml", Some(home), &config),
			Some(PathBuf::from("/home/ana/.config/gh/hosts.yml"))
		);
		assert_eq!(
			expand("{config.vault_path}", Some(home), &config),
			Some(PathBuf::from("/data/Notes"))
		);
		assert_eq!(
			expand("{config.tilde}", Some(home), &config),
			Some(PathBuf::from("/home/ana/Mail"))
		);
		assert_eq!(expand("{config.empty}", Some(home), &config), None);
		assert_eq!(expand("{config.missing}", Some(home), &config), None);
		assert_eq!(expand("relative/path", Some(home), &config), None);
		assert_eq!(expand("~/x", None, &config), None);
	}

	#[test]
	fn network_is_shared_only_when_declared() {
		let mut grant = Grant {
			adapter_dir: PathBuf::from("/opt/adapter"),
			program: PathBuf::from("/usr/bin/python3"),
			..Default::default()
		};
		assert!(!bwrap_args(&grant).contains(&"--share-net".to_string()));
		grant.network = true;
		assert!(bwrap_args(&grant).contains(&"--share-net".to_string()));
	}
}
