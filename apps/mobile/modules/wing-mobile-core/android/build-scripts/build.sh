#!/bin/bash
set -e

cd "$(dirname "$0")/../../core"

echo "Building WingDrive Mobile Core for Android..."

OUTPUT_DIR="../android/src/main/jniLibs"
mkdir -p "$OUTPUT_DIR"

# WING_ANDROID_ABIS picks the targets: arm64-v8a for devices, x86_64 for the emulator.
# Example: WING_ANDROID_ABIS="arm64-v8a x86_64" ./build.sh
ABIS="${WING_ANDROID_ABIS:-arm64-v8a}"

TARGET_ARGS=()
for abi in $ABIS; do
	TARGET_ARGS+=(-t "$abi")
done

# sccache makes rebuilds after a clean or a branch switch much faster when it is installed.
if [ -z "$RUSTC_WRAPPER" ] && command -v sccache >/dev/null 2>&1; then
	export RUSTC_WRAPPER=sccache
fi

echo "Building for: $ABIS"
cargo ndk --platform 24 "${TARGET_ARGS[@]}" -o "$OUTPUT_DIR" build --release

# cargo-ndk also copies hashed dependency dylibs (libiroh-<hash>.so, ...); only our library ships.
find "$OUTPUT_DIR" -name 'lib*-*.so' -delete

echo "Android Rust build complete!"
