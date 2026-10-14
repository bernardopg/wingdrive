import type {File} from '@wingdrive/ts-client';
import {
	createContext,
	useCallback,
	useContext,
	useEffect,
	useRef,
	useState,
	type ReactNode
} from 'react';
import {useFileContextMenu} from './hooks/useFileContextMenu';
import {useSelection} from './SelectionContext';

interface MenuRequest {
	id: number;
	file: File;
	selected: boolean;
	x: number;
	y: number;
}

type ShowFileContextMenu = (
	e: Pick<MouseEvent, 'preventDefault' | 'stopPropagation' | 'clientX' | 'clientY'>,
	file: File,
	selected: boolean
) => void;

const FileContextMenuContext = createContext<ShowFileContextMenu | null>(null);

/**
 * Builds the file context menu once, on demand. Each item used to mount its
 * own menu (a dozen mutations plus an "Open With" query per file), which made
 * every folder change subscribe and tear down hundreds of query observers.
 */
export function FileContextMenuProvider({children}: {children: ReactNode}) {
	const [request, setRequest] = useState<MenuRequest | null>(null);

	const show = useCallback<ShowFileContextMenu>((e, file, selected) => {
		e.preventDefault();
		e.stopPropagation();
		setRequest((prev) => ({
			id: (prev?.id ?? 0) + 1,
			file,
			selected,
			x: e.clientX,
			y: e.clientY
		}));
	}, []);

	return (
		<FileContextMenuContext.Provider value={show}>
			{children}
			{/* Stays mounted after showing so menu actions keep a live owner. */}
			{request && <RequestedFileContextMenu request={request} />}
		</FileContextMenuContext.Provider>
	);
}

function RequestedFileContextMenu({request}: {request: MenuRequest}) {
	const {selectedFiles} = useSelection();
	const menu = useFileContextMenu({
		file: request.file,
		selectedFiles,
		selected: request.selected
	});
	const shownId = useRef(0);

	useEffect(() => {
		// Wait for "Open With" apps so the first menu is complete.
		if (menu.appsLoading || shownId.current === request.id) return;
		shownId.current = request.id;
		void menu.show({
			clientX: request.x,
			clientY: request.y,
			preventDefault() {},
			stopPropagation() {}
		});
	}, [menu, request]);

	return null;
}

export function useShowFileContextMenu(): ShowFileContextMenu {
	const show = useContext(FileContextMenuContext);
	if (!show) {
		throw new Error(
			'useShowFileContextMenu must be used within a FileContextMenuProvider'
		);
	}
	return show;
}
