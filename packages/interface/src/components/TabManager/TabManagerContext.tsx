import type {ColumnSizingState, SortingState} from '@tanstack/react-table';
import type {
	DirectorySortBy,
	MediaSortBy,
	SortDirection
} from '@wingdrive/ts-client';
import {
	createContext,
	useCallback,
	useEffect,
	useMemo,
	useRef,
	useState,
	type ReactNode
} from 'react';
import type {RouteObject} from 'react-router-dom';
import {usePlatform} from '../../contexts/PlatformContext';
import {useWingDriveClient} from '@wingdrive/ts-client';
import {deriveTitleFromPath} from './deriveTitle';
import {remapSelectionIds} from './remapSelectionIds';
import {restoreTabState} from './restoreTabState';
import {createTabRouter} from './tabRouter';

type Router = ReturnType<typeof createTabRouter>;

// ============================================================================
// Types
// ============================================================================

export type ViewMode = 'grid' | 'list' | 'column' | 'media' | 'size';
export type SortBy = DirectorySortBy | MediaSortBy;

export interface Tab {
	id: string;
	title: string;
	icon: string | null;
	isPinned: boolean;
	lastActive: number;
	savedPath: string;
	history?: string[];
	historyIndex?: number;
}

/**
 * All explorer-related state for a single tab.
 * This is the single source of truth - no sync effects needed.
 */
export interface TabExplorerState {
	listSorting?: SortingState;
	/** Folders expanded in place in list view, by absolute path. */
	listExpanded?: string[];
	/** List view widths the user dragged; columns missing here use their defaults. */
	listColumnSizing?: ColumnSizingState;
	/** Terminal docked under the view (F4), and its height in pixels. */
	terminalVisible?: boolean;
	terminalHeight?: number;
	// View settings
	viewMode: ViewMode;
	sortBy: SortBy;
	/** Explicit direction; null uses the natural default for `sortBy`. */
	sortDirection: SortDirection | null;
	gridSize: number;
	gapSize: number;
	foldersFirst: boolean;
	showHiddenFiles: boolean;
	/** Split grid and list views into sections by the active sort key. */
	groupItems: boolean;

	// Column view state (serialized WingPath[] as JSON strings)
	columnStack: string[];

	// Scroll position
	scrollTop: number;
	scrollLeft: number;

	// Size view transform (zoom + pan)
	sizeViewTransform: {k: number; x: number; y: number};
}

/**
 * The second pane of a tab's split view (F3). Its tab never shows in the tab
 * bar; it keeps its own path, history and explorer state, and survives closing
 * the split so reopening it returns to the same folder.
 */
export interface SplitPane {
	tab: Tab;
	open: boolean;
}

export type PaneId = 'primary' | 'secondary';

/** A closed tab plus the explorer state it had, so reopening restores context. */
interface ClosedTab {
	tab: Tab;
	explorerState?: TabExplorerState;
}

/** Default explorer state for new tabs */
const DEFAULT_EXPLORER_STATE: TabExplorerState = {
	viewMode: 'grid',
	sortBy: 'name',
	sortDirection: null,
	gridSize: 120,
	gapSize: 16,
	foldersFirst: true,
	showHiddenFiles: false,
	groupItems: false,
	columnStack: [],
	scrollTop: 0,
	scrollLeft: 0,
	sizeViewTransform: {k: 1, x: 0, y: 0}
};

// ============================================================================
// Persistence
// ============================================================================

const STORAGE_KEY = 'wing-tabs-state';

/**
 * Secondary windows run their own tab manager; sharing one key would make two
 * windows overwrite each other's tabs on every change.
 */
export function storageKeyFor(windowLabel?: string): string {
	if (!windowLabel || windowLabel === 'main') return STORAGE_KEY;
	// Each file chooser window has a new label; they share one entry instead of piling up.
	if (windowLabel.startsWith('file-chooser-')) return `${STORAGE_KEY}:file-chooser`;
	return `${STORAGE_KEY}:${windowLabel}`;
}

interface PersistedState {
	tabs: Tab[];
	activeTabId: string;
	explorerStates: Record<string, TabExplorerState>;
	defaultNewTabPath: string;
	/** Split view of each tab, by the id of the tab that owns it. */
	splits?: Record<string, SplitPane>;
}

function loadPersistedState(storageKey: string): PersistedState | null {
	try {
		const stored = localStorage.getItem(storageKey);
		if (!stored) return null;

		const parsed = JSON.parse(stored) as PersistedState;

		// Validate structure
		if (
			!Array.isArray(parsed.tabs) ||
			typeof parsed.activeTabId !== 'string' ||
			typeof parsed.explorerStates !== 'object'
		) {
			return null;
		}

		const localPath = (path: unknown): path is string =>
			typeof path === 'string' &&
			path.startsWith('/') &&
			!path.startsWith('//');
		if (
			!parsed.tabs.length ||
			parsed.explorerStates === null ||
			parsed.tabs.some(
				(tab) =>
					!tab ||
					typeof tab.id !== 'string' ||
					typeof tab.title !== 'string' ||
					!localPath(tab.savedPath)
			) ||
			new Set(parsed.tabs.map((tab) => tab.id)).size !==
				parsed.tabs.length
		)
			return null;
		for (const tab of parsed.tabs) {
			if (
				!Array.isArray(tab.history) ||
				!tab.history.length ||
				!tab.history.every(localPath) ||
				!Number.isInteger(tab.historyIndex)
			) {
				tab.history = [tab.savedPath];
				tab.historyIndex = 0;
			}
		}
		if (!localPath(parsed.defaultNewTabPath))
			parsed.defaultNewTabPath = '/';
		const splits: Record<string, SplitPane> = {};
		const owners = new Set(parsed.tabs.map((tab) => tab.id));
		for (const [owner, split] of Object.entries(parsed.splits ?? {})) {
			const tab = split?.tab;
			if (!owners.has(owner) || !tab || typeof tab.id !== 'string' || !localPath(tab.savedPath))
				continue;
			if (!Array.isArray(tab.history) || !tab.history.every(localPath) || !Number.isInteger(tab.historyIndex)) {
				tab.history = [tab.savedPath];
				tab.historyIndex = 0;
			}
			splits[owner] = {tab, open: split.open === true};
		}
		parsed.splits = splits;
		return parsed;
	} catch {
		return null;
	}
}

function savePersistedState(storageKey: string, state: PersistedState): void {
	try {
		localStorage.setItem(storageKey, JSON.stringify(state));
	} catch {
		// Silently fail if localStorage is unavailable
	}
}

// ============================================================================
// Context
// ============================================================================

interface TabManagerContextValue {
	// Tab management
	tabs: Tab[];
	activeTabId: string;
	router: Router;
	/** `background` opens the tab without switching to it, as a middle-click does in a browser. */
	createTab: (title?: string, path?: string, options?: {background?: boolean}) => void;
	closeTab: (tabId: string) => void;
	/** Closes several tabs in one state transition (close others / to the right). */
	closeTabs: (tabIds: string[]) => void;
	switchTab: (tabId: string) => void;
	updateTabTitle: (tabId: string, title: string) => void;
	updateTabPath: (tabId: string, path: string) => void;
	reorderTabs: (activeId: string, overId: string) => void;
	nextTab: () => void;
	previousTab: () => void;
	selectTabAtIndex: (index: number) => void;
	reopenTab: () => void;
	hasClosedTabs: boolean;
	setDefaultNewTabPath: (path: string) => void;

	// Explorer state (per-tab)
	getExplorerState: (tabId: string) => TabExplorerState;
	updateExplorerState: (
		tabId: string,
		updates: Partial<TabExplorerState>
	) => void;

	// Selection state (per-tab, ephemeral - not persisted)
	getSelectionIds: (tabId: string) => string[];
	updateSelectionIds: (tabId: string, fileIds: string[]) => void;

	// Split view of the active tab
	/** The active tab's second pane while the split is open, else null. */
	split: {tabId: string; router: Router} | null;
	/** Opens or closes the active tab's split view (F3). */
	toggleSplit: () => void;
	/** Pane that receives keyboard shortcuts and sidebar navigation. */
	focusedPane: PaneId;
	setFocusedPane: (pane: PaneId) => void;
	/** Current URL of a pane of the active tab; null for a closed split. */
	panePath: (pane: PaneId) => string | null;
	/** Navigates a pane of the active tab, as a sidebar click does. */
	navigatePane: (pane: PaneId, to: string) => void;
}

const TabManagerContext = createContext<TabManagerContextValue | null>(null);

// ============================================================================
// Provider
// ============================================================================

interface TabManagerProviderProps {
	children: ReactNode;
	routes: RouteObject[];
}

export function TabManagerProvider({
	children,
	routes
}: TabManagerProviderProps) {
	// Read localStorage once: four separate initializers meant four JSON parses
	// and left room for the slices to disagree with each other.
	const platform = usePlatform();
	const client = useWingDriveClient();
	const storageKey = useRef(
		storageKeyFor(platform.getCurrentWindowLabel?.())
	).current;
	const restored = useRef(loadPersistedState(storageKey)).current;

	const [tabs, setTabs] = useState<Tab[]>(() => {
		if (restored && restored.tabs.length > 0) {
			const deepLink = window.location.pathname + window.location.search;
			return restored.tabs.map((tab) =>
				tab.id === restored.activeTabId && deepLink !== '/'
					? {
							...tab,
							savedPath: deepLink,
							history: [deepLink],
							historyIndex: 0
						}
					: tab
			);
		}

		return [
			{
				id: crypto.randomUUID(),
				title: 'Overview',
				icon: null,
				isPinned: false,
				lastActive: Date.now(),
				savedPath: window.location.pathname + window.location.search
			}
		];
	});

	const [activeTabId, setActiveTabId] = useState<string>(() => {
		const tabExists = tabs.some((t) => t.id === restored?.activeTabId);
		return tabExists ? restored!.activeTabId : tabs[0].id;
	});
	const [splits, setSplits] = useState<Record<string, SplitPane>>(
		() => restored?.splits ?? {}
	);
	const [focusedPane, setFocusedPane] = useState<PaneId>('primary');

	const routers = useRef(new Map<string, Router>());
	const routerFor = useCallback(
		(tab: Tab, save: (savedPath: string, history: string[], historyIndex: number) => void) => {
			const cached = routers.current.get(tab.id);
			if (cached) return cached;
			const created = createTabRouter(routes, tab.savedPath, tab.history, tab.historyIndex, save);
			routers.current.set(tab.id, created);
			return created;
		},
		[routes]
	);
	const router = useMemo(() => {
		const tab = tabs.find((tab) => tab.id === activeTabId) ?? tabs[0];
		return routerFor(tab, (savedPath, history, historyIndex) =>
			setTabs((prev) =>
				prev.map((entry) =>
					entry.id === tab.id
						? {...entry, savedPath, history, historyIndex}
						: entry
				)
			)
		);
	}, [activeTabId, routerFor, tabs]);

	const activeSplit = splits[activeTabId];
	const splitTab = activeSplit?.open ? activeSplit.tab : null;
	const splitRouter = useMemo(() => {
		if (!splitTab) return null;
		const owner = activeTabId;
		return routerFor(splitTab, (savedPath, history, historyIndex) =>
			setSplits((prev) =>
				prev[owner]
					? {...prev, [owner]: {...prev[owner], tab: {...prev[owner].tab, savedPath, history, historyIndex}}}
					: prev
			)
		);
	}, [activeTabId, routerFor, splitTab]);
	const split = useMemo(
		() => (splitTab && splitRouter ? {tabId: splitTab.id, router: splitRouter} : null),
		[splitTab, splitRouter]
	);
	// A closed split or another tab hands shortcuts back to the remaining pane.
	const effectiveFocusedPane: PaneId = split ? focusedPane : 'primary';

	useEffect(() => {
		// ponytail: routers use O(open tabs) memory; evict cold routers if very large sessions need a cap.
		const open = new Set([
			...tabs.map((tab) => tab.id),
			...Object.values(splits).map((split) => split.tab.id)
		]);
		for (const [id, router] of routers.current) {
			if (!open.has(id)) {
				router.dispose();
				routers.current.delete(id);
			}
		}
	}, [tabs]);
	const mounted = useRef(false);
	useEffect(() => {
		mounted.current = true;
		const cache = routers.current;
		return () => {
			mounted.current = false;
			// Strict Mode replays effects before this microtask; only dispose on actual unmount.
			queueMicrotask(() => {
				if (!mounted.current)
					for (const router of cache.values()) router.dispose();
			});
		};
	}, []);

	const [explorerStates, setExplorerStates] = useState<
		Map<string, TabExplorerState>
	>(() => {
		if (restored?.explorerStates) {
			return new Map(
				Object.entries(restored.explorerStates).map(([id, state]) => [
					id,
					restoreTabState({...DEFAULT_EXPLORER_STATE, ...state})
				])
			);
		}

		const initialMap = new Map<string, TabExplorerState>();
		initialMap.set(tabs[0].id, {...DEFAULT_EXPLORER_STATE});
		return initialMap;
	});

	// Per-tab selection state (ephemeral, not persisted to localStorage)
	const [selectionStates, setSelectionStates] = useState<
		Map<string, string[]>
	>(() => {
		const initialMap = new Map<string, string[]>();
		// Initialize with empty selection for first tab
		initialMap.set(tabs[0].id, []);
		return initialMap;
	});

	// Remap selections in every tab, including tabs that are not currently open.
	// The cache handles row identity; the tab manager owns selected IDs.
	useEffect(() => {
		let unsubscribe: (() => void) | undefined;
		let disposed = false;
		let generation = 0;
		const subscribe = () => {
			const current = ++generation;
			unsubscribe?.();
			unsubscribe = undefined;
			const libraryId = client.getCurrentLibraryId();
			if (!libraryId) return;
			void client.subscribeFiltered(
				{ library_id: libraryId, resource_type: "file", event_types: ["ResourceChanged"] },
				(event) => {
					if (disposed || current !== generation) return;
					if (!(typeof event === "object" && event !== null && "ResourceChanged" in event)) return;
					const { resource_type, resource, metadata } = event.ResourceChanged;
					if (resource_type !== "file" || !resource || typeof resource !== "object" || Array.isArray(resource)) return;
					const id = resource.id;
					if (typeof id === "string" && metadata?.alternate_ids.length) {
						setSelectionStates((previous) => remapSelectionIds(previous, metadata.alternate_ids, id));
					}
				},
			).then((stop) => {
				if (disposed || current !== generation) stop();
				else unsubscribe = stop;
			}).catch((error: unknown) => {
				console.error("Failed to subscribe to UUID reconciliation events", error);
			});
		};
		const onLibraryChanged = () => {
			setSelectionStates(new Map());
			subscribe();
		};
		client.on("library-changed", onLibraryChanged);
		subscribe();
		return () => {
			disposed = true;
			generation++;
			client.off("library-changed", onLibraryChanged);
			unsubscribe?.();
		};
	}, [client]);

	// Recently closed tabs (LIFO, max 10) for Cmd+Shift+T reopen. The explorer
	// state travels with the tab so reopening restores view mode and scroll.
	const [closedTabs, setClosedTabs] = useState<ClosedTab[]>([]);

	const [defaultNewTabPath, setDefaultNewTabPathState] = useState<string>(
		() => restored?.defaultNewTabPath ?? '/'
	);

	// Safety net for any path that removes the active tab: keeping the pointer
	// valid here means no state updater has to reach out and set it.
	useEffect(() => {
		if (tabs.length === 0) return;
		if (tabs.some((t) => t.id === activeTabId)) return;
		setActiveTabId(tabs[tabs.length - 1].id);
	}, [tabs, activeTabId]);

	// ========================================================================
	// Persistence
	// ========================================================================

	useEffect(() => {
		const explorerStatesObject = Object.fromEntries(explorerStates);

		savePersistedState(storageKey, {
			tabs,
			activeTabId,
			explorerStates: explorerStatesObject,
			defaultNewTabPath,
			splits
		});
	}, [storageKey, tabs, activeTabId, explorerStates, defaultNewTabPath, splits]);

	// ========================================================================
	// Tab management
	// ========================================================================

	const setDefaultNewTabPath = useCallback((path: string) => {
		setDefaultNewTabPathState(path);
	}, []);

	const createTab = useCallback(
		(title?: string, path?: string, options?: {background?: boolean}) => {
			const tabPath = path ?? defaultNewTabPath;
			const [pathname, search = ''] = tabPath.split('?');
			const derivedTitle =
				title ||
				deriveTitleFromPath(pathname, search ? `?${search}` : '') ||
				'WingDrive';

			const newTab: Tab = {
				id: crypto.randomUUID(),
				title: derivedTitle,
				icon: null,
				isPinned: false,
				lastActive: Date.now(),
				savedPath: tabPath
			};

			// Initialize explorer state for the new tab
			setExplorerStates((prev) =>
				new Map(prev).set(newTab.id, {...DEFAULT_EXPLORER_STATE})
			);

			// Initialize empty selection state for the new tab
			setSelectionStates((prev) => new Map(prev).set(newTab.id, []));

			setTabs((prev) => [...prev, newTab]);
			if (!options?.background) setActiveTabId(newTab.id);
		},
		[defaultNewTabPath]
	);

	// Batch close keeps "close others"/"close to the right" atomic. Looping over
	// a single-tab close ran each iteration against a stale snapshot and could
	// leave activeTabId pointing at a removed tab.
	const closeTabs = useCallback(
		(tabIds: string[]) => {
			const doomed = new Set(tabIds);
			const closing = tabs.filter((t) => doomed.has(t.id));
			if (closing.length === 0) return;

			const remaining = tabs.filter((t) => !doomed.has(t.id));
			// The window always keeps at least one tab open; closing the last one
			// would also push it onto the reopen stack and duplicate its id later.
			if (remaining.length === 0) return;

			setClosedTabs((closed) =>
				[
					...closing
						.map((tab) => ({
							tab,
							explorerState: explorerStates.get(tab.id)
						}))
						.reverse(),
					...closed
				].slice(0, 10)
			);

			setTabs(remaining);

			if (doomed.has(activeTabId)) {
				const activeIndex = tabs.findIndex((t) => t.id === activeTabId);
				// Prefer the nearest surviving tab on the left, like browsers do.
				const fallback =
					tabs
						.slice(0, activeIndex)
						.reverse()
						.find((t) => !doomed.has(t.id)) ??
					tabs.slice(activeIndex + 1).find((t) => !doomed.has(t.id));

				if (fallback) setActiveTabId(fallback.id);
			}

			// A closed tab takes its split pane with it.
			const panes = [...doomed].flatMap((id) => (splits[id] ? [splits[id].tab.id] : []));
			if (panes.length > 0)
				setSplits((prev) => {
					const next = {...prev};
					for (const id of doomed) delete next[id];
					return next;
				});

			setExplorerStates((prev) => {
				const next = new Map(prev);
				for (const id of [...doomed, ...panes]) next.delete(id);
				return next;
			});

			setSelectionStates((prev) => {
				const next = new Map(prev);
				for (const id of [...doomed, ...panes]) next.delete(id);
				return next;
			});
		},
		[activeTabId, tabs, explorerStates, splits]
	);

	const closeTab = useCallback(
		(tabId: string) => closeTabs([tabId]),
		[closeTabs]
	);

	const switchTab = useCallback(
		(newTabId: string) => {
			if (newTabId === activeTabId) {
				return;
			}

			setTabs((prev) =>
				prev.map((tab) =>
					tab.id === newTabId ? {...tab, lastActive: Date.now()} : tab
				)
			);

			setActiveTabId(newTabId);
		},
		[activeTabId]
	);

	const updateTabTitle = useCallback((tabId: string, title: string) => {
		setTabs((prev) =>
			prev.map((tab) => (tab.id === tabId ? {...tab, title} : tab))
		);
	}, []);

	const updateTabPath = useCallback((tabId: string, path: string) => {
		setTabs((prev) =>
			prev.map((tab) =>
				tab.id === tabId ? {...tab, savedPath: path} : tab
			)
		);
	}, []);

	const reorderTabs = useCallback((activeId: string, overId: string) => {
		setTabs((prev) => {
			const oldIndex = prev.findIndex((tab) => tab.id === activeId);
			const newIndex = prev.findIndex((tab) => tab.id === overId);

			if (oldIndex === -1 || newIndex === -1 || oldIndex === newIndex) {
				return prev;
			}

			const newTabs = [...prev];
			const [movedTab] = newTabs.splice(oldIndex, 1);
			newTabs.splice(newIndex, 0, movedTab);

			return newTabs;
		});
	}, []);

	const nextTab = useCallback(() => {
		const currentIndex = tabs.findIndex((t) => t.id === activeTabId);
		const nextIndex = (currentIndex + 1) % tabs.length;
		switchTab(tabs[nextIndex].id);
	}, [tabs, activeTabId, switchTab]);

	const previousTab = useCallback(() => {
		const currentIndex = tabs.findIndex((t) => t.id === activeTabId);
		const prevIndex = (currentIndex - 1 + tabs.length) % tabs.length;
		switchTab(tabs[prevIndex].id);
	}, [tabs, activeTabId, switchTab]);

	const selectTabAtIndex = useCallback(
		(index: number) => {
			if (index >= 0 && index < tabs.length) {
				switchTab(tabs[index].id);
			}
		},
		[tabs, switchTab]
	);

	const reopenTab = useCallback(() => {
		const [lastClosed, ...rest] = closedTabs;
		if (!lastClosed) return;

		const {tab, explorerState} = lastClosed;

		// Guard against re-adding a tab that is somehow still open, which would
		// duplicate the React key and the sortable id.
		setTabs((prev) =>
			prev.some((t) => t.id === tab.id) ? prev : [...prev, tab]
		);
		setExplorerStates((prev) =>
			new Map(prev).set(tab.id, restoreTabState({
				...DEFAULT_EXPLORER_STATE,
				...explorerState
			}))
		);
		setSelectionStates((prev) => new Map(prev).set(tab.id, []));
		setClosedTabs(rest);
		setActiveTabId(tab.id);
	}, [closedTabs]);

	// ========================================================================
	// Explorer state (per-tab)
	// ========================================================================

	const getExplorerState = useCallback(
		(tabId: string): TabExplorerState => {
			return explorerStates.get(tabId) ?? {...DEFAULT_EXPLORER_STATE};
		},
		[explorerStates]
	);

	const updateExplorerState = useCallback(
		(tabId: string, updates: Partial<TabExplorerState>) => {
			setExplorerStates((prev) => {
				const current = prev.get(tabId) ?? {
					...DEFAULT_EXPLORER_STATE
				};
				if (Object.entries(updates).every(([key, value]) => Object.is(current[key as keyof TabExplorerState], value))) return prev;
				return new Map(prev).set(tabId, {...current, ...updates});
			});
		},
		[]
	);

	// ========================================================================
	// Selection state (per-tab)
	// ========================================================================

	const getSelectionIds = useCallback(
		(tabId: string): string[] => {
			return selectionStates.get(tabId) ?? [];
		},
		[selectionStates]
	);

	const updateSelectionIds = useCallback(
		(tabId: string, fileIds: string[]) => {
			setSelectionStates((prev) => {
				const current = prev.get(tabId) ?? [];
				if (current.length === fileIds.length && current.every((id, index) => id === fileIds[index])) return prev;
				return new Map(prev).set(tabId, fileIds);
			});
		},
		[]
	);

	// ========================================================================
	// Split view
	// ========================================================================

	const toggleSplit = useCallback(() => {
		const owner = tabs.find((tab) => tab.id === activeTabId);
		if (!owner) return;
		const existing = splits[owner.id];
		if (existing) {
			const open = !existing.open;
			setSplits((prev) => ({...prev, [owner.id]: {...existing, open}}));
			setFocusedPane(open ? 'secondary' : 'primary');
			return;
		}
		// A new pane starts in the same folder and view, as in Dolphin.
		const tab: Tab = {
			id: crypto.randomUUID(),
			title: owner.title,
			icon: null,
			isPinned: false,
			lastActive: Date.now(),
			savedPath: owner.savedPath,
			history: [owner.savedPath],
			historyIndex: 0
		};
		setExplorerStates((prev) =>
			new Map(prev).set(tab.id, {
				...(prev.get(owner.id) ?? DEFAULT_EXPLORER_STATE),
				scrollTop: 0,
				scrollLeft: 0,
				listExpanded: []
			})
		);
		setSelectionStates((prev) => new Map(prev).set(tab.id, []));
		setSplits((prev) => ({...prev, [owner.id]: {tab, open: true}}));
		setFocusedPane('secondary');
	}, [tabs, activeTabId, splits]);

	const panePath = useCallback(
		(pane: PaneId): string | null => {
			if (pane === 'secondary') return splitTab?.savedPath ?? null;
			return tabs.find((tab) => tab.id === activeTabId)?.savedPath ?? null;
		},
		[splitTab, tabs, activeTabId]
	);

	const navigatePane = useCallback(
		(pane: PaneId, to: string) => {
			const target = pane === 'secondary' && splitRouter ? splitRouter : router;
			void target.navigate(to);
		},
		[router, splitRouter]
	);

	// ========================================================================
	// Context value
	// ========================================================================

	const value = useMemo<TabManagerContextValue>(
		() => ({
			tabs,
			activeTabId,
			router,
			createTab,
			closeTab,
			closeTabs,
			switchTab,
			updateTabTitle,
			updateTabPath,
			reorderTabs,
			nextTab,
			previousTab,
			selectTabAtIndex,
			reopenTab,
			hasClosedTabs: closedTabs.length > 0,
			setDefaultNewTabPath,
			getExplorerState,
			updateExplorerState,
			getSelectionIds,
			updateSelectionIds,
			split,
			toggleSplit,
			focusedPane: effectiveFocusedPane,
			setFocusedPane,
			panePath,
			navigatePane
		}),
		[
			tabs,
			activeTabId,
			router,
			createTab,
			closeTab,
			closeTabs,
			switchTab,
			updateTabTitle,
			updateTabPath,
			reorderTabs,
			nextTab,
			previousTab,
			selectTabAtIndex,
			reopenTab,
			closedTabs,
			setDefaultNewTabPath,
			getExplorerState,
			updateExplorerState,
			getSelectionIds,
			updateSelectionIds,
			split,
			toggleSplit,
			effectiveFocusedPane,
			panePath,
			navigatePane
		]
	);

	return (
		<TabManagerContext.Provider value={value}>
			{children}
		</TabManagerContext.Provider>
	);
}

export {TabManagerContext};
