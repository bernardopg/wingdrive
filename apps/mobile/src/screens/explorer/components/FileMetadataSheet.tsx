import { useState } from "react";
import { Alert, FlatList, Modal, Pressable, Text, TextInput, View } from "react-native";
import { useSafeAreaInsets } from "react-native-safe-area-context";
import { useTranslation } from "react-i18next";
import { useQueryClient } from "@tanstack/react-query";
import { fileDisplayName, type File, type TagTargets } from "@wingdrive/ts-client";
import { useLibraryAction, useLibraryQuery } from "../../../client";

export function FileMetadataSheet({ file, onClose }: { file: File; onClose: () => void }) {
	const { t } = useTranslation("common");
	const insets = useSafeAreaInsets();
	const client = useQueryClient();
	const [search, setSearch] = useState("");
	const current = useLibraryQuery("files.by_id", { file_id: file.id });
	const matches = useLibraryQuery("tags.search", {
		query: search.trim(),
		namespace: null,
		tag_type: null,
		include_archived: false,
		limit: 50,
		resolve_ambiguous: false,
		context_tag_ids: null,
	});
	const refresh = async () => {
		await Promise.all([
			current.refetch(),
			matches.refetch(),
			client.invalidateQueries({ queryKey: ["query:files.directory_listing"] }),
			client.invalidateQueries({ queryKey: ["query:search.files"] }),
		]);
	};
	const showError = (error: Error) => Alert.alert(t("Error"), error.message);
	const favorite = useLibraryAction("metadata.set_favorite", { onError: showError, onSuccess: refresh });
	const apply = useLibraryAction("tags.apply", { onError: showError, onSuccess: async (result) => {
		if (result.warnings.length) Alert.alert(t("Error"), result.warnings.join("\n"));
		await refresh();
	} });
	const unapply = useLibraryAction("tags.unapply", { onError: showError, onSuccess: refresh });
	const create = useLibraryAction("tags.create", {
		onError: showError,
		onSuccess: async () => {
			setSearch("");
			await refresh();
		},
	});
	const pending = favorite.isPending || apply.isPending || unapply.isPending || create.isPending;
	const currentFile = current.data;
	const assigned = currentFile?.tags ?? [];
	const tags = [...new Map([
		...assigned,
		...(matches.data?.tags.map(({ tag }) => tag) ?? []),
	].map((tag) => [tag.id, tag])).values()];
	const targets: TagTargets = { type: "EntryUuid", ids: [file.id] };
	const newName = search.trim();
	const canCreate = newName && matches.isSuccess && !tags.some(
		(tag) => tag.canonical_name.toLowerCase() === newName.toLowerCase(),
	);

	return (
		<Modal visible animationType="slide" onRequestClose={onClose}>
			<View className="flex-1 bg-app px-4" style={{ paddingTop: insets.top + 12, paddingBottom: insets.bottom }}>
				<View className="flex-row items-center gap-3 mb-4">
					<Text className="flex-1 text-ink text-lg font-semibold" numberOfLines={1}>{fileDisplayName(file)}</Text>
					<Pressable accessibilityRole="button" onPress={onClose} className="min-h-12 justify-center px-3">
						<Text className="text-ink">{t("Done")}</Text>
					</Pressable>
				</View>
				{current.isPending ? (
					<Text className="text-ink-dull">{t("Loading")}</Text>
				) : current.isError || !currentFile ? (
					<Text className="text-ink-dull">{t("Could not load file metadata")}</Text>
				) : (
					<>
						<Pressable
							accessibilityRole="switch"
							accessibilityState={{ checked: currentFile.favorite, disabled: pending }}
							disabled={pending}
							onPress={() => favorite.mutate({ entry_uuid: file.id, is_favorite: !currentFile.favorite })}
							className="min-h-12 mb-4 justify-center rounded-lg bg-app-box px-3"
						>
							<Text className="text-ink">{t(currentFile.favorite ? "Remove from Favorites" : "Add to Favorites")}</Text>
						</Pressable>
						<TextInput
							accessibilityLabel={t("Search tags")}
							placeholder={t("Search tags")}
							placeholderTextColor="hsl(235, 10%, 55%)"
							value={search}
							onChangeText={setSearch}
							maxLength={64}
							className="min-h-12 mb-2 px-3 rounded-lg border border-app-line text-ink"
						/>
						{canCreate ? (
							<Pressable
								accessibilityRole="button"
								disabled={pending}
								className="min-h-12 justify-center px-3"
								onPress={() => create.mutate({
									canonical_name: newName,
									display_name: null,
									formal_name: null,
									abbreviation: null,
									aliases: [],
									namespace: null,
									tag_type: null,
									color: null,
									icon: null,
									description: null,
									is_organizational_anchor: null,
									privacy_level: null,
									search_weight: null,
									attributes: null,
									apply_to: targets,
								})}
							>
								<Text className="text-accent">{t("Create tag {{name}}", { name: newName })}</Text>
							</Pressable>
						) : null}
						<FlatList
							data={tags}
							keyExtractor={(tag) => tag.id}
							keyboardShouldPersistTaps="handled"
							ListEmptyComponent={<Text className="text-ink-dull px-3 py-2">
								{t(matches.isError ? "Could not load tags" : matches.isPending ? "Loading" : "No tags")}
							</Text>}
							renderItem={({ item: tag }) => {
								const checked = assigned.some((item) => item.id === tag.id);
								return (
									<Pressable
										accessibilityRole="checkbox"
										accessibilityState={{ checked, disabled: pending }}
										disabled={pending}
										className="min-h-12 flex-row items-center gap-3 border-b border-app-line px-3"
										onPress={() => checked
											? unapply.mutate({ entry_ids: [file.id], tag_ids: [tag.id] })
											: apply.mutate({ targets, tag_ids: [tag.id], source: "User", confidence: null, applied_context: null, instance_attributes: null })
										}
									>
										<Text className={checked ? "text-accent" : "text-ink-dull"}>{checked ? "✓" : "○"}</Text>
										<Text className="text-ink flex-1">{tag.display_name ?? tag.canonical_name}</Text>
									</Pressable>
								);
							}}
						/>
					</>
				)}
			</View>
		</Modal>
	);
}
