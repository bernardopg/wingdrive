import {GlobalRegistrator} from '@happy-dom/global-registrator';
import {afterAll, describe, expect, it} from 'bun:test';

// Registered once per test process and never unregistered: unregistering closes
// the window whose timers React's scheduler keeps using in the next test file.
if (!GlobalRegistrator.isRegistered) GlobalRegistrator.register();

const {startTransition, useLayoutEffect} = await import('react');
const {createRoot} = await import('react-dom/client');
const {Outlet, useLocation} = await import('react-router-dom');
const {TabManagerContext} = await import('./TabManagerContext');
const {createTabRouter} = await import('./tabRouter');
const {renderedLocationKeys, TabView} = await import('./TabView');

// Other test files turn on React's act environment; this test drives the real
// scheduler and timers, outside act().
const actEnvironment = globalThis as {IS_REACT_ACT_ENVIRONMENT?: boolean};
const previousActEnvironment = actEnvironment.IS_REACT_ACT_ENVIRONMENT;

afterAll(() => {
	actEnvironment.IS_REACT_ACT_ENVIRONMENT = previousActEnvironment;
});

/** Waits up to `ms` for `text` to render; the first render can take a while in a full test run. */
async function waitForText(container: HTMLElement, text: string, ms: number) {
	const deadline = Date.now() + ms;
	while (container.textContent !== text && Date.now() < deadline)
		await new Promise((resolve) => setTimeout(resolve, 25));
}

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
		actEnvironment.IS_REACT_ACT_ENVIRONMENT = false;
		const container = document.createElement('div');
		document.body.append(container);
		const value = {router, activeTabId: 'tab'} as never;
		createRoot(container).render(
			<TabManagerContext.Provider value={value}>
				<TabView />
			</TabManagerContext.Provider>
		);
		await waitForText(container, 'home', 2000);
		expect(container.textContent).toBe('home');

		startTransition(() => void router.navigate('/stuck'));
		await new Promise((resolve) => setTimeout(resolve, 200));
		expect(router.state.location.pathname).toBe('/stuck');
		expect(container.textContent).toBe('home');

		// The held render never retries; only the watchdog's remount shows the page.
		stuck = false;
		await waitForText(container, 'destination', 5000);
		expect(container.textContent).toBe('destination');
	}, 10_000);
});
