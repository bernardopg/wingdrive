import {FitAddon} from '@xterm/addon-fit';
import {Terminal, type ITheme} from '@xterm/xterm';
import '@xterm/xterm/css/xterm.css';
import {useEffect, useRef, useState, type KeyboardEvent, type PointerEvent} from 'react';

import {usePlatform, type Platform} from '../../../contexts/PlatformContext';
import {i18n, useTranslation} from '../../../i18n';
import {
	closeDockedTerminal,
	dockedTerminal,
	keepDockedTerminal,
	shellMoved,
	takeTerminalFocus,
	type DockedTerminal
} from './dockedTerminals';
import {clampTerminalHeight, osc7Folder} from './terminalFolder';

/** Tailwind's `font-mono` stack. */
const FONT_FAMILY =
	'ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace';

/** Pixels an arrow key moves the panel's edge. */
const KEYBOARD_STEP = 16;

interface TerminalPanelProps {
	tabId: string;
	/** Folder the shell follows; null leaves it where it is. */
	folder: string | null;
	height: number;
	onHeightChange(height: number): void;
	/** The shell changed folder on its own (you typed `cd`); the Explorer should go there. */
	onShellFolder(path: string): void;
	/** The tab's shell ended, so its panel should close. */
	onExit(tabId: string): void;
}

/** xterm.js needs concrete colors, so read the theme's from the panel. */
function themeOf(element: HTMLElement): ITheme {
	const style = getComputedStyle(element);
	const probe = document.createElement('span');
	probe.className = 'bg-accent/30';
	element.appendChild(probe);
	const selectionBackground = getComputedStyle(probe).backgroundColor;
	probe.remove();
	return {
		background: style.backgroundColor,
		foreground: style.color,
		cursor: style.color,
		cursorAccent: style.backgroundColor,
		selectionBackground
	};
}

function createDockedTerminal(
	platform: Platform,
	tabId: string,
	container: HTMLElement,
	folder: string | null,
	onExit: (tabId: string) => void
): DockedTerminal {
	const terminal = new Terminal({
		cursorBlink: true,
		fontFamily: FONT_FAMILY,
		fontSize: 13,
		scrollback: 5000,
		theme: themeOf(container)
	});
	const fitAddon = new FitAddon();
	terminal.loadAddon(fitAddon);
	const host = document.createElement('div');
	host.className = 'h-full w-full';
	// xterm.js measures its font when it opens, which needs the element on the page.
	container.appendChild(host);
	terminal.open(host);
	fitAddon.fit();

	const session = (async () => {
		const cwd = folder ?? (await platform.homeDir?.()) ?? '/';
		return platform.openTerminalSession!({
			cwd,
			cols: terminal.cols,
			rows: terminal.rows,
			onOutput: (data) => terminal.write(data),
			onFolderChange: (path) => shellMoved(tabId, path),
			onExit: () => {
				closeDockedTerminal(tabId);
				onExit(tabId);
			}
		});
	})().catch((error: unknown) => {
		terminal.write(`\r\n${i18n.t('explorer:terminal.couldNotStart', {error: String(error)})}\r\n`);
		return null;
	});
	// Shells that report their folder (fish, or VTE's script) do it with OSC 7;
	// it also covers platforms where the folder cannot be read from the process.
	let hostname: string | null = null;
	void session.then((shell) => {
		hostname = shell?.hostname ?? null;
	});
	terminal.parser.registerOscHandler(7, (data) => {
		const path = osc7Folder(data, hostname);
		if (path) shellMoved(tabId, path);
		return true;
	});
	terminal.onData((data) => void session.then((shell) => shell?.write(data)));
	terminal.onResize(({cols, rows}) => void session.then((shell) => shell?.resize(cols, rows)));

	return {
		host,
		session,
		folder,
		onFolderChange: null,
		fit: () => {
			if (host.isConnected) fitAddon.fit();
		},
		focus: () => terminal.focus(),
		dispose: () => terminal.dispose()
	};
}

/** Your shell, docked under the Explorer view and following its folder (F4). */
export default function TerminalPanel({
	tabId,
	folder,
	height,
	onHeightChange,
	onShellFolder,
	onExit
}: TerminalPanelProps) {
	const {t} = useTranslation('explorer');
	const platform = usePlatform();
	const panelRef = useRef<HTMLDivElement>(null);
	const containerRef = useRef<HTMLDivElement>(null);
	const [dragHeight, setDragHeight] = useState<number | null>(null);
	const shownHeight = dragHeight ?? height;
	// Read when the terminal is created; later folders arrive through `follow`.
	const folderRef = useRef(folder);
	folderRef.current = folder;
	const onExitRef = useRef(onExit);
	onExitRef.current = onExit;
	const onShellFolderRef = useRef(onShellFolder);
	onShellFolderRef.current = onShellFolder;

	// The tab's terminal outlives this panel: attach it, or create it on first show.
	useEffect(() => {
		const container = containerRef.current;
		if (!container || !platform.openTerminalSession) return;
		let docked = dockedTerminal(tabId);
		if (docked) {
			container.appendChild(docked.host);
		} else {
			docked = createDockedTerminal(platform, tabId, container, folderRef.current, (id) =>
				onExitRef.current(id)
			);
			keepDockedTerminal(tabId, docked);
		}
		const terminal = docked;
		terminal.onFolderChange = (path) => onShellFolderRef.current(path);
		terminal.fit();
		if (takeTerminalFocus(tabId)) terminal.focus();
		const observer = new ResizeObserver(() => terminal.fit());
		observer.observe(container);
		return () => {
			observer.disconnect();
			terminal.onFolderChange = null;
			terminal.host.remove();
		};
	}, [tabId, platform]);

	// Navigating sends the shell along: at once at its prompt, or once a running program ends.
	useEffect(() => {
		const docked = dockedTerminal(tabId);
		if (!docked || !folder || docked.folder === folder) return;
		const previous = docked.folder;
		docked.folder = folder;
		docked.session
			.then(async (shell) => {
				if (!shell || !(await shell.follow(folder))) {
					if (docked.folder === folder) docked.folder = previous;
				}
			})
			.catch(() => {});
	}, [tabId, folder]);

	const available = () => panelRef.current?.parentElement?.clientHeight ?? window.innerHeight;

	const startResize = (event: PointerEvent<HTMLDivElement>) => {
		const handle = event.currentTarget;
		const startY = event.clientY;
		const startHeight = shownHeight;
		const space = available();
		const heightAt = (y: number) => clampTerminalHeight(startHeight + startY - y, space);
		handle.setPointerCapture(event.pointerId);
		const move = (moveEvent: globalThis.PointerEvent) => setDragHeight(heightAt(moveEvent.clientY));
		const end = (endEvent: globalThis.PointerEvent) => {
			handle.removeEventListener('pointermove', move);
			handle.removeEventListener('pointerup', end);
			handle.removeEventListener('pointercancel', end);
			setDragHeight(null);
			onHeightChange(heightAt(endEvent.clientY));
		};
		handle.addEventListener('pointermove', move);
		handle.addEventListener('pointerup', end);
		handle.addEventListener('pointercancel', end);
	};

	const resizeWithKeys = (event: KeyboardEvent<HTMLDivElement>) => {
		const step = event.key === 'ArrowUp' ? KEYBOARD_STEP : event.key === 'ArrowDown' ? -KEYBOARD_STEP : 0;
		if (!step) return;
		event.preventDefault();
		onHeightChange(clampTerminalHeight(shownHeight + step, available()));
	};

	return (
		<div
			ref={panelRef}
			role="region"
			aria-label={t('terminal.label')}
			className="flex shrink-0 flex-col border-t border-app-line"
			style={{height: shownHeight}}
		>
			<div
				role="separator"
				aria-orientation="horizontal"
				aria-label={t('terminal.resize')}
				aria-valuenow={shownHeight}
				tabIndex={0}
				onPointerDown={startResize}
				onKeyDown={resizeWithKeys}
				className="h-1 shrink-0 cursor-row-resize hover:bg-accent/40 focus-visible:bg-accent/40 focus-visible:outline-none"
			/>
			<div
				ref={containerRef}
				className="min-h-0 flex-1 overflow-hidden bg-app px-2 pb-1 text-ink"
			/>
		</div>
	);
}
