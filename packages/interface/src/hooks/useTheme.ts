import { useEffect, useSyncExternalStore } from 'react';
import { toLightIcon } from '@wingdrive/assets/util';
import { useCoreQuery, useWingDriveClient } from '../contexts/WingDriveContext';
import { applyLanguage } from '../i18n';

const THEME_CLASS_MAP: Record<string, string> = {
	light: 'vanilla-theme',
	dark: '', // default, no class needed
	midnight: 'midnight-theme',
	noir: 'noir-theme',
	slate: 'slate-theme',
	nord: 'nord-theme',
	mocha: 'mocha-theme',
	't3-code': 't3-code-theme',
	't3-chat': 't3-chat-theme',
	grove: 'grove-theme',
	ocean: 'ocean-theme',
	ember: 'ember-theme',
	iris: 'iris-theme',
	catppuccin: 'catppuccin-theme',
	gruvbox: 'gruvbox-theme',
	solarized: 'solarized-theme',
	'rose-pine': 'rose-pine-theme',
	dracula: 'dracula-theme',
	'tokyo-night': 'tokyo-night-theme',
};

function resolveTheme(theme: string) {
	if (theme !== 'system') return theme;
	if (typeof window === 'undefined') return 'dark';
	return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
}

/** Class that scopes a subtree to a theme's colors, for previews. */
export function themePreviewClass(theme: string) {
	return THEME_CLASS_MAP[resolveTheme(theme)] || 'dark-theme';
}

export function applyTheme(theme: string) {
	const themeClass = THEME_CLASS_MAP[resolveTheme(theme)];
	const classList = document.documentElement.classList;
	classList.remove(...Object.values(THEME_CLASS_MAP).filter(Boolean));
	// Dark is the default and needs no class
	if (themeClass) classList.add(themeClass);
}

export function useTheme() {
	const client = useWingDriveClient();
	const { data: config, refetch } = useCoreQuery({ type: 'config.app.get', input: null as any });

	// Apply theme on mount and when config changes
	useEffect(() => {
		const theme = config?.preferences?.theme || 'system';
		applyTheme(theme);
	}, [config?.preferences?.theme]);

	useEffect(() => {
		applyLanguage(config?.preferences?.language || 'en');
	}, [config?.preferences?.language]);

	// Listen for config change events from Rust (following useJobs pattern)
	useEffect(() => {
		if (!client) return;

		console.log('[useTheme] Setting up ConfigChanged subscription');
		let unsubscribe: (() => void) | undefined;
		let isCancelled = false;

		const handleEvent = (event: any) => {
			if ('ConfigChanged' in event) {
				console.log('[useTheme] Received ConfigChanged event:', event.ConfigChanged);
				const configEvent = event.ConfigChanged;
				if (configEvent?.field === 'theme' || configEvent?.field === 'language') {
					// Refetch config; the effects above apply the new theme and language
					refetch();
				}
			}
		};

		const filter = {
			event_types: ['ConfigChanged']
		};

		client.subscribeFiltered(filter, handleEvent).then((unsub) => {
			console.log('[useTheme] ConfigChanged subscription created');
			if (isCancelled) {
				console.log('[useTheme] Subscription was cancelled during setup, cleaning up');
				unsub();
			} else {
				unsubscribe = unsub;
			}
		});

		return () => {
			console.log('[useTheme] Cleaning up ConfigChanged subscription');
			isCancelled = true;
			unsubscribe?.();
		};
	}, [client]);

	// Listen for system theme changes when theme is set to "system"
	useEffect(() => {
		const theme = config?.preferences?.theme || 'system';

		if (theme !== 'system') return;

		const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');

		const handleChange = (e: MediaQueryListEvent) => {
			const resolvedTheme = e.matches ? 'dark' : 'light';
			applyTheme(resolvedTheme);
		};

		mediaQuery.addEventListener('change', handleChange);

		return () => mediaQuery.removeEventListener('change', handleChange);
	}, [config?.preferences?.theme]);
}

function readIsDark() {
	if (typeof document === 'undefined') return true;
	// Themes are CSS classes (some switch with the system scheme), so measure the
	// resolved app background instead of mapping theme ids to light or dark.
	const probe = document.createElement('div');
	probe.style.cssText = 'position:absolute;visibility:hidden;background:var(--color-app)';
	document.body.appendChild(probe);
	const [r = 0, g = 0, b = 0] = (getComputedStyle(probe).backgroundColor.match(/[\d.]+/g) ?? []).map(Number);
	probe.remove();
	return 0.2126 * r + 0.7152 * g + 0.0722 * b < 128;
}

// Measured once per theme change, not per render: Explorer renders hundreds of icons
let isDarkCache: boolean | null = null;
const getIsDark = () => (isDarkCache ??= readIsDark());

function subscribeTheme(notify: () => void) {
	const onChange = () => {
		isDarkCache = readIsDark();
		notify();
	};
	const observer = new MutationObserver(onChange);
	observer.observe(document.documentElement, {attributes: true, attributeFilter: ['class']});
	const media = window.matchMedia('(prefers-color-scheme: dark)');
	media.addEventListener('change', onChange);
	return () => {
		observer.disconnect();
		media.removeEventListener('change', onChange);
	};
}

/** Whether the active theme has a dark background; follows theme and system changes. */
export function useIsDarkTheme() {
	return useSyncExternalStore(subscribeTheme, getIsDark, () => true);
}

/** Picks the light variant of an icon URL when the active theme is light. */
export function useThemedIcon() {
	const isDark = useIsDarkTheme();
	return (url: string) => (isDark ? url : toLightIcon(url));
}
