import { memo, useCallback } from "react";
import { useExplorer } from "../../routes/explorer";
import { useSelection } from "../../routes/explorer/SelectionContext";
import { QuickPreviewFullscreen } from "./QuickPreviewFullscreen";

/**
 * QuickPreviewController - Handles QuickPreview with navigation
 *
 * Isolated component that reads selection state for prev/next navigation.
 * Only re-renders when quickPreviewFileId changes, not on every selection change.
 */
export const QuickPreviewController = memo(function QuickPreviewController({
	sidebarWidth,
	inspectorWidth,
}: {
	sidebarWidth: number;
	inspectorWidth: number;
}) {
	const { quickPreviewFileId, closeQuickPreview, currentFiles } =
		useExplorer();
	const { selectFile } = useSelection();

	const currentIndex = currentFiles.findIndex(
		(f) => f.id === quickPreviewFileId,
	);
	const hasPrevious = currentIndex > 0;
	const hasNext = currentIndex >= 0 && currentIndex < currentFiles.length - 1;

	// Stable identities: the slideshow timer in the preview restarts whenever
	// these change, so inline closures would stall auto-advance.
	const handleNext = useCallback(() => {
		const next = currentFiles[currentIndex + 1];
		if (next) selectFile(next, currentFiles, false, false);
	}, [currentFiles, currentIndex, selectFile]);

	const handlePrevious = useCallback(() => {
		const previous = currentFiles[currentIndex - 1];
		if (previous) selectFile(previous, currentFiles, false, false);
	}, [currentFiles, currentIndex, selectFile]);

	// Hooks above run unconditionally; bail out after them.
	if (!quickPreviewFileId) return null;

	return (
		<QuickPreviewFullscreen
			fileId={quickPreviewFileId}
			isOpen={!!quickPreviewFileId}
			onClose={closeQuickPreview}
			onNext={handleNext}
			onPrevious={handlePrevious}
			hasPrevious={hasPrevious}
			hasNext={hasNext}
			sidebarWidth={sidebarWidth}
			inspectorWidth={inspectorWidth}
		/>
	);
});