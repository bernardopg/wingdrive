import { useEffect, useRef } from "react";
import { useLocation, useNavigate } from "react-router-dom";
import { useTabManager } from "./useTabManager";
import { deriveTitleFromPath } from "./deriveTitle";

/**
 * TabNavigationSync - Syncs router navigation with active tab
 *
 * This component runs inside the router context and:
 * 1. Saves the current location to the active tab when navigation occurs
 * 2. Updates the tab title based on the current route
 * 3. Navigates to the saved location when switching to a different tab
 */
export function TabNavigationSync() {
	const location = useLocation();
	const navigate = useNavigate();
	const { activeTabId, tabs, updateTabPath, updateTabTitle } = useTabManager();

	const activeTab = tabs.find((t) => t.id === activeTabId);
	const currentPath = location.pathname + location.search;

	// Track previous activeTabId to detect tab switches
	const prevActiveTabIdRef = useRef(activeTabId);

	// Save current location and update title for active tab (only for in-tab navigation)
	useEffect(() => {
		// Skip saving during tab switch - currentPath belongs to the old tab
		if (prevActiveTabIdRef.current !== activeTabId) {
			prevActiveTabIdRef.current = activeTabId;
			return;
		}

		if (activeTab && currentPath !== activeTab.savedPath) {
			updateTabPath(activeTabId, currentPath);
		}

		// Update title based on current location (null = managed by the route component)
		const newTitle = deriveTitleFromPath(location.pathname, location.search);
		if (activeTab && newTitle !== null && newTitle !== activeTab.title) {
			updateTabTitle(activeTabId, newTitle);
		}
	}, [currentPath, activeTab, activeTabId, updateTabPath, updateTabTitle, location.pathname, location.search]);

	// Navigate to saved location when switching tabs
	useEffect(() => {
		if (activeTab && currentPath !== activeTab.savedPath) {
			navigate(activeTab.savedPath, { replace: true });
		}
		// eslint-disable-next-line react-hooks/exhaustive-deps
	}, [activeTabId]);

	return null;
}
