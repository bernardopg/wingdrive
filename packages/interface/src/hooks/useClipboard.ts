import { create } from "zustand";
import type { WingPath } from "@wingdrive/ts-client";

export interface ClipboardState {
	operation: "copy" | "cut" | null;
	files: WingPath[];
	sourcePath: WingPath | null;
}

interface ClipboardStore extends ClipboardState {
	setClipboard: (
		operation: "copy" | "cut",
		files: WingPath[],
		sourcePath: WingPath | null,
	) => void;
	clearClipboard: () => void;
	hasClipboard: () => boolean;
}

export const useClipboardStore = create<ClipboardStore>((set, get) => ({
	operation: null,
	files: [],
	sourcePath: null,

	setClipboard: (operation, files, sourcePath) => {
		set({ operation, files, sourcePath });
	},

	clearClipboard: () => {
		set({ operation: null, files: [], sourcePath: null });
	},

	hasClipboard: () => {
		const state = get();
		return state.operation !== null && state.files.length > 0;
	},
}));

/**
 * Hook to access clipboard state and operations
 */
export function useClipboard() {
	const store = useClipboardStore();

	return {
		operation: store.operation,
		files: store.files,
		sourcePath: store.sourcePath,
		setClipboard: store.setClipboard,
		clearClipboard: store.clearClipboard,
		hasClipboard: store.hasClipboard,

		// Helper to copy files
		copyFiles: (files: WingPath[], sourcePath: WingPath | null = null) => {
			store.setClipboard("copy", files, sourcePath);
		},

		// Helper to cut files
		cutFiles: (files: WingPath[], sourcePath: WingPath | null = null) => {
			store.setClipboard("cut", files, sourcePath);
		},
	};
}
