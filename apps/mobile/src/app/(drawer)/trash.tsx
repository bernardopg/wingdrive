import { Alert, FlatList, Pressable, Text, View } from "react-native";
import { useSafeAreaInsets } from "react-native-safe-area-context";
import { useTranslation } from "react-i18next";
import { useRouter } from "expo-router";
import { useQueryClient } from "@tanstack/react-query";
import type { TrashEntry } from "@wingdrive/ts-client";
import { useCoreAction, useCoreQuery } from "../../client";

export default function TrashScreen() {
	const { t } = useTranslation("common");
	const router = useRouter();
	const insets = useSafeAreaInsets();
	const client = useQueryClient();
	const trash = useCoreQuery("trash.list", {});
	const refresh = async () => {
		await Promise.all([
			trash.refetch(),
			client.invalidateQueries({ queryKey: ["query:files.directory_listing"] }),
			client.invalidateQueries({ queryKey: ["query:search.files"] }),
		]);
	};
	const showError = (error: Error) => Alert.alert(t("Error"), error.message);
	const restore = useCoreAction("trash.restore", { onError: showError, onSettled: refresh });
	const purge = useCoreAction("trash.purge", { onError: showError, onSettled: refresh });
	const pending = restore.isPending || purge.isPending;
	const confirmPurge = (item: TrashEntry) => Alert.alert(
		t("Delete permanently?"),
		`${item.name}\n\n${t("This cannot be undone.")}`,
		[
			{ text: t("Cancel"), style: "cancel" },
			{ text: t("Delete"), style: "destructive", onPress: () => purge.mutate({ ids: [item.id] }) },
		],
	);

	return (
		<View className="flex-1 bg-app px-4" style={{ paddingTop: insets.top, paddingBottom: insets.bottom }}>
			<View className="flex-row items-center gap-3 border-b border-app-line py-2">
				<Pressable
					accessibilityRole="button"
					onPress={() => router.canGoBack() ? router.back() : router.replace("/browse")}
					className="min-h-12 justify-center px-2"
				>
					<Text className="text-accent">{t("Go back")}</Text>
				</Pressable>
				<Text className="text-ink text-xl font-semibold">{t("Trash")}</Text>
			</View>
			<FlatList
				data={trash.data?.items ?? []}
				keyExtractor={(item) => item.id}
				refreshing={trash.isRefetching}
				onRefresh={() => void trash.refetch()}
				ListEmptyComponent={<Text className="text-ink-dull px-3 py-4">
					{t(trash.isError ? "Could not load Trash" : trash.isPending ? "Loading" : trash.data?.supported ? "Trash is empty" : "Trash is unavailable on this device")}
				</Text>}
				renderItem={({ item }) => (
					<View className="border-b border-app-line px-3 py-3">
						<Text className="text-ink font-semibold" numberOfLines={1}>{item.name}</Text>
						<Text className="text-ink-dull" numberOfLines={2}>{item.original_path}</Text>
						<View className="flex-row flex-wrap gap-3">
							<Pressable
								accessibilityRole="button"
								disabled={pending}
								className="min-h-12 justify-center px-2"
								onPress={() => restore.mutate({ ids: [item.id] })}
							>
								<Text className="text-accent">{t("Restore")}</Text>
							</Pressable>
							<Pressable
								accessibilityRole="button"
								disabled={pending}
								className="min-h-12 justify-center px-2"
								onPress={() => confirmPurge(item)}
							>
								<Text className="text-red-500">{t("Delete permanently")}</Text>
							</Pressable>
						</View>
					</View>
				)}
			/>
		</View>
	);
}
