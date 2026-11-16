#!/usr/bin/env bash
# Copies the fonts and app screenshots the promo pages load into assets/.
# Needs `bun install` in website/ (for the Fontsource fonts) and in this folder (playwright-core).
set -euo pipefail
cd "$(dirname "$0")"
mkdir -p assets/fonts assets/shots
fs=../../node_modules/@fontsource-variable
cp "$fs/inter/files/inter-latin-wght-normal.woff2" "$fs/space-grotesk/files/space-grotesk-latin-wght-normal.woff2" \
	"$fs/jetbrains-mono/files/jetbrains-mono-latin-wght-normal.woff2" assets/fonts/
cp ../../../docs/public/Wing*.webp assets/shots/
