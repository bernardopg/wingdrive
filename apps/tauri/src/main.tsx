import { invoke } from '@tauri-apps/api/core';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { ErrorBoundary, explorerUrlForDirectory } from '@wingdrive/interface';
import React from 'react';
import ReactDOM from 'react-dom/client';
import App from './App';
import './index.css';

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
