import type {File} from '@wingdrive/ts-client';
import {
	createContext,
	useCallback,
	useContext,
	useRef,
	type ReactNode
} from 'react';
import {useOpenFile} from './hooks/useOpenFile';

type OpenFile = (file: File) => Promise<void>;

const OpenFileContext = createContext<OpenFile | null>(null);

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

	return (
		<OpenFileContext.Provider value={open}>{children}</OpenFileContext.Provider>
	);
}

export function useOpenFileAction(): OpenFile {
	const open = useContext(OpenFileContext);
	if (!open) {
		throw new Error('useOpenFileAction must be used within an OpenFileProvider');
	}
	return open;
}
