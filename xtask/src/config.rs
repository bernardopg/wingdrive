//! Cargo config generation from template

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

use crate::system::{get_best_linker, get_rust_targets, Os, SystemInfo};
use crate::template::Value;

/// Half the CPUs on developer machines: a full-width build swaps and overheats the NVMe.
/// CI keeps every CPU. `CARGO_BUILD_JOBS` still overrides the generated value.
fn build_jobs() -> usize {
	let cpus = std::thread::available_parallelism().map_or(1, |n| n.get());
	if std::env::var_os("CI").is_some() {
		cpus
	} else {
		(cpus / 2).max(1)
	}
}

/// Generate .cargo/config.toml from the mustache template
pub fn generate_cargo_config(
	root: &Path,
	native_deps_dir: Option<&Path>,
	mobile_deps_dir: Option<&Path>,
) -> Result<()> {
	println!("️  Generating .cargo/config.toml...");

	let system = SystemInfo::detect()?;
	let rust_targets = get_rust_targets().unwrap_or_default();

	// Check if iOS targets are installed
	let ios_targets = [
		"aarch64-apple-ios",
		"aarch64-apple-ios-sim",
		"x86_64-apple-ios",
	];
	let has_ios = ios_targets
		.iter()
		.any(|t| rust_targets.contains(&t.to_string()));

	let android_targets = [
		"aarch64-linux-android",
		"x86_64-linux-android",
		// add more as needed
	];
	let has_android = android_targets
		.iter()
		.any(|t| rust_targets.contains(&t.to_string()));

	// Get linker info
	// Only the presence of a usable linker matters to the template.
	let has_lld = get_best_linker().is_some();

	// Prefer a system FFmpeg over the bundled native-deps blob when one is available (see
	// `system::has_system_ffmpeg` for why: version/glibc skew between the two breaks bindgen).
	let use_system_ffmpeg = matches!(system.os, Os::Linux) && crate::system::has_system_ffmpeg();
	if use_system_ffmpeg {
		println!("   ✓ Compatible system FFmpeg found via pkg-config, using it instead of bundled native-deps");
	}

	// Convert paths to strings and handle Windows backslashes
	let native_deps =
		native_deps_dir.map(|p| p.to_string_lossy().replace('\\', "\\\\").to_string());

	let protoc = native_deps_dir.map(|p| {
		let protoc_name = if cfg!(target_os = "windows") {
			"protoc.exe"
		} else {
			"protoc"
		};
		p.join("bin")
			.join(protoc_name)
			.to_string_lossy()
			.replace('\\', "\\\\")
			.to_string()
	});

	let mobile_native_deps =
		mobile_deps_dir.map(|p| p.to_string_lossy().replace('\\', "\\\\").to_string());

	let android_ndk_home = std::env::var("ANDROID_NDK")
		.or_else(|_| std::env::var("ANDROID_NDK_HOME"))
		.unwrap_or_else(|_| {
			println!("   ⚠️  Android NDK not found. Android builds will not work.");
			String::new()
		})
		.replace('\\', "\\\\");

	// Get macOS SDK path so bindgen can find system headers (errno.h, etc.)
	let sdk_path = if matches!(system.os, Os::MacOS) {
		std::process::Command::new("xcrun")
			.args(["--show-sdk-path"])
			.output()
			.ok()
			.filter(|o| o.status.success())
			.and_then(|o| String::from_utf8(o.stdout).ok())
			.map(|p| p.trim().to_string())
			.unwrap_or_default()
	} else {
		String::new()
	};

	// Android NDK host tag - the prebuilt directory is always named darwin-x86_64 on macOS,
	// but the binaries are universal (fat) binaries with native ARM64 support.
	// Google kept the path name for backwards compatibility.
	let host_tag = match system.os {
		Os::Windows => "windows-x86_64",
		Os::Linux => "linux-x86_64",
		Os::MacOS => "darwin-x86_64",
	};
	let is_win = matches!(system.os, Os::Windows);
	let is_macos = matches!(system.os, Os::MacOS);
	let is_linux = matches!(system.os, Os::Linux);
	let build_jobs = build_jobs();

	// Read template
	let template_path = root.join(".cargo").join("config.toml.mustache");
	let template =
		fs::read_to_string(&template_path).context("Failed to read config.toml.mustache")?;

	// Render template
	let rendered = crate::template::render(&template, &|name| match name {
		"nativeDeps" => native_deps.as_deref().map(Value::Str),
		"protoc" => protoc.as_deref().map(Value::Str),
		"mobileNativeDeps" => mobile_native_deps.as_deref().map(Value::Str),
		"androidNdkHome" => Some(Value::Str(&android_ndk_home)),
		"hostTag" => Some(Value::Str(host_tag)),
		"isWin" => Some(Value::Num(u64::from(is_win))),
		"isMacOS" => Some(Value::Num(u64::from(is_macos))),
		"isLinux" => Some(Value::Num(u64::from(is_linux))),
		"hasiOS" => Some(Value::Num(u64::from(has_ios))),
		"hasAndroid" => Some(Value::Num(u64::from(has_android))),
		"hasLLD" => has_lld.then_some(Value::Num(1)),
		"sdkPath" => Some(Value::Str(&sdk_path)),
		"useSystemFfmpeg" => Some(Value::Num(u64::from(use_system_ffmpeg))),
		"buildJobs" => Some(Value::Num(build_jobs as u64)),
		_ => None,
	})
	.context("Failed to render template")?;

	// Clean up extra newlines
	let rendered = rendered
		.lines()
		.filter(|line| !line.trim().is_empty() || line.is_empty())
		.collect::<Vec<_>>()
		.join("\n");

	// Validate TOML before writing
	toml::from_str::<toml::Value>(&rendered).context("Generated config is not valid TOML")?;

	// Write output
	let output_path = root.join(".cargo").join("config.toml");
	fs::write(&output_path, rendered).context("Failed to write config.toml")?;

	println!("   ✓ Generated {}", output_path.display());

	Ok(())
}
