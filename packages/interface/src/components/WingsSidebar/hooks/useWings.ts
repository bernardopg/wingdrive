import { useNormalizedQuery } from '@wingdrive/ts-client';
import { useWingDriveClient } from '../../../contexts/WingDriveContext';
import { useQueryClient } from '@tanstack/react-query';
import { useEffect } from 'react';
import type { Event } from '@wingdrive/ts-client';

export function useWings() {
	return useNormalizedQuery({
		query: 'wings.list',
		input: null, // Unit struct serializes as null, not {}
		resourceType: 'wing',
	});
}

export function useWingLayout(wingId: string | null) {
	const client = useWingDriveClient();
	const queryClient = useQueryClient();
	const libraryId = client.getCurrentLibraryId();

	const query = useNormalizedQuery({
		query: 'wings.get_layout',
		input: wingId ? { wing_id: wingId } : null,
		resourceType: 'wing_layout',
		resourceId: wingId || undefined,
		enabled: !!wingId,
	});

	// Subscribe to wing_item deletions to update the layout
	// (wing_item sends its own ResourceDeleted events, separate from wing_layout)
	useEffect(() => {
		if (!wingId || !libraryId) return;

		const handleEvent = (event: Event) => {
			if (typeof event === 'string') return;

			if ('ResourceDeleted' in event) {
				const { resource_type, resource_id } = (event as any).ResourceDeleted;

				if (resource_type === 'wing_item') {
					console.log('[useWingLayout] Wing item deleted, updating layout:', resource_id);

					// Remove the item from the layout cache
					const queryKey = ['query:wings.get_layout', libraryId, { wing_id: wingId }];
					queryClient.setQueryData(queryKey, (oldData: any) => {
						if (!oldData) return oldData;

						// Remove from wing_items array
						const updatedWingItems = oldData.wing_items?.filter(
							(item: any) => item.id !== resource_id
						) || [];

						// Remove from groups
						const updatedGroups = oldData.groups?.map((group: any) => ({
							...group,
							items: group.items.filter((item: any) => item.id !== resource_id),
						})) || [];

						return {
							...oldData,
							wing_items: updatedWingItems,
							groups: updatedGroups,
						};
					});
				}
			}
		};

		let unsubscribe: (() => void) | undefined;

		client.subscribeFiltered(
			{
				resource_type: 'wing_item',
				library_id: libraryId,
				include_descendants: false,
			},
			handleEvent
		).then((unsub) => {
			unsubscribe = unsub;
		});

		return () => {
			unsubscribe?.();
		};
	}, [client, queryClient, wingId, libraryId]);

	return query;
}