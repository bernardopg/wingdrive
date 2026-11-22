#!/usr/bin/env python3
"""Verify fresh production daemon startup and that an idle daemon stays quiet.

Expected conditions (unreadable /tmp entries, transient watcher teardown) must not reach
WARN or ERROR, so the journal stays useful in bug reports (CORE-022).
"""
import json
import os
import re
import socket
import subprocess
import sys
import tempfile
import time
from pathlib import Path

# Own targets and the watcher crates that have produced noise in the past. Third-party
# crates at info are ignored so the check stays stable across dependency updates.
OWN_LOG_PREFIXES = (
    "wing_core",
    "wingdrive",
    "wing_",
    "tauri",
    "notify::",
    "wing_task_system",
    "wing_fs_watcher",
)
LEVEL_RE = re.compile(r"\b(WARN|ERROR)\b")


def log_noise(lines):
    noise = []
    for line in lines:
        if not LEVEL_RE.search(line):
            continue
        if any(prefix in line for prefix in OWN_LOG_PREFIXES):
            noise.append(line.rstrip())
    return noise


binary = Path(sys.argv[1]).resolve()
idle_seconds = int(os.environ.get("WINGDRIVE_DAEMON_SMOKE_IDLE_SECONDS", "30"))
with tempfile.TemporaryDirectory(prefix="wingdrive-daemon-") as directory:
    instance = f"release-preflight-{os.getpid()}"
    port = 6970 + sum(instance.encode()) % 1000
    with socket.socket() as probe:
        probe.bind(("127.0.0.1", port))
    with tempfile.TemporaryFile(mode="w+") as log:
        process = subprocess.Popen(
            [str(binary), "--data-dir", directory, "--instance", instance],
            stdout=log, stderr=log, env=dict(os.environ, RUST_LOG="info"),
        )
        try:
            deadline = time.monotonic() + 20
            while time.monotonic() < deadline:
                assert process.poll() is None, f"Daemon exited with {process.returncode}"
                try:
                    with socket.create_connection(("127.0.0.1", port), timeout=1) as connection:
                        connection.sendall(b'{"Ping":null}\n')
                        response = json.loads(connection.recv(65536).split(b"\n")[0])
                        assert "error" not in response, response
                        assert (Path(directory) / "instances" / instance).is_dir()
                        print("PASS: fresh production daemon replies within 20 seconds")
                        break
                except (OSError, json.JSONDecodeError):
                    time.sleep(.1)
            else:
                raise AssertionError("Fresh production daemon did not become reachable within 20 seconds")

            # Let the daemon sit idle so startup races and watcher teardown settle (CORE-022).
            idle_deadline = time.monotonic() + idle_seconds
            while time.monotonic() < idle_deadline:
                assert process.poll() is None, f"Daemon exited during idle window with {process.returncode}"
                time.sleep(1)
            log.flush()
            log.seek(0)
            lines = log.readlines()
            noise = log_noise(lines)
            assert not noise, (
                f"Idle daemon emitted {len(noise)} WARN/ERROR line(s) in {idle_seconds}s:\n"
                + "\n".join(noise[:20])
            )
            print(f"PASS: no WARN/ERROR from own targets during {idle_seconds}s idle")
        finally:
            if process.poll() is None:
                process.terminate()
                try:
                    process.wait(timeout=10)
                except subprocess.TimeoutExpired:
                    process.kill()
                    process.wait()
            log.seek(0)
            print(log.read())
