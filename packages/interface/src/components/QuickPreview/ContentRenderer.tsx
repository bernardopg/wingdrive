import type { File } from "@wingdrive/ts-client";
import {
	fileDisplayName,
	fileDisplaySize,
	getContentKind,
} from "@wingdrive/ts-client";
import { File as FileComponent } from "../../routes/explorer/File";
import { formatBytes } from "../../routes/explorer/utils";
import { usePlatform } from "../../contexts/PlatformContext";
import { useLibraryMutation } from "../../contexts/WingDriveContext";
import { useServer } from "../../contexts/ServerContext";
import { useTranslation } from "../../i18n";
import {
	useState,
	useEffect,
	useRef,
	useCallback,
	lazy,
	Suspense,
} from "react";
import {
	MagnifyingGlassPlus,
	MagnifyingGlassMinus,
	ArrowCounterClockwise,
	Cube,
} from "@phosphor-icons/react";
import { VideoPlayer } from "./VideoPlayer";
import type {
	VideoControlsState,
	VideoControlsCallbacks,
} from "./VideoControls";
import { AudioPlayer } from "./AudioPlayer";
import { useZoomPan } from "./useZoomPan";
import { TextViewer } from "./TextViewer";
import { WithPrismTheme } from "./prism";
import { sounds } from "@wingdrive/assets/sounds";
import { CircleButton } from "@wingdrive/primitives";
import { DirectoryPreview } from "./DirectoryPreview";

const MeshViewer = lazy(() =>
	import("./MeshViewer").then((m) => ({ default: m.MeshViewer })),
);
const MeshViewerUI = lazy(() =>
	import("./MeshViewer").then((m) => ({ default: m.MeshViewerUI })),
);
const PdfViewer = lazy(() =>
	import("./PdfViewer").then((m) => ({ default: m.PdfViewer })),
);

/** Largest file of unknown kind that is sniffed to see if it is plain text. */
const TEXT_SNIFF_MAX_BYTES = 2 * 1024 * 1024;

interface ContentRendererProps {
	file: File;
	onZoomChange?: (isZoomed: boolean) => void;
	onVideoControlsStateChange?: (state: VideoControlsState) => void;
	onShowVideoControlsChange?: (show: boolean) => void;
	getVideoCallbacks?: (callbacks: VideoControlsCallbacks) => void;
}

function ImageRenderer({ file, onZoomChange }: ContentRendererProps) {
	const { t } = useTranslation("inspector");
	const platform = usePlatform();
	const { buildSidecarUrl } = useServer();
	const containerRef = useRef<HTMLDivElement>(null);
	const [originalLoaded, setOriginalLoaded] = useState(false);
	const [originalUrl, setOriginalUrl] = useState<string | null>(null);
	const [shouldLoadOriginal, setShouldLoadOriginal] = useState(false);
	const [showSplat, setShowSplat] = useState(false);
	const [splatLoaded, setSplatLoaded] = useState(false);
	const { zoom, zoomIn, zoomOut, reset, isZoomed, transform } =
		useZoomPan(containerRef as React.RefObject<HTMLElement>);

	// Track MeshViewer controls state
	const [meshControls, setMeshControls] = useState<{
		autoRotate: boolean;
		swayAmount: number;
		swaySpeed: number;
		cameraDistance: number;
		isGaussianSplat: boolean;
		onResetFocalPoint?: () => void;
	}>({
		autoRotate: true,
		swayAmount: 0.25,
		swaySpeed: 0.5,
		cameraDistance: 0.5,
		isGaussianSplat: false,
	});

	// Get a stable identifier for the image file itself
	const imageFileId = file.content_identity?.uuid || file.id;

	// Check if Gaussian splat sidecar exists and get URL
	const splatSidecar = file.sidecars?.find(
		(s) => s.kind === "gaussian_splat" && s.format === "ply",
	);
	const hasSplat = !!splatSidecar;

	// Build sidecar URL for the splat
	const splatUrl =
		hasSplat && file.content_identity?.uuid
			? buildSidecarUrl(
					file.content_identity.uuid,
					splatSidecar!.kind,
					splatSidecar!.variant,
					splatSidecar!.format,
				)
			: null;

	// Notify parent of zoom state changes
	useEffect(() => {
		onZoomChange?.(isZoomed);
	}, [isZoomed, onZoomChange]);

	// Reset and defer original loading by 50ms to ensure thumbnail renders first
	useEffect(() => {
		setShouldLoadOriginal(false);
		setOriginalLoaded(false);
		setOriginalUrl(null);
		setShowSplat(false);
		setSplatLoaded(false);

		const timer = setTimeout(() => {
			setShouldLoadOriginal(true);
		}, 50);

		return () => clearTimeout(timer);
	}, [imageFileId]);

	useEffect(() => {
		if (!shouldLoadOriginal || !platform.convertFileSrc) {
			return;
		}

		const sdPath = file.wing_path as any;
		const physicalPath = sdPath?.Physical?.path;

		if (!physicalPath) {
			console.log(
				"[ImageRenderer] No physical path available, wing_path:",
				file.wing_path,
			);
			return;
		}

		const url = platform.convertFileSrc(physicalPath);
		console.log(
			"[ImageRenderer] Loading original from:",
			physicalPath,
			"-> URL:",
			url,
		);
		setOriginalUrl(url);
	}, [shouldLoadOriginal, imageFileId, file.wing_path, platform]);

	// Get highest resolution thumbnail first
	const getHighestResThumbnail = () => {
		const thumbnails =
			file.sidecars?.filter((s) => s.kind === "thumb") || [];
		if (thumbnails.length === 0) return null;

		const highest = thumbnails.sort((a, b) => {
			const aSize = parseInt(
				a.variant.split("x")[0]?.replace(/\D/g, "") || "0",
			);
			const bSize = parseInt(
				b.variant.split("x")[0]?.replace(/\D/g, "") || "0",
			);
			return bSize - aSize;
		})[0];

		const contentUuid = file.content_identity?.uuid;
		if (!contentUuid) return null;

		return buildSidecarUrl(
			contentUuid,
			highest.kind,
			highest.variant,
			highest.format,
		);
	};

	const thumbnailUrl = getHighestResThumbnail();

	// Stable callback to prevent re-renders that would reinitialize MeshViewer
	const handleSplatLoaded = useCallback(() => {
		console.log(
			"[ImageRenderer] Splat is fully visible, hiding image overlay",
		);
		setSplatLoaded(true);
		sounds.splat();
	}, []);

	// Persistent pre-mounted shimmer wrapper (stays mounted regardless of view)
	// Disabled for now
	const persistentShimmer = null;

	// Render splat view separately (not overlayed)
	if (showSplat && hasSplat && splatUrl) {
		return (
			<>
				{/* Persistent shimmer - always ready */}
				{persistentShimmer}

				{/* Fullscreen canvas layer */}
				<Suspense
					fallback={
						<div className="relative w-full h-full z-10 pointer-events-none bg-black flex items-center justify-center">
							{thumbnailUrl && (
								<img
									src={thumbnailUrl}
									alt={file.name}
									className="w-full h-full object-contain"
									draggable={false}
								/>
							)}
							{originalUrl && (
								<img
									src={originalUrl}
									alt={file.name}
									className="absolute inset-0 w-full h-full object-contain transition-opacity duration-300"
									style={{
										opacity: originalLoaded ? 1 : 0,
									}}
									draggable={false}
								/>
							)}
						</div>
					}
				>
					<MeshViewer
						file={file}
						splatUrl={splatUrl}
						onSplatLoaded={handleSplatLoaded}
						autoRotate={meshControls.autoRotate}
						swayAmount={meshControls.swayAmount}
						swaySpeed={meshControls.swaySpeed}
						cameraDistance={meshControls.cameraDistance}
						onControlsChange={setMeshControls}
					/>
				</Suspense>

				{/* Image overlay - shown during splat loading, fades out when loaded */}
				{!splatLoaded && (
					<div className="relative w-full h-full z-10 pointer-events-none bg-black flex items-center justify-center">
						{/* Thumbnail (always available) */}
						{thumbnailUrl && (
							<img
								src={thumbnailUrl}
								alt={file.name}
								className="w-full h-full object-contain"
								draggable={false}
							/>
						)}
						{/* Original image (fades in over thumbnail when ready) */}
						{originalUrl && (
							<img
								src={originalUrl}
								alt={file.name}
								className="absolute inset-0 w-full h-full object-contain transition-opacity duration-300"
								style={{ opacity: originalLoaded ? 1 : 0 }}
								draggable={false}
							/>
						)}
					</div>
				)}

				{/* Safe area UI overlay */}
				<div className="relative w-full h-full z-30 pointer-events-none">
					{/* Toggle button */}
					<div className="absolute top-4 left-4 pointer-events-auto">
						<CircleButton
							icon={Cube}
							onClick={() => {
								setShowSplat(false);
								setSplatLoaded(false);
							}}
title={t("inspector.preview.showImage")}
							active={true}
							activeAccent={true}
						/>
					</div>

					{/* MeshViewer UI controls */}
					<Suspense fallback={null}>
						<MeshViewerUI
							autoRotate={meshControls.autoRotate}
							setAutoRotate={(v) =>
								setMeshControls((c) => ({
									...c,
									autoRotate: v,
								}))
							}
							swayAmount={meshControls.swayAmount}
							setSwayAmount={(v) =>
								setMeshControls((c) => ({
									...c,
									swayAmount: v,
								}))
							}
							swaySpeed={meshControls.swaySpeed}
							setSwaySpeed={(v) =>
								setMeshControls((c) => ({ ...c, swaySpeed: v }))
							}
							cameraDistance={meshControls.cameraDistance}
							setCameraDistance={(v) =>
								setMeshControls((c) => ({
									...c,
									cameraDistance: v,
								}))
							}
							isGaussianSplat={meshControls.isGaussianSplat}
							onResetFocalPoint={meshControls.onResetFocalPoint}
						/>
					</Suspense>
				</div>
			</>
		);
	}

	// Render image view with zoom/pan
	return (
		<div
			ref={containerRef}
			className={`relative w-full h-full flex items-center justify-center ${isZoomed ? "overflow-visible" : "overflow-hidden"}`}
		>
			{/* Persistent shimmer - always ready */}
			{persistentShimmer}

			{/* Splat Toggle (top-left) */}
			{hasSplat && (
				<div className="absolute top-4 left-4 z-10">
					<CircleButton
						icon={Cube}
						onClick={() => {
							sounds.splatTrigger();
							setShowSplat(true);
						}}
title={t("inspector.preview.showSplat")}
					/>
				</div>
			)}

			{/* Zoom Controls */}
			<div className="absolute top-4 right-4 z-10 flex flex-col gap-2">
				<button
					onClick={zoomIn}
					className="rounded-lg bg-app-box/80 p-2 text-ink backdrop-blur-xl transition-colors hover:bg-app-hover"
title={t("inspector.preview.zoomIn")}
				>
					<MagnifyingGlassPlus size={20} weight="bold" />
				</button>
				<button
					onClick={zoomOut}
					className="rounded-lg bg-app-box/80 p-2 text-ink backdrop-blur-xl transition-colors hover:bg-app-hover"
title={t("inspector.preview.zoomOut")}
				>
					<MagnifyingGlassMinus size={20} weight="bold" />
				</button>
				{zoom > 1 && (
					<button
						onClick={reset}
						className="rounded-lg bg-app-box/80 p-2 text-ink backdrop-blur-xl transition-colors hover:bg-app-hover"
title={t("inspector.preview.resetZoom")}
					>
						<ArrowCounterClockwise size={20} weight="bold" />
					</button>
				)}
			</div>

			{/* Zoom level indicator */}
			{zoom > 1 && (
				<div className="absolute top-4 left-4 z-10 rounded-lg bg-app-box/80 px-3 py-1.5 text-sm font-medium text-ink backdrop-blur-xl">
					{Math.round(zoom * 100)}%
				</div>
			)}

			{/* Image container with zoom/pan transform */}
			<div
				className="relative w-full h-full flex items-center justify-center"
				style={transform}
			>
				{/* High-res thumbnail (always rendered as background layer) */}
				{thumbnailUrl && (
					<img
						src={thumbnailUrl}
						alt={file.name}
						className="w-full h-full object-contain"
						draggable={false}
					/>
				)}

				{/* Original image (loads async, fades in over thumbnail when ready) */}
				{originalUrl && (
					<img
						src={originalUrl}
						alt={file.name}
						className="absolute inset-0 w-full h-full object-contain"
						style={{
							opacity: originalLoaded ? 1 : 0,
							transition: "opacity 0.3s",
						}}
						onLoad={() => setOriginalLoaded(true)}
						onError={(e) =>
							console.error(
								"[ImageRenderer] Original failed to load:",
								e,
							)
						}
						draggable={false}
					/>
				)}
			</div>
		</div>
	);
}

/**
 * Resolves a playable URL for a media file.
 *
 * The asset protocol works for images because the webview fetches them, but
 * video and audio are decoded by the platform media stack (GStreamer on Linux),
 * which fetches the URL on its own and cannot see the webview's custom scheme.
 * Platforms that expose a streaming endpoint use it; the rest fall back to the
 * asset protocol.
 */
async function resolveMediaSrc(
	platform: { getMediaSrc?: (p: string) => Promise<string>; convertFileSrc?: (p: string) => string },
	physicalPath: string,
): Promise<string | null> {
	if (platform.getMediaSrc) {
		try {
			return await platform.getMediaSrc(physicalPath);
		} catch (err) {
			console.error("[media] streaming URL unavailable, falling back", err);
		}
	}

	return platform.convertFileSrc?.(physicalPath) ?? null;
}

/**
 * URL the webview can `fetch` the file's bytes from, or `null` while it resolves.
 *
 * `fetch` cannot read `asset:` URLs (the CSP's `connect-src` does not list them
 * and the asset scope stops at `$HOME`), so this prefers the loopback stream
 * that media playback uses.
 */
function useFileFetchSrc(file: File): string | null {
	const platform = usePlatform();
	const physicalPath =
		"Physical" in file.wing_path ? file.wing_path.Physical.path : null;
	const [resolved, setResolved] = useState<{
		path: string;
		url: string;
	} | null>(null);

	useEffect(() => {
		if (!physicalPath) return;
		let cancelled = false;

		resolveMediaSrc(platform, physicalPath).then((url) => {
			if (!cancelled && url) setResolved({ path: physicalPath, url });
		});

		return () => {
			cancelled = true;
		};
	}, [physicalPath, platform]);

	return resolved?.path === physicalPath ? resolved.url : null;
}

/**
 * Whether a file of unknown kind is readable text: `null` while checking.
 *
 * Small files are fetched and rejected when they contain NUL bytes or are not
 * valid UTF-8, which covers dotfiles, `LICENSE`, and extensionless configs.
 */
function useIsPlainText(src: string | null, size: number | null): boolean | null {
	const sniffable = !!src && size !== null && size <= TEXT_SNIFF_MAX_BYTES;
	const [result, setResult] = useState<{ src: string; isText: boolean } | null>(
		null,
	);

	useEffect(() => {
		if (!sniffable || !src) return;
		const controller = new AbortController();

		fetch(src, { signal: controller.signal })
			.then((response) => {
				if (!response.ok) throw new Error(`HTTP ${response.status}`);
				return response.arrayBuffer();
			})
			.then((buffer) => {
				const bytes = new Uint8Array(buffer);
				let isText = !bytes.subarray(0, 8192).includes(0);
				if (isText) {
					try {
						new TextDecoder("utf-8", { fatal: true }).decode(bytes);
					} catch {
						isText = false;
					}
				}
				setResult({ src, isText });
			})
			.catch(() => {
				if (!controller.signal.aborted) setResult({ src, isText: false });
			});

		return () => controller.abort();
	}, [sniffable, src]);

	if (!sniffable) return false;
	return result?.src === src ? result.isText : null;
}

function VideoRenderer({
	file,
	onZoomChange,
	onVideoControlsStateChange,
	onShowVideoControlsChange,
	getVideoCallbacks,
}: ContentRendererProps) {
	const platform = usePlatform();
	const [videoUrl, setVideoUrl] = useState<string | null>(null);
	const [shouldLoadVideo, setShouldLoadVideo] = useState(false);

	// Get a stable identifier for the video file itself
	const videoFileId = file.content_identity?.uuid || file.id;

	// Reset and defer video loading by 50ms to ensure thumbnail renders first
	useEffect(() => {
		setShouldLoadVideo(false);
		setVideoUrl(null);

		const timer = setTimeout(() => {
			setShouldLoadVideo(true);
		}, 50);

		return () => clearTimeout(timer);
	}, [videoFileId]);

	useEffect(() => {
		if (!shouldLoadVideo) return;

		const sdPath = file.wing_path as any;
		const physicalPath = sdPath?.Physical?.path;

		if (!physicalPath) return;

		let cancelled = false;

		resolveMediaSrc(platform, physicalPath).then((url) => {
			if (!cancelled && url) setVideoUrl(url);
		});

		return () => {
			cancelled = true;
		};
	}, [shouldLoadVideo, videoFileId, file.wing_path, platform]);

	if (!videoUrl) {
		return (
			<div className="w-full h-full flex items-center justify-center">
				<FileComponent.Thumb
					file={file}
					size={800}
					className="max-w-full max-h-full"
				/>
			</div>
		);
	}

	return (
		<VideoPlayer
			src={videoUrl}
			file={file}
			onZoomChange={onZoomChange}
			onControlsStateChange={onVideoControlsStateChange}
			onShowControlsChange={onShowVideoControlsChange}
			getCallbacks={getVideoCallbacks}
		/>
	);
}

function AudioRenderer({ file }: ContentRendererProps) {
	const platform = usePlatform();
	const [audioUrl, setAudioUrl] = useState<string | null>(null);
	const [shouldLoadAudio, setShouldLoadAudio] = useState(false);

	// Get a stable identifier for the audio file itself
	const audioFileId = file.content_identity?.uuid || file.id;

	// Reset and defer audio loading by 50ms to ensure thumbnail renders first
	useEffect(() => {
		setShouldLoadAudio(false);
		setAudioUrl(null);

		const timer = setTimeout(() => {
			setShouldLoadAudio(true);
		}, 50);

		return () => clearTimeout(timer);
	}, [audioFileId]);

	useEffect(() => {
		if (!shouldLoadAudio) return;

		const sdPath = file.wing_path as any;
		const physicalPath = sdPath?.Physical?.path;

		if (!physicalPath) return;

		let cancelled = false;

		resolveMediaSrc(platform, physicalPath).then((url) => {
			if (!cancelled && url) setAudioUrl(url);
		});

		return () => {
			cancelled = true;
		};
	}, [shouldLoadAudio, audioFileId, file.wing_path, platform]);

	if (!audioUrl) {
		return (
			<div className="w-full h-full flex items-center justify-center">
				<div className="text-center">
					<FileComponent.Thumb file={file} size={200} />
					<div className="mt-6 text-ink text-lg font-medium">
						{file.name}
					</div>
				</div>
			</div>
		);
	}

	return <AudioPlayer src={audioUrl} file={file} />;
}

function PdfRenderer({ file }: ContentRendererProps) {
	const src = useFileFetchSrc(file);
	const placeholder = (
		<div className="w-full h-full flex items-center justify-center">
			<FileComponent.Thumb file={file} size={200} />
		</div>
	);

	if (!src) return placeholder;

	return (
		<Suspense fallback={placeholder}>
			<PdfViewer src={src} />
		</Suspense>
	);
}

function DocumentRenderer({ file }: ContentRendererProps) {
	if (file.extension?.toLowerCase() === "pdf") {
		return <PdfRenderer file={file} />;
	}

	const displaySize = fileDisplaySize(file);
	return (
		<div className="w-full h-full flex items-center justify-center">
			<div className="text-center">
				<FileComponent.Thumb file={file} size={200} />
				<div className="mt-6 text-ink text-lg font-medium">
					{file.name}
				</div>
				<div className="text-ink-dull text-sm mt-2 capitalize">
					{getContentKind(file) ?? "unknown"}
				</div>
				{displaySize !== null && (
					<div className="text-ink-dull text-xs mt-1">
						{formatBytes(displaySize)}
					</div>
				)}
			</div>
		</div>
	);
}

function TextRenderer({ file }: ContentRendererProps) {
	const { t } = useTranslation("inspector");
	const textUrl = useFileFetchSrc(file);
	const [failedUrl, setFailedUrl] = useState<string | null>(null);
	const handleError = useCallback(() => setFailedUrl(textUrl), [textUrl]);

	const extension = file.extension?.toLowerCase() || undefined;

	if (textUrl && failedUrl === textUrl) {
		return (
			<div className="w-full h-full flex items-center justify-center">
				<div className="text-center">
					<FileComponent.Thumb file={file} size={120} />
					<div className="mt-4 text-ink text-lg font-medium">
						{fileDisplayName(file)}
					</div>
					<div className="text-ink-dull text-sm mt-2">
						{t("inspector.preview.textLoadFailed")}
					</div>
				</div>
			</div>
		);
	}

	if (!textUrl) {
		return (
			<div className="w-full h-full flex items-center justify-center">
				<div className="text-center">
					<FileComponent.Thumb file={file} size={120} />
					<div className="mt-4 text-ink text-lg font-medium">
						{fileDisplayName(file)}
					</div>
<div className="text-ink-dull text-sm mt-2">{t("inspector.loading")}</div>
				</div>
			</div>
		);
	}

	return (
		<>
			<WithPrismTheme />
			<TextViewer
				src={textUrl}
				codeExtension={extension}
				onError={handleError}
				className="w-full h-full overflow-auto bg-app p-4 text-ink"
			/>
		</>
	);
}

/**
 * The file as the renderers can load it: a file inside an archive points at
 * its extracted copy, or is `null` while that copy is being made.
 */
function useOnDiskFile(file: File): File | null {
	const openMember = useLibraryMutation("archive.open_member");
	const physical = "Physical" in file.wing_path ? file.wing_path.Physical : null;
	const inArchive =
		file.kind === "File" && !!physical && !!file.archive;
	const [resolved, setResolved] = useState<File | null>(null);

	useEffect(() => {
		if (!inArchive || !physical) return;
		let cancelled = false;
		openMember
			.mutateAsync({ path: file.wing_path })
			.then(({ path }) => {
				if (!cancelled) {
					setResolved({ ...file, wing_path: { Physical: { ...physical, path } } });
				}
			})
			.catch((error) => console.error("[QuickPreview] Cannot extract from archive", error));
		return () => {
			cancelled = true;
		};
		// Keyed by the file; the mutation object changes every render.
		// eslint-disable-next-line react-hooks/exhaustive-deps
	}, [file.id, inArchive]);

	if (!inArchive) return file;
	return resolved?.id === file.id ? resolved : null;
}

function UnknownRenderer({ file }: ContentRendererProps) {
	const src = useFileFetchSrc(file);
	const isText = useIsPlainText(
		file.kind === "File" ? src : null,
		fileDisplaySize(file),
	);

	if (isText) return <TextRenderer file={file} />;
	return <DefaultRenderer file={file} />;
}

function DefaultRenderer({ file }: ContentRendererProps) {
	const displaySize = fileDisplaySize(file);
	return (
		<div className="w-full h-full flex items-center justify-center">
			<div className="text-center">
				<FileComponent.Thumb file={file} size={200} />
				<div className="mt-6 text-ink text-lg font-medium">
					{file.name}
				</div>
				<div className="text-ink-dull text-sm mt-2 capitalize">
					{getContentKind(file) ?? "unknown"}
				</div>
				{displaySize !== null && (
					<div className="text-ink-dull text-xs mt-1">
						{formatBytes(displaySize)}
					</div>
				)}
			</div>
		</div>
	);
}

export function ContentRenderer({
	file: listed,
	onZoomChange,
	onVideoControlsStateChange,
	onShowVideoControlsChange,
	getVideoCallbacks,
}: ContentRendererProps) {
	const file = useOnDiskFile(listed);
	if (!file) return <DefaultRenderer file={listed} />;

	// Handle directories with grid preview of subdirectories
	if (file.kind === "Directory") {
		return <DirectoryPreview file={file} />;
	}

	const kind = getContentKind(file);

	switch (kind) {
		case "image":
			return <ImageRenderer file={file} onZoomChange={onZoomChange} />;
		case "video":
			return (
				<VideoRenderer
					file={file}
					onZoomChange={onZoomChange}
					onVideoControlsStateChange={onVideoControlsStateChange}
					onShowVideoControlsChange={onShowVideoControlsChange}
					getVideoCallbacks={getVideoCallbacks}
				/>
			);
		case "audio":
			return <AudioRenderer file={file} />;
		case "mesh":
			return (
				<Suspense
					fallback={
						<div className="w-full h-full flex items-center justify-center">
							<FileComponent.Thumb file={file} size={200} />
						</div>
					}
				>
					<MeshViewer file={file} />
				</Suspense>
			);
		case "document":
		case "book":
		case "spreadsheet":
		case "presentation":
			return <DocumentRenderer file={file} />;
		case "text":
		case "code":
		case "config":
			return <TextRenderer file={file} />;
		case "unknown":
			return <UnknownRenderer file={file} />;
		default:
			return <DefaultRenderer file={file} />;
	}
}
