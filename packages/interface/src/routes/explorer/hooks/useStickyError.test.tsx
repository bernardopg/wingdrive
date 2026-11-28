import {GlobalRegistrator} from '@happy-dom/global-registrator';
import {describe, expect, it} from 'bun:test';

if (!GlobalRegistrator.isRegistered) GlobalRegistrator.register();
(globalThis as {IS_REACT_ACT_ENVIRONMENT?: boolean}).IS_REACT_ACT_ENVIRONMENT = true;

const {act} = await import('react');
const {createRoot} = await import('react-dom/client');
const {useStickyError} = await import('./useStickyError');
type QueryErrorState = import('./useStickyError').QueryErrorState;

const notFound = new Error('No such file or directory');

type Props = {query: QueryErrorState | null; key: string};

/** Renders the hook and exposes its last result, like `renderHook`. */
function render(query: QueryErrorState | null, key = 'directory:/missing') {
	const result: {current: Error | null} = {current: null};
	const root = createRoot(document.createElement('div'));
	function Probe({query, keyName}: {query: Props['query']; keyName: string}) {
		result.current = useStickyError(query, keyName);
		return null;
	}
	const rerender = (props: Props) => {
		act(() => root.render(<Probe query={props.query} keyName={props.key} />));
	};
	rerender({query, key});
	return {result, rerender};
}

describe('useStickyError', () => {
	it('keeps the error while a query with no data fetches again', () => {
		const hook = render({error: notFound, status: 'error'});
		expect(hook.result.current).toBe(notFound);
		// A refetch of a query without data goes back to pending and clears the error.
		hook.rerender({query: {error: null, status: 'pending'}, key: 'directory:/missing'});
		expect(hook.result.current).toBe(notFound);
	});

	it('drops the error once a fetch succeeds', () => {
		const hook = render({error: notFound, status: 'error'});
		hook.rerender({query: {error: null, status: 'success'}, key: 'directory:/missing'});
		expect(hook.result.current).toBeNull();
		hook.rerender({query: {error: null, status: 'pending'}, key: 'directory:/missing'});
		expect(hook.result.current).toBeNull();
	});

	it('drops the error when the view lists something else', () => {
		const hook = render({error: notFound, status: 'error'});
		hook.rerender({query: {error: null, status: 'pending'}, key: 'directory:/other'});
		expect(hook.result.current).toBeNull();
	});

	it('shows the newest error', () => {
		const denied = new Error('Permission denied');
		const hook = render({error: notFound, status: 'error'});
		hook.rerender({query: {error: denied, status: 'error'}, key: 'directory:/missing'});
		expect(hook.result.current).toBe(denied);
	});

	it('has no error without a query', () => {
		const hook = render({error: notFound, status: 'error'});
		hook.rerender({query: null, key: 'virtual'});
		expect(hook.result.current).toBeNull();
	});
});
