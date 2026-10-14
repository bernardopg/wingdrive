import { useState, useRef } from "react";
import { Copy, Trash, Eye, Share } from "@phosphor-icons/react";
import { useDragOperation } from "../hooks/useDragOperation";
import { useDropZone } from "../hooks/useDropZone";
import { useContextMenu, useTranslation } from "@wingdrive/interface";
import type { DragItem } from "../lib/drag";

export function DragDemo() {
	const { t } = useTranslation("common");
	const [selectedFiles, setSelectedFiles] = useState<string[]>([
		"/Users/example/Documents/report.pdf",
		"/Users/example/Pictures/photo.jpg",
	]);
	const [selectedFile, setSelectedFile] = useState<string | null>(null);
	const [draggingFile, setDraggingFile] = useState<string | null>(null);
	const dragStartPos = useRef<{ x: number; y: number } | null>(null);

	// Context menu for files
	const contextMenu = useContextMenu({
		items: [
			{
				icon: Copy,
				label: t("contextMenu.copy"),
				onClick: () => alert(t("dragDemo.copying", {file: selectedFile})),
				keybind: "⌘C",
				condition: () => selectedFile !== null,
			},
			{
				icon: Eye,
				label: t("contextMenu.quickLook"),
				onClick: () => alert(t("dragDemo.quickLook", {file: selectedFile})),
				keybind: "Space",
			},
			{ type: "separator" },
			{
				icon: Share,
				label: t("contextMenu.share"),
				submenu: [
					{
						label: "AirDrop",
						onClick: () => alert(t("dragDemo.airDropShare")),
					},
					{
						label: t("dragDemo.messages"),
						onClick: () => alert(t("dragDemo.messagesShare")),
					},
				],
			},
			{ type: "separator" },
			{
				icon: Trash,
				label: t("contextMenu.delete"),
				onClick: () => {
					if (selectedFile && confirm(t("dragDemo.confirmDelete", {file: selectedFile}))) {
						setSelectedFiles((files) =>
							files.filter((f) => f !== selectedFile),
						);
						setSelectedFile(null);
					}
				},
				keybind: "⌘⌫",
				variant: "danger" as const,
			},
		],
	});

	const { isDragging, startDrag, cursorPosition } = useDragOperation({
		onDragStart: (sessionId) => {
			console.log("Drag started:", sessionId);
		},
		onDragEnd: (result) => {
			console.log("Drag ended:", result);
			setDraggingFile(null);
			dragStartPos.current = null;
		},
	});

	const { isHovered, dropZoneProps } = useDropZone({
		onDrop: (items) => {
			console.log("Files dropped:", items);
		},
		onDragEnter: () => {
			console.log("Drag entered drop zone");
		},
		onDragLeave: () => {
			console.log("Drag left drop zone");
		},
	});

	const handleMouseDown = (file: string, e: React.MouseEvent) => {
		setDraggingFile(file);
		dragStartPos.current = { x: e.clientX, y: e.clientY };
	};

	const handleMouseMove = async (e: React.MouseEvent) => {
		if (!draggingFile || !dragStartPos.current || isDragging) return;

		const distance = Math.sqrt(
			Math.pow(e.clientX - dragStartPos.current.x, 2) +
				Math.pow(e.clientY - dragStartPos.current.y, 2),
		);

		// Start native drag after moving 10px
		if (distance > 10) {
			const items: DragItem[] = [
				{
					id: `file-${draggingFile}`,
					kind: {
						type: "file" as const,
						path: draggingFile,
					},
				},
			];

			try {
				await startDrag({
					items,
					allowedOperations: ["copy", "move"],
				});
			} catch (error) {
				console.error("Failed to start drag:", error);
				setDraggingFile(null);
			}
		}
	};

	const handleMouseUp = () => {
		setDraggingFile(null);
		dragStartPos.current = null;
	};

	return (
		<div
			className="p-8 space-y-6 bg-gray-900 text-white min-h-screen"
			onMouseMove={handleMouseMove}
			onMouseUp={handleMouseUp}
			onMouseLeave={handleMouseUp}
		>
			<h1 className="text-3xl font-bold">{t("dragDemo.title")}</h1>

			{/* Draggable items */}
			<div className="space-y-4">
				<h2 className="text-xl font-semibold">{t("dragDemo.files")}</h2>
				<div className="space-y-2">
					{selectedFiles.map((file, idx) => (
						<div
							key={idx}
							className={`bg-gray-800 p-4 rounded-lg border transition-colors cursor-move select-none ${
								draggingFile === file
									? "border-accent bg-blue-900/20"
									: selectedFile === file
										? "border-green-500 bg-green-900/20"
										: "border-gray-700 hover:border-accent"
							}`}
							onMouseDown={(e) => {
								e.preventDefault();
								handleMouseDown(file, e);
							}}
							onClick={() => setSelectedFile(file)}
							onContextMenu={(e) => {
								setSelectedFile(file);
								contextMenu.show(e);
							}}
						>
							<div className="flex items-center gap-3">
								<div className="text-2xl"></div>
								<div className="flex-1">
									<div className="font-medium">
										{file.split("/").pop()}
									</div>
									<div className="text-sm text-gray-400">
										{file}
									</div>
								</div>
							</div>
						</div>
					))}
				</div>
				<p className="text-sm text-gray-400">
					{t("dragDemo.dragInstructions")}
					<br />
					{t("dragDemo.contextMenuInstructions")}
				</p>
			</div>

			{/* Drop zone */}
			<div className="space-y-4">
				<h2 className="text-xl font-semibold">{t("dragDemo.dropZone")}</h2>
				<div
					{...dropZoneProps}
					className={`
            border-2 border-dashed rounded-lg p-8 text-center transition-all
            ${isHovered ? "border-accent bg-accent/10" : "border-gray-700 bg-gray-800/50"}
          `}
				>
					<div className="text-4xl mb-2">{isHovered ? "" : ""}</div>
					<div className="text-lg font-medium">
						{isHovered ? t("dragDemo.dropFiles") : t("dragDemo.dragFiles")}
					</div>
					<div className="text-sm text-gray-400 mt-1">
						{t("dragDemo.dropZoneHint")}
					</div>
				</div>
			</div>

			{/* Status */}
			<div className="space-y-2">
				<h2 className="text-xl font-semibold">{t("dragDemo.status")}</h2>
				<div className="bg-gray-800 p-4 rounded-lg space-y-2 font-mono text-sm">
					<div>
						<span className="text-gray-400">{t("dragDemo.dragging")}:</span>{" "}
						<span
							className={
								isDragging ? "text-green-400" : "text-gray-500"
							}
						>
							{isDragging ? t("dragDemo.yes") : t("dragDemo.no")}
						</span>
					</div>
					<div>
						<span className="text-gray-400">
							{t("dragDemo.dropZoneHovered")}:
						</span>{" "}
						<span
							className={
								isHovered ? "text-blue-400" : "text-gray-500"
							}
						>
							{isHovered ? t("dragDemo.yes") : t("dragDemo.no")}
						</span>
					</div>
					{cursorPosition && (
						<div>
							<span className="text-gray-400">{t("dragDemo.cursor")}:</span>{" "}
							<span className="text-gray-300">
								({Math.round(cursorPosition.x)},{" "}
								{Math.round(cursorPosition.y)})
							</span>
						</div>
					)}
				</div>
			</div>

			<div className="text-sm text-gray-500 border-t border-gray-800 pt-4">
				<p className="font-semibold mb-2">{t("dragDemo.howItWorks")}</p>
				<ul className="list-disc list-inside space-y-1">
					<li>
						{t("dragDemo.howItWorksItems.nativeFiles")}
					</li>
					<li>
						{t("dragDemo.howItWorksItems.overlay")}
					</li>
					<li>
						{t("dragDemo.howItWorksItems.dropZones")}
					</li>
					<li>
						{t("dragDemo.howItWorksItems.sync")}
					</li>
					<li>
						<strong>
							{t("dragDemo.howItWorksItems.contextMenu")}
						</strong>{" "}
						{t("dragDemo.howItWorksItems.transparentWindow")}
					</li>
				</ul>
			</div>
		</div>
	);
}
