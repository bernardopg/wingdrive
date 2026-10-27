import {GlobalRegistrator} from '@happy-dom/global-registrator';
import {describe, expect, it, mock} from 'bun:test';

// Registered once per test process and never unregistered: unregistering closes
// the window whose timers React's scheduler keeps using in the next test file.
if (!GlobalRegistrator.isRegistered) GlobalRegistrator.register();

let handlerVersion = 0;
const opened: string[] = [];
let built = 0;

mock.module('./hooks/useOpenFile', () => ({
	useOpenFile: () => {
		built++;
		const version = handlerVersion;
		return async (file: {id: string}) => {
			opened.push(`${file.id}@${version}`);
		};
	}
}));

const {act, useState} = await import('react');
const {createRoot} = await import('react-dom/client');
const {OpenFileProvider, useOpenFileAction} = await import('./OpenFileProvider');

(globalThis as {IS_REACT_ACT_ENVIRONMENT?: boolean}).IS_REACT_ACT_ENVIRONMENT = true;

const seen: unknown[] = [];
let rerender: () => void = () => {};
function Item() {
	seen.push(useOpenFileAction());
	return null;
}
function Shell() {
	const [, setTick] = useState(0);
	rerender = () => setTick((tick) => tick + 1);
	return (
		<OpenFileProvider>
			<Item />
			<Item />
		</OpenFileProvider>
	);
}

describe('OpenFileProvider', () => {
	it('gives every item one stable handler that runs the latest open logic', async () => {
		const root = createRoot(document.createElement('div'));
		await act(async () => root.render(<Shell />));
		handlerVersion = 1;
		await act(async () => rerender());

		// One handler for the whole tree, not one per item.
		expect(built).toBe(2);
		expect(new Set(seen).size).toBe(1);

		const open = seen[0] as (file: {id: string}) => Promise<void>;
		await open({id: 'a'});
		expect(opened).toEqual(['a@1']);
		await act(async () => root.unmount());
	});
});
