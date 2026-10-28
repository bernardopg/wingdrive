import {
	ArrowCounterClockwise,
	MagnifyingGlassMinus,
	MagnifyingGlassPlus,
} from "@phosphor-icons/react";
import * as pdfjs from "pdfjs-dist/legacy/build/pdf.mjs";
import type { PDFDocumentProxy, RenderTask } from "pdfjs-dist/legacy/build/pdf.mjs";
import workerUrl from "pdfjs-dist/legacy/build/pdf.worker.min.mjs?url";
import { useEffect, useRef, useState } from "react";

import { useTranslation } from "../../i18n";

pdfjs.GlobalWorkerOptions.workerSrc = workerUrl;

/** Widest a page is drawn at 100% zoom, so wide windows stay readable. */
const MAX_FIT_WIDTH = 960;
const PAGE_GUTTER = 48;
const ZOOM_STEP = 0.25;
const MIN_ZOOM = 0.5;
const MAX_ZOOM = 4;

interface PdfViewerProps {
	src: string;
}

/**
 * Renders a PDF page by page with pdf.js.
 *
 * No webview the app runs in shows PDFs inline (WebKitGTK has no viewer), so
 * the bytes are fetched once and each page is drawn to a canvas only while it
 * is near the viewport, which keeps long documents cheap.
 */
export function PdfViewer({ src }: PdfViewerProps) {
	const { t } = useTranslation("inspector");
	const containerRef = useRef<HTMLDivElement>(null);
	const [loaded, setLoaded] = useState<{
		src: string;
		doc: PDFDocumentProxy;
		aspect: number;
	} | null>(null);
	const [failedSrc, setFailedSrc] = useState<string | null>(null);
	const [zoom, setZoom] = useState(1);
	const [containerWidth, setContainerWidth] = useState(0);

	useEffect(() => {
		const element = containerRef.current;
		if (!element) return;
		const observer = new ResizeObserver(([entry]) => {
			if (entry) setContainerWidth(entry.contentRect.width);
		});
		observer.observe(element);
		return () => observer.disconnect();
	}, []);

	useEffect(() => {
		const controller = new AbortController();
		let task: ReturnType<typeof pdfjs.getDocument> | null = null;

		fetch(src, { signal: controller.signal })
			.then((response) => {
				if (!response.ok) throw new Error(`HTTP ${response.status}`);
				return response.arrayBuffer();
			})
			.then((bytes) => {
				task = pdfjs.getDocument({ data: new Uint8Array(bytes) });
				return task.promise;
			})
			.then(async (doc) => {
				const first = await doc.getPage(1);
				const viewport = first.getViewport({ scale: 1 });
				if (controller.signal.aborted) return;
				setLoaded({ src, doc, aspect: viewport.height / viewport.width });
				setZoom(1);
			})
			.catch((error) => {
				if (controller.signal.aborted) return;
				console.error("[PdfViewer] Cannot open document", error);
				setFailedSrc(src);
			});

		return () => {
			controller.abort();
			// Destroying the loading task also releases the document and its worker.
			task?.destroy();
		};
	}, [src]);

	const current = loaded?.src === src ? loaded : null;
	const pageWidth =
		Math.max(Math.min(containerWidth - PAGE_GUTTER, MAX_FIT_WIDTH), 0) * zoom;

	if (failedSrc === src) {
		return (
			<div className="flex h-full w-full items-center justify-center p-8">
				<div className="text-sm font-medium text-ink-dull">
					{t("inspector.preview.documentLoadFailed")}
				</div>
			</div>
		);
	}

	return (
		<div className="relative h-full w-full">
			<div ref={containerRef} className="h-full w-full overflow-auto">
				{current && pageWidth > 0 ? (
					<div className="flex w-max min-w-full flex-col items-center gap-4 px-6 py-6">
						{Array.from({ length: current.doc.numPages }, (_, index) => (
							<PdfPage
								key={index}
								doc={current.doc}
								pageNumber={index + 1}
								width={pageWidth}
								defaultAspect={current.aspect}
							/>
						))}
					</div>
				) : (
					<div className="flex h-full items-center justify-center">
						<div className="animate-pulse text-sm text-ink-dull">
							{t("inspector.loading")}
						</div>
					</div>
				)}
			</div>

			{current && (
				<>
					<div className="absolute left-4 top-4 z-10 rounded-lg bg-app-box/80 px-3 py-1.5 text-sm font-medium text-ink backdrop-blur-xl">
						{t("inspector.preview.pageCount", {
							count: current.doc.numPages,
						})}
						{zoom !== 1 && ` · ${Math.round(zoom * 100)}%`}
					</div>

					<div className="absolute right-4 top-4 z-10 flex flex-col gap-2">
						<button
							onClick={() =>
								setZoom((z) => Math.min(MAX_ZOOM, z + ZOOM_STEP))
							}
							className="rounded-lg bg-app-box/80 p-2 text-ink backdrop-blur-xl transition-colors hover:bg-app-hover"
							title={t("inspector.preview.zoomIn")}
						>
							<MagnifyingGlassPlus size={20} weight="bold" />
						</button>
						<button
							onClick={() =>
								setZoom((z) => Math.max(MIN_ZOOM, z - ZOOM_STEP))
							}
							className="rounded-lg bg-app-box/80 p-2 text-ink backdrop-blur-xl transition-colors hover:bg-app-hover"
							title={t("inspector.preview.zoomOut")}
						>
							<MagnifyingGlassMinus size={20} weight="bold" />
						</button>
						{zoom !== 1 && (
							<button
								onClick={() => setZoom(1)}
								className="rounded-lg bg-app-box/80 p-2 text-ink backdrop-blur-xl transition-colors hover:bg-app-hover"
								title={t("inspector.preview.resetZoom")}
							>
								<ArrowCounterClockwise size={20} weight="bold" />
							</button>
						)}
					</div>
				</>
			)}
		</div>
	);
}

interface PdfPageProps {
	doc: PDFDocumentProxy;
	pageNumber: number;
	width: number;
	/** Height/width ratio used until this page's own size is known. */
	defaultAspect: number;
}

function PdfPage({ doc, pageNumber, width, defaultAspect }: PdfPageProps) {
	const frameRef = useRef<HTMLDivElement>(null);
	const canvasRef = useRef<HTMLCanvasElement>(null);
	const [visible, setVisible] = useState(false);
	const [aspect, setAspect] = useState(defaultAspect);

	useEffect(() => {
		const frame = frameRef.current;
		if (!frame) return;
		// Draw a screen ahead so pages are ready before they scroll in, and drop
		// the canvas once they are far away to bound memory on long documents.
		const observer = new IntersectionObserver(
			([entry]) => setVisible(!!entry?.isIntersecting),
			{ rootMargin: "100% 0px" },
		);
		observer.observe(frame);
		return () => observer.disconnect();
	}, []);

	useEffect(() => {
		const canvas = canvasRef.current;
		if (!visible || !canvas) return;

		let cancelled = false;
		let task: RenderTask | null = null;

		doc.getPage(pageNumber)
			.then((page) => {
				if (cancelled) return;
				const base = page.getViewport({ scale: 1 });
				setAspect(base.height / base.width);

				const pixelRatio = window.devicePixelRatio || 1;
				const viewport = page.getViewport({
					scale: (width / base.width) * pixelRatio,
				});
				canvas.width = Math.floor(viewport.width);
				canvas.height = Math.floor(viewport.height);

				task = page.render({ canvas, viewport });
				return task.promise;
			})
			.catch((error) => {
				if (cancelled || error?.name === "RenderingCancelledException") return;
				console.error("[PdfViewer] Cannot render page", pageNumber, error);
			});

		return () => {
			cancelled = true;
			task?.cancel();
		};
	}, [visible, doc, pageNumber, width]);

	return (
		<div
			ref={frameRef}
			className="shrink-0 overflow-hidden rounded-sm bg-white shadow-lg"
			style={{ width, height: width * aspect }}
		>
			{visible && (
				<canvas
					ref={canvasRef}
					className="block"
					style={{ width, height: width * aspect }}
				/>
			)}
		</div>
	);
}
