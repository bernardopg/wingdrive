// The adapter store catalog, read at build time from the registry's latest signed index.
// The app verifies the signature before installing; the site only lists what is published.
export type StoreAdapter = {
	id: string;
	name: string;
	summary: string;
	description: string;
	version: string;
	author: string;
	license: string;
	homepage?: string;
	categories: string[];
	trust: "official" | "verified" | "community";
	platforms: string[];
	permissions: { read?: string[]; network?: string[]; secrets?: string[]; tools?: string[] };
	icon: { url: string; sha256: string };
	/** The icon as a data URL, so it renders without a request and never as a document. */
	iconData?: string;
};

export type Catalog = { sequence: number; generated: string; adapters: StoreAdapter[] };

export const REGISTRY = "https://github.com/bernardopg/wingdrive-adapters";
const INDEX = `${REGISTRY}/releases/latest/download/index.json`;

let cached: Catalog | undefined;

export async function catalog(): Promise<Catalog> {
	if (cached) return cached;
	try {
		const res = await fetch(INDEX, { headers: { "User-Agent": "wingdrive-site-build" } });
		if (!res.ok) throw new Error(String(res.status));
		const j = await res.json();
		const adapters: StoreAdapter[] = [...j.adapters].sort((a: StoreAdapter, b: StoreAdapter) => a.name.localeCompare(b.name));
		await Promise.all(adapters.map(async (a) => (a.iconData = await iconData(a))));
		cached = { sequence: j.sequence, generated: j.generated_at, adapters };
	} catch (err) {
		if (process.env.CI) throw new Error(`adapter index lookup failed: ${err}`);
		cached = { sequence: 0, generated: new Date(0).toISOString(), adapters: [] };
	}
	return cached;
}

async function iconData(a: StoreAdapter): Promise<string | undefined> {
	try {
		const res = await fetch(a.icon.url, { headers: { "User-Agent": "wingdrive-site-build" } });
		if (!res.ok) return undefined;
		const bytes = new Uint8Array(await res.arrayBuffer());
		const digest = Buffer.from(await crypto.subtle.digest("SHA-256", bytes)).toString("hex");
		if (digest !== a.icon.sha256) return undefined;
		return `data:image/svg+xml;base64,${Buffer.from(bytes).toString("base64")}`;
	} catch {
		return undefined;
	}
}
