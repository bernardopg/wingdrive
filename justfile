# WingDrive development commands

setup:
	bun install
	cargo xtask setup

dev-daemon *ARGS:
	cargo run --features ffmpeg,heif --bin sd-daemon {{ARGS}}

dev-desktop:
	cd apps/tauri && bun run tauri:dev

dev-mobile:
	cd apps/mobile && bun run start

dev-mobile-ios:
	cd apps/mobile && bun run ios

dev-mobile-android:
	cd apps/mobile && bun run android

build-mobile:
	cargo xtask build-mobile

dev-server *ARGS:
	cargo run --bin sd-server {{ARGS}}

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

cli *ARGS:
	cargo run --bin sd-cli {{ARGS}}
