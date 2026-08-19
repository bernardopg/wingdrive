import { createPortal } from "react-dom";
import { motion, AnimatePresence } from "framer-motion";
import { X, ArrowLeft, ArrowRight, Info, Play, Pause } from "@phosphor-icons/react";
import { useEffect, useState, useMemo, useRef } from "react";
import { ContentRenderer } from "./ContentRenderer";
import { MetadataPanel } from "./MetadataPanel";
import {
	VideoControls,
	type VideoControlsState,
	type VideoControlsCallbacks,
} from "./VideoControls";
import { TopBarPortal, TopBarItem } from "../../TopBar";
import { getContentKind } from "@sd/ts-client";
import { useExplorer } from "../../routes/explorer/context";
import { isInputFocused } from "../../util/keybinds";

interface QuickPreviewFullscreenProps {
	fileId: string;
	isOpen: boolean;
	onClose: () => void;
	onNext?: () => void;
	onPrevious?: () => void;
	hasPrevious?: boolean;
	hasNext?: boolean;
	sidebarWidth?: number;
	inspectorWidth?: number;
}

const PREVIEW_LAYER_ID = "quick-preview-layer";

/** Kept in sync with MetadataPanel so the content area can make room for it. */
const METADATA_PANEL_WIDTH = 300;

export function QuickPreviewFullscreen({
	fileId,
	isOpen,
	onClose,
	onNext,
	onPrevious,
	hasPrevious,
	hasNext,
	sidebarWidth = 0,
	inspectorWidth = 0,
}: QuickPreviewFullscreenProps) {
	const [portalTarget, setPortalTarget] = useState<HTMLElement | null>(null);
	const [isZoomed, setIsZoomed] = useState(false);
	const [videoControlsState, setVideoControlsState] =
		useState<VideoControlsState | null>(null);
	const [showVideoControls, setShowVideoControls] = useState(false);
	const [videoCallbacks, setVideoCallbacks] =
		useState<VideoControlsCallbacks | null>(null);
	const [showMetadata, setShowMetadata] = useState(false);
	const [isSlideshow, setIsSlideshow] = useState(false);
	const { currentFiles } = useExplorer();

	// Zoom is per-file, but the metadata panel stays open across navigation so
	// browsing a folder with details visible doesn't flicker on every step.
	useEffect(() => {
		setIsZoomed(false);
	}, [fileId]);

	// Read through a ref so re-renders from the parent don't restart the timer.
	const onNextRef = useRef(onNext);
	useEffect(() => {
		onNextRef.current = onNext;
	}, [onNext]);

	// Auto-advance while slideshow is active
	useEffect(() => {
		if (!isSlideshow || !hasNext) return;
		const timer = setInterval(() => onNextRef.current?.(), 3000);
		return () => clearInterval(timer);
	}, [isSlideshow, hasNext]);

	// Stop the slideshow at the last item and when the preview closes
	useEffect(() => {
		if ((!hasNext || !isOpen) && isSlideshow) {
			setIsSlideshow(false);
		}
	}, [hasNext, isOpen, isSlideshow]);

	// Get file directly from currentFiles - instant, no network request
	const file = useMemo(
		() => currentFiles.find((f) => f.id === fileId) ?? null,
		[currentFiles, fileId],
	);

	// No query needed - files are already loaded by the explorer views
	const isLoading = false;
	const error = null as Error | null;

	// Find portal target on mount
	useEffect(() => {
		const target = document.getElementById(PREVIEW_LAYER_ID);
		setPortalTarget(target);
	}, []);

	useEffect(() => {
		if (!isOpen) return;

		const handleKeyDown = (e: KeyboardEvent) => {
			// Typing in the tag search or any other field must not close the preview.
			if (isInputFocused()) return;

			// Escape peels one layer at a time: metadata panel first, then the preview.
			if (e.code === "Escape" && showMetadata) {
				e.preventDefault();
				e.stopImmediatePropagation();
				setShowMetadata(false);
				return;
			}

			// Only handle close events - let Explorer handle navigation
			if (e.code === "Escape" || e.code === "Space") {
				e.preventDefault();
				e.stopImmediatePropagation();
				onClose();
			}
		};

		window.addEventListener("keydown", handleKeyDown, { capture: true });
		return () =>
			window.removeEventListener("keydown", handleKeyDown, {
				capture: true,
			});
	}, [isOpen, onClose, showMetadata]);

	// Get background style based on content type
	const getBackgroundClass = () => {
		if (!file) return "bg-black/90";

		switch (getContentKind(file)) {
			case "video":
				return "bg-black";
			case "audio":
				return "audio-gradient";
			case "image":
				return "bg-black/95";
			default:
				return "bg-black/90";
		}
	};

	// Memoize TopBarItem children to prevent infinite re-renders
	const navigationButtons = useMemo(
		() => (
			<div className="flex items-center gap-2">
				<button
					onClick={onPrevious}
					disabled={!hasPrevious}
					className="rounded-md p-1.5 text-white/70 transition-colors hover:bg-white/10 hover:text-white disabled:opacity-30"
				>
					<ArrowLeft size={16} weight="bold" />
				</button>
				<button
					onClick={onNext}
					disabled={!hasNext}
					className="rounded-md p-1.5 text-white/70 transition-colors hover:bg-white/10 hover:text-white disabled:opacity-30"
				>
					<ArrowRight size={16} weight="bold" />
				</button>
				<div className="h-4 w-px bg-white/20 mx-1" />
			</div>
		),
		[onPrevious, onNext, hasPrevious, hasNext]
	);

	const filenameDisplay = useMemo(
		() => (
			<div className="truncate text-sm font-medium text-white/90">
				{file?.name}
			</div>
		),
		[file?.name]
	);

	const closeButton = useMemo(
		() => (
			<button
				onClick={onClose}
				className="rounded-md p-1.5 text-white/70 transition-colors hover:bg-white/10 hover:text-white"
			>
				<X size={16} weight="bold" />
			</button>
		),
		[onClose]
	);

	const infoButton = useMemo(
		() => (
			<button
				onClick={() => setShowMetadata((s) => !s)}
				className={`rounded-md p-1.5 transition-colors hover:bg-white/10 hover:text-white ${showMetadata ? "bg-white/15 text-white" : "text-white/70"}`}
				title="Details"
			>
				<Info size={16} weight="bold" />
			</button>
		),
		[showMetadata]
	);

	const slideshowButton = useMemo(
		() => (
			<button
				onClick={() => setIsSlideshow((s) => !s)}
				className={`rounded-md p-1.5 transition-colors hover:bg-white/10 hover:text-white ${isSlideshow ? "bg-white/15 text-white" : "text-white/70"}`}
				title={isSlideshow ? "Stop slideshow" : "Start slideshow"}
			>
				{isSlideshow ? (
					<Pause size={16} weight="bold" />
				) : (
					<Play size={16} weight="fill" />
				)}
			</button>
		),
		[isSlideshow]
	);

	if (!portalTarget) return null;

	const content = (
		<AnimatePresence mode="wait">
			{isOpen && (
				<motion.div
					key="fullscreen-preview"
					initial={{ opacity: 0 }}
					animate={{ opacity: 1 }}
					exit={{ opacity: 0 }}
					transition={{ duration: 0.2 }}
					className={`absolute inset-0 flex flex-col ${getBackgroundClass()}`}
				>
					{!file && isLoading ? (
						<div className="flex h-full items-center justify-center text-ink">
							<div className="animate-pulse">Loading...</div>
						</div>
					) : !file && error ? (
						<div className="flex h-full items-center justify-center text-red-400">
							<div>
								<div className="mb-2 text-lg font-medium">
									Error loading file
								</div>
								<div className="text-sm">{error.message}</div>
							</div>
						</div>
					) : !file ? (
						<div className="flex h-full items-center justify-center text-ink-dull">
							<div>File not found</div>
						</div>
					) : (
						<>
							{/* TopBar content via portal */}
							<TopBarPortal
								left={
									<>
										{(hasPrevious || hasNext) && (
											<TopBarItem
												id="preview-navigation"
												label="Navigation"
												priority="high"
											>
												{navigationButtons}
											</TopBarItem>
										)}
									</>
								}
								center={
									<TopBarItem
										id="preview-filename"
										label="File Name"
										priority="high"
									>
										{filenameDisplay}
									</TopBarItem>
								}
								right={
									<>
										{hasNext && (
											<TopBarItem
												id="preview-slideshow"
												label="Slideshow"
												priority="high"
											>
												{slideshowButton}
											</TopBarItem>
										)}
										<TopBarItem
											id="preview-details"
											label="Details"
											priority="high"
										>
											{infoButton}
										</TopBarItem>
										<TopBarItem
											id="preview-close"
											label="Close"
											priority="high"
											onClick={onClose}
										>
											{closeButton}
										</TopBarItem>
									</>
								}
							/>

							{/* Content Area - padded to fit between sidebar/inspector, expands on zoom */}
							<div
								className={`flex-1 pt-14 pb-10 ${isZoomed ? "overflow-visible" : "overflow-hidden"}`}
								style={{
									paddingLeft: isZoomed ? 0 : sidebarWidth,
									paddingRight: isZoomed
										? 0
										: inspectorWidth +
											(showMetadata ? METADATA_PANEL_WIDTH : 0),
									transition: "padding 0.3s ease-out",
								}}
							>
								<ContentRenderer
									file={file}
									onZoomChange={setIsZoomed}
									onVideoControlsStateChange={
										setVideoControlsState
									}
									onShowVideoControlsChange={
										setShowVideoControls
									}
									getVideoCallbacks={setVideoCallbacks}
								/>
							</div>

							{/* Video Controls Overlay - fixed position, always uses sidebar/inspector padding */}
							{videoControlsState &&
								videoCallbacks &&
								getContentKind(file) === "video" && (
									<div
										className="absolute inset-0"
										style={{
											paddingTop: "56px", // TopBar height
											paddingBottom: "40px", // Footer height
											pointerEvents: "none", // Let clicks through except on controls themselves
										}}
									>
										<VideoControls
											file={file}
											state={videoControlsState}
											callbacks={videoCallbacks}
											showControls={showVideoControls}
											sidebarWidth={sidebarWidth}
											inspectorWidth={inspectorWidth}
										/>
									</div>
								)}

							{/* Footer with keyboard hints */}
							<div className="absolute bottom-0 left-0 right-0 z-10 px-6 py-3">
								<div className="text-center text-xs text-white/50">
									<span className="text-white/70">ESC</span>{" "}
									or{" "}
									<span className="text-white/70">Space</span>{" "}
									to close
									{(hasPrevious || hasNext) && (
										<>
											{" · "}
											<span className="text-white/70">
												←
											</span>{" "}
											/{" "}
											<span className="text-white/70">
												→
											</span>{" "}
											to navigate
										</>
									)}
								</div>
							</div>

							{/* Metadata panel, offset so the inspector (which sits above
							    this layer) never covers it */}
							<AnimatePresence>
								{showMetadata && file && (
									<MetadataPanel
										key={file.id}
										file={file}
										offsetRight={inspectorWidth}
										onClose={() => setShowMetadata(false)}
									/>
								)}
							</AnimatePresence>
						</>
					)}
				</motion.div>
			)}
		</AnimatePresence>
	);

	return createPortal(content, portalTarget);
}

export { PREVIEW_LAYER_ID };