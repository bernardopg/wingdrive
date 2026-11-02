export function reorderItems<T extends {id: string}>(items: T[], activeId: string, overId: string): T[] | null {
	const from = items.findIndex((item) => item.id === activeId);
	const to = items.findIndex((item) => item.id === overId);
	if (from < 0 || to < 0 || from === to) return null;
	const reordered = [...items];
	const [item] = reordered.splice(from, 1);
	reordered.splice(to, 0, item);
	return reordered;
}

export function orderDynamicItems<T extends {id: string}>(items: T[], saved: string[]): T[] {
	const positions = new Map(saved.map((id, index) => [id, index]));
	return [...items].sort((a, b) => (positions.get(a.id) ?? Infinity) - (positions.get(b.id) ?? Infinity));
}

export function rememberDynamicOrder(visible: string[], saved: string[]): string[] {
	const present = new Set(visible);
	let index = 0;
	const merged = saved.map((id) => present.has(id) ? visible[index++] : id);
	return [...merged, ...visible.slice(index)];
}

export function reorderLayoutItems<T extends {id: string}, G extends {group: {id: string}; items: T[]}, L extends {space_items: T[]; groups: G[]}>(layout: L, activeId: string, overId: string): {layout: L; groupId: string | null; itemIds: string[]} | null {
	const group = layout.groups.find((entry) => entry.items.some((item) => item.id === activeId));
	const reordered = reorderItems(group ? group.items : layout.space_items, activeId, overId);
	if (!reordered) return null;
	return {
		layout: group
			? {...layout, groups: layout.groups.map((entry) => entry === group ? {...entry, items: reordered} : entry)}
			: {...layout, space_items: reordered},
		groupId: group?.group.id ?? null,
		itemIds: reordered.map((item) => item.id)
	};
}

export function sidebarSections(groupIds: string[], treeId: string, after: string | null | undefined): string[] {
	const result = [...groupIds];
	const index = after === null ? 0 : after === undefined ? result.length : result.indexOf(after) + 1;
	result.splice(index < 1 && after != null ? result.length : index, 0, treeId);
	return result;
}

export function isSidebarSortable(data: {label?: unknown; sortable?: unknown} | undefined): boolean {
	return data?.label != null && data.sortable != null;
}
