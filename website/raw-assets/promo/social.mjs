// Screenshots social.html into the media kit: bun social.mjs <out-dir>
import { chromium } from "playwright-core";
import { resolve } from "node:path";
import { mkdirSync } from "node:fs";

const out = resolve(process.argv[2] || "../../../media-kit/social");
mkdirSync(out, { recursive: true });
const browser = await chromium.launch({ executablePath: process.env.CHROME || "/usr/bin/google-chrome-stable", args: ["--allow-file-access-from-files", "--force-color-profile=srgb"] });
const page = await browser.newPage({ viewport: { width: 2560, height: 2000 } });
const names = { thumb: "youtube-thumbnail-1280x720", social: "github-social-preview-1280x640", banner: "youtube-banner-2560x1440", poster: "vertical-poster-1080x1920" };
for (const [k, name] of Object.entries(names)) {
	await page.goto("file://" + resolve("social.html") + "?kind=" + k);
	await page.evaluate(() => window.__ready);
	await page.evaluate(() => document.fonts.ready);
	await page.locator("#c").screenshot({ path: `${out}/${name}.png` });
	console.log(name);
}
await browser.close();
