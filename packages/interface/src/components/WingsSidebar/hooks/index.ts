// Wing item utilities
export {
	isOverviewItem,
	isRecentsItem,
	isFavoritesItem,
	isFileKindsItem,
	isLocationItem,
	isVolumeItem,
	isTagItem,
	isPathItem,
	isRawLocation,
	isDropTargetItem,
	getDropTargetType,
	buildDropTargetPath,
	resolveItemMetadata,
	type IconData,
	type ItemMetadata,
	type ResolveMetadataOptions,
	type DropTargetType,
} from "./wingItemUtils";

// Wing item hooks
export { useWingItemActive } from "./useWingItemActive";
export { useWingItemDropZones, type UseWingItemDropZonesResult } from "./useWingItemDropZones";
export { useWingItemContextMenu } from "./useWingItemContextMenu";

// Wing data hooks
export { useWings, useWingLayout } from "./useWings";

