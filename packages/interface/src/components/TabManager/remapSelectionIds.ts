// Keep per-tab selections stable when a scan-time UUID is reconciled.
export function remapSelectionIds(
	selections: Map<string, string[]>,
	alternateIds: readonly string[],
	persistentId: string,
): Map<string, string[]> {
	if (alternateIds.length === 0) return selections;
	const aliases = new Set(alternateIds);
	const next = new Map(selections);
	let changed = false;
	for (const [tabId, ids] of selections) {
		if (!ids.some((id) => aliases.has(id))) continue;
		const seen = new Set<string>();
		const remapped = ids.map((id) => aliases.has(id) ? persistentId : id)
			.filter((id) => {
				if (seen.has(id)) return false;
				seen.add(id);
				return true;
			});
		next.set(tabId, remapped);
		changed = true;
	}
	return changed ? next : selections;
}
