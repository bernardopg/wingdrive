import { fileDisplayName, fileDisplaySize } from "@wingdrive/ts-client";
import { useNormalizedQuery } from "../../contexts/WingDriveContext";
import { usePlatform } from "../../contexts/PlatformContext";
import type { File } from "@wingdrive/ts-client";
import { getContentKind } from "@wingdrive/ts-client";
import { useEffect, useState } from "react";
import { formatBytes } from "../../routes/explorer/utils";
import { X } from "@phosphor-icons/react";
import { ContentRenderer } from "./ContentRenderer";
import { useTranslation } from "../../i18n";
import {uiLocale} from "../../i18n";

function MetadataPanel({ file }: { file: File }) {
	const { t } = useTranslation("inspector");
	const displaySize = fileDisplaySize(file);
	return (
		<div className="w-[280px] min-w-[280px] bg-sidebar-box border-l border-sidebar-line p-4 overflow-y-auto">
			<div className="space-y-4">
				<div>
					<div className="text-xs text-ink-dull mb-1">{t("inspector.preview.name")}</div>
					<div className="text-sm text-ink break-words">
						{file.name}
					</div>
				</div>

				<div>
					<div className="text-xs text-ink-dull mb-1">{t("inspector.preview.kind")}</div>
					<div className="text-sm text-ink capitalize">
						{getContentKind(file)}
					</div>
				</div>

				<div>
					<div className="text-xs text-ink-dull mb-1">{t("inspector.preview.size")}</div>
					<div className="text-sm text-ink">
					{displaySize === null ? "—" : formatBytes(displaySize)}
					</div>
				</div>

				{file.extension && (
					<div>
						<div className="text-xs text-ink-dull mb-1">
							{t("ui.extension")}
						</div>
						<div className="text-sm text-ink">{file.extension}</div>
					</div>
				)}

				{file.created_at && (
					<div>
						<div className="text-xs text-ink-dull mb-1">
							{t("ui.created")}
						</div>
						<div className="text-sm text-ink">
							{new Date(file.created_at).toLocaleString(uiLocale())}
						</div>
					</div>
				)}

				{file.modified_at && (
					<div>
						<div className="text-xs text-ink-dull mb-1">
							{t("ui.modified")}
						</div>
						<div className="text-sm text-ink">
							{new Date(file.modified_at).toLocaleString(uiLocale())}
						</div>
					</div>
				)}
			</div>
		</div>
	);
}

export function QuickPreview() {
	const { t } = useTranslation("inspector");
	const platform = usePlatform();
	const [fileId, setFileId] = useState<string | null>(null);

	useEffect(() => {
		// Extract file_id from window label
		if (platform.getCurrentWindowLabel) {
			const label = platform.getCurrentWindowLabel();

			// Label format: "quick-preview-{file_id}"
			const match = label.match(/^quick-preview-(.+)$/);
			if (match) {
				setFileId(match[1]);
			}
		}
	}, [platform]);

	const {
		data: file,
		isLoading,
		error,
	} = useNormalizedQuery<{ file_id: string }, File>({
		query: "files.by_id",
		input: { file_id: fileId! },
		resourceType: "file",
		resourceId: fileId!,
		enabled: !!fileId,
	});

	const handleClose = () => {
		if (platform.closeCurrentWindow) {
			platform.closeCurrentWindow();
		}
	};

	// Keyboard shortcuts
	useEffect(() => {
		const handleKeyDown = (e: KeyboardEvent) => {
			if (e.code === "Escape") {
				handleClose();
			}
		};

		window.addEventListener("keydown", handleKeyDown);
		return () => window.removeEventListener("keydown", handleKeyDown);
	}, []);

	if (isLoading || !file) {
		return (
			<div className="h-screen flex items-center justify-center bg-app text-ink">
				<div className="animate-pulse">{t("inspector.loading")}</div>
			</div>
		);
	}

	if (error) {
		return (
			<div className="h-screen flex items-center justify-center bg-app text-red-400">
				<div>
					<div className="text-lg font-medium mb-2">
						{t("ui.errorLoadingFile")}
					</div>
					<div className="text-sm">{error.message}</div>
				</div>
			</div>
		);
	}

	return (
		<div className="h-screen flex flex-col bg-app text-ink">
			{/* Header */}
			<div className="flex items-center justify-between px-4 py-3 border-b border-app-line">
				<div className="text-sm font-medium truncate flex-1">
					{fileDisplayName(file)}
				</div>
				<button
					onClick={handleClose}
					className="p-1 rounded-md hover:bg-app-hover text-ink-dull hover:text-ink"
				>
					<X size={16} weight="bold" />
				</button>
			</div>

			{/* Content Area */}
			<div className="flex-1 flex overflow-hidden">
				{/* File Content */}
				<div className="flex-1 p-6 bg-app-box/30">
					<ContentRenderer file={file} />
				</div>

				{/* Metadata Sidebar */}
				<MetadataPanel file={file} />
			</div>

			{/* Footer with keyboard hints */}
			<div className="px-4 py-2 border-t border-app-line bg-app-box/30">
				<div className="text-xs text-ink-dull text-center">
					{t("inspector.preview.closeSimple")}
				</div>
			</div>
		</div>
	);
}
