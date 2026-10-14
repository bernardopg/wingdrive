import {GlobalRegistrator} from '@happy-dom/global-registrator';
import {afterAll, describe, expect, it, mock} from 'bun:test';

GlobalRegistrator.register();

const shown: Array<{file: string; x: number; y: number}> = [];
const built: string[] = [];
let appsLoading = false;

mock.module('./hooks/useFileContextMenu', () => ({
	useFileContextMenu: ({file}: {file: {id: string}}) => {
		built.push(file.id);
		return {
			appsLoading,
			show: async (e: {clientX: number; clientY: number}) => {
				shown.push({file: file.id, x: e.clientX, y: e.clientY});
			}
		};
	}
}));
mock.module('./SelectionContext', () => ({
	useSelection: () => ({selectedFiles: []})
}));

const {act} = await import('react');
const {createRoot} = await import('react-dom/client');
const {FileContextMenuProvider, useShowFileContextMenu} = await import(
	'./FileContextMenuProvider'
);

(globalThis as {IS_REACT_ACT_ENVIRONMENT?: boolean}).IS_REACT_ACT_ENVIRONMENT = true;

let show: ReturnType<typeof useShowFileContextMenu> | undefined;
function Item() {
	show = useShowFileContextMenu();
	return null;
}

const event = (x: number, y: number) => ({
	clientX: x,
	clientY: y,
	preventDefault() {},
	stopPropagation() {}
});

afterAll(() => GlobalRegistrator.unregister());

describe('FileContextMenuProvider', () => {
	it('builds no menu until one is requested, then shows each request once', async () => {
		const root = createRoot(document.createElement('div'));
		await act(async () => {
			root.render(
				<FileContextMenuProvider>
					<Item />
					<Item />
				</FileContextMenuProvider>
			);
		});
		expect(built).toEqual([]);

		appsLoading = true;
		await act(async () => show!(event(10, 20), {id: 'a'} as never, false));
		expect(shown).toEqual([]);

		appsLoading = false;
		await act(async () => show!(event(10, 20), {id: 'a'} as never, false));
		expect(shown).toEqual([{file: 'a', x: 10, y: 20}]);

		await act(async () => show!(event(30, 40), {id: 'b'} as never, true));
		expect(shown).toEqual([
			{file: 'a', x: 10, y: 20},
			{file: 'b', x: 30, y: 40}
		]);
		await act(async () => root.unmount());
	});
});
