import {createContext, useContext, useEffect, useRef} from 'react';
import type {PaneId} from './TabManagerContext';

/**
 * # Panes of the split view
 *
 * With the split view open (F3) the window shows two tab routers side by side,
 * each with its own Explorer, selection and history. Code that listens to the
 * whole window (shortcuts, arrow keys, typeahead) reads this context so only
 * the focused pane reacts. Outside a split there is one pane, always focused.
 */
export interface PaneState {
	pane: PaneId;
	/** Whether the window shows two panes. */
	split: boolean;
	/** Whether this pane receives keyboard input. */
	focused: boolean;
}

const SINGLE_PANE: PaneState = {pane: 'primary', split: false, focused: true};

export const PaneContext = createContext<PaneState>(SINGLE_PANE);

export function usePane(): PaneState {
	return useContext(PaneContext);
}

/**
 * Whether this pane has keyboard focus, as a ref for listeners registered once
 * on `window`.
 */
export function usePaneFocusedRef() {
	const {focused} = usePane();
	const ref = useRef(focused);
	useEffect(() => {
		ref.current = focused;
	}, [focused]);
	return ref;
}
