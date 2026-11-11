// Hero sparkle layer. Every sparkle sits on a bright feather pixel sampled from the hero art
// (scripts/hero-points.mjs), mapped through the same object-fit/object-position as the <img>,
// so the glitter stays glued to the wing at any viewport size. Sparkles twinkle, a few flare into
// four-point glints, and the ones near the pointer brighten.
import points from "../data/hero-points.json";

type P = { u: number; v: number; w: number; phase: number; speed: number; size: number; glint: boolean };

export function mountHero(canvas: HTMLCanvasElement, img: HTMLImageElement) {
	const ctx = canvas.getContext("2d");
	if (!ctx) return;
	const reduce = matchMedia("(prefers-reduced-motion: reduce)").matches;
	const IW = 1672;
	const IH = 941;

	let seed = 11;
	const rand = () => (seed = (seed * 16807) % 2147483647) / 2147483647;
	const all: P[] = (points as number[][]).map(([u, v, w]) => ({
		u,
		v,
		w,
		phase: rand() * Math.PI * 2,
		speed: 0.6 + rand() * 1.8,
		size: 0.5 + rand() * 1.3,
		glint: rand() < 0.035,
	}));

	let dpr = 1;
	let map = { s: 1, ox: 0, oy: 0 };
	let list = all;
	const resize = () => {
		dpr = Math.min(devicePixelRatio, 2);
		const w = canvas.clientWidth;
		const h = canvas.clientHeight;
		canvas.width = Math.round(w * dpr);
		canvas.height = Math.round(h * dpr);
		// Mirror object-fit: cover + the image's computed object-position.
		const s = Math.max(w / IW, h / IH);
		const [px, py] = getComputedStyle(img)
			.objectPosition.split(" ")
			.map((p) => parseFloat(p) / 100);
		map = { s, ox: (w - IW * s) * (isNaN(px) ? 0.5 : px), oy: (h - IH * s) * (isNaN(py) ? 0.5 : py) };
		list = w < 700 ? all.filter((_, i) => i % 2 === 0) : all;
	};
	resize();
	new ResizeObserver(resize).observe(canvas);

	const ptr = { x: -9999, y: -9999 };
	addEventListener(
		"pointermove",
		(e) => {
			const r = canvas.getBoundingClientRect();
			ptr.x = e.clientX - r.left;
			ptr.y = e.clientY - r.top;
		},
		{ passive: true },
	);

	let visible = true;
	new IntersectionObserver(([e]) => (visible = e.isIntersecting)).observe(canvas);

	const t0 = performance.now();
	const frame = () => {
		requestAnimationFrame(frame);
		if (!visible) return;
		const t = reduce ? 1.3 : (performance.now() - t0) / 1000;
		const w = canvas.width;
		const h = canvas.height;
		ctx.setTransform(1, 0, 0, 1, 0, 0);
		ctx.clearRect(0, 0, w, h);
		ctx.globalCompositeOperation = "lighter";
		ctx.scale(dpr, dpr);
		const R2 = 160 * 160;
		for (const p of list) {
			const x = map.ox + p.u * IW * map.s + Math.sin(t * 0.7 + p.phase) * 0.8;
			const y = map.oy + p.v * IH * map.s + Math.cos(t * 0.5 + p.phase) * 0.8;
			const tw = 0.5 + 0.5 * Math.sin(t * p.speed + p.phase);
			const dx = x - ptr.x;
			const dy = y - ptr.y;
			const near = Math.max(0, 1 - (dx * dx + dy * dy) / R2);
			const a = Math.min(1, (0.15 + tw * tw * 0.85) * p.w + near * 0.8);
			if (a < 0.04) continue;
			const r = p.size * (1 + near * 0.8);
			ctx.fillStyle = `rgba(233, 222, 255, ${a})`;
			ctx.beginPath();
			ctx.arc(x, y, r, 0, Math.PI * 2);
			ctx.fill();
			if (p.glint && tw > 0.82) {
				const L = 7 + 9 * (tw - 0.82) * 5.5;
				const g = ctx.createRadialGradient(x, y, 0, x, y, L);
				g.addColorStop(0, `rgba(245, 240, 255, ${a})`);
				g.addColorStop(1, "rgba(196, 181, 253, 0)");
				ctx.fillStyle = g;
				ctx.fillRect(x - L, y - 0.6, L * 2, 1.2);
				ctx.fillRect(x - 0.6, y - L, 1.2, L * 2);
			}
		}
	};
	frame();
}
