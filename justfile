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
ci-local:
	./scripts/ci-local.sh

cli *ARGS:
	cargo run --features ffmpeg,heif --bin wing {{ARGS}}
