#!/usr/bin/env bash
# Runs the administrator-retry test through the real pkexec and polkit, unattended.
#
# A process-specific pkttyagent answers the password prompts for the test binary
# only, so the session's graphical agent never shows a dialog. The password comes
# from WINGDRIVE_POLKIT_PASSWORD and is never written to disk or printed.
#
# Usage: WINGDRIVE_POLKIT_PASSWORD=... scripts/check-admin-retry.sh
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

: "${WINGDRIVE_POLKIT_PASSWORD:?set WINGDRIVE_POLKIT_PASSWORD to your login password}"
command -v pkttyagent expect > /dev/null

# Builds the desktop crate's test binary and prints its path.
binary=$(cargo test --locked -p wingdrive --bin WingDrive --no-run --message-format=json 2> /dev/null |
	jq -r 'select(.profile.test == true and .target.name == "WingDrive") | .executable' | tail -1)
test -x "$binary"

# `exec` keeps this shell's PID and start time, so the agent registered for $$
# answers the prompts that pkexec raises for the test binary.
(
	expect -f - "$$" <<'EXPECT'
set timeout 120
log_user 0
spawn pkttyagent --process [lindex $argv 0]
expect {
	-re {[Pp]assword[^:]*:} { send -- "$env(WINGDRIVE_POLKIT_PASSWORD)\r"; exp_continue }
	-re {AUTHENTICATION FAILED} { puts stderr "polkit refused the password"; exit 1 }
	eof
}
EXPECT
) &
sleep 1
exec "$binary" admin::tests::pkexec_round_trip_on_root_owned_files --ignored --exact --nocapture
