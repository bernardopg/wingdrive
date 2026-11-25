import { createHash } from "node:crypto";
import type { Lang } from "./strings";

type AdapterText = { summary: string; description: string };

export type StoreAdapter = AdapterText & {
	id: string;
	name: string;
	version: string;
	author: string;
	license: string;
	categories: string[];
	trust: "official" | "verified" | "community";
	platforms: string[];
	permissions: { read?: string[]; network?: string[]; secrets?: string[]; tools?: string[] };
	source: { type: "local" | "git"; path?: string; repository?: string; commit: string };
	icon: { url: string; sha256: string; png?: { url: string; sha256: string; size: number } };
	iconData?: string;
	i18n?: { pt?: Partial<AdapterText> };
};

type Catalog = { sequence: number; adapters: StoreAdapter[] };

export const REGISTRY = "https://github.com/bernardopg/wingdrive-adapters";
const INDEX = `${REGISTRY}/releases/latest/download/index.json`;
const request = { headers: { "User-Agent": "wingdrive-site-build" } };

let cached: Promise<Catalog> | undefined;

export function catalog(): Promise<Catalog> {
	return (cached ??= loadCatalog());
}

async function loadCatalog(): Promise<Catalog> {
	try {
		const res = await fetch(INDEX, { ...request, signal: AbortSignal.timeout(15_000) });
		if (!res.ok) throw new Error(`HTTP ${res.status}`);
		const index = await res.json();
		if (!Number.isSafeInteger(index.sequence) || !Array.isArray(index.adapters)) {
			throw new Error("Invalid adapter index");
		}
		const adapters: StoreAdapter[] = index.adapters;
		const ids = new Set<string>();
		for (const adapter of adapters) {
			if (typeof adapter.id !== "string" || !/^[a-z0-9][a-z0-9_-]*$/.test(adapter.id) || ids.has(adapter.id)) {
				throw new Error("Invalid or duplicate adapter ID");
			}
			ids.add(adapter.id);
		}
		adapters.sort((a, b) => a.name.localeCompare(b.name));
		await Promise.all(adapters.map(async (adapter) => (adapter.iconData = await iconData(adapter))));
		return { sequence: index.sequence, adapters };
	} catch (err) {
		const message = `Adapter index lookup failed: ${err}`;
		if (process.env.CI) throw new Error(message);
		console.warn(message);
		return { sequence: 0, adapters: [] };
	}
}

export function webUrl(value?: string, base?: string): string | undefined {
	if (!value) return;
	try {
		const url = new URL(value, base);
		if ((url.protocol === "https:" || url.protocol === "http:") && !url.username && !url.password) {
			return url.href;
		}
	} catch {
		return;
	}
}

async function iconData(adapter: StoreAdapter): Promise<string | undefined> {
	try {
		const url = webUrl(adapter.icon.url);
		if (!url) return;
		const res = await fetch(url, { ...request, signal: AbortSignal.timeout(10_000) });
		if (!res.ok) return;
		const bytes = Buffer.from(await res.arrayBuffer());
		if (createHash("sha256").update(bytes).digest("hex") !== adapter.icon.sha256) return;
		return `data:image/svg+xml;base64,${bytes.toString("base64")}`;
	} catch {
		return;
	}
}

export function adapterText(adapter: StoreAdapter, lang: Lang): AdapterText {
	const translated = lang === "pt" ? adapter.i18n?.pt : undefined;
	return {
		summary: translated?.summary?.trim() || adapter.summary,
		description: translated?.description?.trim() || adapter.description,
	};
}

const platforms: Record<string, string> = { linux: "Linux", macos: "macOS", windows: "Windows", android: "Android", ios: "iOS" };

export function platformLabel(platform: string): string {
	return Object.hasOwn(platforms, platform) ? platforms[platform] : platform;
}
