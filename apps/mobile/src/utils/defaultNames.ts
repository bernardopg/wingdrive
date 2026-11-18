/**
 * Names the core seeds in English: the first library and the wing every library starts with.
 * They are shown translated while the user keeps them; a name the user typed is shown as is.
 */
const SEEDED_NAMES = ["My Library", "All Devices"] as const;

export function displayName(name: string, t: (key: string) => string): string {
	return (SEEDED_NAMES as readonly string[]).includes(name) ? t(name) : name;
}
