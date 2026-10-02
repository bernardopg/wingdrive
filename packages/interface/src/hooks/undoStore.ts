import { create } from "zustand";

type UndoEntry = { id: string; libraryId: string; label: string; run: () => Promise<void> };

// ponytail: 20 session-only entries; persist a daemon journal if restart recovery is required.
export const useUndoStore = create<{
	entries: UndoEntry[];
	busy: boolean;
	record: (entry: Omit<UndoEntry, "id">) => void;
	undo: (libraryId: string) => Promise<string | null>;
}>((set, get) => ({
	entries: [],
	busy: false,
	record: (entry) => set((state) => ({entries: [...state.entries, {...entry, id: crypto.randomUUID()}].slice(-20)})),
	undo: async (libraryId) => {
		if (get().busy) return null;
		const entry = get().entries.slice().reverse().find((entry) => entry.libraryId === libraryId);
		if (!entry) return null;
		set({busy: true});
		try {
			await entry.run();
			set((state) => ({entries: state.entries.filter((item) => item.id !== entry.id)}));
			return entry.label;
		} finally { set({busy: false}); }
	},
}));
