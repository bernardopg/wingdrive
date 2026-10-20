import {useEffect, useLayoutEffect} from 'react';
import {useLocation} from 'react-router-dom';
import {deriveTitleFromPath} from './deriveTitle';
import {renderedLocationKeys} from './TabView';
import {useTabManager} from './useTabManager';

export function TabNavigationSync() {
	const location = useLocation();
	const {activeTabId, router, tabs, updateTabTitle} = useTabManager();
	useLayoutEffect(() => {
		renderedLocationKeys.set(router, location.key);
	}, [router, location.key]);
	useEffect(() => {
		window.history.replaceState(
			null,
			'',
			location.pathname + location.search + location.hash
		);
		const title = deriveTitleFromPath(location.pathname, location.search);
		const tab = tabs.find((tab) => tab.id === activeTabId);
		if (title !== null && tab && title !== tab.title)
			updateTabTitle(activeTabId, title);
	}, [location, activeTabId, tabs, updateTabTitle]);
	return null;
}
