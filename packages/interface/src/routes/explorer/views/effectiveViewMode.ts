import type {ViewMode} from '../context';

/**
 * View that actually renders for a stored view mode. Knowledge is a
 * development prototype, so a tab that persisted it falls back to the grid in
 * production builds instead of showing an unfinished screen.
 */
export function effectiveViewMode(
	viewMode: ViewMode,
	isDevelopment: boolean
): ViewMode {
	if (viewMode === 'knowledge' && !isDevelopment) return 'grid';
	return viewMode;
}
