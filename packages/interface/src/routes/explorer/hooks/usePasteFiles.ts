import type { WingPath } from "@wingdrive/ts-client";
import { toast } from "@wingdrive/primitives";
import { useClipboard } from "../../../hooks/useClipboard";
import { useFileOperationDialog } from "../../../components/modals/FileOperationModal";
import { useTranslation } from "../../../i18n";

export function usePasteFiles() {
	const {t} = useTranslation("explorer");
	const clipboard = useClipboard();
	const openFileOperation = useFileOperationDialog();
	return async (destination: WingPath | null) => {
		if (!destination) return;
		try {
			const contents = await clipboard.readFiles();
			if (contents.files.length === 0) return;
			openFileOperation({
				operation: contents.operation === "cut" ? "move" : "copy",
				sources: contents.files,
				destination,
				onComplete: () => {
					void clipboard.finishPaste(contents).catch((error) => toast.error(t("errors.couldNotUpdateClipboard", {error})));
				},
			});
		} catch (error) {
			toast.error(t("errors.couldNotPasteFiles", {error}));
		}
	};
}
