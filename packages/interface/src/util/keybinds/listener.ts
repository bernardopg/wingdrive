import type { KeyCombo, Platform } from './types';
import { getCurrentPlatform, isInputFocused, normalizeModifiers } from './platform';

export type KeybindHandler = () => void | Promise<void>;

/**
 * Physical keys that produce a different `event.key` once Shift is held
 * (Shift+] reports `}`), which silently broke every Shift combo bound to
 * punctuation. Matching on `event.code` as well keeps those shortcuts working.
 */
const CODE_TO_UNSHIFTED_KEY: Record<string, string> = {
	BracketLeft: '[',
	BracketRight: ']',
	Comma: ',',
	Period: '.',
	Slash: '/',
	Semicolon: ';',
	Quote: "'",
	Backslash: '\\',
	Minus: '-',
	Equal: '=',
	Backquote: '`'
};

function unshiftedKeyFromCode(code: string): string | null {
	if (CODE_TO_UNSHIFTED_KEY[code]) return CODE_TO_UNSHIFTED_KEY[code];
	if (/^Key[A-Z]$/.test(code)) return code.slice(3).toLowerCase();
	if (/^Digit[0-9]$/.test(code)) return code.slice(5);
	return null;
}

interface RegisteredKeybind {
	combo: KeyCombo;
	handler: KeybindHandler;
	scope: string;
	preventDefault: boolean;
	ignoreWhenInputFocused: boolean;
}

// Singleton listener for web platform
class WebKeybindListener {
	private registeredKeybinds = new Map<string, RegisteredKeybind>();
	private activeScopes = new Set<string>(['global']);
	private platform: Platform;
	private boundHandleKeyDown: (e: KeyboardEvent) => void;

	constructor() {
		this.platform = getCurrentPlatform();
		this.boundHandleKeyDown = this.handleKeyDown.bind(this);

		if (typeof window !== 'undefined') {
			window.addEventListener('keydown', this.boundHandleKeyDown);
		}
	}

	register(
		id: string,
		combo: KeyCombo,
		handler: KeybindHandler,
		scope: string,
		preventDefault = false,
		ignoreWhenInputFocused = true
	): void {
		this.registeredKeybinds.set(id, {
			combo,
			handler,
			scope,
			preventDefault,
			ignoreWhenInputFocused
		});
	}

	unregister(id: string): void {
		this.registeredKeybinds.delete(id);
	}

	pushScope(scope: string): void {
		this.activeScopes.add(scope);
	}

	popScope(scope: string): void {
		this.activeScopes.delete(scope);
	}

	hasScope(scope: string): boolean {
		return this.activeScopes.has(scope);
	}

	private handleKeyDown(e: KeyboardEvent): void {
		// Find matching keybind
		for (const [_id, keybind] of this.registeredKeybinds) {
			// Check if scope is active
			if (!this.isScopeActive(keybind.scope)) continue;

			// Check if we should ignore when input is focused
			if (keybind.ignoreWhenInputFocused && isInputFocused()) continue;

			// Check if key combo matches
			if (this.matchesCombo(e, keybind.combo)) {
				if (keybind.preventDefault) {
					e.preventDefault();
					e.stopPropagation();
				}

				// Fire and forget - don't await
				void keybind.handler();
				return;
			}
		}
	}

	private isScopeActive(scope: string): boolean {
		return scope === 'global' || this.activeScopes.has(scope);
	}

	private matchesCombo(e: KeyboardEvent, combo: KeyCombo): boolean {
		const normalizedModifiers = normalizeModifiers(combo.modifiers, this.platform);

		// Count required modifiers
		const requiredCmd = normalizedModifiers.includes('Cmd');
		const requiredCtrl = normalizedModifiers.includes('Ctrl');
		const requiredAlt = normalizedModifiers.includes('Alt');
		const requiredShift = normalizedModifiers.includes('Shift');

		// Check modifier states
		const hasMeta = e.metaKey;
		const hasCtrl = e.ctrlKey;
		const hasAlt = e.altKey;
		const hasShift = e.shiftKey;

		// On macOS, Cmd = metaKey
		// On Windows/Linux, Cmd has been normalized to Ctrl
		if (this.platform === 'macos') {
			// macOS: Cmd maps to metaKey
			if (requiredCmd !== hasMeta) return false;
			if (requiredCtrl !== hasCtrl) return false;
		} else {
			// Windows/Linux: Both Cmd and Ctrl map to ctrlKey
			const requiresCtrlKey = requiredCmd || requiredCtrl;
			if (requiresCtrlKey !== hasCtrl) return false;
			// Meta key should not be pressed unless required
			if (hasMeta) return false;
		}

		if (requiredAlt !== hasAlt) return false;
		if (requiredShift !== hasShift) return false;

		// Check key
		const target = this.normalizeKey(combo.key);
		if (this.normalizeKey(e.key) === target) return true;

		const fromCode = unshiftedKeyFromCode(e.code);
		return fromCode !== null && this.normalizeKey(fromCode) === target;
	}

	private normalizeKey(key: string): string {
		// Normalize key names for comparison
		const normalized = key.toLowerCase();

		// Handle special cases
		if (normalized === ' ') return 'space';
		if (normalized === 'arrowup') return 'arrowup';
		if (normalized === 'arrowdown') return 'arrowdown';
		if (normalized === 'arrowleft') return 'arrowleft';
		if (normalized === 'arrowright') return 'arrowright';

		return normalized;
	}

	destroy(): void {
		if (typeof window !== 'undefined') {
			window.removeEventListener('keydown', this.boundHandleKeyDown);
		}
		this.registeredKeybinds.clear();
		this.activeScopes.clear();
		this.activeScopes.add('global');
	}
}

// Global singleton instance
let listenerInstance: WebKeybindListener | null = null;

export function getWebListener(): WebKeybindListener {
	if (!listenerInstance) {
		listenerInstance = new WebKeybindListener();
	}
	return listenerInstance;
}

// For testing purposes
export function resetWebListener(): void {
	if (listenerInstance) {
		listenerInstance.destroy();
		listenerInstance = null;
	}
}
