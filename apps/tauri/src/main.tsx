import { invoke } from '@tauri-apps/api/core';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { ErrorBoundary, explorerUrlForDirectory, i18n } from '@wingdrive/interface';
import React from 'react';
import ReactDOM from 'react-dom/client';
import App from './App';
import { configureRendering } from './lib/rendering';
import './index.css';
import './rendering.css';

configureRendering(
	document.documentElement,
	navigator.platform,
	getCurrentWebviewWindow().label
);

// The tray is native and built in English; rename it whenever the UI language changes.
if (getCurrentWebviewWindow().label === 'main') {
	i18n.on('languageChanged', () => {
		invoke('set_tray_labels', {
			labels: {
				open: i18n.t('common:tray.open'),
				new_tab: i18n.t('common:tray.newTab'),
				quit: i18n.t('common:tray.quit')
			}
		}).catch(() => {
			// Hosts without a tray (background mode off on other platforms) have nothing to rename.
		});
	});
}

// A launch with a folder opens it as the first route, so the tab manager
// starts there instead of painting the overview and navigating afterwards.
async function openLaunchFolder() {
	if (getCurrentWebviewWindow().label !== 'main') return;
	try {
		const request = await invoke<{ directory: string } | null>('take_initial_open_request');
		if (request) window.history.replaceState(null, '', explorerUrlForDirectory(request.directory));
	} catch {
		// Older hosts without the command fall back to LaunchRequestSync.
	}
}

void openLaunchFolder().then(() => {
	ReactDOM.createRoot(document.getElementById('root')!).render(
		<React.StrictMode>
			<ErrorBoundary>
				<App />
			</ErrorBoundary>
		</React.StrictMode>
	);
});
