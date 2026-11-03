#!/usr/bin/env python3
"""Start an extracted desktop bundle with a fresh isolated daemon."""
import json
import os
import shutil
import signal
import socket
import subprocess
import sys
import tempfile
import time
from pathlib import Path

image = Path(sys.argv[1]).resolve()

# The resident-open check goes over the session bus, and GTK waits about 25 s
# for one that does not exist, so give a headless runner its own.
if not os.environ.get("DBUS_SESSION_BUS_ADDRESS") and shutil.which("dbus-run-session"):
    os.execvp("dbus-run-session", ["dbus-run-session", "--", sys.executable, *sys.argv])

# WebKit memory still differs by tens of MiB between fresh launches of the same
# build, so the budget holds when most of MEMORY_SESSIONS sessions stay under it.
# A real regression raises them all.
MEMORY_SESSIONS = 3
# Garbage made while idle shows as a sawtooth between collections, so each session
# reports its peak over this window instead of one snapshot taken at a random phase.
IDLE_WINDOW_SECONDS = 30
IDLE_SAMPLE_SECONDS = 5


def process_memory(pid, verbose=True):
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
        if verbose:
            print(f"  {name} ({child}): {pss / 1048576:.1f} MiB")
    return memory


def idle_peak(pid):
    """Return the highest total over the idle window, and print how it moved."""
    samples = []
    for _ in range(IDLE_WINDOW_SECONDS // IDLE_SAMPLE_SECONDS + 1):
        samples.append(process_memory(pid, verbose=False))
        time.sleep(IDLE_SAMPLE_SECONDS)
    shown = " ".join(f"{m / 1048576:.0f}" for m in samples)
    print(f"  idle every {IDLE_SAMPLE_SECONDS} s: {shown} MiB")
    return max(samples)


def session(app, root, number, full):
    """Launch one isolated desktop, open 15 tabs and return its memory.

    The first session also checks cold start, the visible window and the
    hand-over of a second launch; later ones only repeat the memory scenario.
    """
    root.mkdir()
    instance = f"release-{os.getpid()}-{number}"
    port = 6970 + sum(instance.encode()) % 1000
    with socket.socket() as probe:
        # Only a listening socket makes the port busy, not a closed one in TIME_WAIT.
        probe.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
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
        if full:
            startup_budget = float(os.environ.get('WINGDRIVE_STARTUP_BUDGET_SECONDS', '20'))
            print(f"Cold start to daemon ready: {startup:.1f} s (budget: {startup_budget:.0f} s)")
            assert startup < startup_budget, f"Cold start exceeded {startup_budget:g} s"
        time.sleep(3)
        windows = subprocess.check_output(["xdotool", "search", "--onlyvisible", "--name", "WingDrive"], text=True)
        assert windows.strip(), "Main window not visible"
        assert process.poll() is None, "Desktop exited after connection"
        if full:
            # A second launch with a folder must reach this instance and exit
            # without starting GTK.
            target = root / "open-target"
            target.mkdir()
            resident_budget_ms = float(os.environ.get('WINGDRIVE_RESIDENT_OPEN_BUDGET_MS', '300'))
            started = time.monotonic()
            second = subprocess.run([launcher, str(target)], cwd=app, env=env,
                                    capture_output=True, text=True, timeout=60)
            resident_ms = (time.monotonic() - started) * 1000
            print(f"Resident open handed over: {resident_ms:.0f} ms (budget: {resident_budget_ms:.0f} ms)")
            assert second.returncode == 0, second.stdout + second.stderr
            assert "Handed launch to the running WingDrive" in second.stdout + second.stderr, \
                "Second launch started its own app instead of handing over"
            deadline = time.monotonic() + 10
            while "Delivering open requests" not in (root / "desktop.log").read_text(errors="replace"):
                assert time.monotonic() < deadline, "Running instance did not receive the folder"
                time.sleep(.1)
            assert resident_ms < resident_budget_ms, f"Resident open exceeded {resident_budget_ms:g} ms"
        window = windows.splitlines()[0]
        time.sleep(10)
        subprocess.run(['xdotool', 'windowfocus', '--sync', window], check=True)
        subprocess.run(['xdotool', 'key', '--clearmodifiers', '--repeat', '14',
                        '--repeat-delay', '200', 'ctrl+t'], check=True)
        time.sleep(20)
        print(f"Session {number}:")
        process_memory(process.pid)
        memory = idle_peak(process.pid)
        if output := os.environ.get('WINGDRIVE_TEST_OUTPUT'):
            evidence = Path(output)
            evidence.mkdir(parents=True, exist_ok=True)
            subprocess.run(['import', '-window', window, str(evidence / f'native-15-tabs-{number}.png')],
                           check=True)
        return memory
    except BaseException:
        log.seek(0)
        print(log.read())
        raise
    finally:
        os.killpg(process.pid, signal.SIGTERM)
        try:
            process.wait(timeout=10)
        except subprocess.TimeoutExpired:
            os.killpg(process.pid, signal.SIGKILL)
            process.wait()
        log.close()


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
    majority = MEMORY_SESSIONS // 2 + 1
    samples = []
    while (sum(m < memory_budget for m in samples) < majority
           and sum(m >= memory_budget for m in samples) < majority):
        number = len(samples) + 1
        samples.append(session(app, root / f"session-{number}", number, full=number == 1))
    shown = ", ".join(f"{m / 1048576:.1f}" for m in samples)
    print(f"Native desktop with 15 tabs and children, idle peak PSS per session: {shown} MiB "
          f"(budget: {memory_budget_mib:.1f} MiB in {majority} of {MEMORY_SESSIONS} sessions)")
    if output := os.environ.get('WINGDRIVE_TEST_OUTPUT'):
        (Path(output) / 'native-memory.json').write_text(json.dumps(
            {'tabs': 15, 'budgetMiB': memory_budget_mib, 'sessionsPssMiB': [m / 1048576 for m in samples]}))
    assert sum(m < memory_budget for m in samples) >= majority, (
        f'Native desktop and children exceeded {memory_budget_mib:g} MiB in most sessions'
    )
    print("PASS: isolated packaged daemon replies and WingDrive window is visible")
