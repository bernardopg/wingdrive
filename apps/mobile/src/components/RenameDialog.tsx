import { useState } from "react";
import { Modal, Pressable, TextInput, View } from "react-native";
import { useTranslation } from "react-i18next";
import { LocalizedText as Text } from "../i18n/LocalizedText";
import { Button } from "./primitive";

/** One-field text prompt; Android has no `Alert.prompt`. */
export function RenameDialog({
	title,
	initialValue,
	selectUntil,
	onSubmit,
	onClose,
}: {
	title: string;
	initialValue: string;
	/** Selects only the first characters, such as a file name without its extension */
	selectUntil?: number;
	onSubmit: (value: string) => void;
	onClose: () => void;
}) {
	const { t } = useTranslation("common");
	const [value, setValue] = useState(initialValue);
	// Applied once on focus, then left to the user.
	const [selection, setSelection] = useState<{ start: number; end: number } | undefined>(
		selectUntil === undefined ? undefined : { start: 0, end: selectUntil },
	);
	const trimmed = value.trim();

	return (
		<Modal transparent animationType="fade" onRequestClose={onClose}>
			<Pressable className="flex-1 bg-black/60 items-center justify-center px-6" onPress={onClose}>
				<Pressable className="w-full bg-app-box border border-app-line rounded-2xl p-5" onPress={() => {}}>
					<Text className="text-ink text-lg font-semibold mb-4">{title}</Text>
					<TextInput
						value={value}
						onChangeText={setValue}
						autoFocus
						selectTextOnFocus={selectUntil === undefined}
						selection={selection}
						onSelectionChange={() => selection && setSelection(undefined)}
						maxLength={64}
						className="h-12 px-4 rounded-xl border border-app-line text-ink"
						placeholderTextColor="hsl(235, 10%, 55%)"
					/>
					<View className="flex-row gap-3 mt-5">
						<View className="flex-1">
							<Button variant="gray" onPress={onClose}>
								<Text className="text-ink font-medium">{t("Cancel")}</Text>
							</Button>
						</View>
						<View className="flex-1">
							<Button
								variant="accent"
								disabled={!trimmed || trimmed === initialValue}
								onPress={() => onSubmit(trimmed)}
							>
								<Text className="text-white font-medium">{t("Save")}</Text>
							</Button>
						</View>
					</View>
				</Pressable>
			</Pressable>
		</Modal>
	);
}
