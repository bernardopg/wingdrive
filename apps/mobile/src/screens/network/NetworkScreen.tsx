import { LocalizedText as Text } from "../../i18n/LocalizedText";
import React, { useState } from "react";
import { View, ScrollView, Pressable, Alert, RefreshControl } from "react-native";
import { useSafeAreaInsets } from "react-native-safe-area-context";
import { useTranslation } from "react-i18next";
import type { PairedDeviceInfo } from "@wingdrive/ts-client/src/generated/types";
import { Card, Button } from "../../components/primitive";
import { PairingPanel } from "../../components/PairingPanel";
import { useCoreAction, useCoreQuery } from "../../client";

function DeviceRow({ device, onRevoke }: { device: PairedDeviceInfo; onRevoke: () => void }) {
	return (
		<Pressable onLongPress={onRevoke} className="flex-row items-center py-3">
			<View
				className={`w-2.5 h-2.5 rounded-full mr-3 ${device.isConnected ? "bg-green-500" : "bg-ink-faint"}`}
			/>
			<View className="flex-1">
				<Text className="text-ink font-medium">{device.name}</Text>
				<Text className="text-ink-dull text-sm">
					{`${device.deviceType} · ${device.osVersion} · ${device.appVersion}`}
				</Text>
			</View>
			<Text className="text-ink-dull text-sm">{device.isConnected ? "Connected" : "Offline"}</Text>
		</Pressable>
	);
}

export function NetworkScreen() {
	const { t } = useTranslation("common");
	const insets = useSafeAreaInsets();
	const [pairingMode, setPairingMode] = useState<"generate" | "join" | null>(null);

	// Poll so a peer that connects or drops shows up without leaving the screen.
	const status = useCoreQuery("network.status", null, { refetchInterval: 5000 });
	const devices = useCoreQuery("network.devices.list", { connectedOnly: false }, { refetchInterval: 5000 });
	const revoke = useCoreAction("network.device.revoke");

	const confirmRevoke = (device: PairedDeviceInfo) => {
		Alert.alert(t("Unpair device"), t("Remove the pairing with this device?"), [
			{ text: t("Cancel"), style: "cancel" },
			{
				text: t("Unpair"),
				style: "destructive",
				onPress: () =>
					revoke.mutate(
						{ device_id: device.id, remove_from_library: false },
						{ onSettled: () => devices.refetch() },
					),
			},
		]);
	};

	const running = status.data?.running ?? false;
	const paired = devices.data?.devices ?? [];

	return (
		<View className="flex-1 bg-app">
			<ScrollView
				contentContainerStyle={{
					paddingTop: insets.top + 16,
					paddingBottom: insets.bottom + 100,
					paddingHorizontal: 16,
				}}
				refreshControl={
					<RefreshControl
						refreshing={status.isRefetching || devices.isRefetching}
						onRefresh={() => {
							status.refetch();
							devices.refetch();
						}}
					/>
				}
			>
				<Text className="text-2xl font-bold text-ink mb-6">Network</Text>

				<Card className="mb-6">
					<View className="flex-row items-center">
						<View className={`w-3 h-3 rounded-full mr-3 ${running ? "bg-green-500" : "bg-red-500"}`} />
						<View className="flex-1">
							<Text className="text-ink font-medium">Network Status</Text>
							<Text className="text-ink-dull text-sm">{running ? "P2P enabled" : "P2P disabled"}</Text>
						</View>
						{devices.data && (
							<Text className="text-ink-dull text-sm">
								{`${devices.data.connected}/${devices.data.total}`}
							</Text>
						)}
					</View>
					{status.data?.node_id && (
						<Text selectable className="text-ink-faint text-xs mt-3">
							{status.data.node_id}
						</Text>
					)}
				</Card>

				<Text className="text-lg font-semibold text-ink mb-3">Paired Devices</Text>
				<Card className="mb-4">
					{paired.length === 0 ? (
						<Text className="text-ink-dull">No paired devices yet</Text>
					) : (
						paired.map((device) => (
							<DeviceRow key={device.id} device={device} onRevoke={() => confirmRevoke(device)} />
						))
					)}
				</Card>

				<View className="flex-row gap-3">
					<View className="flex-1">
						<Button variant="accent" onPress={() => setPairingMode("generate")}>
							<Text className="text-white font-medium">Generate Code</Text>
						</Button>
					</View>
					<View className="flex-1">
						<Button variant="gray" onPress={() => setPairingMode("join")}>
							<Text className="text-ink font-medium">Join with Code</Text>
						</Button>
					</View>
				</View>
			</ScrollView>

			<PairingPanel
				key={pairingMode ?? "closed"}
				isOpen={pairingMode !== null}
				initialMode={pairingMode ?? "generate"}
				onClose={() => {
					setPairingMode(null);
					devices.refetch();
				}}
			/>
		</View>
	);
}
