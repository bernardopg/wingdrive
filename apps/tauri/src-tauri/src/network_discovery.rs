//! # Servers on the local network
//!
//! Finds file servers the user can connect to without typing an address:
//!
//! - services announced over mDNS/DNS-SD (SFTP, SSH, SMB, FTP, WebDAV, NFS),
//!   read from `avahi-browse`, which every desktop running Avahi has;
//! - hosts named in `~/.ssh/config`, which covers machines that do not
//!   announce themselves, such as a headless home server reached by alias.
//!
//! Each result carries a gvfs URI, so connecting reuses
//! [`crate::gvfs::mount_network_location`].
//!
//! ## Example
//! ```rust,ignore
//! let servers = network_discovery::discover_network_servers().await?;
//! ```

use std::collections::HashSet;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use serde::Serialize;

/// How long `avahi-browse` may run; it answers from its cache in well under that.
const BROWSE_TIMEOUT: Duration = Duration::from_secs(5);

/// A file server found on the network or in the SSH configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DiscoveredServer {
	/// Readable name, such as `homesystem (SFTP)`.
	pub name: String,
	/// Address passed to `gio mount`, such as `sftp://homesystem/`.
	pub uri: String,
	/// URI scheme: `sftp`, `smb`, `ftp`, `dav`, `davs` or `nfs`.
	pub kind: String,
	/// Where the server came from: `mdns` or `ssh-config`.
	pub source: String,
}

/// DNS-SD service types WingDrive can mount, with the scheme and label for each.
const SERVICES: &[(&str, &str, &str)] = &[
	("_sftp-ssh._tcp", "sftp", "SFTP"),
	("_ssh._tcp", "sftp", "SFTP"),
	("_smb._tcp", "smb", "SMB"),
	("_ftp._tcp", "ftp", "FTP"),
	("_webdav._tcp", "dav", "WebDAV"),
	("_webdavs._tcp", "davs", "WebDAV"),
	("_nfs._tcp", "nfs", "NFS"),
];

/// Default port of each scheme, left out of the URI when the service uses it.
fn default_port(scheme: &str) -> u16 {
	match scheme {
		"sftp" => 22,
		"smb" => 445,
		"ftp" => 21,
		"dav" => 80,
		"davs" => 443,
		"nfs" => 2049,
		_ => 0,
	}
}

/// Decodes the `\DDD` decimal escapes `avahi-browse -p` writes for bytes
/// outside printable ASCII, such as `Roku\032Express`.
fn unescape(text: &str) -> String {
	let bytes = text.as_bytes();
	let mut out = Vec::with_capacity(bytes.len());
	let mut i = 0;
	while i < bytes.len() {
		if bytes[i] == b'\\' && i + 4 <= bytes.len() {
			let digits = &bytes[i + 1..i + 4];
			if digits.iter().all(u8::is_ascii_digit) {
				let value = digits
					.iter()
					.fold(0u32, |n, d| n * 10 + u32::from(d - b'0'));
				if let Ok(value) = u8::try_from(value) {
					out.push(value);
					i += 4;
					continue;
				}
			}
		}
		out.push(bytes[i]);
		i += 1;
	}
	String::from_utf8_lossy(&out).into_owned()
}

/// Reads `path=/share` from a TXT record such as `"path=/dav" "u=me"`.
fn txt_path(txt: &str) -> Option<String> {
	txt.split('"')
		.find_map(|field| field.strip_prefix("path="))
		.map(|path| path.trim_start_matches('/').to_string())
		.filter(|path| !path.is_empty())
}

/// Turns the resolved lines of `avahi-browse -ptrk` into servers, one per URI.
///
/// A resolved line reads
/// `=;iface;proto;name;type;domain;hostname;address;port;txt`.
pub fn parse_avahi(listing: &str) -> Vec<DiscoveredServer> {
	let mut seen = HashSet::new();
	let mut servers = Vec::new();
	for line in listing.lines() {
		let fields: Vec<&str> = line.splitn(10, ';').collect();
		if fields.len() < 9 || fields[0] != "=" {
			continue;
		}
		let Some(&(_, scheme, label)) = SERVICES.iter().find(|(kind, _, _)| *kind == fields[4])
		else {
			continue;
		};
		let host = fields[6].trim_end_matches('.');
		if host.is_empty() {
			continue;
		}
		let port: u16 = fields[8].parse().unwrap_or(0);
		let authority = if port == 0 || port == default_port(scheme) {
			host.to_string()
		} else {
			format!("{host}:{port}")
		};
		let path = fields
			.get(9)
			.and_then(|txt| txt_path(txt))
			.unwrap_or_default();
		let uri = format!("{scheme}://{authority}/{path}");
		if !seen.insert(uri.clone()) {
			continue;
		}
		servers.push(DiscoveredServer {
			name: format!("{} ({label})", unescape(fields[3])),
			uri,
			kind: scheme.to_string(),
			source: "mdns".to_string(),
		});
	}
	servers
}

/// Host aliases from an OpenSSH client configuration, skipping patterns.
pub fn parse_ssh_config(config: &str) -> Vec<String> {
	let mut hosts = Vec::new();
	for line in config.lines() {
		let line = line.trim();
		// OpenSSH also accepts `Host=name` and `Host = name`.
		let end = line
			.find(|c: char| c.is_whitespace() || c == '=')
			.unwrap_or(line.len());
		let (keyword, rest) = line.split_at(end);
		if !keyword.eq_ignore_ascii_case("host") {
			continue;
		}
		let rest = rest.trim_start();
		let rest = rest.strip_prefix('=').unwrap_or(rest);
		for host in rest.split_whitespace() {
			if host.starts_with('#') {
				break;
			}
			if host.contains(['*', '?', '!']) || hosts.iter().any(|h| h == host) {
				continue;
			}
			hosts.push(host.to_string());
		}
	}
	hosts
}

fn ssh_config_servers(home: &Path) -> Vec<DiscoveredServer> {
	let Ok(config) = std::fs::read_to_string(home.join(".ssh/config")) else {
		return Vec::new();
	};
	parse_ssh_config(&config)
		.into_iter()
		.map(|host| DiscoveredServer {
			name: format!("{host} (SFTP)"),
			uri: format!("sftp://{host}/"),
			kind: "sftp".to_string(),
			source: "ssh-config".to_string(),
		})
		.collect()
}

/// Runs `avahi-browse` once, terminated after [`BROWSE_TIMEOUT`].
///
/// A host without Avahi yields no servers instead of an error: the SSH
/// configuration still lists hosts there.
fn browse_mdns() -> Vec<DiscoveredServer> {
	let mut command = Command::new("avahi-browse");
	file_opening_linux::use_host_environment(&mut command);
	command.args([
		"--all",
		"--terminate",
		"--resolve",
		"--parsable",
		"--no-db-lookup",
	]);
	browse_with(command, BROWSE_TIMEOUT)
}

/// Runs a prepared browse command for at most `timeout` and parses what it
/// printed until then.
fn browse_with(mut command: Command, timeout: Duration) -> Vec<DiscoveredServer> {
	let Ok(mut child) = command
		.stdin(Stdio::null())
		.stdout(Stdio::piped())
		.stderr(Stdio::null())
		.spawn()
	else {
		return Vec::new();
	};
	// Read while it runs: on a busy network the listing outgrows the pipe
	// buffer, and a blocked writer would only stop at the time limit.
	let reader = child.stdout.take().map(|mut stdout| {
		std::thread::spawn(move || {
			let mut listing = String::new();
			let _ = stdout.read_to_string(&mut listing);
			listing
		})
	});
	let deadline = Instant::now() + timeout;
	loop {
		match child.try_wait() {
			Ok(Some(_)) => break,
			Ok(None) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(50)),
			_ => {
				let _ = child.kill();
				let _ = child.wait();
				break;
			}
		}
	}
	let listing = reader
		.and_then(|reader| reader.join().ok())
		.unwrap_or_default();
	parse_avahi(&listing)
}

/// File servers announced on the network plus hosts from `~/.ssh/config`.
#[tauri::command]
pub async fn discover_network_servers() -> Result<Vec<DiscoveredServer>, String> {
	tokio::task::spawn_blocking(|| {
		let home = std::env::var_os("HOME").map(PathBuf::from);
		merge_servers(browse_mdns(), home.as_deref())
	})
	.await
	.map_err(|e| e.to_string())
}

/// Announced servers first, then SSH hosts whose address none of them has.
fn merge_servers(mut servers: Vec<DiscoveredServer>, home: Option<&Path>) -> Vec<DiscoveredServer> {
	if let Some(home) = home {
		for server in ssh_config_servers(home) {
			if !servers.iter().any(|s| s.uri == server.uri) {
				servers.push(server);
			}
		}
	}
	servers
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn ssh_aliases_are_unique_and_pattern_only_blocks_add_no_servers() {
		assert_eq!(parse_ssh_config("# Host ignored\nHost nas nas *.lan !blocked ?\nHOST = backup nas # skip\nHostName other\nMatch host ignored\nHost\n"), ["nas", "backup"]);
	}

	#[test]
	fn unreadable_or_non_utf8_ssh_configs_yield_no_servers() {
		let root = tempfile::tempdir().unwrap();
		let ssh = root.path().join(".ssh");
		std::fs::create_dir(&ssh).unwrap();
		std::fs::write(ssh.join("config"), b"Host \xff\n").unwrap();
		assert!(ssh_config_servers(root.path()).is_empty());
		std::fs::remove_file(ssh.join("config")).unwrap();
		std::fs::create_dir(ssh.join("config")).unwrap();
		assert!(ssh_config_servers(root.path()).is_empty());
	}

	#[test]
	fn mdns_deduplicates_by_uri_and_preserves_nondefault_ports_and_txt_paths() {
		let listing = "=;e;IPv4;Jos\\195\\169;_ssh._tcp;local;nas.local.;1;22;\"u=me\" \"path=///docs;2026\"\n=;e;IPv6;Duplicate;_sftp-ssh._tcp;local;nas.local;::1;22;\"path=/docs;2026\"\n=;e;IPv4;Other;_sftp-ssh._tcp;local;nas.local;1;2222;\"path=/\"\n";
		let servers = parse_avahi(listing);
		assert_eq!(servers.len(), 2);
		assert_eq!(servers[0].name, "José (SFTP)");
		assert_eq!(servers[0].uri, "sftp://nas.local/docs;2026");
		assert_eq!(servers[1].uri, "sftp://nas.local:2222/");
	}

	const LISTING: &str = "\
+;eth0;IPv4;nas;_smb._tcp;local
=;eth0;IPv4;nas;_smb._tcp;local;nas.local;192.168.1.5;445;
=;eth0;IPv6;nas;_smb._tcp;local;nas.local;fe80::1;445;
=;eth0;IPv4;My\\032Box;_sftp-ssh._tcp;local;box.local;192.168.1.6;2222;
=;eth0;IPv4;cloud;_webdavs._tcp;local;cloud.local;192.168.1.7;443;\"path=/remote.php/dav\"
=;eth0;IPv4;Casa;_home-assistant._tcp;local;ha.local;192.168.1.8;8123;
";

	#[test]
	fn mdns_services_become_mount_uris() {
		let servers = parse_avahi(LISTING);
		let uris: Vec<&str> = servers.iter().map(|s| s.uri.as_str()).collect();
		assert_eq!(
			uris,
			[
				"smb://nas.local/",
				"sftp://box.local:2222/",
				"davs://cloud.local/remote.php/dav"
			]
		);
		assert_eq!(servers[1].name, "My Box (SFTP)");
		assert_eq!(servers[0].source, "mdns");
	}

	#[test]
	fn ssh_config_lists_concrete_hosts() {
		let config = "\
Host homesystem hs
  HostName 192.168.15.20
Host *.example.com !bad
host  github.com  # code
Host=nas
Host = backup
Match host foo
";
		assert_eq!(
			parse_ssh_config(config),
			["homesystem", "hs", "github.com", "nas", "backup"]
		);
	}

	#[test]
	fn ssh_config_hosts_become_sftp_servers_and_missing_config_is_empty() {
		let home = tempfile::tempdir().unwrap();
		assert!(ssh_config_servers(home.path()).is_empty());
		std::fs::create_dir(home.path().join(".ssh")).unwrap();
		std::fs::write(home.path().join(".ssh/config"), "Host nas *.lan\n").unwrap();
		let servers = ssh_config_servers(home.path());
		assert_eq!(servers.len(), 1);
		assert_eq!(servers[0].uri, "sftp://nas/");
		assert_eq!(servers[0].name, "nas (SFTP)");
		assert_eq!(servers[0].source, "ssh-config");
	}

	#[test]
	fn avahi_lines_without_host_or_known_service_are_skipped() {
		let listing = "\
=;eth0;IPv4;x;_smb._tcp;local;;192.168.1.5;445;
=;eth0;IPv4;x;_ftp._tcp;local;ftp.local.;192.168.1.5;2121;
=;eth0;IPv4;x;_nfs._tcp;local;nfs.local;192.168.1.5;notaport;
=;short;line
";
		let uris: Vec<String> = parse_avahi(listing).into_iter().map(|s| s.uri).collect();
		assert_eq!(uris, ["ftp://ftp.local:2121/", "nfs://nfs.local/"]);
		assert_eq!(unescape("\\999x"), "\\999x");
	}

	#[test]
	fn unescape_decodes_decimal_bytes() {
		assert_eq!(unescape("Roku\\032Express"), "Roku Express");
		assert_eq!(unescape("plain\\"), "plain\\");
	}
}
