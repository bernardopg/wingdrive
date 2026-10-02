#[cfg(target_os = "macos")]
use std::process::Command;

fn main() {
	// Compile .icon to Assets.car on macOS
	#[cfg(target_os = "macos")]
	{
		let project_root = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
		let icon_source = format!("{}/../WingDrive.icon", project_root);
		let gen_dir = format!("{}/gen", project_root);

		// Create gen directory
		std::fs::create_dir_all(&gen_dir).expect("Failed to create gen directory");

		// Check if .icon file exists
		if std::path::Path::new(&icon_source).exists() {
			println!("cargo:rerun-if-changed={}", icon_source);

			// Run actool to compile .icon to Assets.car
			let output = Command::new("xcrun")
				.args([
					"actool",
					&icon_source,
					"--compile",
					&gen_dir,
					"--output-format",
					"human-readable-text",
					"--notices",
					"--warnings",
					"--errors",
					"--output-partial-info-plist",
					&format!("{}/partial.plist", gen_dir),
					"--app-icon",
					"WingDrive",
					"--include-all-app-icons",
					"--enable-on-demand-resources",
					"NO",
					"--development-region",
					"en",
					"--target-device",
					"mac",
					"--minimum-deployment-target",
					"11.0",
					"--platform",
					"macosx",
				])
				.output()
				.expect("Failed to execute actool");

			if !output.status.success() {
				eprintln!("actool failed: {}", String::from_utf8_lossy(&output.stderr));
			} else {
				println!("Successfully compiled WingDrive.icon to Assets.car");
			}
		} else {
			println!("cargo:warning=WingDrive.icon not found at {}", icon_source);
		}
	}

	// Create target-suffixed daemon binary for Tauri bundler
	// Tauri's externalBin expects binaries with target triple suffix
	let target_triple = std::env::var("TARGET").expect("TARGET not set");

	// Expose target triple to runtime code for daemon binary resolution
	println!("cargo:rustc-env=WING_TARGET_TRIPLE={}", target_triple);
	let profile = std::env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());
	let workspace_dir = std::env::var("CARGO_WORKSPACE_DIR")
		.or_else(|_| std::env::var("CARGO_MANIFEST_DIR").map(|d| format!("{}/../../..", d)))
		.expect("Could not find workspace directory");

	let exe_ext = if target_triple.contains("windows") {
		".exe"
	} else {
		""
	};

	let out_dir = std::path::PathBuf::from(std::env::var_os("OUT_DIR").expect("OUT_DIR not set"));
	let profile_dir = out_dir
		.ancestors()
		.nth(3)
		.expect("Missing Cargo profile directory");
	let daemon_source = profile_dir.join(format!("wing-daemon{exe_ext}"));
	let fallback = std::path::PathBuf::from(format!(
		"{workspace_dir}/target/{profile}/wing-daemon{exe_ext}"
	));
	let daemon_source = if daemon_source.exists() {
		daemon_source
	} else {
		fallback
	};
	println!("cargo:rerun-if-changed={}", daemon_source.display());
	if daemon_source.exists() {
		let destination = std::path::PathBuf::from(format!(
			"{workspace_dir}/target/release/wing-daemon-{target_triple}{exe_ext}"
		));
		std::fs::create_dir_all(destination.parent().expect("Missing sidecar directory"))
			.expect("Cannot create sidecar directory");
		// Debug builds must not replace the daemon used by release bundles.
		if profile == "release" || !destination.exists() {
			std::fs::copy(&daemon_source, destination).expect("Cannot stage daemon sidecar");
		}
	} else if profile == "release" {
		panic!(
			"Release daemon is missing at {}; run the daemon build first",
			daemon_source.display()
		);
	}

	tauri_build::build()
}
