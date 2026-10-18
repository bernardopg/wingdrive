/** Saturated, mid-light colors that read on both light and dark themes. */
export const TAG_COLORS = [
	'#EF4444',
	'#F97316',
	'#F59E0B',
	'#84CC16',
	'#22C55E',
	'#14B8A6',
	'#06B6D4',
	'#3B82F6',
	'#6366F1',
	'#8B5CF6',
	'#D946EF',
	'#EC4899'
];

/** A color for a new tag, avoiding the ones already in use while any is free. */
export function nextTagColor(used: readonly (string | null | undefined)[] = []): string {
	const taken = new Set(used.map((color) => color?.toUpperCase()));
	const free = TAG_COLORS.filter((color) => !taken.has(color));
	const pool = free.length > 0 ? free : TAG_COLORS;
	return pool[Math.floor(Math.random() * pool.length)];
}
