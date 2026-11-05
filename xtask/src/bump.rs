use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

enum FileType {
	CargoToml,
	Json,
}

/// All files that contain the WingDrive product version
fn version_files(root: &Path) -> Vec<(PathBuf, FileType)> {
	vec![
		(root.join("core/Cargo.toml"), FileType::CargoToml),
		(root.join("apps/server/Cargo.toml"), FileType::CargoToml),
		(root.join("apps/cli/Cargo.toml"), FileType::CargoToml),
		(
			root.join("apps/tauri/src-tauri/Cargo.toml"),
			FileType::CargoToml,
		),
		(
			root.join("apps/tauri/wing-tauri-core/Cargo.toml"),
			FileType::CargoToml,
		),
		(
			root.join("apps/tauri/src-tauri/tauri.conf.json"),
			FileType::Json,
		),
		(root.join("apps/tauri/package.json"), FileType::Json),
	]
}

fn validate_version(version: &str) -> Result<()> {
	let (core, pre) = match version.split_once('-') {
		Some((core, pre)) => (core, Some(pre)),
		None => (version, None),
	};
	// The release workflow, the AUR packages and the Android version code accept these only.
	if let Some(pre) = pre {
		let valid = pre.split_once('.').is_some_and(|(label, n)| {
			["alpha", "beta", "rc"].contains(&label) && n.parse::<u32>().is_ok()
		});
		if !valid {
			anyhow::bail!("Invalid pre-release '{}'. Use alpha.N, beta.N or rc.N", pre);
		}
	}

	let parts: Vec<&str> = core.split('.').collect();
	if parts.len() != 3 {
		anyhow::bail!(
			"Invalid version: '{}'. Expected format: X.Y.Z or X.Y.Z-pre.N",
			version
		);
	}

	for part in &parts {
		part.parse::<u32>()
			.context(format!("Invalid version number: '{}'", part))?;
	}

	Ok(())
}

fn update_cargo_toml(content: &str, new_version: &str) -> Result<(String, String)> {
	let mut lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();
	let mut old_version = String::new();
	let mut in_package = false;
	let mut found = false;

	for line in &mut lines {
		let trimmed = line.trim();

		if trimmed == "[package]" {
			in_package = true;
			continue;
		}
		if trimmed.starts_with('[') {
			in_package = false;
			continue;
		}

		if in_package && !found {
			if let Some(rest) = trimmed.strip_prefix("version") {
				let rest = rest.trim();
				if let Some(rest) = rest.strip_prefix('=') {
					let rest = rest.trim();
					if rest.starts_with('"') && rest.ends_with('"') {
						old_version = rest[1..rest.len() - 1].to_string();
						*line = line.replace(
							&format!("\"{}\"", old_version),
							&format!("\"{}\"", new_version),
						);
						found = true;
					}
				}
			}
		}
	}

	if !found {
		anyhow::bail!("Could not find version in [package] section");
	}

	let mut result = lines.join("\n");
	if content.ends_with('\n') {
		result.push('\n');
	}

	Ok((result, old_version))
}

fn update_json(content: &str, new_version: &str) -> Result<(String, String)> {
	let mut lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();
	let mut old_version = String::new();
	let mut found = false;

	for line in &mut lines {
		let trimmed = line.trim();

		if !found && trimmed.starts_with("\"version\"") {
			if let Some(colon_pos) = trimmed.find(':') {
				let after_colon = trimmed[colon_pos + 1..].trim();
				let version_str = after_colon.trim_end_matches(',');
				if version_str.starts_with('"') && version_str.ends_with('"') {
					old_version = version_str[1..version_str.len() - 1].to_string();
					*line = line.replace(
						&format!("\"{}\"", old_version),
						&format!("\"{}\"", new_version),
					);
					found = true;
				}
			}
		}
	}

	if !found {
		anyhow::bail!("Could not find \"version\" field");
	}

	let mut result = lines.join("\n");
	if content.ends_with('\n') {
		result.push('\n');
	}

	Ok((result, old_version))
}

pub fn bump(root: &Path, new_version: &str) -> Result<()> {
	validate_version(new_version)?;

	println!("Bumping version to {}...", new_version);
	println!();

	for (path, file_type) in version_files(root) {
		let relative = path.strip_prefix(root).unwrap_or(&path);

		if !path.exists() {
			println!("  ⚠ {} (not found, skipping)", relative.display());
			continue;
		}

		let content = std::fs::read_to_string(&path)
			.context(format!("Failed to read {}", relative.display()))?;

		let (new_content, old_version) = match file_type {
			FileType::CargoToml => update_cargo_toml(&content, new_version)?,
			FileType::Json => update_json(&content, new_version)?,
		};

		if content != new_content {
			std::fs::write(&path, &new_content)
				.context(format!("Failed to write {}", relative.display()))?;
			println!(
				"  ✓ {} ({} → {})",
				relative.display(),
				old_version,
				new_version
			);
		} else {
			println!("  - {} (already {})", relative.display(), new_version);
		}
	}

	// The workspace crates carry the version too, so Cargo.lock changes with them.
	let status = Command::new("cargo")
		.args(["update", "--workspace", "--offline"])
		.current_dir(root)
		.status()
		.context("Failed to run cargo update")?;
	if !status.success() {
		anyhow::bail!("cargo update --workspace --offline failed");
	}

	// The release workflow runs the same check against the tag, so a mismatch fails here first.
	let tag = format!("v{}", new_version);
	let status = Command::new("python3")
		.args(["scripts/release/version.py", &tag])
		.current_dir(root)
		.status()
		.context("Failed to run scripts/release/version.py")?;
	if !status.success() {
		anyhow::bail!("scripts/release/version.py rejected {tag}");
	}

	// main only changes through pull requests, and the tag goes on the merge commit,
	// so committing or tagging here would put the tag on a commit main never has.
	println!();
	println!("Updated the version files and Cargo.lock to {new_version}. Next:");
	println!("  1. Add the CHANGELOG section, run `just ci-local` and open a pull request.");
	println!("  2. After it merges and main's CI finishes, tag the merge commit:");
	println!("     git tag -a {tag} -m \"WingDrive {new_version}\" && git push origin {tag}");

	Ok(())
}
