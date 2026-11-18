import { create } from "zustand";
import type { WingPath } from "@wingdrive/ts-client";
import { pasteBlocker } from "../screens/explorer/fileLogic";

export type ClipboardMode = "copy" | "move";

export interface ClipboardItem {
	path: WingPath;
	/** Physical path on this phone, to check where it may be pasted */
	localPath: string;
	name: string;
	isDirectory: boolean;
}

interface FileClipboardStore {
	pendingPaths: WingPath[] | null;
	mode: ClipboardMode | null;
	items: ClipboardItem[];
	/** Holds files to paste in the folder the user opens next (MOB-007) */
	set: (mode: ClipboardMode, items: ClipboardItem[]) => void;
	clear: () => void;
	beginPaste: (destination: string) => { mode: ClipboardMode; paths: WingPath[] } | null;
	finishPaste: (paths: WingPath[], accepted: boolean) => void;
}

export const useFileClipboard = create<FileClipboardStore>((set, get) => ({
	pendingPaths: null,
	mode: null,
	items: [],
	set: (mode, items) => set({ mode, items, pendingPaths: null }),
	clear: () => set({ mode: null, items: [], pendingPaths: null }),
	beginPaste: (destination) => {
		const { mode, items, pendingPaths } = get();
		if (!mode || !items.length || pendingPaths || pasteBlocker(mode, items, destination)) {
			return null;
		}
		const paths = items.map((item) => item.path);
		set({ pendingPaths: paths });
		return { mode, paths };
	},
	finishPaste: (paths, accepted) => {
		if (get().pendingPaths !== paths) return;
		if (accepted) get().clear();
		else set({ pendingPaths: null });
	},
}));
