import {expect, test} from 'bun:test';
import {restoreTabState} from './restoreTabState';

test('restoring a tab keeps its settings but never starts its previous terminal', () => {
	const previous = {terminalVisible: true, terminalHeight: 320, viewMode: 'list'};
	expect(restoreTabState(previous)).toEqual({terminalVisible: false, terminalHeight: 320, viewMode: 'list'});
	expect(previous.terminalVisible).toBe(true);
});
