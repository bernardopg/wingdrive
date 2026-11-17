import type { ReactNode } from "react";
import { Modal, Pressable, Text as NativeText, View } from "react-native";
import { useSafeAreaInsets } from "react-native-safe-area-context";
import { fileDisplayName, type File } from "@wingdrive/ts-client";
import { LocalizedText as Text } from "../../../i18n/LocalizedText";

export interface FileAction {
	label: string;
	icon: ReactNode;
	onPress: () => void;
	destructive?: boolean;
}

/** Bottom sheet with the actions for one file, opened by a long press (MOB-007). */
export function FileActionsSheet({
	file,
	actions,
	onClose,
}: {
	file: File;
	actions: FileAction[];
	onClose: () => void;
}) {
	const insets = useSafeAreaInsets();
	return (
		<Modal transparent animationType="fade" onRequestClose={onClose}>
			<Pressable className="flex-1 bg-black/60 justify-end" onPress={onClose}>
				<Pressable
					className="bg-app-box border-t border-app-line rounded-t-2xl pt-2"
					style={{ paddingBottom: insets.bottom + 8 }}
					onPress={() => {}}
				>
					<NativeText className="text-ink-dull text-sm px-5 py-3" numberOfLines={1}>
						{fileDisplayName(file)}
					</NativeText>
					{actions.map((action) => (
						<Pressable
							key={action.label}
							accessibilityRole="button"
							onPress={() => {
								onClose();
								action.onPress();
							}}
							className="flex-row items-center gap-4 px-5 py-4 active:bg-app-hover"
						>
							<View className="w-6 items-center">{action.icon}</View>
							<Text
								className={`text-base ${action.destructive ? "text-red-500" : "text-ink"}`}
							>
								{action.label}
							</Text>
						</Pressable>
					))}
				</Pressable>
			</Pressable>
		</Modal>
	);
}
