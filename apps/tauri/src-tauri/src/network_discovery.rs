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
use std::path::Path;
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
		let mut words = line.split_whitespace();
		let Some(keyword) = words.next() else {
			continue;
		};
		if !keyword.eq_ignore_ascii_case("host") {
			continue;
		}
		for host in words {
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
	let Ok(mut child) = command
		.args([
			"--all",
			"--terminate",
			"--resolve",
			"--parsable",
			"--no-db-lookup",
		])
		.stdin(Stdio::null())
		.stdout(Stdio::piped())
		.stderr(Stdio::null())
		.spawn()
	else {
		return Vec::new();
	};
	let deadline = Instant::now() + BROWSE_TIMEOUT;
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
	let mut listing = String::new();
	if let Some(mut stdout) = child.stdout.take() {
		let _ = stdout.read_to_string(&mut listing);
	}
	parse_avahi(&listing)
}

/// File servers announced on the network plus hosts from `~/.ssh/config`.
#[tauri::command]
pub async fn discover_network_servers() -> Result<Vec<DiscoveredServer>, String> {
	tokio::task::spawn_blocking(|| {
		let mut servers = browse_mdns();
		if let Some(home) = std::env::var_os("HOME") {
			for server in ssh_config_servers(Path::new(&home)) {
				if !servers.iter().any(|s| s.uri == server.uri) {
					servers.push(server);
				}
			}
		}
		servers
	})
	.await
	.map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
	use super::*;

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
Match host foo
";
		assert_eq!(parse_ssh_config(config), ["homesystem", "hs", "github.com"]);
	}

	#[test]
	fn unescape_decodes_decimal_bytes() {
		assert_eq!(unescape("Roku\\032Express"), "Roku Express");
		assert_eq!(unescape("plain\\"), "plain\\");
	}
}
