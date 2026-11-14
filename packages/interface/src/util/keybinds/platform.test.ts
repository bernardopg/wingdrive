import {GlobalRegistrator} from '@happy-dom/global-registrator';
import {describe, expect, it} from 'bun:test';

// Registered once per test process and never unregistered: unregistering closes
// the window whose timers React's scheduler keeps using in the next test file.
if (!GlobalRegistrator.isRegistered) GlobalRegistrator.register();

const {
	getComboForPlatform,
	getCurrentPlatform,
	isInputFocused,
	normalizeModifiers,
	revealLabel,
	toDisplayString,
	toTauriAccelerator
} = await import('./platform');

const copy = {modifiers: ['Cmd'], key: 'c'} as never;

describe('keybind platform helpers', () => {
	it('detects the platform once from the user agent', () => {
		// happy-dom reports an X11 Linux user agent.
		expect(getCurrentPlatform()).toBe('linux');
		expect(getCurrentPlatform()).toBe('linux');
		expect(revealLabel()).toBeString();
	});

	it('picks the combo for a platform or its default', () => {
		const perPlatform = {
			default: {modifiers: ['Ctrl'], key: 'k'},
			macos: {modifiers: ['Cmd'], key: 'k'}
		} as never;
		expect(getComboForPlatform(perPlatform, 'macos')).toEqual({modifiers: ['Cmd'], key: 'k'});
		expect(getComboForPlatform(perPlatform, 'linux')).toEqual({modifiers: ['Ctrl'], key: 'k'});
		expect(getComboForPlatform(copy, 'windows')).toBe(copy);
		expect(getComboForPlatform(copy)).toBe(copy);
	});

	it('maps Cmd to Ctrl everywhere but macOS', () => {
		expect(normalizeModifiers(['Cmd', 'Shift'], 'macos')).toEqual(['Cmd', 'Shift']);
		expect(normalizeModifiers(['Cmd', 'Alt'], 'linux')).toEqual(['Ctrl', 'Alt']);
	});

	it('builds Tauri accelerators', () => {
		expect(toTauriAccelerator(copy, 'macos')).toBe('Command+c');
		expect(toTauriAccelerator(copy, 'windows')).toBe('Control+c');
		expect(
			toTauriAccelerator({modifiers: ['Ctrl', 'Alt', 'Shift'], key: 'ArrowUp'} as never, 'linux')
		).toBe('Control+Alt+Shift+Up');
		for (const [key, expected] of [
			['ArrowDown', 'Down'],
			['ArrowLeft', 'Left'],
			['ArrowRight', 'Right'],
			['Space', 'Space']
		]) {
			expect(toTauriAccelerator({modifiers: [], key} as never, 'linux')).toBe(expected!);
		}
	});

	it('formats shortcuts with symbols on macOS and plus signs elsewhere', () => {
		expect(
			toDisplayString({modifiers: ['Cmd', 'Ctrl', 'Alt', 'Shift'], key: 'a'} as never, 'macos')
		).toBe('⌘⌃⌥⇧A');
		expect(
			toDisplayString({modifiers: ['Cmd', 'Ctrl', 'Alt', 'Shift'], key: 'a'} as never, 'linux')
		).toBe('Ctrl+Ctrl+Alt+Shift+A');
		const keys: Array<[string, string]> = [
			['ArrowUp', '↑'],
			['ArrowDown', '↓'],
			['ArrowLeft', '←'],
			['ArrowRight', '→'],
			['Backspace', '⌫'],
			['Delete', '⌦'],
			['Enter', '⏎'],
			['Escape', 'Esc'],
			['Tab', '⇥'],
			['Space', 'Space'],
			['F2', 'F2']
		];
		for (const [key, shown] of keys) {
			expect(toDisplayString({modifiers: [], key} as never, 'windows')).toBe(shown);
		}
	});

	it('knows when a text field has the focus', () => {
		const input = document.createElement('input');
		const editable = document.createElement('div');
		editable.setAttribute('contenteditable', 'true');
		editable.tabIndex = 0;
		const button = document.createElement('button');
		document.body.append(input, editable, button);

		input.focus();
		expect(isInputFocused()).toBe(true);
		editable.focus();
		expect(isInputFocused()).toBe(true);
		button.focus();
		expect(isInputFocused()).toBe(false);
		for (const tag of ['textarea', 'select']) {
			const element = document.createElement(tag);
			document.body.append(element);
			element.focus();
			expect(isInputFocused()).toBe(true);
		}
	});
});
