import {useEffect} from 'react';
import {useNavigate} from 'react-router-dom';

import {usePlatform} from '../../contexts/PlatformContext';
import {requestReveal} from '../../routes/explorer/pendingReveal';
import {useTabManager} from './useTabManager';

export function explorerUrlForDirectory(directory: string): string {
	const path = {Physical: {device_slug: 'local', path: directory}};
	return `/explorer?path=${encodeURIComponent(JSON.stringify(path))}`;
}

/**
 * Opens folders requested from outside the app (launcher arguments, `xdg-open`,
 * a second launch). The launch that started the app reuses the current tab;
 * later requests open new tabs so the user's work is not replaced.
 */
export function LaunchRequestSync() {
	const platform = usePlatform();
	const navigate = useNavigate();
	const {createTab} = useTabManager();

	useEffect(() => {
		const {takeOpenRequests, onOpenRequests} = platform;
		if (!takeOpenRequests || !onOpenRequests) return;

		let cancelled = false;
		let reuseCurrentTab = true;
		const drain = async () => {
			const requests = await takeOpenRequests();
			if (cancelled) return;
			for (const request of requests) {
				if (request.select) requestReveal(request.select);
				const url = explorerUrlForDirectory(request.directory);
				if (reuseCurrentTab) navigate(url);
				else createTab(undefined, url);
				reuseCurrentTab = false;
			}
			reuseCurrentTab = false;
		};

		let unlisten: (() => void) | undefined;
		void onOpenRequests(() => void drain()).then((stop) => {
			if (cancelled) stop();
			else unlisten = stop;
		});
		void drain();

		return () => {
			cancelled = true;
			unlisten?.();
		};
		// The platform and tab callbacks are stable for the app's lifetime.
		// eslint-disable-next-line react-hooks/exhaustive-deps
	}, []);

	return null;
}
