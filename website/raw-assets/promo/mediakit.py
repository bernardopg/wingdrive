"""Builds the WingDrive media kit: vector logos, lockups, palette and social images.

    .venv/bin/python mediakit.py <out-dir>

The wordmark is set in Space Grotesk SemiBold and converted to outlines, so the
SVGs render the same everywhere without the font installed.
"""

import io
import json
import subprocess
import sys
from pathlib import Path

from fontTools.pens.svgPathPen import SVGPathPen
from fontTools.pens.transformPen import TransformPen
from fontTools.ttLib import TTFont
from fontTools.varLib.instancer import instantiateVariableFont

HERE = Path(__file__).parent
OUT = Path(sys.argv[1] if len(sys.argv) > 1 else HERE / "../../../media-kit")

FEATHERS = [
	("M22 78 C40 46 66 28 104 24 C96 46 78 62 52 70 Z", (22, 24, 71.608, 99.331), 1.0),
	("M26 92 C44 66 68 52 100 48 C92 66 74 79 50 85 Z", (26, 48, 64.657, 113.015), 0.72),
	("M30 104 C46 84 66 74 94 71 C87 85 71 95 50 99 Z", (30, 71, 56.884, 123.138), 0.44),
]

PALETTE = [
	("Violet", "#8B5CF6", "Primary accent, feather highlight"),
	("Indigo", "#4338CA", "Feather shadow, gradient end"),
	("Lilac", "#C4B5FD", "Highlights on dark, gradient text start"),
	("Lavender", "#EDE9FE", "Text on dark, lightest tint"),
	("Night", "#0F0A1F", "App icon tile"),
	("Ink", "#05040B", "Video and page background"),
	("Mist", "#A19FB8", "Body text on dark"),
	("Paper", "#FAF9FF", "Light background"),
]


def font(weight=600):
	f = TTFont(HERE / "assets/fonts/space-grotesk-latin-wght-normal.woff2")
	return instantiateVariableFont(f, {"wght": weight}, inplace=False)


def text_path(text, size, weight=600, tracking=-0.02):
	"""Returns (svg path d, advance width, ascender, descender) for text at size px."""
	f = font(weight)
	cmap = f.getBestCmap()
	gs = f.getGlyphSet()
	upm = f["head"].unitsPerEm
	scale = size / upm
	hmtx = f["hmtx"]
	x = 0.0
	ds = []
	kern = {}
	if "GPOS" not in f:
		pass
	for ch in text:
		g = cmap[ord(ch)]
		pen = SVGPathPen(gs)
		tp = TransformPen(pen, (scale, 0, 0, -scale, x, 0))
		gs[g].draw(tp)
		ds.append(pen.getCommands())
		x += hmtx[g][0] * scale + tracking * size
	x -= tracking * size
	os2 = f["OS/2"]
	return " ".join(ds), x, os2.sCapHeight * scale, kern


def defs(prefix, mono=None):
	if mono:
		return ""
	out = []
	for i, (_, (x1, y1, x2, y2), _) in enumerate(FEATHERS):
		out.append(f'<linearGradient id="{prefix}f{i}" x1="{x1}" y1="{y1}" x2="{x2}" y2="{y2}" gradientUnits="userSpaceOnUse"><stop offset="0" stop-color="#8b5cf6"/><stop offset="1" stop-color="#4338ca"/></linearGradient>')
	return "<defs>" + "".join(out) + "</defs>"


def feathers(prefix, mono=None):
	parts = []
	for i, (d, _, op) in enumerate(FEATHERS):
		fill = mono if mono else f"url(#{prefix}f{i})"
		parts.append(f'<path d="{d}" fill="{fill}" opacity="{op}"/>')
	return "".join(parts)


def mark_svg(tile=True, mono=None, tile_fill="#0f0a1f", size=512):
	inner = defs("m", mono) + (f'<rect width="128" height="128" rx="28" fill="{tile_fill}"/>' if tile else "") + feathers("m", mono)
	vb = "0 0 128 128" if tile else "18 20 90 88"
	return f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="{vb}" width="{size}" height="{size}" role="img" aria-label="WingDrive"><title>WingDrive</title>{inner}</svg>\n'


def lockup_svg(theme="dark", layout="horizontal", tile=True):
	ink = "#f5f3ff" if theme == "dark" else "#0f0a1f"
	accent = "#a78bfa" if theme == "dark" else "#6d28d9"
	size = 100
	d_wing, w_wing, cap, _ = text_path("Wing", size)
	d_drive, w_drive, _, _ = text_path("Drive", size)
	gap_txt = -0.02 * size
	mark = 128
	if layout == "horizontal":
		gap = 34
		tx = mark + gap
		ty = 64 + cap / 2
		W = tx + w_wing + gap_txt + w_drive + 4
		H = 128
		text = f'<g transform="translate({tx:.2f} {ty:.2f})"><path d="{d_wing}" fill="{ink}"/><g transform="translate({w_wing + gap_txt:.2f} 0)"><path d="{d_drive}" fill="{accent}"/></g></g>'
		mk = f'<g>{"<rect width=\"128\" height=\"128\" rx=\"28\" fill=\"#0f0a1f\"/>" if tile else ""}{feathers("l")}</g>'
	else:
		tw = w_wing + gap_txt + w_drive
		ms = 1.7
		W = max(tw, mark * ms) + 8
		mx = (W - mark * ms) / 2
		ty = mark * ms + 44 + cap
		H = ty + 26
		tx = (W - tw) / 2
		text = f'<g transform="translate({tx:.2f} {ty:.2f})"><path d="{d_wing}" fill="{ink}"/><g transform="translate({w_wing + gap_txt:.2f} 0)"><path d="{d_drive}" fill="{accent}"/></g></g>'
		mk = f'<g transform="translate({mx:.2f} 0) scale({ms})">{"<rect width=\"128\" height=\"128\" rx=\"28\" fill=\"#0f0a1f\"/>" if tile else ""}{feathers("l")}</g>'
	return f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {W:.1f} {H:.1f}" width="{W * 4:.0f}" height="{H * 4:.0f}" role="img" aria-label="WingDrive"><title>WingDrive</title>{defs("l")}{mk}{text}</svg>\n'


def wordmark_svg(fill="#f5f3ff", accent=None):
	size = 100
	d_wing, w_wing, cap, _ = text_path("Wing", size)
	d_drive, w_drive, _, _ = text_path("Drive", size)
	gap = -0.02 * size
	W = w_wing + gap + w_drive + 4
	H = cap + 34
	return f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 {-cap - 4:.1f} {W:.1f} {H:.1f}" width="{W * 4:.0f}" height="{H * 4:.0f}" role="img" aria-label="WingDrive"><title>WingDrive</title><path d="{d_wing}" fill="{fill}"/><g transform="translate({w_wing + gap:.2f} 0)"><path d="{d_drive}" fill="{accent or fill}"/></g></svg>\n'


def palette_svg():
	cw, ch = 300, 360
	W = cw * 4 + 40 * 5
	H = ch * 2 + 40 * 3 + 120
	cells = []
	for i, (name, hexv, use) in enumerate(PALETTE):
		x = 40 + (i % 4) * (cw + 40)
		y = 160 + (i // 4) * (ch + 40)
		cells.append(f'<rect x="{x}" y="{y}" width="{cw}" height="{ch - 110}" rx="22" fill="{hexv}" stroke="rgba(255,255,255,.12)"/>'
			f'<text x="{x}" y="{y + ch - 70}" font-family="Space Grotesk, Inter, sans-serif" font-weight="600" font-size="30" fill="#f5f3ff">{name}</text>'
			f'<text x="{x}" y="{y + ch - 36}" font-family="JetBrains Mono, monospace" font-size="22" fill="#a19fb8">{hexv}</text>'
			f'<text x="{x}" y="{y + ch - 8}" font-family="Inter, sans-serif" font-size="17" fill="#6b6886">{use}</text>')
	return (f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {W} {H}" width="{W}" height="{H}"><rect width="{W}" height="{H}" fill="#05040b"/>'
		f'<text x="40" y="96" font-family="Space Grotesk, sans-serif" font-weight="600" font-size="56" fill="#f5f3ff">WingDrive colors</text>{"".join(cells)}</svg>\n')


def main():
	OUT.mkdir(parents=True, exist_ok=True)
	(OUT / "logo").mkdir(exist_ok=True)
	files = {
		"logo/wingdrive-icon.svg": mark_svg(),
		"logo/wingdrive-icon-light.svg": mark_svg(tile_fill="#f3f0ff"),
		"logo/wingdrive-mark.svg": mark_svg(tile=False),
		"logo/wingdrive-mark-white.svg": mark_svg(tile=False, mono="#ffffff"),
		"logo/wingdrive-mark-black.svg": mark_svg(tile=False, mono="#0f0a1f"),
		"logo/wingdrive-lockup-horizontal-dark.svg": lockup_svg("dark", "horizontal"),
		"logo/wingdrive-lockup-horizontal-light.svg": lockup_svg("light", "horizontal"),
		"logo/wingdrive-lockup-stacked-dark.svg": lockup_svg("dark", "stacked"),
		"logo/wingdrive-lockup-stacked-light.svg": lockup_svg("light", "stacked"),
		"logo/wingdrive-wordmark-white.svg": wordmark_svg("#ffffff"),
		"logo/wingdrive-wordmark-black.svg": wordmark_svg("#0f0a1f"),
		"logo/wingdrive-wordmark-color-dark.svg": wordmark_svg("#f5f3ff", "#a78bfa"),
		"logo/wingdrive-wordmark-color-light.svg": wordmark_svg("#0f0a1f", "#6d28d9"),
		"palette.svg": palette_svg(),
	}
	for name, content in files.items():
		(OUT / name).write_text(content)
	(OUT / "colors.json").write_text(json.dumps({n.lower(): {"hex": h, "use": u} for n, h, u in PALETTE}, indent=2) + "\n")
	# PNG exports of every logo SVG.
	png = OUT / "logo/png"
	png.mkdir(exist_ok=True)
	for name in files:
		if not name.startswith("logo/"):
			continue
		src = OUT / name
		stem = src.stem
		for w in ((1024, 512, 256) if "icon" in stem or "mark" in stem else (2400, 1200)):
			subprocess.run(["rsvg-convert", "-w", str(w), "-o", str(png / f"{stem}-{w}.png"), str(src)], check=True)
	subprocess.run(["rsvg-convert", "-o", str(OUT / "palette.png"), str(OUT / "palette.svg")], check=True)
	print("wrote", len(files), "svgs to", OUT)


if __name__ == "__main__":
	main()
