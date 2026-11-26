// Latest release metadata, fetched from GitHub at build time with a pinned fallback.
export type Release = { tag: string; version: string; date: string; assets: { name: string; url: string; size: number }[] };

const FALLBACK: Release = {
	tag: "v3.8.1",
	version: "3.8.1",
	date: "2026-11-26T00:00:00Z",
	assets: [
		"WingDrive-3.8.1-x86_64.AppImage",
		"WingDrive-3.8.1-amd64.deb",
		"WingDrive-3.8.1-x86_64.flatpak",
		"WingDrive-3.8.1-x64-setup.exe",
		"WingDrive-3.8.1-x64.msi",
		"WingDrive-3.8.1-aarch64.dmg",
		"WingDrive-3.8.1-android-arm64-v8a.apk",
		"wing-server-3.8.1-linux-x86_64.tar.xz",
		"wing-server-3.8.1-linux-x86_64.tar.gz",
		"wing-server-3.8.1-linux-x86_64.zip",
		"wing-3.8.1-windows-x64.zip",
		"SHA256SUMS",
	].map((name) => ({ name, url: `https://github.com/bernardopg/wingdrive/releases/download/v3.8.1/${name}`, size: 0 })),
};

let cached: Release | undefined;

export async function latestRelease(): Promise<Release> {
	if (cached) return cached;
	try {
		// CI passes GITHUB_TOKEN so the build is not rate limited; locally it is optional.
		const headers: Record<string, string> = { Accept: "application/vnd.github+json", "User-Agent": "wingdrive-site-build" };
		if (process.env.GITHUB_TOKEN) headers.Authorization = `Bearer ${process.env.GITHUB_TOKEN}`;
		const res = await fetch("https://api.github.com/repos/bernardopg/wingdrive/releases/latest", { headers });
		if (!res.ok) throw new Error(String(res.status));
		const j = await res.json();
		cached = {
			tag: j.tag_name,
			version: String(j.tag_name).replace(/^v/, ""),
			date: j.published_at,
			assets: j.assets.map((a: { name: string; browser_download_url: string; size: number }) => ({
				name: a.name,
				url: a.browser_download_url,
				size: a.size,
			})),
		};
	} catch (err) {
		// A CI build must not deploy stale download links; a local build may fall back.
		if (process.env.CI) throw new Error(`latest release lookup failed: ${err}`);
		cached = FALLBACK;
	}
	return cached;
}

export function pick(r: Release, re: RegExp) {
	return r.assets.find((a) => re.test(a.name));
}

export function fmtSize(bytes: number) {
	if (!bytes) return "";
	return `${(bytes / 1024 / 1024).toFixed(0)} MB`;
}

/** "3.3" for 3.3.1: the feature release the badges announce. */
export function minorVersion(r: Release) {
	return r.version.split(".").slice(0, 2).join(".");
}
