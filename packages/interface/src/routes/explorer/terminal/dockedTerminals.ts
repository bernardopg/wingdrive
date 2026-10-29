import type {TerminalSession} from '../../../contexts/PlatformContext';

/**
 * A tab's docked terminal. Every tab renders its own router, so switching tabs
 * unmounts the panel; the terminal lives here instead and its element moves
 * into each new panel, which keeps the shell, its scrollback and any running
 * program across tab switches.
 */
export interface DockedTerminal {
	/** Element xterm.js draws into, moved between panel mounts. */
	host: HTMLDivElement;
	/** Resolves to null when the shell could not start. */
	session: Promise<TerminalSession | null>;
	/** Folder the shell was last sent to or reported, so neither side echoes the other. */
	folder: string | null;
	/** Moves the Explorer when the shell changes folder; set while the panel is on screen. */
	onFolderChange: ((path: string) => void) | null;
	/** Refits the terminal to its panel and reports the new size to the shell. */
	fit(): void;
	focus(): void;
	dispose(): void;
}

const terminals = new Map<string, DockedTerminal>();

export function dockedTerminal(tabId: string): DockedTerminal | undefined {
	return terminals.get(tabId);
}

export function keepDockedTerminal(tabId: string, terminal: DockedTerminal): void {
	terminals.set(tabId, terminal);
}

/** Hangs up the tab's shell and drops its terminal. */
export function closeDockedTerminal(tabId: string): void {
	const terminal = terminals.get(tabId);
	if (!terminal) return;
	terminals.delete(tabId);
	terminal.dispose();
	void terminal.session.then((session) => session?.close()).catch(() => {});
}

/** Closes the terminals of tabs that no longer exist. */
export function closeDockedTerminalsExcept(tabIds: string[]): void {
	const open = new Set(tabIds);
	for (const tabId of [...terminals.keys()]) {
		if (!open.has(tabId)) closeDockedTerminal(tabId);
	}
}

const focusRequests = new Set<string>();

/** Asks the tab's next panel mount to focus the terminal, as when you open it with F4. */
export function requestTerminalFocus(tabId: string): void {
	focusRequests.add(tabId);
}

/** Whether the tab's terminal should take focus now; answers true once per request. */
export function takeTerminalFocus(tabId: string): boolean {
	return focusRequests.delete(tabId);
}

/**
 * The tab's shell moved to `path` on its own. The Explorer follows only while
 * the panel is on screen; a hidden terminal keeps its folder, and the Explorer
 * sends it back to the folder on screen when the panel shows again.
 */
export function shellMoved(tabId: string, path: string): void {
	const terminal = terminals.get(tabId);
	if (!terminal?.onFolderChange || terminal.folder === path) return;
	terminal.folder = path;
	terminal.onFolderChange(path);
}
