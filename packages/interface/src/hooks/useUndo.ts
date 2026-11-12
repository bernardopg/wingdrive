import { toast } from "@wingdrive/primitives";
import type {WingPath} from '@wingdrive/ts-client';
import { useWingDriveClient, useLibraryMutation, useLibraryQuery } from "../contexts/WingDriveContext";
import { useRefetchFileListings } from "./useRefetchFileListings";
import { useUndoStore } from "./undoStore";
import { journaledUndo } from "./journalUndo";
import { useWaitForJob } from "./useWaitForJob";
import { i18n } from "../i18n";

/**
 * Ctrl+Z for the Explorer. Every file operation the core journals (copy, move,
 * rename, trash, new folder, new file, new link) is undone by the core's
 * `files.undo` job, which leaves alone anything that changed since.
 */
export function useUndo() {
	const client = useWingDriveClient();
	const store = useUndoStore();
	const refetch = useRefetchFileListings();
	const undoFiles = useLibraryMutation('files.undo');
	const waitForJob = useWaitForJob();
	const libraryId = client.getCurrentLibraryId();
	const {data: devices} = useLibraryQuery({type: 'devices.list', input: {include_offline: true, include_details: false}});
	const localSlug = devices?.find(device => device.is_current)?.slug;
	const record = (label: string, run: () => Promise<void>) => {
		if (libraryId) store.record({libraryId, label, run});
	};
	return {
		isLocalPath: (path: WingPath) => 'Physical' in path && (path.Physical.device_slug === 'local' || (localSlug !== undefined && path.Physical.device_slug === localSlug)),
		canUndo: !store.busy && store.entries.some((entry) => entry.libraryId === libraryId),
		record,
		/**
		 * Records journaled operations (job ids, or the `undo_id` of a created
		 * entry) as one undo step. Several ids are undone newest first.
		 */
		recordJournaled: (label: string, ids: string | readonly (string | null | undefined)[]) => {
			const list = (typeof ids === 'string' ? [ids] : ids).filter(
				(id): id is string => typeof id === 'string'
			);
			if (list.length === 0) return;
			record(
				label,
				journaledUndo(
					list,
					async (id) =>
						(await waitForJob(() => undoFiles.mutateAsync({job_id: id}))).result,
					{
						skipped: (count, reason) => i18n.t("common:undo.skipped", {count, reason}),
						cancelled: () => i18n.t("common:undo.cancelled"),
					}
				)
			);
		},
		undo: async () => {
			if (!libraryId) return;
			try {
				const label = await store.undo(libraryId);
				if (label) {
					refetch();
					const localizedLabel = i18n.exists(`common:undo.labels.${label}`)
						? i18n.t(`common:undo.labels.${label}`)
						: label;
					toast.success(i18n.t("common:undo.done", { label: localizedLabel }));
				}
			} catch (error) {
				// A partial undo still changed the disk.
				refetch();
				toast.error(i18n.t("common:undo.failed", { error: error instanceof Error ? error.message : String(error) }));
			}
		},
	};
}
