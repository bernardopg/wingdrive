import {GlobalRegistrator} from '@happy-dom/global-registrator';
import {afterEach, describe, expect, it} from 'bun:test';

// Registered once per test process and never unregistered: unregistering closes
// the window whose timers React's scheduler keeps using in the next test file.
if (!GlobalRegistrator.isRegistered) GlobalRegistrator.register();

const {getWebListener, resetWebListener} = await import('./listener');
const {getCurrentPlatform} = await import('./platform');

// happy-dom reports the host's user agent: Cmd combos match Meta on macOS and Ctrl elsewhere.
const mac = getCurrentPlatform() === 'macos';
const cmd = (init: KeyboardEventInit): KeyboardEventInit =>
	mac ? {...init, metaKey: true} : {...init, ctrlKey: true};

function press(init: KeyboardEventInit) {
	const event = new KeyboardEvent('keydown', {bubbles: true, cancelable: true, ...init});
	window.dispatchEvent(event);
	return event;
}

afterEach(() => {
	resetWebListener();
	(document.activeElement as HTMLElement | null)?.blur();
});

describe('web keybind listener', () => {
	it('fires the first matching global combo and prevents the default when asked', () => {
		const listener = getWebListener();
		expect(getWebListener()).toBe(listener);
		const fired: string[] = [];
		listener.register('copy', {modifiers: ['Cmd'], key: 'c'} as never, () => void fired.push('copy'), 'global', true);
		listener.register('del', {modifiers: [], key: 'Delete'} as never, () => void fired.push('del'), 'global');

		const copy = press(cmd({key: 'c', code: 'KeyC'}));
		expect(fired).toEqual(['copy']);
		expect(copy.defaultPrevented).toBe(true);

		const del = press({key: 'Delete', code: 'Delete'});
		expect(fired).toEqual(['copy', 'del']);
		expect(del.defaultPrevented).toBe(false);

		// Wrong or extra modifiers do not match.
		press({key: 'c', code: 'KeyC'});
		press({key: 'c', code: 'KeyC', ctrlKey: true, metaKey: true});
		press(cmd({key: 'c', code: 'KeyC', altKey: true}));
		press(cmd({key: 'c', code: 'KeyC', shiftKey: true}));
		expect(fired).toEqual(['copy', 'del']);

		listener.unregister('copy');
		press(cmd({key: 'c', code: 'KeyC'}));
		expect(fired).toEqual(['copy', 'del']);
	});

	it('matches shifted punctuation and space by physical key', () => {
		const listener = getWebListener();
		const fired: string[] = [];
		listener.register('next', {modifiers: ['Ctrl', 'Shift'], key: ']'} as never, () => void fired.push(']'), 'global');
		listener.register('one', {modifiers: ['Alt'], key: '1'} as never, () => void fired.push('1'), 'global');
		listener.register('space', {modifiers: [], key: 'Space'} as never, () => void fired.push('space'), 'global');
		listener.register('up', {modifiers: [], key: 'ArrowUp'} as never, () => void fired.push('up'), 'global');

		press({key: '}', code: 'BracketRight', ctrlKey: true, shiftKey: true});
		press({key: '¡', code: 'Digit1', altKey: true});
		press({key: ' ', code: 'Space'});
		press({key: 'ArrowUp', code: 'ArrowUp'});
		press({key: 'x', code: 'Numpad1'});
		expect(fired).toEqual([']', '1', 'space', 'up']);
	});

	it('runs scoped combos only while their scope is active', () => {
		const listener = getWebListener();
		const fired: string[] = [];
		listener.register('esc', {modifiers: [], key: 'Escape'} as never, () => void fired.push('esc'), 'modal');
		press({key: 'Escape', code: 'Escape'});
		expect(fired).toEqual([]);
		listener.pushScope('modal');
		expect(listener.hasScope('modal')).toBe(true);
		press({key: 'Escape', code: 'Escape'});
		expect(fired).toEqual(['esc']);
		listener.popScope('modal');
		expect(listener.hasScope('modal')).toBe(false);
		press({key: 'Escape', code: 'Escape'});
		expect(fired).toEqual(['esc']);
	});

	it('ignores combos while typing unless told otherwise', () => {
		const listener = getWebListener();
		const fired: string[] = [];
		listener.register('a', {modifiers: [], key: 'a'} as never, () => void fired.push('a'), 'global');
		listener.register('save', {modifiers: ['Cmd'], key: 's'} as never, () => void fired.push('save'), 'global', true, false);
		const input = document.createElement('input');
		document.body.append(input);
		input.focus();
		press({key: 'a', code: 'KeyA'});
		press(cmd({key: 's', code: 'KeyS'}));
		expect(fired).toEqual(['save']);
		input.remove();
	});

	it('reset stops listening and forgets every combo and scope', () => {
		const listener = getWebListener();
		const fired: string[] = [];
		listener.register('a', {modifiers: [], key: 'a'} as never, () => void fired.push('a'), 'global');
		listener.pushScope('modal');
		resetWebListener();
		resetWebListener();
		press({key: 'a', code: 'KeyA'});
		expect(fired).toEqual([]);
		const fresh = getWebListener();
		expect(fresh).not.toBe(listener);
		expect(fresh.hasScope('modal')).toBe(false);
	});
});
