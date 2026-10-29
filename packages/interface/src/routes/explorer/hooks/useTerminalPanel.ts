import {useCallback} from 'react';

import {useTabManager} from '../../../components/TabManager';
import {usePlatform} from '../../../contexts/PlatformContext';
import {requestTerminalFocus} from '../terminal/dockedTerminals';
import {DEFAULT_TERMINAL_HEIGHT} from '../terminal/terminalFolder';

/** Whether the active tab shows the terminal docked under the view (F4), and its size. */
export function useTerminalPanel() {
	const platform = usePlatform();
	const {activeTabId, getExplorerState, updateExplorerState} = useTabManager();
	const supported = !!platform.openTerminalSession;
	const state = getExplorerState(activeTabId);

	const toggle = useCallback(() => {
		const visible = !getExplorerState(activeTabId).terminalVisible;
		if (visible) requestTerminalFocus(activeTabId);
		updateExplorerState(activeTabId, {terminalVisible: visible});
	}, [activeTabId, getExplorerState, updateExplorerState]);

	// Takes the tab id: the shell can exit while its tab is in the background.
	const hide = useCallback(
		(tabId: string) => updateExplorerState(tabId, {terminalVisible: false}),
		[updateExplorerState]
	);

	const setHeight = useCallback(
		(height: number) => updateExplorerState(activeTabId, {terminalHeight: height}),
		[activeTabId, updateExplorerState]
	);

	return {
		supported,
		visible: supported && !!state.terminalVisible,
		height: state.terminalHeight ?? DEFAULT_TERMINAL_HEIGHT,
		tabId: activeTabId,
		toggle,
		hide,
		setHeight
	};
}
