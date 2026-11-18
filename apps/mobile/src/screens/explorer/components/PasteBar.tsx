import { Pressable, Text as NativeText, View } from "react-native";
import { useSafeAreaInsets } from "react-native-safe-area-context";
import { useTranslation } from "react-i18next";
import type { ClipboardItem, ClipboardMode } from "../../../stores/fileClipboard";

/** Shown while files wait to be pasted: the folder on screen is where they go. */
export function PasteBar({
	mode,
	items,
	blocker,
	pending,
	onPaste,
	onCancel,
}: {
	mode: ClipboardMode;
	items: ClipboardItem[];
	/** Catalog key saying why this folder cannot receive them */
	blocker: string | null;
	pending: boolean;
	onPaste: () => void;
	onCancel: () => void;
}) {
	const { t } = useTranslation("common");
	const insets = useSafeAreaInsets();
	const disabled = !!blocker || pending;
	const names = items.map((item) => item.name).join(", ");
	return (
		<View
			className="bg-app-box border-t border-app-line px-4 pt-3 flex-row items-center gap-3"
			style={{ paddingBottom: insets.bottom + 12 }}
		>
			<View className="flex-1">
				<NativeText className="text-ink font-semibold" numberOfLines={1}>
					{t(mode === "copy" ? "Copy here" : "Move here")}
				</NativeText>
				<NativeText className="text-ink-dull text-xs" numberOfLines={blocker ? undefined : 1}>
					{blocker ? t(blocker) : names}
				</NativeText>
			</View>
			<Pressable
				accessibilityRole="button"
				accessibilityState={{ disabled: pending }}
				disabled={pending}
				onPress={onCancel}
				className="min-h-12 justify-center px-3"
			>
				<NativeText className="text-ink-dull font-medium">{t("Cancel")}</NativeText>
			</Pressable>
			<Pressable
				accessibilityRole="button"
				accessibilityState={{ disabled, busy: pending }}
				disabled={disabled}
				onPress={onPaste}
				className={`min-h-12 justify-center px-4 rounded-lg ${disabled ? "bg-app-line" : "bg-accent"}`}
			>
				<NativeText className={`font-semibold ${disabled ? "text-ink-faint" : "text-white"}`}>
					{t(pending ? "Pasting…" : "Paste")}
				</NativeText>
			</Pressable>
		</View>
	);
}
