// Converts raw generated art and app screenshots into web-ready AVIF/WebP sizes.
import sharp from "sharp";
import { readdirSync, mkdirSync } from "node:fs";
import { join, parse } from "node:path";

const jobs = [
	{ dir: "raw-assets/gen", out: "public/img/art", widths: [640, 1280, 1920] },
	{ dir: "../docs/public", out: "public/img/shots", widths: [800, 1600], match: /^Wing.*\.webp$/ },
];
for (const job of jobs) {
	mkdirSync(job.out, { recursive: true });
	for (const file of readdirSync(job.dir)) {
		if (!/\.(png|webp)$/.test(file) || (job.match && !job.match.test(file))) continue;
		const { name } = parse(file);
		const src = sharp(join(job.dir, file));
		const meta = await src.metadata();
		for (const w of job.widths) {
			if (w > meta.width * 1.25) continue;
			const width = Math.min(w, meta.width);
			const base = join(job.out, `${name}-${w}`);
			await sharp(join(job.dir, file)).resize({ width }).avif({ quality: 58, effort: 6 }).toFile(`${base}.avif`);
			await sharp(join(job.dir, file)).resize({ width }).webp({ quality: 82 }).toFile(`${base}.webp`);
		}
		console.log("ok", name, meta.width, meta.height);
	}
}
