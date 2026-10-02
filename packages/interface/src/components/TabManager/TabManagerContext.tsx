import type {SortingState} from '@tanstack/react-table';
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
import {deriveTitleFromPath} from './deriveTitle';
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
	// View settings
	viewMode: ViewMode;
	sortBy: SortBy;
	/** Explicit direction; null uses the natural default for `sortBy`. */
	sortDirection: SortDirection | null;
	gridSize: number;
	gapSize: number;
	foldersFirst: boolean;
	showHiddenFiles: boolean;

	// Column view state (serialized WingPath[] as JSON strings)
	columnStack: string[];

	// Scroll position
	scrollTop: number;
	scrollLeft: number;

	// Size view transform (zoom + pan)
	sizeViewTransform: {k: number; x: number; y: number};
}

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
function storageKeyFor(windowLabel?: string): string {
	return !windowLabel || windowLabel === 'main'
		? STORAGE_KEY
		: `${STORAGE_KEY}:${windowLabel}`;
}

interface PersistedState {
	tabs: Tab[];
	activeTabId: string;
	explorerStates: Record<string, TabExplorerState>;
	defaultNewTabPath: string;
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
	createTab: (title?: string, path?: string) => void;
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
	const routers = useRef(new Map<string, Router>());
	const router = useMemo(() => {
		const cached = routers.current.get(activeTabId);
		if (cached) return cached;
		const tab = tabs.find((tab) => tab.id === activeTabId) ?? tabs[0];
		const created = createTabRouter(
			routes,
			tab.savedPath,
			tab.history,
			tab.historyIndex,
			(savedPath, history, historyIndex) =>
				setTabs((prev) =>
					prev.map((entry) =>
						entry.id === tab.id
							? {...entry, savedPath, history, historyIndex}
							: entry
					)
				)
		);
		routers.current.set(tab.id, created);
		return created;
	}, [activeTabId, routes, tabs]);
	useEffect(() => {
		// ponytail: routers use O(open tabs) memory; evict cold routers if very large sessions need a cap.
		const open = new Set(tabs.map((tab) => tab.id));
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
					{...DEFAULT_EXPLORER_STATE, ...state}
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
			defaultNewTabPath
		});
	}, [storageKey, tabs, activeTabId, explorerStates, defaultNewTabPath]);

	// ========================================================================
	// Tab management
	// ========================================================================

	const setDefaultNewTabPath = useCallback((path: string) => {
		setDefaultNewTabPathState(path);
	}, []);

	const createTab = useCallback(
		(title?: string, path?: string) => {
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
			setActiveTabId(newTab.id);
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

			setExplorerStates((prev) => {
				const next = new Map(prev);
				for (const id of doomed) next.delete(id);
				return next;
			});

			setSelectionStates((prev) => {
				const next = new Map(prev);
				for (const id of doomed) next.delete(id);
				return next;
			});
		},
		[activeTabId, tabs, explorerStates]
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
			new Map(prev).set(tab.id, {
				...DEFAULT_EXPLORER_STATE,
				...explorerState
			})
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
			updateSelectionIds
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
			updateSelectionIds
		]
	);

	return (
		<TabManagerContext.Provider value={value}>
			{children}
		</TabManagerContext.Provider>
	);
}

export {TabManagerContext};
