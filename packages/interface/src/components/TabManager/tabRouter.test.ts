import { expect, test } from "bun:test";
import { createTabRouter } from "./tabRouter";

test("tabs isolate history, preserve duplicate URLs, restore after eviction, and dispose", async () => {
	const routes = [{path: "*", element: null}];
	let saved: [string, string[], number] = ["/a", ["/a"], 0];
	const first = createTabRouter(routes, "/a", undefined, undefined, (...state) => { saved = state; });
	const second = createTabRouter(routes, "/b");
	await first.navigate("/c");
	await first.navigate("/a");
	await first.navigate(-1);
	expect(first.state.location.pathname).toBe("/c");
	expect(second.state.location.pathname).toBe("/b");
	first.dispose();
	const restored = createTabRouter(routes, ...saved);
	await restored.navigate(-1);
	expect(restored.state.location.pathname).toBe("/a");
	await restored.navigate(1);
	expect(restored.state.location.pathname).toBe("/c");
	restored.dispose();
	second.dispose();
	for (let i = 0; i < 100; i++) {
		const router = createTabRouter(routes, "/");
		await router.navigate("/explorer?path=test");
		router.dispose();
	}
});
