import {isVirtualFile, type File} from '@wingdrive/ts-client';
import {
	createContext,
	useCallback,
	useContext,
	useRef,
	type ReactNode
} from 'react';
import {TabManagerContext} from '../../components/TabManager/TabManagerContext';
import {targetToUrl} from './context';
import {useOpenFile} from './hooks/useOpenFile';

type OpenFile = (file: File) => Promise<void>;
/** Opens an app route in a new background tab, as a middle-click does in a browser. */
type OpenInNewTab = (route: string, title?: string) => void;

const OpenFileContext = createContext<OpenFile | null>(null);
const OpenInNewTabContext = createContext<OpenInNewTab | null>(null);

/** The route that browses a folder (or a virtual item such as a volume) in a tab, or null for files. */
export function folderRoute(file: File): string | null {
	if (file.kind !== 'Directory' && !isVirtualFile(file)) return null;
	return targetToUrl({type: 'path', path: file.wing_path});
}

/**
 * Owns the one "open file" handler that grid cards, list rows and media items
 * call on double-click. Each item used to build its own: an "Open With" query
 * observer, a mutation and the Explorer context per item, so every Explorer
 * change re-rendered every visible item.
 */
export function OpenFileProvider({children}: {children: ReactNode}) {
	const openFile = useOpenFile();
	const latest = useRef(openFile);
	latest.current = openFile;
	// Stable, so items memoized on their props never re-render for it.
	const open = useCallback<OpenFile>((file) => latest.current(file), []);

	// The tab manager changes on every navigation; read it through a ref so
	// this action stays stable too.
	const tabs = useContext(TabManagerContext);
	const tabsRef = useRef(tabs);
	tabsRef.current = tabs;
	const openInNewTab = useCallback<OpenInNewTab>((route, title) => {
		tabsRef.current?.createTab(title, route, {background: true});
	}, []);

	return (
		<OpenFileContext.Provider value={open}>
			<OpenInNewTabContext.Provider value={openInNewTab}>
				{children}
			</OpenInNewTabContext.Provider>
		</OpenFileContext.Provider>
	);
}

export function useOpenFileAction(): OpenFile {
	const open = useContext(OpenFileContext);
	if (!open) {
		throw new Error('useOpenFileAction must be used within an OpenFileProvider');
	}
	return open;
}

/**
 * Middle-click handler for a listed item: a folder (or volume, device) opens
 * in a new background tab, as in Dolphin and in browsers.
 */
export function useMiddleClickOpensTab(file: File) {
	const openInNewTab = useOpenInNewTabAction();
	return (e: React.MouseEvent) => {
		if (e.button !== 1) return;
		const route = folderRoute(file);
		if (!route) return;
		e.preventDefault();
		e.stopPropagation();
		openInNewTab(route);
	};
}

/** Opens a route in a new background tab; a no-op outside the tab manager. */
export function useOpenInNewTabAction(): OpenInNewTab {
	return useContext(OpenInNewTabContext) ?? noop;
}

function noop() {}
