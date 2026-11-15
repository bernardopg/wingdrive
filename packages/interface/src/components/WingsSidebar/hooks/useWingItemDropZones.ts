import { useDroppable, useDndContext } from "@dnd-kit/core";
import type { WingItem as WingItemType, WingPath } from "@wingdrive/ts-client";
import {
	isDropTargetItem,
	getDropTargetType,
	buildDropTargetPath,
	type DropTargetType,
} from "./wingItemUtils";

interface UseWingItemDropZonesOptions {
	item: WingItemType;
	allowInsertion: boolean;
	wingId?: string;
	groupId?: string | null;
	volumeData?: { device_slug: string; mount_path: string };
}

interface DropZoneRefs {
	setTopRef: (node: HTMLElement | null) => void;
	setBottomRef: (node: HTMLElement | null) => void;
	setMiddleRef: (node: HTMLElement | null) => void;
}

interface DropZoneState {
	isOverTop: boolean;
	isOverBottom: boolean;
	isOverMiddle: boolean;
	isDropTarget: boolean;
	targetType: DropTargetType;
	targetPath: WingPath | undefined;
	isDraggingSortableItem: boolean;
}

export type UseWingItemDropZonesResult = DropZoneRefs & DropZoneState;

/**
 * Manages drop zones for a wing item.
 *
 * WingItems support two types of drop interactions:
 * 1. Insertion (reordering): Blue line above/below for sidebar item reordering
 * 2. Move-into (file operations): Blue ring for moving files into location/folder
 */
export function useWingItemDropZones({
	item,
	allowInsertion,
	wingId,
	groupId,
	volumeData,
}: UseWingItemDropZonesOptions): UseWingItemDropZonesResult {
	const { active } = useDndContext();

	// Disable insertion zones when dragging groups or wing items (they have 'label' in data)
	const isDraggingSortableItem = active?.data?.current?.label != null;

	// Determine if this item can receive file drops
	const isDropTarget = isDropTargetItem(item);
	const targetType = getDropTargetType(item);
	const targetPath = buildDropTargetPath(item, volumeData);

	// Top zone: insertion above
	const { setNodeRef: setTopRef, isOver: isOverTop } = useDroppable({
		id: `wing-item-${item.id}-top`,
		disabled: !allowInsertion || isDraggingSortableItem,
		data: {
			action: "insert-before",
			itemId: item.id,
			wingId,
			groupId,
		},
	});

	// Bottom zone: insertion below
	const { setNodeRef: setBottomRef, isOver: isOverBottom } = useDroppable({
		id: `wing-item-${item.id}-bottom`,
		disabled: !allowInsertion || isDraggingSortableItem,
		data: {
			action: "insert-after",
			itemId: item.id,
			wingId,
			groupId,
		},
	});

	// Middle zone: drop into folder/location
	const { setNodeRef: setMiddleRef, isOver: isOverMiddle } = useDroppable({
		id: `wing-item-${item.id}-middle`,
		disabled: !isDropTarget || isDraggingSortableItem,
		data: {
			action: "move-into",
			targetType,
			targetId: item.id,
			targetPath,
		},
	});

	return {
		setTopRef,
		setBottomRef,
		setMiddleRef,
		isOverTop,
		isOverBottom,
		isOverMiddle,
		isDropTarget,
		targetType,
		targetPath,
		isDraggingSortableItem,
	};
}
