import {useCallback} from 'react';
import {createPath, useNavigate, type NavigateOptions, type To} from 'react-router-dom';
import {usePane} from './PaneContext';
import {useTabManager} from './useTabManager';

/**
 * `useNavigate` for the sidebar. The sidebar lives in the left pane of a split
 * view but acts on the focused pane, as Dolphin's Places panel does, so a
 * click while the right pane has focus navigates the right pane.
 */
export function useSidebarNavigate() {
	const navigate = useNavigate();
	const {pane} = usePane();
	const {split, focusedPane, navigatePane} = useTabManager();

	return useCallback(
		(to: To, options?: NavigateOptions) => {
			if (split && focusedPane !== pane) {
				navigatePane(focusedPane, typeof to === 'string' ? to : createPath(to));
				return;
			}
			void navigate(to, options);
		},
		[navigate, pane, split, focusedPane, navigatePane]
	);
}
