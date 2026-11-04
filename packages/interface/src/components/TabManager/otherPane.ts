import type {WingPath} from '@wingdrive/ts-client';
import {wingPathFromParam} from '../../routes/explorer/wingPathParam';
import type {PaneId} from './TabManagerContext';

/** The pane across from `pane` in a split view. */
export function otherPane(pane: PaneId): PaneId {
	return pane === 'primary' ? 'secondary' : 'primary';
}

/**
 * The folder an Explorer URL shows, or null for views without one (Overview,
 * Recents, a tag). Only those URLs can take copied or moved files.
 */
export function folderOfUrl(url: string | null): WingPath | null {
	if (!url) return null;
	const query = url.split('?')[1];
	if (!query) return null;
	const raw = new URLSearchParams(query).get('path');
	return raw ? wingPathFromParam(raw) : null;
}
