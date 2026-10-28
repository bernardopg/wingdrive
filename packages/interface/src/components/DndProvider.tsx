import {
	DndContext,
	DragOverlay,
	PointerSensor,
	useSensor,
	useSensors,
	pointerWithin,
} from "@dnd-kit/core";
import type { CollisionDetection, DragMoveEvent } from "@dnd-kit/core";
import { useMemo, useRef, useState } from "react";
import { House, Clock, Heart, Folders } from "@phosphor-icons/react";
import { useQueryClient } from "@tanstack/react-query";
import { toast } from "@wingdrive/primitives";
import { usePlatform } from "../contexts/PlatformContext";
import {
	useLibraryMutation,
	useNormalizedQuery,
	useWingDriveClient,
} from "../contexts/WingDriveContext";
import { useSidebarStore } from "@wingdrive/ts-client";
import type { Device, File, WingPath } from "@wingdrive/ts-client";
import { useSpaces } from "./SpacesSidebar/hooks/useSpaces";
import { useFileOperationDialog } from "./modals/FileOperationModal";
import { File as FileComponent } from "../routes/explorer/File";
import { selectionCapabilities } from "../routes/explorer/fileCapabilities";
import {
	externalDragPaths,
	pointerLeftViewport,
} from "../routes/explorer/hooks/externalDragPaths";
import { useTabManager } from "./TabManager/useTabManager";
import { useTranslation } from "../i18n";
import {
	dropOperation,
	modifierKeysNow,
	useModifierKeyTracker,
} from "../hooks/useModifierKeys";

/**
 * DndProvider - Global drag-and-drop coordinator
 *
 * Handles all drag-and-drop operations in the Explorer using @dnd-kit/core.
 *
 * Drop Actions:
 *
 * 1. insert-before / insert-after
 *    - Pins a file to the sidebar before/after an existing item
 *    - Shows a blue line indicator
 *    - Data: { action, itemId }
 *
 * 2. move-into
 *    - Moves a file into a location/volume/folder
 *    - Shows a blue ring around the target
 *    - Data: { action, targetType, targetId, targetPath? }
 *    - targetType: "location" | "volume" | "folder"
 *    - targetPath: WingPath (for locations, directly usable)
 *    - Ctrl forces copy and Shift forces move; dnd-kit carries no modifier
 *      state, so the window tracker answers
 *
 * 3. type: "space" | "group"
 *    - Legacy: Drops on the space root or group area (no specific item)
 *    - Adds item to space/group
 *    - Data: { type, spaceId, groupId? }
 */
// Module-level so the sensor keeps its identity across renders. dnd-kit
// memoizes sensors on the options object: a new literal on every render gave
// every draggable and droppable (each file and sidebar item) a new context
// value, re-rendering all of them whenever this provider rendered.
const POINTER_SENSOR_OPTIONS = {
	activationConstraint: {
		distance: 8, // Require 8px movement before activating drag
	},
};

/**
 * Keeps the horizontal scroll of the dragged item's containers where it was.
 * dnd-kit auto-scrolls on both axes, and heading for the window edge to drag
 * files out to another app slid wide list views sideways, hiding the Name
 * column. Vertical auto-scroll still works. Returns the release function.
 */
function holdScrollLeft(target: EventTarget | null): () => void {
	const held = new Map<Element, number>();
	for (let el = target instanceof Element ? target : null; el; el = el.parentElement) {
		if (el.scrollWidth > el.clientWidth) held.set(el, el.scrollLeft);
	}
	if (held.size === 0) return () => {};
	const restore = (event: Event) => {
		const left = event.target instanceof Element ? held.get(event.target) : undefined;
		if (left !== undefined && (event.target as Element).scrollLeft !== left) {
			(event.target as Element).scrollLeft = left;
		}
	};
	document.addEventListener("scroll", restore, true);
	// The last auto-scroll step can land after the drag ends; its scroll event
	// fires before the next animation frame, so release only then.
	return () =>
		requestAnimationFrame(() => {
			for (const [el, left] of held) el.scrollLeft = left;
			document.removeEventListener("scroll", restore, true);
		});
}

/** Whether every dragged file already sits directly in the destination folder. */
function alreadyIn(dragData: any, destination: WingPath): boolean {
	if (!("Physical" in destination)) return false;
	const dragged: File[] = dragData.selectedFiles ?? [dragData.file];
	const folder = destination.Physical.path.replace(/\/+$/, "") || "/";
	return dragged.every((file) => {
		const path = file.wing_path;
		if (!("Physical" in path)) return false;
		if (path.Physical.device_slug !== destination.Physical.device_slug) return false;
		const parent = path.Physical.path.slice(0, path.Physical.path.lastIndexOf("/")) || "/";
		return parent === folder;
	});
}

export function DndProvider({ children }: { children: React.ReactNode }) {
	const { t } = useTranslation("common");
	const sensors = useSensors(useSensor(PointerSensor, POINTER_SENSOR_OPTIONS));
	const addItem = useLibraryMutation("spaces.add_item");
	const reorderItems = useLibraryMutation("spaces.reorder_items");
	const reorderGroups = useLibraryMutation("spaces.reorder_groups");
	const openFileOperation = useFileOperationDialog();
	useModifierKeyTracker();
	const [activeItem, setActiveItem] = useState<any>(null);
	const client = useWingDriveClient();
	const queryClient = useQueryClient();
	const { currentSpaceId } = useSidebarStore();
	const { data: spacesData } = useSpaces();
	const spaces = spacesData?.spaces;
	const { reorderTabs } = useTabManager();

	// Custom collision detection: prefer -top zones over -bottom zones to avoid double lines
	const customCollision: CollisionDetection = (args) => {
		const collisions = pointerWithin(args);
		if (!collisions || collisions.length === 0) return collisions;

		// If we have multiple collisions, prefer -top over -bottom
		const hasTop = collisions.find((c) => String(c.id).endsWith("-top"));
		const hasMiddle = collisions.find((c) =>
			String(c.id).endsWith("-middle"),
		);

		if (hasMiddle) return [hasMiddle]; // Middle zone takes priority
		if (hasTop) return [hasTop]; // Top zone over bottom
		return [collisions[0]]; // First collision
	};

	const platform = usePlatform();
	const { data: devicesData } = useNormalizedQuery({
		query: "devices.list",
		input: { include_offline: true, include_details: false },
		resourceType: "device",
		enabled: !!platform.startFileDrag,
	});
	const localSlug = useMemo(
		() => (devicesData as Device[] | undefined)?.find((d) => d.is_current)?.slug,
		[devicesData],
	);
	// One handoff per drag: the OS drag owns the pointer once it starts.
	const handedOffRef = useRef(false);
	const releaseScrollRef = useRef<() => void>(() => {});
	const endDrag = () => {
		releaseScrollRef.current();
		releaseScrollRef.current = () => {};
		setActiveItem(null);
	};

	const handleDragStart = (event: any) => {
		handedOffRef.current = false;
		releaseScrollRef.current();
		releaseScrollRef.current =
			event.active.data.current?.type === "explorer-file"
				? holdScrollLeft(event.activatorEvent?.target ?? null)
				: () => {};
		setActiveItem(event.active.data.current);
	};

	// Files dragged past the window edge leave as an OS drag, so other apps
	// (browsers, chats, editors, file managers) can take them.
	const handleDragMove = (event: DragMoveEvent) => {
		if (!platform.startFileDrag || handedOffRef.current) return;
		const data = event.active.data.current;
		if (data?.type !== "explorer-file") return;
		const start = event.activatorEvent;
		if (!(start instanceof PointerEvent || start instanceof MouseEvent)) return;
		const x = start.clientX + event.delta.x;
		const y = start.clientY + event.delta.y;
		if (!pointerLeftViewport(x, y, window.innerWidth, window.innerHeight)) return;
		const paths = externalDragPaths(data.file, data.selectedFiles ?? [], localSlug);
		if (paths.length === 0) return;

		handedOffRef.current = true;
		// End the in-app drag first: under the OS drag grab the webview never
		// sees the pointer go up. dnd-kit cancels on Escape at the document;
		// not bubbling keeps window keybinds from seeing it.
		document.dispatchEvent(
			new KeyboardEvent("keydown", { key: "Escape", code: "Escape", bubbles: false }),
		);
		platform.startFileDrag(paths).catch((error) => {
			toast.error({
				title: t("dnd.dragOutFailed"),
				body: String(error),
			});
		});
	};

	// Opens the copy/move dialog for the dragged files and a destination folder.
	const dropFilesInto = (dragData: any, destination: WingPath) => {
		const dragged: File[] = dragData.selectedFiles ?? [dragData.file];
		// A virtual entry stands for a whole location, volume or device root;
		// dragging it along with real files must not move that root.
		if (!selectionCapabilities(dragged).canCopy) {
			toast.error(t("dnd.virtualLocationsCannotMove"));
			return;
		}
		const sources: WingPath[] = dragged.map((f) => f.wing_path);

		// Native file managers move within a volume and copy across volumes;
		// copying by default silently duplicated files during reorganization.
		// "Same volume" is approximated by matching device slugs on physical
		// paths; Ctrl or Shift overrides the guess, and the operation can
		// still be switched in the dialog.
		const sourceDevice =
			"Physical" in dragData.sdPath
				? dragData.sdPath.Physical.device_slug
				: null;
		const targetDevice =
			"Physical" in destination ? destination.Physical.device_slug : null;
		const sameVolume = sourceDevice !== null && sourceDevice === targetDevice;
		const operation = dropOperation(
			sameVolume ? "move" : "copy",
			modifierKeysNow(),
		);

		openFileOperation({
			operation,
			sources,
			destination,
		});
	};

	const handleDragEnd = async (event: any) => {
		const { active, over } = event;

		endDrag();

		if (!over) return;
		// Switching tabs unmounts the source, so retain its data from drag start.
		const dragData = activeItem ?? active.data.current;

		// Files dropped on another tab move into the folder that tab shows.
		if (
			dragData?.type === "explorer-file" &&
			over.data.current?.type === "tab"
		) {
			const destination = over.data.current.targetPath as WingPath | null;
			// The hover switch can make the target the source's own tab; files
			// already in that folder have nowhere to go.
			if (destination && !alreadyIn(dragData, destination)) {
				dropFilesInto(dragData, destination);
			}
			return;
		}

		// Handle tab reordering
		if (
			active.id !== over.id &&
			dragData?.type === "tab" &&
			over.data.current?.type === "tab"
		) {
			reorderTabs(active.id, over.id);
			return;
		}

		// Handle sortable reordering (no drag data, just active/over IDs)
		if (active.id !== over.id && !dragData?.type) {
			const libraryId = client.getCurrentLibraryId();
			const currentSpace =
				spaces?.find((s: any) => s.id === currentSpaceId) ??
				spaces?.[0];

			if (!currentSpace || !libraryId) return;

			const queryKey = [
				"query:spaces.get_layout",
				libraryId,
				{ space_id: currentSpace.id },
			];
			const layout = queryClient.getQueryData(queryKey) as any;

			if (!layout) return;

			// Check if we're reordering groups
			const groups = layout.groups?.map((g: any) => g.group) || [];
			const isGroupReorder = groups.some((g: any) => g.id === active.id);

			if (isGroupReorder) {
				console.log("[DnD] Reordering groups");

				const oldIndex = groups.findIndex(
					(g: any) => g.id === active.id,
				);
				const newIndex = groups.findIndex((g: any) => g.id === over.id);

				if (
					oldIndex !== -1 &&
					newIndex !== -1 &&
					oldIndex !== newIndex
				) {
					// Optimistically update the UI
					const newGroups = [...layout.groups];
					const [movedGroup] = newGroups.splice(oldIndex, 1);
					newGroups.splice(newIndex, 0, movedGroup);

					queryClient.setQueryData(queryKey, {
						...layout,
						groups: newGroups,
					});

					// Send reorder mutation
					try {
						await reorderGroups.mutateAsync({
							space_id: currentSpace.id,
							group_ids: newGroups.map((g: any) => g.group.id),
						});
						console.log("[DnD] Group reorder successful");
					} catch (err) {
						console.error("[DnD] Group reorder failed:", err);
						// Revert on error
						queryClient.setQueryData(queryKey, layout);
					}
				}

				return;
			}

			// Reordering space items
			if (layout?.space_items) {
				const items = layout.space_items;
				const oldIndex = items.findIndex(
					(item: any) => item.id === active.id,
				);

				// Extract item ID from over.id (could be a drop zone ID like "space-item-{id}-top")
				let overItemId = String(over.id);
				if (overItemId.startsWith("space-item-")) {
					// Extract the UUID from "space-item-{uuid}-top/bottom/middle"
					const parts = overItemId.split("-");
					// Remove "space" and "item" and the last part (top/bottom/middle)
					overItemId = parts.slice(2, -1).join("-");
				}

				const newIndex = items.findIndex(
					(item: any) => item.id === overItemId,
				);

				console.log("[DnD] Reorder space items:", {
					oldIndex,
					newIndex,
					activeId: active.id,
					extractedOverId: overItemId,
				});

				if (
					oldIndex !== -1 &&
					newIndex !== -1 &&
					oldIndex !== newIndex
				) {
					// Optimistically update the UI
					const newItems = [...items];
					const [movedItem] = newItems.splice(oldIndex, 1);
					newItems.splice(newIndex, 0, movedItem);

					queryClient.setQueryData(queryKey, {
						...layout,
						space_items: newItems,
					});

					// Send reorder mutation
					try {
						await reorderItems.mutateAsync({
							group_id: null, // Space-level items
							item_ids: newItems.map((item: any) => item.id),
						});
						console.log("[DnD] Space items reorder successful");
					} catch (err) {
						console.error("[DnD] Space items reorder failed:", err);
						// Revert on error
						queryClient.setQueryData(queryKey, layout);
					}
				}
			}

			return;
		}

		if (!active.data.current) return;

		const dropData = over.data.current;

		// Handle palette item drops (from customization panel)
		if (dragData?.type === "palette-item") {
			const libraryId = client.getCurrentLibraryId();
			const currentSpace =
				spaces?.find((s: any) => s.id === currentSpaceId) ??
				spaces?.[0];

			if (!currentSpace || !libraryId) return;

			try {
				await addItem.mutateAsync({
					space_id: currentSpace.id,
					group_id: dropData?.groupId || null,
					item_type: dragData.itemType,
				});
				console.log("[DnD] Successfully added palette item");
			} catch (err) {
				console.error("[DnD] Failed to add palette item:", err);
			}
			return;
		}

		if (!dragData || dragData.type !== "explorer-file") return;

		// Add to space (root-level drop zones between groups)
		if (dropData?.action === "add-to-space") {
			if (!dropData.spaceId) return;

			try {
				await addItem.mutateAsync({
					space_id: dropData.spaceId,
					group_id: null,
					item_type: { Path: { wing_path: dragData.sdPath } },
				});
				console.log("[DnD] Successfully added to space root");
			} catch (err) {
				console.error("[DnD] Failed to add to space:", err);
			}
			return;
		}

		// Add to group (empty group drop zone)
		if (dropData?.action === "add-to-group") {
			if (!dropData.spaceId || !dropData.groupId) return;

			console.log("[DnD] Adding to group:", {
				spaceId: dropData.spaceId,
				groupId: dropData.groupId,
				sdPath: dragData.sdPath,
			});

			try {
				await addItem.mutateAsync({
					space_id: dropData.spaceId,
					group_id: dropData.groupId,
					item_type: { Path: { wing_path: dragData.sdPath } },
				});
				console.log("[DnD] Successfully added to group");
			} catch (err) {
				console.error("[DnD] Failed to add to group:", err);
			}
			return;
		}

		// Insert before/after sidebar items (adds item to space/group)
		if (
			dropData?.action === "insert-before" ||
			dropData?.action === "insert-after"
		) {
			if (!dropData.spaceId) return;

			console.log("[DnD] Inserting item:", {
				action: dropData.action,
				spaceId: dropData.spaceId,
				groupId: dropData.groupId,
				sdPath: dragData.sdPath,
			});

			try {
				await addItem.mutateAsync({
					space_id: dropData.spaceId,
					group_id: dropData.groupId || null,
					item_type: { Path: { wing_path: dragData.sdPath } },
				});
				console.log("[DnD] Successfully inserted item");
				// TODO: Implement proper ordering relative to itemId
			} catch (err) {
				console.error("[DnD] Failed to add item:", err);
			}
			return;
		}

		// Move file into location/volume/folder
		if (dropData?.action === "move-into") {
			const destination: WingPath = dropData.targetPath;
			if (!destination) {
				console.error("[DnD] No target path for move-into action");
				return;
			}
			dropFilesInto(dragData, destination);
			return;
		}

		// Drop on space root area (adds to space)
		if (dropData?.type === "space" && dragData.type === "explorer-file") {
			console.log("[DnD] Adding to space (type=space):", {
				spaceId: dropData.spaceId,
				sdPath: dragData.sdPath,
			});

			try {
				await addItem.mutateAsync({
					space_id: dropData.spaceId,
					group_id: null,
					item_type: { Path: { wing_path: dragData.sdPath } },
				});
				console.log("[DnD] Successfully added to space");
			} catch (err) {
				console.error("[DnD] Failed to add item:", err);
			}
		}

		// Drop on group area (adds to group)
		if (dropData?.type === "group" && dragData.type === "explorer-file") {
			console.log("[DnD] Adding to group (type=group):", {
				spaceId: dropData.spaceId,
				groupId: dropData.groupId,
				sdPath: dragData.sdPath,
			});

			try {
				await addItem.mutateAsync({
					space_id: dropData.spaceId,
					group_id: dropData.groupId,
					item_type: { Path: { wing_path: dragData.sdPath } },
				});
				console.log("[DnD] Successfully added to group");
			} catch (err) {
				console.error("[DnD] Failed to add item to group:", err);
			}
		}
	};

	return (
		<DndContext
			sensors={sensors}
			collisionDetection={customCollision}
			onDragStart={handleDragStart}
			onDragMove={handleDragMove}
			onDragEnd={handleDragEnd}
			onDragCancel={endDrag}
		>
			{children}
			<DragOverlay dropAnimation={null}>
				{activeItem?.type === "palette-item" ? (
					// Palette item preview
					<div className="flex items-center gap-2 px-3 py-2 rounded-lg bg-accent text-white shadow-lg min-w-[180px]">
						{activeItem.itemType === "Overview" && (
							<House size={20} weight="bold" />
						)}
						{activeItem.itemType === "Recents" && (
							<Clock size={20} weight="bold" />
						)}
						{activeItem.itemType === "Favorites" && (
							<Heart size={20} weight="bold" />
						)}
						{activeItem.itemType === "FileKinds" && (
							<Folders size={20} weight="bold" />
						)}
						<span className="text-sm font-medium">
							{activeItem.itemType === "Overview" && t("dnd.itemTypes.overview")}
							{activeItem.itemType === "Recents" && t("dnd.itemTypes.recents")}
							{activeItem.itemType === "Favorites" && t("dnd.itemTypes.favorites")}
							{activeItem.itemType === "FileKinds" &&
								t("dnd.itemTypes.fileKinds")}
						</span>
					</div>
				) : activeItem?.label ? (
					// Group or SpaceItem preview (from sortable context)
					<div className="flex items-center gap-2 px-3 py-2 rounded-lg bg-sidebar/95 backdrop-blur-sm text-sidebar-ink shadow-lg border border-sidebar-line min-w-[180px]">
						<span className="text-sm font-medium">
							{activeItem.label}
						</span>
					</div>
				) : activeItem?.file ? (
					activeItem.gridSize ? (
						// Grid view preview
						<div style={{ width: activeItem.gridSize }}>
							<div className="flex flex-col items-center gap-2 p-1 rounded-lg relative">
								<div className="rounded-lg p-2">
									<FileComponent.Thumb
										file={activeItem.file}
										size={Math.max(
											activeItem.gridSize * 0.6,
											60,
										)}
									/>
								</div>
								<div className="text-sm truncate px-2 py-0.5 rounded-md bg-accent text-white max-w-full">
									{activeItem.name}
								</div>
								{/* Show count badge if dragging multiple files */}
								{activeItem.selectedFiles &&
									activeItem.selectedFiles.length > 1 && (
										<div className="absolute -top-2 -right-2 size-6 rounded-full bg-accent text-white text-xs font-bold flex items-center justify-center shadow-lg border-2 border-app">
											{activeItem.selectedFiles.length}
										</div>
									)}
							</div>
						</div>
					) : (
						// Column/List view preview
						<div className="flex items-center gap-2 px-3 py-1.5 rounded-md bg-accent text-white shadow-lg min-w-[200px] max-w-[300px]">
							<FileComponent.Thumb
								file={activeItem.file}
								size={24}
							/>
							<span className="text-sm font-medium truncate">
								{activeItem.name}
							</span>
							{/* Show count badge if dragging multiple files */}
							{activeItem.selectedFiles &&
								activeItem.selectedFiles.length > 1 && (
									<div className="ml-auto size-5 rounded-full bg-white text-accent text-xs font-bold flex items-center justify-center">
										{activeItem.selectedFiles.length}
									</div>
								)}
						</div>
					)
				) : null}
			</DragOverlay>
		</DndContext>
	);
}
