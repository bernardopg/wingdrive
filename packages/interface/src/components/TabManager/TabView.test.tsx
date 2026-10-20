import {GlobalRegistrator} from '@happy-dom/global-registrator';
import {afterAll, describe, expect, it} from 'bun:test';

GlobalRegistrator.register();

const {startTransition, useLayoutEffect} = await import('react');
const {createRoot} = await import('react-dom/client');
const {Outlet, useLocation} = await import('react-router-dom');
const {TabManagerContext} = await import('./TabManagerContext');
const {createTabRouter} = await import('./tabRouter');
const {renderedLocationKeys, TabView} = await import('./TabView');

afterAll(() => GlobalRegistrator.unregister());

// Suspends on a promise that never settles, so a transition that renders this
// route never commits: the tab looks frozen while its router has moved on.
let stuck = true;
const never = new Promise<never>(() => {});
function Stuck() {
	if (stuck) throw never;
	return <p>destination</p>;
}

function Layout() {
	const location = useLocation();
	// Committed renders only, as TabNavigationSync does.
	useLayoutEffect(() => {
		renderedLocationKeys.set(router, location.key);
	}, [location.key]);
	return <Outlet />;
}

const router = createTabRouter(
	[
		{
			path: '/',
			element: <Layout />,
			children: [
				{index: true, element: <p>home</p>},
				{path: 'stuck', element: <Stuck />}
			]
		}
	],
	'/'
);

describe('TabView', () => {
	it('remounts a tab router whose navigation never renders', async () => {
		const container = document.createElement('div');
		document.body.append(container);
		const value = {router, activeTabId: 'tab'} as never;
		createRoot(container).render(
			<TabManagerContext.Provider value={value}>
				<TabView />
			</TabManagerContext.Provider>
		);
		await new Promise((resolve) => setTimeout(resolve, 50));
		expect(container.textContent).toBe('home');

		startTransition(() => void router.navigate('/stuck'));
		await new Promise((resolve) => setTimeout(resolve, 200));
		expect(router.state.location.pathname).toBe('/stuck');
		expect(container.textContent).toBe('home');

		// The held render never retries; only the watchdog's remount shows the page.
		stuck = false;
		await new Promise((resolve) => setTimeout(resolve, 3300));
		expect(container.textContent).toBe('destination');
	}, 10_000);
});
