#!/usr/bin/env bun

/**
 * Development script that:
 * 1. Builds the daemon (dev profile)
 * 2. Starts the daemon
 * 3. Waits for it to be ready
 * 4. Starts Vite dev server
 * 5. Cleans up daemon on exit
 */
import {execSync, spawn} from 'child_process';
import {existsSync} from 'fs';
import {platform} from 'os';
import {dirname, join, resolve} from 'path';
import {fileURLToPath} from 'url';

// Get script directory
const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

// Detect Platform
const IS_WIN = platform() === 'win32';

// Paths relative to this script (apps/tauri/scripts/)
// Script is at: PROJECT_ROOT/apps/tauri/scripts/
// So PROJECT_ROOT is: ../../../
const PROJECT_ROOT = resolve(__dirname, '../../../');

// Resolve target directory from Cargo config (supports custom target-dir)
function getCargoTargetDir(): string {
	try {
		const output = execSync('cargo metadata --format-version 1 --no-deps', {
			cwd: PROJECT_ROOT,
			encoding: 'utf8',
			stdio: ['pipe', 'pipe', 'pipe']
		});
		const metadata = JSON.parse(output);
		return metadata.target_directory;
	} catch {
		return join(PROJECT_ROOT, 'target');
	}
}

const BIN_NAME = IS_WIN ? 'sd-daemon.exe' : 'sd-daemon';
const DAEMON_BIN = join(getCargoTargetDir(), 'debug', BIN_NAME);

const DATA_DIR = process.env.WINGDRIVE_DATA_DIR;
const INSTANCE = process.env.WINGDRIVE_INSTANCE;

if (!DATA_DIR || !INSTANCE) {
	throw new Error(
		'WINGDRIVE_DATA_DIR and WINGDRIVE_INSTANCE are required. Start development with ./scripts/dev-isolated.sh.'
	);
}

if (!/^[A-Za-z0-9_-]{1,64}$/.test(INSTANCE)) {
	throw new Error(
		'WINGDRIVE_INSTANCE must contain only letters, numbers, hyphens, and underscores.'
	);
}

const DAEMON_PORT =
	6970 +
	([...Buffer.from(INSTANCE)].reduce((sum, byte) => sum + byte, 0) % 1000);
const DAEMON_ADDR = `127.0.0.1:${DAEMON_PORT}`;

let daemonProcess: any = null;
let viteProcess: any = null;
let startedDaemon = false;

// Cleanup function
function cleanup() {
	console.log('\nCleaning up...');

	if (viteProcess) {
		console.log('Stopping Vite...');
		viteProcess.kill();
	}

	if (daemonProcess && startedDaemon) {
		console.log('Stopping daemon (started by us)...');
		daemonProcess.kill();
	} else if (!startedDaemon) {
		console.log('Leaving existing daemon running...');
	}

	process.exit(0);
}

// Handle signals
process.on('SIGINT', cleanup);
process.on('SIGTERM', cleanup);

async function main() {
	// Check if daemon is already running by trying to connect to TCP port
	let daemonAlreadyRunning = false;
	console.log(`Checking if daemon is running on ${DAEMON_ADDR}...`);
	try {
		const {connect} = await import('net');
		await new Promise<void>((resolve, reject) => {
			const client = connect(DAEMON_PORT, '127.0.0.1');
			client.on('connect', () => {
				daemonAlreadyRunning = true;
				client.end();
				resolve();
			});
			client.on('error', () => {
				reject();
			});
			setTimeout(() => reject(), 1000);
		});
	} catch (e) {
		// Connection failed, daemon not running
		daemonAlreadyRunning = false;
	}

	if (daemonAlreadyRunning) {
		console.log(
			'Daemon already running, skipping build and using existing instance'
		);
		startedDaemon = false;
	} else {
		console.log('Building daemon (dev profile)...');
		console.log('Project root:', PROJECT_ROOT);
		console.log('Daemon binary:', DAEMON_BIN);

		// Build daemon
		// On Windows, the binary target name is still just "sd-daemon" (Cargo handles the .exe)
		const build = spawn('cargo', ['build', '--bin', 'sd-daemon'], {
			cwd: PROJECT_ROOT,
			stdio: 'inherit',
			shell: IS_WIN // shell: true is often needed on Windows for spawn to work correctly
		});

		await new Promise<void>((resolve, reject) => {
			build.on('exit', (code) => {
				if (code === 0) {
					resolve();
				} else {
					reject(new Error(`Daemon build failed with code ${code}`));
				}
			});
		});

		console.log('Daemon built successfully');
		// Start daemon
		console.log(
			`Starting isolated daemon instance ${INSTANCE} in ${DATA_DIR}...`
		);
		startedDaemon = true;

		// Verify binary exists
		if (!existsSync(DAEMON_BIN)) {
			throw new Error(`Daemon binary not found at: ${DAEMON_BIN}`);
		}

		const depsLibPath = join(PROJECT_ROOT, 'apps/.deps/lib');
		const depsBinPath = join(PROJECT_ROOT, 'apps/.deps/bin');

		daemonProcess = spawn(
			DAEMON_BIN,
			['--data-dir', DATA_DIR, '--instance', INSTANCE],
			{
				cwd: PROJECT_ROOT,
				stdio: ['ignore', 'pipe', 'pipe'],
				env: {
					...process.env,
					// macOS library path
					DYLD_LIBRARY_PATH: depsLibPath,
					// Windows: Add DLLs directory to PATH
					PATH: IS_WIN
						? `${depsBinPath};${process.env.PATH || ''}`
						: process.env.PATH
				}
			}
		);

		// Log daemon output
		daemonProcess.stdout.on('data', (data: Buffer) => {
			const lines = data.toString().trim().split('\n');
			for (const line of lines) {
				console.log(`[daemon] ${line}`);
			}
		});

		daemonProcess.stderr.on('data', (data: Buffer) => {
			const lines = data.toString().trim().split('\n');
			for (const line of lines) {
				console.log(`[daemon] ${line}`);
			}
		});

		// Wait for daemon to be ready
		console.log('Waiting for daemon to be ready...');
		for (let i = 0; i < 30; i++) {
			try {
				const {connect} = await import('net');
				await new Promise<void>((resolve, reject) => {
					const client = connect(DAEMON_PORT, '127.0.0.1');
					client.on('connect', () => {
						client.end();
						resolve();
					});
					client.on('error', reject);
					setTimeout(() => reject(), 500);
				});
				console.log(`Daemon ready at ${DAEMON_ADDR}`);
				break;
			} catch (e) {
				if (i === 29) {
					throw new Error(
						'Daemon failed to start (connection not available)'
					);
				}
				await new Promise((resolve) => setTimeout(resolve, 1000));
			}
		}
	}

	// Start Vite
	console.log('Starting Vite dev server...');

	// Use 'bun' explicitly, with shell true for Windows compatibility
	viteProcess = spawn('bun', ['run', 'dev'], {
		stdio: 'inherit',
		shell: IS_WIN
	});

	// Keep running
	await new Promise(() => {});
}

main().catch((error) => {
	console.error('Error:', error);
	cleanup();
	process.exit(1);
});
