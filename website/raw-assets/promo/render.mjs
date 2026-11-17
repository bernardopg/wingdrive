// Renders a cut of promo.html to video.
//   bun render.mjs <cut> [--fps 60] [--sub 4] [--workers 3] [--stills 1,2.5,...] [--from s --to s]
// Each output frame averages `sub` sub-frames spread over the shutter, which is
// real motion blur. Workers render disjoint frame ranges into lossless segments
// that are concatenated at the end.
import { chromium } from "playwright-core";
import { spawn } from "node:child_process";
import { mkdirSync, writeFileSync, existsSync } from "node:fs";
import { resolve } from "node:path";

const args = process.argv.slice(2);
const cut = args[0] || "youtube";
const opt = (k, d) => { const i = args.indexOf("--" + k); return i >= 0 ? args[i + 1] : d; };
const FPS = +opt("fps", 60);
const SUB = +opt("sub", 4);
const WORKERS = +opt("workers", 3);
const SHUTTER = +opt("shutter", 0.6); // fraction of a frame the shutter stays open
const stills = opt("stills", null);
const OUT = resolve(opt("out", "out"));
mkdirSync(OUT, { recursive: true });
const url = "file://" + resolve("promo.html") + "?cut=" + cut;

async function openPage(browser) {
	const ctx = await browser.newContext({ viewport: { width: 1920, height: 1920 }, deviceScaleFactor: 1 });
	const page = await ctx.newPage();
	page.on("pageerror", (e) => console.error("pageerror", e.message));
	page.on("console", (m) => { if (m.type() === "error") console.error("console", m.text()); });
	await page.goto(url);
	await page.evaluate(() => window.__ready);
	const info = await page.evaluate(() => ({ total: window.TOTAL, w: window.W, h: window.H, cues: window.CUES }));
	await page.setViewportSize({ width: info.w, height: info.h });
	const cdp = await ctx.newCDPSession(page);
	return { page, cdp, info };
}

async function shot(p, t, tb = t) {
	await p.page.evaluate(([t, tb]) => window.seek(t, tb), [t, tb]);
	let r;
	for (let attempt = 0; ; attempt++) {
		try {
			r = await p.cdp.send("Page.captureScreenshot", { format: "png", optimizeForSpeed: true, captureBeyondViewport: false });
			break;
		} catch (e) {
			// A busy compositor sometimes refuses one capture; give up after a few tries.
			if (attempt >= 4) throw e;
			await new Promise((res) => setTimeout(res, 250 * (attempt + 1)));
		}
	}
	return Buffer.from(r.data, "base64");
}

const browser = await chromium.launch({
	executablePath: process.env.CHROME || "/usr/bin/google-chrome-stable",
	args: ["--force-color-profile=srgb", "--hide-scrollbars", "--disable-lcd-text", "--font-render-hinting=none", "--allow-file-access-from-files"],
});

const first = await openPage(browser);
const { total, w, h, cues } = first.info;
console.log(`cut=${cut} ${w}x${h} total=${total.toFixed(2)}s cues=${cues.length}`);
writeFileSync(`${OUT}/${cut}.cues.json`, JSON.stringify({ total, fps: FPS, cues }, null, 1));

if (stills) {
	for (const s of stills.split(",")) {
		const t = +s;
		writeFileSync(`${OUT}/${cut}-still-${t.toFixed(2)}.png`, await shot(first, t));
	}
	await browser.close();
	process.exit(0);
}

const from = +opt("from", 0), to = Math.min(+opt("to", total), total);
const nFrames = Math.round((to - from) * FPS);
const pages = [first];
for (let i = 1; i < WORKERS; i++) pages.push(await openPage(browser));

const chunk = Math.ceil(nFrames / WORKERS);
const t0 = Date.now();
let done = 0;
let skipped = 0;
const segs = [];
await Promise.all(pages.map(async (p, wi) => {
	const a = wi * chunk, b = Math.min(nFrames, a + chunk);
	if (a >= b) return;
	const seg = `${OUT}/${cut}-seg${wi}.mkv`;
	segs[wi] = seg;
	// tmix averages each group of SUB sub-frames; select keeps one per group.
	const ff = spawn("ffmpeg", ["-y", "-loglevel", "error", "-f", "image2pipe", "-framerate", String(FPS * SUB), "-i", "-",
		"-vf", `format=rgb48le,tmix=frames=${SUB},select='eq(mod(n\\,${SUB})\\,${SUB - 1})',setpts=N/(${FPS}*TB),format=yuv444p10le`,
		"-r", String(FPS), "-c:v", "libx264", "-preset", "veryfast", "-qp", "0", seg], { stdio: ["pipe", "inherit", "inherit"] });
	for (let f = a; f < b; f++) {
		// Sub-frames share the frame's background time. When the first and last
		// sub-frame match, nothing moved: reuse it instead of rendering the rest.
		const tb = from + f / FPS;
		const ts = (s) => Math.max(0, from + (f + (s / SUB - 0.5) * SHUTTER) / FPS);
		const firstPng = await shot(p, ts(0), tb);
		const lastPng = await shot(p, ts(SUB - 1), tb);
		const pngs = [firstPng];
		if (firstPng.equals(lastPng)) {
			for (let s = 1; s < SUB; s++) pngs.push(firstPng);
			skipped++;
		} else {
			for (let s = 1; s < SUB - 1; s++) pngs.push(await shot(p, ts(s), tb));
			pngs.push(lastPng);
		}
		for (const png of pngs) if (!ff.stdin.write(png)) await new Promise((r) => ff.stdin.once("drain", r));
		done++;
		if (wi === 0 && done % 60 === 0) {
			const el = (Date.now() - t0) / 1000;
			console.log(`${done}/${nFrames} frames (${skipped} static)  ${el.toFixed(0)}s  eta ${((el / done) * (nFrames - done)).toFixed(0)}s`);
		}
	}
	ff.stdin.end();
	await new Promise((r) => ff.on("close", r));
}));
await browser.close();
const list = `${OUT}/${cut}-segs.txt`;
writeFileSync(list, segs.filter(Boolean).map((s) => `file '${s}'`).join("\n"));
await new Promise((r) => spawn("ffmpeg", ["-y", "-loglevel", "error", "-f", "concat", "-safe", "0", "-i", list, "-c", "copy", `${OUT}/${cut}-video.mkv`], { stdio: "inherit" }).on("close", r));
console.log(`done ${nFrames} frames in ${((Date.now() - t0) / 1000).toFixed(0)}s -> ${OUT}/${cut}-video.mkv`);
