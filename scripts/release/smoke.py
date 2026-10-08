#!/usr/bin/env python3
"""Start an extracted desktop bundle with a fresh isolated daemon."""
import json
import os
import signal
import socket
import subprocess
import sys
import tempfile
import time
from pathlib import Path

image = Path(sys.argv[1]).resolve()


def process_memory(pid):
    """Sum proportional memory for the desktop and its children, including the daemon."""
    descendants = {pid}
    parents = {}
    for entry in Path('/proc').iterdir():
        if entry.name.isdigit():
            try:
                status = (entry / 'status').read_text()
                parents[int(entry.name)] = int(next(line for line in status.splitlines()
                                                    if line.startswith('PPid:')).split()[1])
            except (OSError, StopIteration):
                pass
    while True:
        children = {child for child, parent in parents.items() if parent in descendants}
        if children <= descendants:
            break
        descendants |= children
    memory = 0
    for child in sorted(descendants):
        try:
            status = Path(f'/proc/{child}/smaps_rollup').read_text()
            pss = int(next(line for line in status.splitlines()
                           if line.startswith('Pss:')).split()[1]) * 1024
            name = Path(f'/proc/{child}/comm').read_text().strip()
        except (OSError, StopIteration):
            continue
        memory += pss
        print(f"  {name} ({child}): {pss / 1048576:.1f} MiB")
    return memory


with tempfile.TemporaryDirectory(prefix="wingdrive-bundle-") as directory:
    root = Path(directory)
    memory_budget_mib = float(os.environ.get('WINGDRIVE_NATIVE_MEMORY_BUDGET_MIB', '550'))
    memory_budget = int(memory_budget_mib * 1048576)
    if image.is_dir():
        app = image
    else:
        subprocess.run([str(image), "--appimage-extract"], cwd=root, check=True, stdout=subprocess.DEVNULL)
        app = root / "squashfs-root"
    assert (app / "usr/bin/wing-daemon").is_file(), "Packaged daemon missing"
    sections = subprocess.check_output(["readelf", "--sections", str(app / "usr/bin/wing-daemon")], text=True)
    assert ".symtab" not in sections, "Packaged daemon contains debug symbols"
    instance = f"release-{os.getpid()}"
    port = 6970 + sum(instance.encode()) % 1000
    with socket.socket() as probe:
        probe.bind(("127.0.0.1", port))
    env = dict(os.environ, WINGDRIVE_DATA_DIR=str(root / "data"),
               WINGDRIVE_INSTANCE=instance, WEBKIT_DISABLE_COMPOSITING_MODE="1", RUST_LOG="info",
               XDG_DATA_HOME=str(root / "xdg/data"), XDG_CACHE_HOME=str(root / "xdg/cache"),
               XDG_CONFIG_HOME=str(root / "xdg/config"))
    env.pop("GDK_BACKEND", None)
    log = open(root / "desktop.log", "w+")
    launcher = os.environ.get("WINGDRIVE_TEST_LAUNCHER", str(app / "AppRun"))
    process = subprocess.Popen([launcher], cwd=app, env=env,
                               stdout=log, stderr=log, start_new_session=True)
    try:
        deadline = time.monotonic() + 90
        while time.monotonic() < deadline:
            assert process.poll() is None, "Desktop exited during startup"
            try:
                with socket.create_connection(("127.0.0.1", port), timeout=1) as connection:
                    connection.sendall(b'{"Ping":null}\n')
                    response = connection.recv(65536)
                    if response:
                        decoded = json.loads(response.split(b"\n")[0])
                        assert "error" not in decoded, decoded
                        break
            except (OSError, json.JSONDecodeError):
                pass
            time.sleep(.25)
        else:
            raise AssertionError("Packaged daemon did not become ready")
        startup = 90 - (deadline - time.monotonic())
        startup_budget = float(os.environ.get('WINGDRIVE_STARTUP_BUDGET_SECONDS', '20'))
        print(f"Cold start to daemon ready: {startup:.1f} s (budget: {startup_budget:.0f} s)")
        assert startup < startup_budget, f"Cold start exceeded {startup_budget:g} s"
        time.sleep(3)
        windows = subprocess.check_output(["xdotool", "search", "--onlyvisible", "--name", "WingDrive"], text=True)
        assert windows.strip(), "Main window not visible"
        assert process.poll() is None, "Desktop exited after connection"
        window = windows.splitlines()[0]
        time.sleep(10)
        subprocess.run(['xdotool', 'windowfocus', '--sync', window], check=True)
        subprocess.run(['xdotool', 'key', '--clearmodifiers', '--repeat', '14',
                        '--repeat-delay', '200', 'ctrl+t'], check=True)
        time.sleep(20)
        memory = process_memory(process.pid)
        print(f"Native desktop with 15 tabs and children PSS: {memory / 1048576:.1f} MiB "
              f"(budget: {memory_budget_mib:.1f} MiB)")
        if output := os.environ.get('WINGDRIVE_TEST_OUTPUT'):
            evidence = Path(output)
            evidence.mkdir(parents=True, exist_ok=True)
            subprocess.run(['import', '-window', window, str(evidence / 'native-15-tabs.png')], check=True)
            (evidence / 'native-memory.json').write_text(json.dumps({'tabs': 15, 'pssMiB': memory / 1048576}))
        assert memory < memory_budget, (
            f'Native desktop and children exceeded {memory_budget_mib:g} MiB'
        )
        print("PASS: isolated packaged daemon replies and WingDrive window is visible")
    finally:
        os.killpg(process.pid, signal.SIGTERM)
        try:
            process.wait(timeout=10)
        except subprocess.TimeoutExpired:
            os.killpg(process.pid, signal.SIGKILL)
            process.wait()
        log.seek(0)
        print(log.read())
