import {useDroppable} from '@dnd-kit/core';
import clsx from 'clsx';
import {useLayoutEffect, useMemo, useState} from 'react';
import {RouterProvider} from 'react-router-dom';
import {folderOfUrl} from './otherPane';
import {PaneContext} from './PaneContext';
import {TabManagerContext, type PaneId} from './TabManagerContext';
import {useTabManager} from './useTabManager';

/** Location key each tab router last rendered; written by `TabNavigationSync`. */
export const renderedLocationKeys = new WeakMap<object, string>();

const STALE_RENDER_MS = 3000;

type Router = ReturnType<typeof useTabManager>['router'];

/**
 * A navigation could update the router without ever reaching the screen,
 * leaving every click in the tab without effect until another tab was opened,
 * which remounts the router. When the rendered location still lags the router
 * after `STALE_RENDER_MS`, remount it here the same way.
 */
function useStaleRenderRemounts(router: Router) {
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

	return remounts;
}

function Pane({pane, tabId, router}: {pane: PaneId; tabId: string; router: Router}) {
	const manager = useTabManager();
	const remounts = useStaleRenderRemounts(router);
	const split = !!manager.split;
	const focused = manager.focusedPane === pane;
	const paneState = useMemo(() => ({pane, split, focused}), [pane, split, focused]);
	// Everything inside a pane keys its state by `activeTabId`, so the second
	// pane sees its own hidden tab as the active one.
	// In a split view the whole pane takes files dragged from the other one,
	// into the folder it shows; folders inside it remain their own targets.
	const folder = split ? folderOfUrl(manager.panePath(pane)) : null;
	const {setNodeRef, isOver} = useDroppable({
		id: `pane-drop-${pane}`,
		disabled: !folder,
		data: {action: 'move-into', targetType: 'pane', targetPath: folder}
	});
	const scoped = useMemo(
		() => (pane === 'primary' ? manager : {...manager, activeTabId: tabId, router}),
		[manager, pane, tabId, router]
	);

	return (
		<TabManagerContext.Provider value={scoped}>
			<PaneContext.Provider value={paneState}>
				<div
					ref={setNodeRef}
					className={clsx(
						'relative min-w-0 flex-1',
						split && 'basis-0',
						isOver && 'after:pointer-events-none after:absolute after:inset-2 after:z-[80] after:rounded-lg after:ring-2 after:ring-accent'
					)}
					onPointerDownCapture={(e) => {
						// The sidebar navigates the focused pane, so a click there must
						// not move the focus to the pane that holds it.
						if ((e.target as Element).closest('[data-pane-neutral]')) return;
						if (!focused) manager.setFocusedPane(pane);
					}}
					onFocusCapture={(e) => {
						if ((e.target as Element).closest('[data-pane-neutral]')) return;
						if (!focused) manager.setFocusedPane(pane);
					}}
				>
					<RouterProvider key={`${tabId}:${remounts}`} router={router} />
				</div>
			</PaneContext.Provider>
		</TabManagerContext.Provider>
	);
}

/** Renders the active tab's router, and its second pane while the split view is open. */
export function TabView() {
	const {router, activeTabId, split} = useTabManager();

	if (!split) return <Pane pane="primary" tabId={activeTabId} router={router} />;

	return (
		<div className="flex h-screen w-screen overflow-hidden">
			<Pane pane="primary" tabId={activeTabId} router={router} />
			<Pane pane="secondary" tabId={split.tabId} router={split.router} />
		</div>
	);
}
