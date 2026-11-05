//! # `wing update`
//!
//! Reports whether a newer WingDrive release exists and how to install it. It
//! never downloads or replaces binaries: every published `wing` ships inside a
//! package that owns its files (the AUR packages, the `.deb`, the AppImage, the
//! Windows installers), so that package's own update path is the only safe one.
//! Overwriting `/opt/wingdrive/usr/bin/wing` behind pacman's back, for example,
//! would leave the package database describing files that are no longer there.
//!
//! The `stable` channel (the default) only considers stable releases. The `beta`
//! and `nightly` channels, or an installed pre-release, consider alpha, beta and
//! rc releases too, matching the `wingdrive-nightly` AUR package.

use std::path::{Component, Path, PathBuf};

use anyhow::{Context, Result};
use semver::Version;
use serde::Deserialize;

use crate::config::CliConfig;

const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// The fields of a GitHub release that `wing update` reads.
#[derive(Debug, Clone, Deserialize)]
struct Release {
	tag_name: String,
	#[serde(default)]
	prerelease: bool,
	#[serde(default)]
	draft: bool,
	html_url: String,
	#[serde(default)]
	assets: Vec<Asset>,
}

/// One downloadable file of a release.
#[derive(Debug, Clone, Deserialize)]
struct Asset {
	name: String,
	browser_download_url: String,
}

/// How the running `wing` was installed, which decides how to update it.
#[derive(Debug, Clone, PartialEq, Eq)]
enum InstallKind {
	/// `wingdrive-bin` or `wingdrive-nightly`, under `/opt/wingdrive`. The name is
	/// `None` when pacman could not tell which one owns the file.
	Aur { package: Option<String> },
	/// The Debian package, under `/usr`.
	Deb,
	/// Running from inside the AppImage.
	AppImage,
	/// The NSIS installer (per user) or the MSI (per machine, under Program Files).
	WindowsInstaller { msi: bool },
	/// A `cargo` build from a checkout.
	Source,
	/// Anything else, such as a copied binary.
	Unknown,
}

/// Checks the release repository and prints how to update this installation.
pub async fn run(data_dir: PathBuf) -> Result<()> {
	let config = CliConfig::load(&data_dir)?;
	let current = Version::parse(CURRENT_VERSION).context("parsing this build's version")?;
	let include_prereleases = config.update.channel != "stable" || !current.pre.is_empty();

	println!("Current version: {CURRENT_VERSION}");
	println!(
		"Update channel: {} ({})",
		config.update.channel,
		if include_prereleases {
			"stable and pre-releases"
		} else {
			"stable releases only"
		}
	);

	let releases = fetch_releases(&config.update.repo).await?;
	let Some((release, latest)) = newest_release(&releases, include_prereleases) else {
		println!("No release found in {}.", config.update.repo);
		return Ok(());
	};
	if latest <= current {
		println!("WingDrive is up to date (latest release: {latest}).");
		return Ok(());
	}

	println!("Update available: {current} -> {latest}");
	println!();
	println!("{}", instructions(&detect_install_kind(), release, &latest));
	Ok(())
}

/// Lists the repository's releases, newest first, as GitHub returns them.
async fn fetch_releases(repo: &str) -> Result<Vec<Release>> {
	let url = format!("https://api.github.com/repos/{repo}/releases?per_page=50");
	let response = reqwest::Client::builder()
		.user_agent("wingdrive-cli")
		.build()?
		.get(&url)
		.send()
		.await
		.with_context(|| format!("fetching releases of {repo}"))?;
	if !response.status().is_success() {
		anyhow::bail!(
			"Failed to fetch releases of {repo}: HTTP {}",
			response.status()
		);
	}
	Ok(response.json().await?)
}

/// The newest published release whose tag is a version, skipping pre-releases
/// unless `include_prereleases` is set.
fn newest_release(releases: &[Release], include_prereleases: bool) -> Option<(&Release, Version)> {
	releases
		.iter()
		.filter(|release| !release.draft)
		.filter_map(|release| {
			let version = Version::parse(release.tag_name.trim_start_matches('v')).ok()?;
			let prerelease = release.prerelease || !version.pre.is_empty();
			(include_prereleases || !prerelease).then_some((release, version))
		})
		.max_by(|(_, a), (_, b)| a.cmp(b))
}

/// Works out the install kind of the running executable.
fn detect_install_kind() -> InstallKind {
	let exe = std::env::current_exe()
		.and_then(|path| path.canonicalize())
		.unwrap_or_default();
	let kind = install_kind(
		&exe,
		std::env::var_os("APPIMAGE").is_some(),
		std::env::consts::OS,
	);
	match kind {
		InstallKind::Aur { .. } => InstallKind::Aur {
			package: pacman_owner(&exe),
		},
		other => other,
	}
}

/// Classifies `exe`, the canonical path of the running binary.
fn install_kind(exe: &Path, in_appimage: bool, os: &str) -> InstallKind {
	if in_appimage {
		return InstallKind::AppImage;
	}
	if exe
		.components()
		.any(|c| c == Component::Normal("target".as_ref()))
	{
		return InstallKind::Source;
	}
	match os {
		"linux" if exe.starts_with("/opt/wingdrive") => InstallKind::Aur { package: None },
		"linux" if exe.starts_with("/usr") => InstallKind::Deb,
		"windows" => {
			let path = exe.to_string_lossy().to_ascii_lowercase();
			InstallKind::WindowsInstaller {
				msi: path.contains("\\program files"),
			}
		}
		_ => InstallKind::Unknown,
	}
}

/// Asks pacman which package owns `exe`, such as `wingdrive-nightly`.
fn pacman_owner(exe: &Path) -> Option<String> {
	let output = std::process::Command::new("pacman")
		.args(["-Qqo", "--"])
		.arg(exe)
		.output()
		.ok()?;
	let name = String::from_utf8(output.stdout).ok()?.trim().to_string();
	(output.status.success() && !name.is_empty()).then_some(name)
}

/// Download link of the first asset whose name ends with `suffix`.
fn asset_url<'a>(release: &'a Release, suffix: &str) -> Option<&'a str> {
	release
		.assets
		.iter()
		.find(|asset| asset.name.ends_with(suffix))
		.map(|asset| asset.browser_download_url.as_str())
}

/// What to run or download to move this installation to `release`.
fn instructions(kind: &InstallKind, release: &Release, version: &Version) -> String {
	let page = &release.html_url;
	let download = |suffix: &str, what: &str| match asset_url(release, suffix) {
		Some(url) => format!("{what}:\n  {url}"),
		None => format!("Download it from the release page:\n  {page}"),
	};
	match kind {
		InstallKind::Aur { package } => {
			let target = if version.pre.is_empty() {
				package.as_deref().unwrap_or("wingdrive-bin")
			} else {
				"wingdrive-nightly"
			};
			let mut text = format!(
				"Update with your AUR helper:\n  yay -S {target}\n(or paru -S {target})"
			);
			if package.as_deref().is_some_and(|installed| installed != target) {
				text.push_str(&format!(
					"\n\nPre-releases ship as {target}, which replaces {}.",
					package.as_deref().unwrap_or_default()
				));
			}
			text
		}
		InstallKind::Deb => match release.assets.iter().find(|a| a.name.ends_with("-amd64.deb")) {
			Some(asset) => format!(
				"Download the Debian package and install it:\n  {}\n  sudo apt install ./{}",
				asset.browser_download_url, asset.name
			),
			None => format!("Download the Debian package from the release page:\n  {page}"),
		},
		InstallKind::AppImage => download(
			"-x86_64.AppImage",
			"Download the new AppImage and replace the one you run",
		),
		InstallKind::WindowsInstaller { msi: true } => download(
			"-x64.msi",
			"Download and run the new installer, which upgrades this one",
		),
		InstallKind::WindowsInstaller { msi: false } => download(
			"-x64-setup.exe",
			"Download and run the new installer, which upgrades this one",
		),
		InstallKind::Source => format!(
			"This is a source build. Check out the release and rebuild:\n  git fetch --tags && git checkout {}",
			release.tag_name
		),
		InstallKind::Unknown => format!("Download WingDrive {version} from:\n  {page}"),
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	/// The asset names `v3.0.0` published, the first release with Windows installers.
	const V3_ASSETS: [&str; 11] = [
		"LICENSE",
		"PKGBUILD",
		"SHA256SUMS",
		"SHA256SUMS-android",
		"SHA256SUMS-windows",
		"WingDrive-3.0.0-amd64.deb",
		"WingDrive-3.0.0-android-arm64-v8a.apk",
		"WingDrive-3.0.0-android.aab",
		"WingDrive-3.0.0-x64-setup.exe",
		"WingDrive-3.0.0-x64.msi",
		"WingDrive-3.0.0-x86_64.AppImage",
	];

	fn release(tag: &str, prerelease: bool) -> Release {
		let base = "https://github.com/bernardopg/wingdrive/releases";
		Release {
			tag_name: tag.into(),
			prerelease,
			draft: false,
			html_url: format!("{base}/tag/{tag}"),
			assets: V3_ASSETS
				.iter()
				.map(|name| Asset {
					name: name.replace("3.0.0", tag.trim_start_matches('v')),
					browser_download_url: format!(
						"{base}/download/{tag}/{}",
						name.replace("3.0.0", tag.trim_start_matches('v'))
					),
				})
				.collect(),
		}
	}

	#[test]
	fn picks_the_newest_release_of_the_channel() {
		let releases = vec![
			release("v3.1.0-beta.1", true),
			release("v3.0.0", false),
			release("v2.4.1", false),
			release("not-a-version", false),
		];
		let (stable, version) = newest_release(&releases, false).unwrap();
		assert_eq!(
			(stable.tag_name.as_str(), version.to_string()),
			("v3.0.0", "3.0.0".into())
		);
		let (any, _) = newest_release(&releases, true).unwrap();
		assert_eq!(any.tag_name, "v3.1.0-beta.1");

		// A stable release outranks the pre-releases that led to it.
		let releases = vec![release("v3.1.0", false), release("v3.1.0-rc.2", true)];
		assert_eq!(
			newest_release(&releases, true).unwrap().0.tag_name,
			"v3.1.0"
		);

		let mut draft = release("v9.0.0", false);
		draft.draft = true;
		assert_eq!(
			newest_release(&[draft], true).map(|(r, _)| r.tag_name.clone()),
			None
		);
	}

	#[test]
	fn classifies_each_install_kind() {
		let linux = |path: &str| install_kind(Path::new(path), false, "linux");
		assert_eq!(
			linux("/opt/wingdrive/usr/bin/wing"),
			InstallKind::Aur { package: None }
		);
		assert_eq!(linux("/usr/bin/wing"), InstallKind::Deb);
		assert_eq!(
			linux("/home/me/wingdrive/target/release/wing"),
			InstallKind::Source
		);
		assert_eq!(linux("/home/me/bin/wing"), InstallKind::Unknown);
		assert_eq!(
			install_kind(
				Path::new("/tmp/.mount_WingDrABC/usr/bin/wing"),
				true,
				"linux"
			),
			InstallKind::AppImage
		);
		assert_eq!(
			install_kind(
				Path::new(r"C:\Users\me\AppData\Local\WingDrive\wing.exe"),
				false,
				"windows"
			),
			InstallKind::WindowsInstaller { msi: false }
		);
		assert_eq!(
			install_kind(
				Path::new(r"C:\Program Files\WingDrive\wing.exe"),
				false,
				"windows"
			),
			InstallKind::WindowsInstaller { msi: true }
		);
	}

	#[test]
	fn points_each_install_kind_at_a_real_v3_asset() {
		let v3 = release("v3.0.0", false);
		let version = Version::parse("3.0.0").unwrap();
		let download = "https://github.com/bernardopg/wingdrive/releases/download/v3.0.0";
		let text = |kind: InstallKind| instructions(&kind, &v3, &version);

		assert!(text(InstallKind::Deb).contains(&format!("{download}/WingDrive-3.0.0-amd64.deb")));
		assert!(text(InstallKind::Deb).contains("sudo apt install ./WingDrive-3.0.0-amd64.deb"));
		assert!(text(InstallKind::AppImage)
			.contains(&format!("{download}/WingDrive-3.0.0-x86_64.AppImage")));
		assert!(text(InstallKind::WindowsInstaller { msi: false })
			.contains(&format!("{download}/WingDrive-3.0.0-x64-setup.exe")));
		assert!(text(InstallKind::WindowsInstaller { msi: true })
			.contains(&format!("{download}/WingDrive-3.0.0-x64.msi")));
		assert!(text(InstallKind::Aur {
			package: Some("wingdrive-bin".into())
		})
		.contains("yay -S wingdrive-bin"));
		assert!(text(InstallKind::Source).contains("git checkout v3.0.0"));
		assert!(text(InstallKind::Unknown).contains("/releases/tag/v3.0.0"));
	}

	#[test]
	fn sends_aur_users_to_wingdrive_nightly_for_a_prerelease() {
		let beta = release("v3.1.0-beta.1", true);
		let version = Version::parse("3.1.0-beta.1").unwrap();
		let text = instructions(
			&InstallKind::Aur {
				package: Some("wingdrive-bin".into()),
			},
			&beta,
			&version,
		);
		assert!(text.contains("yay -S wingdrive-nightly"));
		assert!(text.contains("replaces wingdrive-bin"));
	}

	#[test]
	fn falls_back_to_the_release_page_when_an_asset_is_missing() {
		let mut bare = release("v3.0.0", false);
		bare.assets.clear();
		let version = Version::parse("3.0.0").unwrap();
		for kind in [
			InstallKind::Deb,
			InstallKind::AppImage,
			InstallKind::WindowsInstaller { msi: false },
		] {
			assert!(
				instructions(&kind, &bare, &version).contains("/releases/tag/v3.0.0"),
				"{kind:?}"
			);
		}
	}
}
