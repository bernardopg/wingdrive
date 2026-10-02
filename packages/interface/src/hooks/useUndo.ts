import { toast } from "@wingdrive/primitives";
import type {WingPath} from '@wingdrive/ts-client';
import { useWingDriveClient, useLibraryQuery } from "../contexts/WingDriveContext";
import { useRefetchFileListings } from "./useRefetchFileListings";
import { useUndoStore } from "./undoStore";

export function useUndo() {
	const client = useWingDriveClient();
	const store = useUndoStore();
	const refetch = useRefetchFileListings();
	const libraryId = client.getCurrentLibraryId();
	const {data: devices} = useLibraryQuery({type: 'devices.list', input: {include_offline: true, include_details: false}});
	const localSlug = devices?.find(device => device.is_current)?.slug;
	return {
		isLocalPath: (path: WingPath) => 'Physical' in path && (path.Physical.device_slug === 'local' || (localSlug !== undefined && path.Physical.device_slug === localSlug)),
		canUndo: !store.busy && store.entries.some((entry) => entry.libraryId === libraryId),
		record: (label: string, run: () => Promise<void>) => {
			if (libraryId) store.record({libraryId, label, run});
		},
		undo: async () => {
			if (!libraryId) return;
			try {
				const label = await store.undo(libraryId);
				if (label) { refetch(); toast.success(`Undid ${label}`); }
			} catch (error) { toast.error(`Could not undo: ${error}`); }
		},
	};
}
