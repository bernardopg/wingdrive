# WingDrive development commands

setup:
	bun install
	cargo xtask setup

dev-daemon *ARGS:
	cargo run --features ffmpeg,heif --bin wing-daemon {{ARGS}}

dev-desktop *ARGS:
	./scripts/dev-isolated.sh {{ARGS}}

dev-mobile:
	cd apps/mobile && bun run start

dev-mobile-ios:
	cd apps/mobile && bun run ios

dev-mobile-android:
	cd apps/mobile && bun run android

build-mobile:
	cargo xtask build-mobile

# Boot the Android emulator used for mobile development (docs/core/mobile-development.mdx).
mobile-emulator avd="wingdrive-api36":
	"$ANDROID_HOME/emulator/emulator" -avd {{avd}} -no-snapshot-save -no-boot-anim

# Build and install the Android debug dev client. Only needed after Rust, Kotlin or native
# dependency changes; JS changes reload through `just dev-mobile`. abi: x86_64 (emulator) or arm64-v8a.
mobile-install-android abi="x86_64":
	cd apps/mobile/android && ANDROID_NDK_HOME="$ANDROID_HOME/ndk/27.1.12297006" WING_ANDROID_ABIS={{abi}} ./gradlew installDebug -PreactNativeArchitectures={{abi}}

# Same features as dev-daemon and Tauri dev, so all three reuse one wing-core build.
dev-server *ARGS:
	cargo run --features ffmpeg,heif --bin wing-server {{ARGS}}

test:
	cargo test --workspace

build:
	cargo build --features ffmpeg,heif

build-release:
	cargo build --release

check:
	cargo fmt --check
	cargo clippy --workspace
	./scripts/check-wingdrive-independence.sh

fmt:
	cargo fmt

# Drops incremental caches and build artifacts untouched for 7 days; target/ grows past 250 GB otherwise.
clean-stale:
	rm -rf target/debug/incremental target/*/debug/incremental
	cargo sweep --time 7 || echo "cargo-sweep missing: cargo install cargo-sweep"

# Same gates as CI, run against the local warm target/ before every push.
# Rust steps are skipped when no Rust-relevant file changed against origin/main.
ci-local *ARGS:
	./scripts/ci-local.sh {{ARGS}}

# Type checks the desktop frontend and the interface package it imports.
typecheck:
	bun run typecheck

cli *ARGS:
	cargo run --features ffmpeg,heif --bin wing {{ARGS}}
