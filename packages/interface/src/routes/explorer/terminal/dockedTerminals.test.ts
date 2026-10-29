import {describe, expect, mock, test} from 'bun:test';

import type {TerminalSession} from '../../../contexts/PlatformContext';
import {
	closeDockedTerminal,
	closeDockedTerminalsExcept,
	dockedTerminal,
	keepDockedTerminal,
	requestTerminalFocus,
	takeTerminalFocus,
	type DockedTerminal
} from './dockedTerminals';

function fakeTerminal() {
	const session = {
		write: mock(() => {}),
		resize: mock(async () => {}),
		follow: mock(async () => true),
		close: mock(async () => {})
	} satisfies TerminalSession;
	const terminal: DockedTerminal = {
		host: {} as HTMLDivElement,
		session: Promise.resolve(session),
		folder: null,
		fit: () => {},
		focus: () => {},
		dispose: mock(() => {})
	};
	return {terminal, session};
}

describe('docked terminals', () => {
	test('closing a tab terminal disposes it and hangs up its shell', async () => {
		const {terminal, session} = fakeTerminal();
		keepDockedTerminal('tab-a', terminal);
		expect(dockedTerminal('tab-a')).toBe(terminal);

		closeDockedTerminal('tab-a');
		expect(dockedTerminal('tab-a')).toBeUndefined();
		expect(terminal.dispose).toHaveBeenCalledTimes(1);
		await terminal.session;
		expect(session.close).toHaveBeenCalledTimes(1);
	});

	test('closed tabs lose their terminals and open tabs keep theirs', async () => {
		const kept = fakeTerminal();
		const gone = fakeTerminal();
		keepDockedTerminal('open', kept.terminal);
		keepDockedTerminal('closed', gone.terminal);

		closeDockedTerminalsExcept(['open', 'other']);
		await gone.terminal.session;
		expect(dockedTerminal('open')).toBe(kept.terminal);
		expect(dockedTerminal('closed')).toBeUndefined();
		expect(gone.session.close).toHaveBeenCalledTimes(1);
		expect(kept.session.close).not.toHaveBeenCalled();
		closeDockedTerminal('open');
	});

	test('a shell that never started is closed without error', () => {
		const {terminal} = fakeTerminal();
		keepDockedTerminal('failed', {...terminal, session: Promise.resolve(null)});
		expect(() => closeDockedTerminal('failed')).not.toThrow();
	});

	test('a focus request is answered once', () => {
		expect(takeTerminalFocus('tab-f')).toBe(false);
		requestTerminalFocus('tab-f');
		expect(takeTerminalFocus('tab-f')).toBe(true);
		expect(takeTerminalFocus('tab-f')).toBe(false);
	});
});
