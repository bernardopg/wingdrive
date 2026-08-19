import { useTabManager } from "./useTabManager";
import { useKeybind } from "../../hooks/useKeybind";
import { usePlatform } from "../../contexts/PlatformContext";

/**
 * TabKeyboardHandler - Handles keyboard shortcuts for tab operations
 *
 * Uses the keybind system to listen for tab-related shortcuts and trigger actions.
 */
export function TabKeyboardHandler() {
	const { createTab, closeTab, nextTab, previousTab, selectTabAtIndex, reopenTab, tabs, activeTabId } =
		useTabManager();
	const platform = usePlatform();

	// New Tab (Cmd+T)
	useKeybind("global.newTab", () => {
		createTab();
	});

	// Reopen Closed Tab (Cmd+Shift+T)
	useKeybind("global.reopenTab", () => {
		reopenTab();
	});

	// Close Tab (Cmd+W). On the last tab this closes the window instead, which
	// is what every tabbed app does and what the intercepted native shortcut
	// would otherwise have done.
	useKeybind("global.closeTab", () => {
		if (tabs.length > 1) {
			closeTab(activeTabId);
			return;
		}

		void platform.closeCurrentWindow?.();
	});

	// Next Tab (Cmd+Shift+])
	useKeybind("global.nextTab", () => {
		nextTab();
	});

	// Previous Tab (Cmd+Shift+[)
	useKeybind("global.previousTab", () => {
		previousTab();
	});

	// Select Tab 1-9 (Cmd+1-9)
	useKeybind("global.selectTab1", () => selectTabAtIndex(0));
	useKeybind("global.selectTab2", () => selectTabAtIndex(1));
	useKeybind("global.selectTab3", () => selectTabAtIndex(2));
	useKeybind("global.selectTab4", () => selectTabAtIndex(3));
	useKeybind("global.selectTab5", () => selectTabAtIndex(4));
	useKeybind("global.selectTab6", () => selectTabAtIndex(5));
	useKeybind("global.selectTab7", () => selectTabAtIndex(6));
	useKeybind("global.selectTab8", () => selectTabAtIndex(7));
	useKeybind("global.selectTab9", () => selectTabAtIndex(8));

	return null;
}
