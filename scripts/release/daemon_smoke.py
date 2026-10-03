#!/usr/bin/env python3
"""Verify fresh production daemon startup and preserve its failure output."""
import json
import os
import socket
import subprocess
import sys
import tempfile
import time
from pathlib import Path

binary = Path(sys.argv[1]).resolve()
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
