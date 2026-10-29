import {useQueryClient} from '@tanstack/react-query';
import type {WingPath} from '@wingdrive/ts-client';
import {useEffect} from 'react';

import {useWingDriveClient} from '../../../../contexts/WingDriveContext';

/** Gathers a burst of changes into one refetch, as the shown folder does. */
const COALESCE_MS = 300;

/**
 * Refetches the listings and Git state of folders expanded in list view when
 * files change inside them.
 *
 * The shown folder's listing follows change events through
 * `useNormalizedQuery`, scoped to that folder alone. Expanded folders are
 * fetched with `useLibraryQueries`, which subscribes to nothing, so their rows
 * went stale until the next navigation. One subscription over the shown
 * folder's descendants covers every expanded folder at once.
 */
export function useExpandedFolderEvents(shownFolder: WingPath | null, active: boolean) {
	const client = useWingDriveClient();
	const queryClient = useQueryClient();
	const libraryId = client.getCurrentLibraryId();
	const scope = active && shownFolder ? JSON.stringify(shownFolder) : null;

	useEffect(() => {
		if (!scope || !libraryId) return;
		let cancelled = false;
		let unsubscribe: (() => void) | undefined;
		let timer: ReturnType<typeof setTimeout> | undefined;

		const refetch = () => {
			if (timer) return;
			timer = setTimeout(() => {
				timer = undefined;
				for (const type of ['files.directory_listing', 'files.vcs_status']) {
					void queryClient.invalidateQueries(
						{queryKey: [type, libraryId], exact: false},
						{cancelRefetch: false}
					);
				}
			}, COALESCE_MS);
		};

		client
			.subscribeFiltered(
				{
					resource_type: 'file',
					path_scope: JSON.parse(scope) as WingPath,
					library_id: libraryId,
					include_descendants: true
				},
				refetch
			)
			.then((unsub) => {
				if (cancelled) unsub();
				else unsubscribe = unsub;
			})
			.catch(() => {});

		return () => {
			cancelled = true;
			if (timer) clearTimeout(timer);
			unsubscribe?.();
		};
	}, [client, queryClient, libraryId, scope]);
}
