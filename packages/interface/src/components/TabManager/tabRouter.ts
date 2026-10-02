import { createMemoryRouter, type RouteObject } from "react-router-dom";

export function createTabRouter(
	routes: RouteObject[],
	path: string,
	history: string[] = [path],
	index = history.length - 1,
	onChange: (path: string, history: string[], index: number) => void = () => {},
) {
	let entries: {path: string; key: string}[] = history.map((path) => ({ path, key: crypto.randomUUID() }));
	let currentIndex = Math.max(0, Math.min(index, entries.length - 1));
	const router = createMemoryRouter(routes, {
		initialEntries: entries.map(({path, key}) => ({...parsePath(path), key})),
		initialIndex: currentIndex,
	});
	let lastKey = router.state.location.key;
	router.subscribe((state) => {
		if (state.navigation.state !== "idle" || state.location.key === lastKey) return;
		lastKey = state.location.key;
		const path = state.location.pathname + state.location.search + state.location.hash;
		if (state.historyAction === "POP") {
			currentIndex = entries.findIndex((entry) => entry.key === lastKey);
		} else if (state.historyAction === "REPLACE") {
			entries[currentIndex] = {path, key: lastKey};
		} else {
			entries = [...entries.slice(0, currentIndex + 1), {path, key: lastKey}];
			currentIndex = entries.length - 1;
		}
		onChange(path, entries.map((entry) => entry.path), currentIndex);
	});
	return router;
}

function parsePath(path: string) {
	const url = new URL(path, "http://localhost");
	return {pathname: url.pathname, search: url.search, hash: url.hash};
}
