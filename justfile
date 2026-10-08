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

dev-server *ARGS:
	cargo run --bin wing-server {{ARGS}}

test:
	cargo test --workspace

build:
	cargo build

build-release:
	cargo build --release

check:
	cargo fmt --check
	cargo clippy --workspace
	./scripts/check-wingdrive-independence.sh

fmt:
	cargo fmt

# Same gates as CI, run against the local warm target/ before every push.
# Rust steps are skipped when no Rust-relevant file changed against origin/main.
ci-local:
	./scripts/ci-local.sh

cli *ARGS:
	cargo run --bin wing {{ARGS}}
