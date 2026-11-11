// Samples bright feather pixels from the hero art so the sparkle layer sits exactly on the wing.
// Output: src/data/hero-points.json with [u, v, weight] triples in 0..1 image coordinates.
import sharp from "sharp";
import { writeFileSync } from "node:fs";

const W = 320;
const H = 180;
const { data } = await sharp("raw-assets/gen/hero.webp").resize(W, H, { fit: "fill" }).raw().toBuffer({ resolveWithObject: true });
const cand = [];
for (let y = 0; y < H; y++) {
	for (let x = 0; x < W; x++) {
		const i = (y * W + x) * 3;
		const r = data[i];
		const g = data[i + 1];
		const b = data[i + 2];
		const lum = 0.2126 * r + 0.7152 * g + 0.0722 * b;
		const violet = b > g + 25 && r > g; // the wing is violet; skip grey drives
		if (lum > 95 && violet) cand.push([x, y, lum]);
	}
}
// Weighted sample without replacement, favouring the brightest feather edges.
let seed = 7;
const rand = () => ((seed = (seed * 16807) % 2147483647) / 2147483647);
const picked = [];
const N = 1600;
for (let k = 0; k < N && cand.length; k++) {
	const total = cand.reduce((s, c) => s + c[2] ** 2, 0);
	let t = rand() * total;
	let j = 0;
	for (; j < cand.length; j++) {
		t -= cand[j][2] ** 2;
		if (t <= 0) break;
	}
	const [x, y, lum] = cand.splice(Math.min(j, cand.length - 1), 1)[0];
	picked.push([+((x + rand()) / W).toFixed(4), +((y + rand()) / H).toFixed(4), +(lum / 255).toFixed(2)]);
}
writeFileSync("src/data/hero-points.json", JSON.stringify(picked));
console.log("points", picked.length, "of", cand.length + picked.length);
