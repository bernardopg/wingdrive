import {useLayoutEffect, useState} from 'react';
import {RouterProvider} from 'react-router-dom';
import {useTabManager} from './useTabManager';

/** Location key each tab router last rendered; written by `TabNavigationSync`. */
export const renderedLocationKeys = new WeakMap<object, string>();

const STALE_RENDER_MS = 3000;

/**
 * Renders the active tab's router. A navigation could update the router without
 * ever reaching the screen, leaving every click in the tab without effect until
 * another tab was opened, which remounts the router. When the rendered location
 * still lags the router after `STALE_RENDER_MS`, remount it here the same way.
 */
export function TabView() {
	const {router, activeTabId} = useTabManager();
	const [remounts, setRemounts] = useState(0);

	// Subscribe at commit, as RouterProvider does, so no navigation slips in before the watch starts.
	useLayoutEffect(() => {
		let timer: ReturnType<typeof setTimeout> | undefined;
		const unsubscribe = router.subscribe((state) => {
			clearTimeout(timer);
			if (state.navigation.state !== 'idle') return;
			const key = state.location.key;
			timer = setTimeout(() => {
				const rendered = renderedLocationKeys.get(router);
				if (rendered === undefined || rendered === key) return;
				console.warn(
					`[TabView] ${state.location.pathname} did not render after ${STALE_RENDER_MS} ms; remounting the tab router`
				);
				setRemounts((count) => count + 1);
			}, STALE_RENDER_MS);
		});
		return () => {
			clearTimeout(timer);
			unsubscribe();
		};
	}, [router]);

	return <RouterProvider key={`${activeTabId}:${remounts}`} router={router} />;
}
