export function formatMode(mode: number): string {
	return (mode & 0o7777).toString(8).padStart(4, '0');
}

/** Parses an octal mode such as "755" or "0644"; null when invalid. */
export function parseMode(text: string): number | null {
	const trimmed = text.trim();
	if (!/^[0-7]{3,4}$/.test(trimmed)) return null;
	return parseInt(trimmed, 8);
}
