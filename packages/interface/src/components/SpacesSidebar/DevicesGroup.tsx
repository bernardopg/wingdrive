import { WifiHigh, WifiSlashIcon, Trash } from "@phosphor-icons/react";
import { useNormalizedQuery, getDeviceIcon, useCoreMutation } from "../../contexts/WingDriveContext";
import { useExplorer } from "../../routes/explorer/context";
import { SpaceItem } from "./SpaceItem";
import { GroupHeader } from "./GroupHeader";
import type { ListLibraryDevicesInput, Device } from "@wingdrive/ts-client";
import { useTranslation } from "../../i18n";
import { useWebContextMenuController } from "../../contexts/WebContextMenuContext";

interface DevicesGroupProps {
	isCollapsed: boolean;
	onToggle: () => void;
	sortableAttributes?: any;
	sortableListeners?: any;
}

export function DevicesGroup({
	isCollapsed,
	onToggle,
	sortableAttributes,
	sortableListeners,
}: DevicesGroupProps) {
const { t } = useTranslation("navigation");
	const webMenu = useWebContextMenuController();
	const { navigateToView, loadPreferencesForSpaceItem } = useExplorer();

	// Use normalized query for automatic updates when device events are emitted
	const { data: devices, isLoading } = useNormalizedQuery<
		ListLibraryDevicesInput,
		Device[]
	>({
		query: "devices.list",
		input: {
			include_offline: true,
			include_details: false,
			show_paired: true,
		},
		resourceType: "device",
	});

	// Mutation for unpairing devices
	const revokeDevice = useCoreMutation("network.device.revoke");

	// Handler for device context menu
	const handleDeviceContextMenu = (device: Device) => async (e: React.MouseEvent) => {
		e.preventDefault();
		e.stopPropagation();

		// Only show context menu for non-current devices
		if (device.is_current) return;

		// Create context menu items for this device
		const items = [
			{
				icon: Trash,
                label: t("devices.unpair"),
				onClick: async () => {
					await revokeDevice.mutateAsync({
						device_id: device.id,
						remove_from_library: false, // Keep device in library
					});
				},
				variant: "default" as const,
			},
			{
				icon: Trash,
                label: t("devices.remove"),
				onClick: async () => {
					await revokeDevice.mutateAsync({
						device_id: device.id,
						remove_from_library: true, // Remove from library too
					});
				},
				variant: "danger" as const,
			},
		];

		// Show platform-appropriate context menu
		const wingdrive =
			(window as any).__WINGDRIVE__;
		if (wingdrive?.showContextMenu) {
			// Tauri native menu
			await wingdrive.showContextMenu(items, {
				x: e.clientX,
				y: e.clientY,
			});
		} else {
			webMenu?.show(items, e.clientX, e.clientY);
		}
	};

	return (
		<div>
			<GroupHeader
				label={t("sidebar.devices")}
				isCollapsed={isCollapsed}
				onToggle={onToggle}
				sortableAttributes={sortableAttributes}
				sortableListeners={sortableListeners}
			/>

			{/* Items */}
			{!isCollapsed && (
				<div className="space-y-0.5">
					{isLoading ? (
						<div className="px-2 py-1 text-xs text-sidebar-ink-faint">
							{t("common.loading")}
						</div>
					) : !devices || devices.length === 0 ? (
						<div className="px-2 py-1 text-xs text-sidebar-ink-faint">
							{t("devices.empty")}
						</div>
					) : (
						devices.map((device, index) => {
							// Create a minimal SpaceItem structure for the device
							const deviceItem = {
								id: device.id,
								item_type: "Overview" as const,
							};

							return (
								<SpaceItem
									key={device.id}
									item={deviceItem as any}
									customIcon={getDeviceIcon(device)}
									customLabel={device.name}
									onClick={() => {
										loadPreferencesForSpaceItem(`device:${device.id}`);
										navigateToView("device", device.id);
									}}
									onContextMenu={handleDeviceContextMenu(device)}
									allowInsertion={false}
									isLastItem={index === devices.length - 1}
									className="text-sidebar-ink-dull"
									rightComponent={
										<div className="flex items-center gap-1">
											{!device.is_current &&
												!device.is_connected && (
													<WifiSlashIcon
														size={12}
														weight="bold"
														className="text-ink-dull"
													/>
												)}
											{!device.is_current &&
												device.is_connected && (
													<WifiHigh
														size={12}
														weight="bold"
														className="text-accent"
													/>
												)}
										</div>
									}
								/>
							);
						})
					)}
				</div>
			)}
		</div>
	);
}
