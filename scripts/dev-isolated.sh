#!/usr/bin/env bash
# Start the Linux desktop app against disposable WingDrive state.
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
state_root="${WINGDRIVE_DEV_ROOT:-${XDG_STATE_HOME:-$HOME/.local/state}/wingdrive-dev}"
instance="${WINGDRIVE_DEV_INSTANCE:-desktop-dev}"

if [[ ! "$instance" =~ ^[[:alnum:]_-]{1,64}$ ]]; then
	echo "WINGDRIVE_DEV_INSTANCE must contain only letters, numbers, hyphens, and underscores." >&2
	exit 2
fi

export WINGDRIVE_DATA_DIR="$state_root/data"
export WINGDRIVE_INSTANCE="$instance"
export RUST_LOG="${RUST_LOG:-sd_core=info,sd_tauri=info}"

mkdir -p "$WINGDRIVE_DATA_DIR"
printf 'WingDrive development state: %s\nDaemon instance: %s\n' \
	"$WINGDRIVE_DATA_DIR/instances/$WINGDRIVE_INSTANCE" "$WINGDRIVE_INSTANCE"
printf 'Production data directories are not used by this command.\n'

cd "$repo_root/apps/tauri"
exec bun run tauri:dev "$@"
