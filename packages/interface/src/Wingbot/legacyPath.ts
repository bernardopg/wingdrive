/**
 * Maps a pre-rename `/spacebot` URL to its `/wingbot` equivalent.
 *
 * Tabs, deep links and bookmarks saved before BRAND-002 still point at
 * `/spacebot/...`; they land on the same screen instead of a 404.
 */
export function legacyWingbotPath(pathname: string): string {
	return pathname.replace(/^\/spacebot(?=\/|$)/, '/wingbot');
}
