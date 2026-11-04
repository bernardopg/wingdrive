import {useEffect, useLayoutEffect} from 'react';
import {useLocation} from 'react-router-dom';
import {deriveTitleFromPath} from './deriveTitle';
import {usePane} from './PaneContext';
import {renderedLocationKeys} from './TabView';
import {useTabManager} from './useTabManager';

export function TabNavigationSync() {
	const location = useLocation();
	const {activeTabId, router, tabs, updateTabTitle} = useTabManager();
	const {pane} = usePane();
	useLayoutEffect(() => {
		renderedLocationKeys.set(router, location.key);
	}, [router, location.key]);
	useEffect(() => {
		// The window URL restores the tab on reload; the split pane keeps its
		// own path in the tab state instead.
		if (pane === 'primary')
			window.history.replaceState(
				null,
				'',
				location.pathname + location.search + location.hash
			);
		const title = deriveTitleFromPath(location.pathname, location.search);
		const tab = tabs.find((tab) => tab.id === activeTabId);
		if (title !== null && tab && title !== tab.title)
			updateTabTitle(activeTabId, title);
	}, [location, activeTabId, tabs, updateTabTitle, pane]);
	return null;
}
