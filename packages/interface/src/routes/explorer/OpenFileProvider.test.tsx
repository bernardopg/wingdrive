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
const {OpenFileProvider, useOpenFileAction, useOpenInNewTabAction, folderRoute} =
	await import('./OpenFileProvider');
const {TabManagerContext} = await import('../../components/TabManager/TabManagerContext');

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

	it('opens routes in a background tab through one stable action', async () => {
		const created: unknown[][] = [];
		const actions: unknown[] = [];
		let setVersion: (v: number) => void = () => {};
		function Tabs({children}: {children: React.ReactNode}) {
			const [version, set] = useState(0);
			setVersion = set;
			// A new context value per render, as the tab manager gives on every navigation.
			const value = {version, createTab: (...args: unknown[]) => created.push(args)};
			return (
				<TabManagerContext.Provider value={value as never}>{children}</TabManagerContext.Provider>
			);
		}
		function Opener() {
			actions.push(useOpenInNewTabAction());
			return null;
		}
		const root = createRoot(document.createElement('div'));
		await act(async () =>
			root.render(
				<Tabs>
					<OpenFileProvider>
						<Opener />
					</OpenFileProvider>
				</Tabs>
			)
		);
		await act(async () => setVersion(1));

		expect(new Set(actions).size).toBe(1);
		(actions[0] as (route: string, title?: string) => void)('/explorer?x', 'Root');
		expect(created).toEqual([['Root', '/explorer?x', {background: true}]]);
		await act(async () => root.unmount());
	});

	it('gives folders and virtual items a route, and files none', () => {
		const path = {Physical: {device_slug: 'd', path: '/a'}};
		expect(folderRoute({kind: 'Directory', wing_path: path} as never)).toContain('/explorer?path=');
		expect(folderRoute({kind: 'File', wing_path: path} as never)).toBeNull();
	});
});
