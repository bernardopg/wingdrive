// Builds favicons, the web manifest icons and the 1200×630 social card.
import sharp from "sharp";
import { readFileSync, writeFileSync } from "node:fs";

const logo = readFileSync("public/logo.svg");
await sharp(logo, { density: 300 }).resize(32, 32).png().toFile("public/favicon-32.png");
await sharp(logo, { density: 600 }).resize(180, 180).png().toFile("public/apple-touch-icon.png");
await sharp(logo, { density: 600 }).resize(192, 192).png().toFile("public/icon-192.png");
await sharp(logo, { density: 600 }).resize(512, 512).png().toFile("public/icon-512.png");

const W = 1200;
const H = 630;
const art = await sharp("raw-assets/gen/hero.webp").resize(W, H, { fit: "cover", position: "right" }).toBuffer();
const overlay = Buffer.from(`
<svg width="${W}" height="${H}" xmlns="http://www.w3.org/2000/svg">
  <defs>
    <linearGradient id="v" x1="0" x2="1"><stop offset="0" stop-color="#0a0614" stop-opacity=".96"/><stop offset=".55" stop-color="#0a0614" stop-opacity=".7"/><stop offset="1" stop-color="#0a0614" stop-opacity="0"/></linearGradient>
    <linearGradient id="g" x1="0" x2="1"><stop offset="0" stop-color="#f5f3ff"/><stop offset=".5" stop-color="#c4b5fd"/><stop offset="1" stop-color="#8b5cf6"/></linearGradient>
  </defs>
  <rect width="${W}" height="${H}" fill="url(#v)"/>
  <text x="72" y="250" font-family="Space Grotesk, Inter, sans-serif" font-weight="700" font-size="76" fill="#f5f3ff" letter-spacing="-2">Every drive you own,</text>
  <text x="72" y="338" font-family="Space Grotesk, Inter, sans-serif" font-weight="700" font-size="76" fill="url(#g)" letter-spacing="-2">in one file manager.</text>
  <text x="74" y="410" font-family="Inter, sans-serif" font-size="28" fill="#a7a0c9">Local-first · Open source · No telemetry</text>
  <text x="166" y="128" font-family="Space Grotesk, Inter, sans-serif" font-weight="700" font-size="40" fill="#f5f3ff">WingDrive</text>
  <text x="74" y="560" font-family="JetBrains Mono, monospace" font-size="24" fill="#c4b5fd">wingdrive.org</text>
</svg>`);
const mark = await sharp(logo, { density: 600 }).resize(72, 72).png().toBuffer();
await sharp(art)
	.composite([
		{ input: overlay, top: 0, left: 0 },
		{ input: mark, top: 76, left: 72 },
	])
	.png({ compressionLevel: 9 })
	.toFile("public/og.png");

writeFileSync(
	"public/site.webmanifest",
	JSON.stringify(
		{
			name: "WingDrive",
			short_name: "WingDrive",
			icons: [
				{ src: "/icon-192.png", sizes: "192x192", type: "image/png" },
				{ src: "/icon-512.png", sizes: "512x512", type: "image/png" },
			],
			theme_color: "#0a0614",
			background_color: "#0a0614",
			display: "standalone",
		},
		null,
		"\t",
	),
);
console.log("icons ok");
