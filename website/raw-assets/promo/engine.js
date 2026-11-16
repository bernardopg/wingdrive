// WingDrive promo engine: every frame is a pure function of time, so a headless
// browser can render any instant (and sub-instants for motion blur) on demand.
"use strict";

const Q = new URLSearchParams(location.search);
const CUT = Q.get("cut") || "youtube";
const V = CUT === "shorts";
const W = V ? 1080 : 1920;
const H = V ? 1920 : 1080;
const U = 1;
const OV = 0.55; // crossfade between scenes, seconds

const stage = document.getElementById("stage");
stage.style.width = W + "px";
stage.style.height = H + "px";
stage.style.setProperty("--u", U);
const px = (n) => n * U + "px";

// ---------------------------------------------------------------- math
const clamp = (x, a = 0, b = 1) => Math.min(b, Math.max(a, x));
const mix = (a, b, k) => a + (b - a) * k;
const E = {
	lin: (x) => x,
	outCubic: (x) => 1 - Math.pow(1 - x, 3),
	outQuint: (x) => 1 - Math.pow(1 - x, 5),
	outExpo: (x) => (x >= 1 ? 1 : 1 - Math.pow(2, -10 * x)),
	inCubic: (x) => x * x * x,
	inOutCubic: (x) => (x < 0.5 ? 4 * x * x * x : 1 - Math.pow(-2 * x + 2, 3) / 2),
	inOutQuart: (x) => (x < 0.5 ? 8 * x ** 4 : 1 - Math.pow(-2 * x + 2, 4) / 2),
	inOutExpo: (x) =>
		x <= 0 ? 0 : x >= 1 ? 1 : x < 0.5 ? Math.pow(2, 20 * x - 10) / 2 : (2 - Math.pow(2, -20 * x + 10)) / 2,
	outBack: (x) => {
		const c1 = 1.2, c3 = c1 + 1;
		return 1 + c3 * Math.pow(x - 1, 3) + c1 * Math.pow(x - 1, 2);
	},
};
const P = (t, a, d, e = E.outExpo) => e(clamp((t - a) / d));

// ---------------------------------------------------------------- dom
function el(tag, cls, style, html) {
	const n = document.createElement(tag);
	if (cls) n.className = cls;
	if (style) Object.assign(n.style, style);
	if (html != null) n.innerHTML = html;
	return n;
}
function abs(node, x, y, w) {
	Object.assign(node.style, { position: "absolute", left: px(x), top: px(y) });
	if (w != null) node.style.width = px(w);
	return node;
}
// Splits text into inline-block spans ("w" for words, "c" for characters).
function split(parent, text, mode = "w", cls = "") {
	const spans = [];
	const parts = mode === "c" ? [...text] : text.split(/(\s+)/);
	for (const p of parts) {
		if (/^\s+$/.test(p)) { parent.appendChild(document.createTextNode(" ")); continue; }
		const s = el("span", "w " + cls, null);
		s.textContent = p;
		parent.appendChild(s);
		spans.push(s);
	}
	return spans;
}
function line(parent, text, cls, size, mode = "w", style = {}) {
	const grad = /\bgrad\b/.test(cls);
	const d = el("div", cls.replace(/\bgrad\b/, "").trim(), Object.assign({ fontSize: px(size) }, style));
	parent.appendChild(d);
	const spans = split(d, text, mode);
	if (grad) spans.forEach((x) => x.classList.add("grad"));
	return spans;
}
function rise(node, k, o = {}) {
	const y = o.y ?? 34, x = o.x ?? 0, blur = o.blur ?? 14, s = o.s ?? 0;
	node.style.opacity = clamp(k * (o.o ?? 1));
	node.style.transform = `translate3d(${(1 - k) * x * U}px,${(1 - k) * y * U}px,0) scale(${1 - (1 - k) * s})`;
	node.style.filter = k < 0.999 && blur ? `blur(${((1 - k) * blur).toFixed(2)}px)` : "none";
}
function stagger(nodes, t, start, step, dur, o, ease = E.outExpo) {
	nodes.forEach((n, i) => rise(n, P(t, start + i * step, dur, ease), o));
}
function set(node, k) { node.style.opacity = k; }

// ---------------------------------------------------------------- icons
const ICON = {
	ssd: '<rect x="3" y="6" width="18" height="12" rx="2.5"/><circle cx="7.5" cy="12" r="1.1"/><path d="M11 12h6"/>',
	hdd: '<rect x="3.5" y="3.5" width="17" height="17" rx="2.5"/><circle cx="12" cy="11" r="4.2"/><circle cx="12" cy="11" r=".9"/><path d="M7 17.5h.01"/>',
	usb: '<rect x="8.5" y="2.5" width="7" height="6" rx="1"/><rect x="6.5" y="8.5" width="11" height="13" rx="2.5"/><path d="M10.5 5h.01M13.5 5h.01"/>',
	server: '<rect x="3" y="3.5" width="18" height="7" rx="2"/><rect x="3" y="13.5" width="18" height="7" rx="2"/><path d="M7 7h.01M7 17h.01M11 7h6M11 17h6"/>',
	cloud: '<path d="M7.5 19h9.5a4.5 4.5 0 0 0 .6-8.96A6.5 6.5 0 0 0 5.2 10.4 4.4 4.4 0 0 0 7.5 19z"/>',
	laptop: '<rect x="4" y="4.5" width="16" height="11" rx="1.6"/><path d="M2 19h20"/>',
	phone: '<rect x="7" y="2.5" width="10" height="19" rx="2.4"/><path d="M11 18.5h2"/>',
	terminal: '<rect x="2.5" y="4" width="19" height="16" rx="2.4"/><path d="m7 10 3 2.5L7 15M12.5 15H17"/>',
	globe: '<circle cx="12" cy="12" r="9"/><path d="M3 12h18M12 3c2.6 3 2.6 15 0 18M12 3c-2.6 3-2.6 15 0 18"/>',
	check: '<path d="m5 12.5 4.5 4.5L19 7.5"/>',
	lock: '<rect x="5" y="10.5" width="14" height="10" rx="2.2"/><path d="M8.5 10.5V8a3.5 3.5 0 0 1 7 0v2.5"/>',
	window: '<rect x="3" y="4" width="18" height="16" rx="2.4"/><path d="M3 8.5h18"/>',
	box: '<path d="M21 8 12 3 3 8v8l9 5 9-5z"/><path d="M3 8l9 5 9-5M12 13v8"/>',
	disk: '<ellipse cx="12" cy="6" rx="8" ry="3"/><path d="M4 6v12c0 1.7 3.6 3 8 3s8-1.3 8-3V6"/><path d="M4 12c0 1.7 3.6 3 8 3s8-1.3 8-3"/>',
};
function icon(name, size, color = "#c4b5fd", sw = 1.6) {
	return `<svg width="${size * U}" height="${size * U}" viewBox="0 0 24 24" fill="none" stroke="${color}" stroke-width="${sw}" stroke-linecap="round" stroke-linejoin="round">${ICON[name]}</svg>`;
}

// ---------------------------------------------------------------- logo
let logoSeq = 0;
function makeLogo(size) {
	const id = "lg" + logoSeq++;
	const wrap = el("div", "", { position: "absolute", width: px(size), height: px(size) });
	const feathers = [
		["M22 78 C40 46 66 28 104 24 C96 46 78 62 52 70 Z", 22, 24, 71.608, 99.331, 1],
		["M26 92 C44 66 68 52 100 48 C92 66 74 79 50 85 Z", 26, 48, 64.657, 113.015, 0.72],
		["M30 104 C46 84 66 74 94 71 C87 85 71 95 50 99 Z", 30, 71, 56.884, 123.138, 0.44],
	];
	wrap.innerHTML = `
<svg viewBox="0 0 128 128" width="100%" height="100%" style="overflow:visible">
 <defs>
  ${feathers.map((f, i) => `<linearGradient id="${id}f${i}" x1="${f[1]}" y1="${f[2]}" x2="${f[3]}" y2="${f[4]}" gradientUnits="userSpaceOnUse"><stop offset="0" stop-color="#9d74ff"/><stop offset="1" stop-color="#4338ca"/></linearGradient>`).join("")}
  <linearGradient id="${id}sq" x1="0" y1="0" x2="128" y2="128" gradientUnits="userSpaceOnUse"><stop offset="0" stop-color="#17112d"/><stop offset="1" stop-color="#0b0718"/></linearGradient>
  <linearGradient id="${id}st" x1="0" y1="0" x2="128" y2="128" gradientUnits="userSpaceOnUse"><stop offset="0" stop-color="#ddd6fe"/><stop offset=".5" stop-color="#8b5cf6"/><stop offset="1" stop-color="#4338ca"/></linearGradient>
  <linearGradient id="${id}sh" x1="0" y1="0" x2="1" y2="0"><stop offset="0" stop-color="#fff" stop-opacity="0"/><stop offset=".5" stop-color="#fff" stop-opacity=".75"/><stop offset="1" stop-color="#fff" stop-opacity="0"/></linearGradient>
  <clipPath id="${id}cp">${feathers.map((f) => `<path d="${f[0]}"/>`).join("")}</clipPath>
 </defs>
 <rect class="sq" width="128" height="128" rx="28" fill="url(#${id}sq)"/>
 <rect class="edge" x=".5" y=".5" width="127" height="127" rx="27.5" fill="none" stroke="rgba(196,181,253,.16)" stroke-width="1"/>
 <rect class="draw" x="1" y="1" width="126" height="126" rx="27" fill="none" stroke="url(#${id}st)" stroke-width="1.6" stroke-dasharray="460" stroke-dashoffset="460"/>
 ${feathers.map((f, i) => `<path class="fe" d="${f[0]}" fill="url(#${id}f${i})" opacity="${f[5]}"/>`).join("")}
 <g clip-path="url(#${id}cp)"><rect class="shine" x="-80" y="-20" width="60" height="170" fill="url(#${id}sh)" transform="rotate(18 64 64)"/></g>
</svg>`;
	const q = (s) => wrap.querySelector(s);
	const fe = [...wrap.querySelectorAll(".fe")];
	return {
		node: wrap,
		// k* in 0..1: square fill, outline draw, feathers (per index), shine position.
		set({ sq = 1, draw = 0, drawFade = 1, f = [1, 1, 1], shine = -1 }) {
			q(".sq").style.opacity = sq;
			q(".edge").style.opacity = sq;
			const d = q(".draw");
			d.setAttribute("stroke-dashoffset", (460 * (1 - draw)).toFixed(1));
			d.style.opacity = drawFade;
			fe.forEach((p, i) => {
				const k = f[i];
				p.setAttribute("transform", `translate(${((1 - k) * -16).toFixed(2)} ${((1 - k) * 12).toFixed(2)}) rotate(${((1 - k) * -42).toFixed(2)} 24 96)`);
				p.style.opacity = clamp(k * 1.8) * feathers[i][5];
			});
			q(".shine").setAttribute("x", mix(-90, 170, clamp(shine)).toFixed(1));
			q(".shine").style.opacity = shine < 0 || shine > 1 ? 0 : 1;
		},
	};
}

// ---------------------------------------------------------------- components
function win(src, w) {
	const n = el("div", "win", { width: px(w) });
	n.innerHTML = `<div class="bar"><i></i><i></i><i></i></div><img src="assets/shots/${src}.webp">`;
	return n;
}
function bulletList(items, size, gap = 22) {
	const box = el("div", "", { display: "flex", flexDirection: "column", gap: px(gap) });
	const rows = items.map((t) => {
		const r = el("div", "body", { display: "flex", alignItems: "center", gap: px(18), fontSize: px(size), color: "#d4d0ea" });
		r.innerHTML = `<span class="dot" style="flex:none"></span><span>${t}</span>`;
		box.appendChild(r);
		return r;
	});
	return { box, rows };
}
function headBlock(parent, { eyebrow, lines, size, align = "left", body, bodySize = 30, width }) {
	const box = el("div", "", { display: "flex", flexDirection: "column", alignItems: align === "center" ? "center" : "flex-start", textAlign: align, gap: px(18) });
	if (width) box.style.width = px(width);
	parent.appendChild(box);
	const out = { box, eyebrow: null, words: [], body: null };
	if (eyebrow) {
		const e = el("div", "eyebrow", { fontSize: px(V ? 24 : 21), marginBottom: px(6) }, eyebrow);
		box.appendChild(e);
		out.eyebrow = e;
	}
	const h = el("div", "", {});
	box.appendChild(h);
	for (const [text, grad] of lines) out.words.push(...line(h, text, "h2" + (grad ? " grad" : ""), size, "w", { display: "block" }));
	if (body) {
		const b = el("div", "body", { fontSize: px(bodySize), marginTop: px(10), maxWidth: px(width || 900) }, body);
		box.appendChild(b);
		out.body = b;
	}
	return out;
}
function revealHead(hb, t, t0 = 0.15) {
	if (hb.eyebrow) rise(hb.eyebrow, P(t, t0, 0.9), { y: 16, blur: 6 });
	stagger(hb.words, t, t0 + 0.12, 0.065, 1.0, { y: 46, blur: 16 });
	if (hb.body) rise(hb.body, P(t, t0 + 0.35 + hb.words.length * 0.065, 1.0), { y: 24, blur: 10 });
}
function counter(node, t, a, d, to, fmt = (v) => Math.round(v)) {
	node.textContent = fmt(to * P(t, a, d, E.outQuint));
}

// ---------------------------------------------------------------- background
const bg = document.getElementById("bg");
const bgGlow = el("div", "layer");
const bgVig = el("div", "layer", { background: "radial-gradient(ellipse 75% 70% at 50% 50%, transparent 55%, rgba(0,0,0,.6) 100%)" });
const grain = el("canvas", "layer", { opacity: 0.03 });
bg.append(bgGlow, grain, bgVig);
(function makeGrain() {
	grain.width = W; grain.height = H;
	grain.style.width = W + "px"; grain.style.height = H + "px";
	const c = grain.getContext("2d");
	const im = c.createImageData(grain.width, grain.height);
	let s = 1234567;
	for (let i = 0; i < im.data.length; i += 4) {
		s = (s * 1103515245 + 12345) & 0x7fffffff;
		const v = (s >> 8) & 1 ? 255 : 0;
		im.data[i] = im.data[i + 1] = im.data[i + 2] = v;
		im.data[i + 3] = 255;
	}
	c.putImageData(im, 0, 0);
})();
function renderBg(t) {
	const a = t * 0.07, b = t * 0.05 + 2;
	const x1 = 50 + Math.sin(a) * 22, y1 = 30 + Math.cos(a * 1.3) * 14;
	const x2 = 50 + Math.cos(b) * 28, y2 = 78 + Math.sin(b * 1.1) * 10;
	bgGlow.style.background = `
		radial-gradient(ellipse 55% 45% at ${x1}% ${y1}%, rgba(109,40,217,.20), transparent 70%),
		radial-gradient(ellipse 50% 40% at ${x2}% ${y2}%, rgba(67,56,202,.16), transparent 70%),
		radial-gradient(ellipse 90% 60% at 50% 115%, rgba(139,92,246,.10), transparent 70%),
		#05040b`;
}

// ---------------------------------------------------------------- scenes
// Each scene builds its DOM once and returns render(t), t in the scene's own
// (speed-scaled) seconds. api.D is the scene's length in those seconds.
const S = {};

S.intro = (root, api) => {
	const L = V ? 300 : 250;
	const glow = el("div", "", { position: "absolute", width: px(1100), height: px(1100), left: px(W / 2 - 550), top: px(H / 2 - 550), borderRadius: "50%", background: "radial-gradient(circle, rgba(139,92,246,.42), rgba(67,56,202,.14) 38%, transparent 68%)" });
	const streak = el("div", "", { position: "absolute", left: 0, top: px(H / 2), width: px(W), height: px(2), background: "linear-gradient(90deg, transparent, rgba(221,214,254,.95) 50%, transparent)", transformOrigin: "50% 50%" });
	const logo = makeLogo(L);
	const wm = el("div", "h1", { position: "absolute", fontSize: px(V ? 150 : 168), whiteSpace: "nowrap" });
	const chars = split(wm, "WingDrive", "c");
	chars.slice(4).forEach((c) => c.classList.add("grad"));
	const tag = el("div", "eyebrow", { position: "absolute", fontSize: px(V ? 26 : 22), color: "#b8a6f8", whiteSpace: "nowrap", letterSpacing: ".34em", textAlign: "center", lineHeight: 1.9 }, V ? "The file manager<br>for every drive you own" : "The file manager for every drive you own");
	root.append(glow, streak, logo.node, wm, tag);
	// measure the lockup once fonts are in
	const wmW = wm.offsetWidth, wmH = wm.offsetHeight, tagW = tag.offsetWidth;
	const gap = V ? 40 : 56;
	const lockW = L + gap + wmW;
	api.cue("riser", 0.0, { d: 1.3 });
	api.cue("swish", 1.18);
	api.cue("hit", 1.3);
	api.cue("swish", 1.34, { g: 0.6 });
	api.cue("swish", 1.5, { g: 0.45 });
	api.cue("shimmer", 2.35);
	api.cue("whoosh", 3.0, { g: 0.7, d: 1.0 });
	return (t) => {
		const s = P(t, 0.2, 1.0, E.outCubic);
		streak.style.opacity = Math.sin(clamp((t - 0.05) / 1.3) * Math.PI) * 0.9;
		streak.style.transform = `scaleX(${s})`;
		const g = P(t, 1.2, 0.35) * (1 - 0.55 * P(t, 1.6, 2.6, E.outCubic));
		glow.style.opacity = g;
		glow.style.transform = `scale(${0.7 + 0.35 * P(t, 1.2, 2.2)})`;
		logo.set({
			draw: P(t, 0.35, 1.25, E.inOutCubic),
			drawFade: 1 - 0.85 * P(t, 1.9, 1.0),
			sq: P(t, 1.05, 0.9, E.outCubic),
			f: [0, 1, 2].map((i) => P(t, 1.18 + i * 0.16, 1.15, E.outExpo)),
			shine: clamp((t - 2.3) / 0.95) * 1.0 + (t < 2.3 ? -1 : 0),
		});
		const sc = 0.9 + 0.1 * P(t, 0.3, 2.6, E.outCubic);
		const m = P(t, 3.0, 1.25, E.inOutQuart);
		let lx, ly, wx, wy;
		if (V) {
			const total = L + gap + wmH;
			lx = W / 2 - L / 2; ly = mix(H / 2 - L / 2, H / 2 - total / 2 - 40, m);
			wx = W / 2 - wmW / 2; wy = H / 2 - total / 2 - 40 + L + gap;
		} else {
			lx = mix(W / 2 - L / 2, W / 2 - lockW / 2, m); ly = H / 2 - L / 2 - 20;
			wx = W / 2 - lockW / 2 + L + gap; wy = H / 2 - wmH / 2 - 20;
		}
		Object.assign(logo.node.style, { left: px(lx), top: px(ly), transform: `scale(${sc})` });
		Object.assign(wm.style, { left: px(wx), top: px(wy) });
		chars.forEach((c, i) => rise(c, P(t, 3.3 + i * 0.05, 1.0), { x: V ? 0 : -30, y: V ? 30 : 0, blur: 18 }));
		Object.assign(tag.style, { left: px(W / 2 - tagW / 2), top: px(V ? wy + wmH + 70 : ly + L + 90) });
		rise(tag, P(t, 4.15, 1.1), { y: 14, blur: 8, o: 0.9 });
	};
};

S.statement = (root, api) => {
	root.classList.add("center");
	const box = el("div", "", {});
	root.appendChild(box);
	const size = V ? 118 : 136;
	const a = V ? [["Every drive"], ["you own."]] : [["Every drive you own."]];
	const b = V ? [["One fast"], ["file manager."]] : [["One fast file manager."]];
	const wa = [], wb = [];
	for (const [tx] of a) wa.push(...line(box, tx, "h1", size));
	const gap = el("div", "", { height: px(V ? 30 : 10) });
	box.appendChild(gap);
	for (const [tx] of b) wb.push(...line(box, tx, "h1", size));
	wb.forEach((s) => s.classList.add("grad"));
	api.cue("whoosh", 0.05, { g: 0.55, d: 0.9 });
	api.cue("tick", 0.32, { g: 0.6 });
	api.cue("tick", 1.22, { g: 0.6, f: 1.5 });
	return (t) => {
		stagger(wa, t, 0.2, 0.09, 1.1, { y: 60, blur: 22 });
		stagger(wb, t, 1.1, 0.09, 1.1, { y: 60, blur: 22 });
		box.style.transform = `scale(${1 + 0.035 * clamp(t / api.D)})`;
	};
};

S.hero = (root, api) => {
	root.style.perspective = px(2200);
	const hb = headBlock(root, { eyebrow: "Meet WingDrive", lines: [["Fast to read."], ["Calm to use.", true]], size: V ? 104 : 92, align: "center" });
	abs(hb.box, 0, V ? 230 : 70, W);
	const ww = V ? 1180 : 1240;
	const w = win("WingOverview", ww);
	abs(w, (W - ww) / 2, V ? 700 : 330);
	w.style.transformOrigin = "50% 0%";
	root.appendChild(w);
	api.cue("whoosh", 0.55, { g: 0.9, d: 1.6 });
	return (t) => {
		revealHead(hb, t, 0.1);
		const k = P(t, 0.45, 2.2, E.outQuint);
		w.style.opacity = clamp(k * 2);
		w.style.transform = `translate3d(0,${(1 - k) * 420 - 14 * clamp((t - 2) / 6)}px,0) rotateX(${(1 - k) * 34}deg) scale(${0.86 + 0.14 * k + 0.02 * clamp((t - 2) / 6)})`;
	};
};

S.library = (root, api) => {
	const hb = headBlock(root, { eyebrow: "One library", lines: [["Your SSD, your HDD,"], ["your USB stick and"], ["your clouds."], ["In one view.", true]], size: V ? 86 : 74, width: V ? 940 : 760, body: "Files stay where they are. The index makes them fast to browse, search and recognize.", bodySize: V ? 32 : 28 });
	abs(hb.box, V ? 70 : 140, V ? 200 : 210);
	const cx = V ? W / 2 : 1390, cy = V ? 1310 : 560, rx = V ? 360 : 360, ry = V ? 360 : 330;
	const svg = document.createElementNS("http://www.w3.org/2000/svg", "svg");
	Object.assign(svg.style, { position: "absolute", left: 0, top: 0, width: px(W), height: px(H), overflow: "visible" });
	root.appendChild(svg);
	const hubR = 92;
	const hub = el("div", "", { position: "absolute", width: px(hubR * 2), height: px(hubR * 2), left: px(cx - hubR), top: px(cy - hubR), borderRadius: "50%", background: "radial-gradient(circle, rgba(139,92,246,.35), rgba(67,56,202,.08) 60%, transparent 72%)" });
	const lg = makeLogo(118);
	Object.assign(lg.node.style, { left: px(hubR - 59), top: px(hubR - 59) });
	hub.appendChild(lg.node);
	lg.set({});
	root.appendChild(hub);
	const items = [["ssd", "Internal SSD"], ["hdd", "External HDD"], ["usb", "USB stick"], ["server", "Home server"], ["cloud", "Cloud volumes"]];
	const nodes = items.map(([ic, label], i) => {
		const ang = -Math.PI / 2 + (i / items.length) * Math.PI * 2;
		const x = cx + Math.cos(ang) * rx, y = cy + Math.sin(ang) * ry;
		const ln = document.createElementNS("http://www.w3.org/2000/svg", "line");
		const len = Math.hypot(x - cx, y - cy);
		const sx = cx + ((x - cx) / len) * (hubR - 8), sy = cy + ((y - cy) / len) * (hubR - 8);
		ln.setAttribute("x1", sx); ln.setAttribute("y1", sy); ln.setAttribute("x2", x); ln.setAttribute("y2", y);
		ln.setAttribute("stroke", "rgba(167,139,250,.45)"); ln.setAttribute("stroke-width", "1.5");
		const L = len - hubR + 8;
		ln.setAttribute("stroke-dasharray", L); ln.setAttribute("stroke-dashoffset", L);
		svg.appendChild(ln);
		const pulse = document.createElementNS("http://www.w3.org/2000/svg", "circle");
		pulse.setAttribute("r", 4); pulse.setAttribute("fill", "#ddd6fe");
		pulse.style.filter = "drop-shadow(0 0 6px #a78bfa)";
		svg.appendChild(pulse);
		const chip = el("div", "chip", { position: "absolute", fontSize: px(V ? 30 : 26), padding: px(V ? 16 : 14) + " " + px(V ? 28 : 24) });
		chip.innerHTML = icon(ic, V ? 34 : 30) + `<span>${label}</span>`;
		root.appendChild(chip);
		return { ln, L, chip, x, y, sx, sy, pulse };
	});
	const nodeT = (i) => 0.9 + i * 0.22;
	nodes.forEach((_, i) => api.cue("pop", nodeT(i) + 0.15, { f: 1 + i * 0.12 }));
	const stat = el("div", "", { display: "flex", alignItems: "baseline", gap: px(18) });
	stat.innerHTML = `<span class="h1 grad" style="font-size:${px(V ? 96 : 84)}">40+</span><span class="body" style="font-size:${px(V ? 30 : 26)}">cloud services through OpenDAL</span>`;
	abs(stat, V ? 70 : 140, V ? 1780 - 120 : 860);
	root.appendChild(stat);
	return (t) => {
		revealHead(hb, t, 0.1);
		const hk = P(t, 0.5, 1.2);
		hub.style.opacity = hk;
		hub.style.transform = `scale(${0.7 + 0.3 * hk + 0.03 * Math.sin(t * 2.2)})`;
		nodes.forEach((n, i) => {
			const lk = P(t, nodeT(i), 0.9, E.outCubic);
			n.ln.setAttribute("stroke-dashoffset", (n.L * (1 - lk)).toFixed(1));
			const ck = P(t, nodeT(i) + 0.15, 0.9, E.outBack);
			n.chip.style.left = px(n.x - n.chip.offsetWidth / 2);
			n.chip.style.top = px(n.y - n.chip.offsetHeight / 2);
			n.chip.style.opacity = clamp(ck * 1.5);
			n.chip.style.transform = `scale(${0.6 + 0.4 * ck})`;
			n.chip.style.filter = ck < 1 ? `blur(${(1 - clamp(ck)) * 10}px)` : "none";
			const ph = ((t - 2.2 - i * 0.37) / 1.6) % 1;
			const on = t > 2.2 + i * 0.37;
			const pk = E.inOutCubic(clamp(ph));
			n.pulse.setAttribute("cx", mix(n.sx, n.x, pk));
			n.pulse.setAttribute("cy", mix(n.sy, n.y, pk));
			n.pulse.style.opacity = on ? Math.sin(clamp(ph) * Math.PI) : 0;
		});
		rise(stat, P(t, 2.3, 1.1), { y: 24 });
	};
};

function splitScene(root, api, { eyebrow, lines, bullets, shot, size = 84, body }) {
	root.style.perspective = px(2400);
	const hb = headBlock(root, { eyebrow, lines, size: V ? size + 10 : size, width: V ? 940 : 700, body, bodySize: V ? 32 : 28 });
	abs(hb.box, V ? 70 : 140, V ? 210 : 250);
	const bl = bulletList(bullets, V ? 32 : 27, V ? 26 : 22);
	root.appendChild(bl.box);
	const ww = V ? 1000 : 1000;
	const w = win(shot, ww);
	abs(w, V ? 40 : 820, V ? 1010 : 215);
	w.style.transformOrigin = V ? "50% 0%" : "0% 50%";
	root.appendChild(w);
	const bt = 0.95 + hb.words.length * 0.05;
	bullets.forEach((_, i) => api.cue("tick", bt + i * 0.22 + 0.05, { g: 0.5, f: 1 + i * 0.1 }));
	api.cue("whoosh", 0.3, { g: 0.75, d: 1.3 });
	return (t) => {
		revealHead(hb, t, 0.1);
		const hbH = hb.box.offsetHeight;
		bl.box.style.position = "absolute";
		bl.box.style.left = px(V ? 70 : 140);
		bl.box.style.top = px((V ? 210 : 250) + hbH + (V ? 50 : 56));
		stagger(bl.rows, t, bt, 0.22, 0.9, { x: -24, y: 0, blur: 8 });
		const k = P(t, 0.3, 1.8, E.outQuint);
		const drift = clamp(t / api.D);
		w.style.opacity = clamp(k * 1.6);
		w.style.transform = V
			? `translate3d(0,${(1 - k) * 300 - 20 * drift}px,0) rotateX(${(1 - k) * 26}deg) scale(${0.9 + 0.1 * k})`
			: `translate3d(${(1 - k) * 380 - 26 * drift}px,0,0) rotateY(${-24 * (1 - k) - 3}deg) scale(${0.92 + 0.08 * k})`;
	};
}

S.search = (root, api) => splitScene(root, api, {
	eyebrow: "Search",
	lines: [["Find it"], ["in seconds.", true]],
	bullets: ["Refine by kind, extension, size and date", "Hidden files on demand", "Counts over the whole result"],
	shot: "WingSearch",
	size: 96,
});

S.views = (root, api) => {
	root.style.perspective = px(2000);
	const hb = headBlock(root, { eyebrow: "Views", lines: [["See folders"], ["your way.", true]], size: V ? 104 : 88, align: "center" });
	abs(hb.box, 0, V ? 200 : 60, W);
	const shots = [["WingGridView", "Grid + Inspector"], ["WingListView", "List + terminal"], ["WingColumnView", "Columns"], ["WingMediaView", "Media"], ["WingSizeView", "Size map"]];
	const cw = V ? 960 : 1000;
	const cy = V ? 780 : 330;
	const cards = shots.map(([s]) => {
		const w = win(s, cw);
		abs(w, (W - cw) / 2, cy);
		root.appendChild(w);
		return w;
	});
	const labels = shots.map(([, l]) => {
		const n = el("div", "chip", { position: "absolute", fontSize: px(V ? 32 : 26), padding: px(12) + " " + px(26) }, l);
		root.appendChild(n);
		return n;
	});
	const n = shots.length - 1;
	const t0 = 1.4, step = Math.max(0.9, (api.D - t0 - 1.4) / n);
	for (let i = 1; i <= n; i++) api.cue("whoosh", t0 + (i - 1) * step, { g: 0.4, d: 0.7, f: 1 + i * 0.08 });
	api.cue("whoosh", 0.25, { g: 0.7, d: 1.2 });
	return (t) => {
		revealHead(hb, t, 0.1);
		let c = 0;
		for (let i = 1; i <= n; i++) c += P(t, t0 + (i - 1) * step, Math.min(0.85, step * 0.8), E.inOutCubic);
		const intro = P(t, 0.25, 1.6, E.outQuint);
		const ch = cw * (900 / 1400) + 30;
		cards.forEach((w, i) => {
			const d = i - c;
			const ad = Math.abs(d);
			const x = d * (V ? 620 : 700);
			const z = -ad * 420;
			w.style.zIndex = String(100 - Math.round(ad * 10));
			w.style.opacity = clamp((1 - ad * 0.55) * clamp(intro * 1.4));
			w.style.transform = `translate3d(${x}px,${(1 - intro) * 300}px,${z}px) rotateY(${-d * 24}deg)`;
			w.style.filter = ad > 0.05 ? `brightness(${1 - Math.min(ad, 1) * 0.45})` : "none";
		});
		labels.forEach((l, i) => {
			const ad = Math.abs(i - c);
			const k = clamp(1 - ad * 2.2) * intro;
			l.style.left = px(W / 2 - l.offsetWidth / 2);
			l.style.top = px(cy + ch + (V ? 60 : 42));
			l.style.opacity = k;
			l.style.transform = `translateY(${(1 - k) * 12}px)`;
		});
	};
};

S.undo = (root, api) => {
	root.classList.add("center");
	const keys = el("div", "", { display: "flex", gap: px(28), alignItems: "center", marginBottom: px(V ? 110 : 80) });
	const kh = V ? 180 : 160;
	const k1 = el("div", "key", { width: px(kh * 1.75), height: px(kh), fontSize: px(kh * 0.32) }, "Ctrl");
	const plus = el("div", "h2", { fontSize: px(kh * 0.4), color: "#6b6886" }, "+");
	const k2 = el("div", "key", { width: px(kh), height: px(kh), fontSize: px(kh * 0.42) }, "Z");
	keys.append(k1, plus, k2);
	const ring = el("div", "", { position: "absolute", width: px(560), height: px(560), borderRadius: "50%", border: "2px solid rgba(167,139,250,.7)", left: px(W / 2 - 280), top: 0 });
	root.append(ring, keys);
	const hb = headBlock(root, { eyebrow: "Undo", lines: [["Undo that"], ["means it.", true]], size: V ? 110 : 100, align: "center", body: "Copy, overwrite, move and trash are journaled. Ctrl+Z reverses them, and anything that changed since is left alone.", bodySize: V ? 34 : 30, width: V ? 900 : 1000 });
	api.cue("key", 1.3, { f: 0.9 });
	api.cue("key", 1.52, { f: 1.15 });
	api.cue("swell", 1.55);
	const press = (k, t, a) => {
		const p = Math.sin(clamp((t - a) / 0.32) * Math.PI);
		k.style.transform = `translateY(${p * 12}px)`;
		k.style.boxShadow = `0 ${px(16 - p * 12)} 0 #0c0a16, 0 ${px(30 - p * 20)} ${px(60)} rgba(0,0,0,.6), 0 0 ${px(80 * p)} rgba(139,92,246,${0.55 * p})`;
		k.style.borderColor = `rgba(167,139,250,${0.15 + 0.6 * p})`;
	};
	return (t) => {
		const kk = P(t, 0.15, 1.1);
		rise(keys, kk, { y: 50, blur: 16, s: 0.08 });
		press(k1, t, 1.3);
		press(k2, t, 1.52);
		const r = clamp((t - 1.58) / 1.4);
		ring.style.top = px(keys.offsetTop + keys.offsetHeight / 2 - 280);
		ring.style.opacity = r > 0 && r < 1 ? (1 - r) * 0.7 : 0;
		ring.style.transform = `scale(${0.35 + E.outCubic(r) * 0.85})`;
		revealHead(hb, t, 1.0);
	};
};

S.toolbox = (root, api) => {
	root.classList.add("center");
	const hb = headBlock(root, { eyebrow: "The full toolbox", lines: [["Everything a file manager"], ["should do. Then some.", true]], size: V ? 84 : 86, align: "center", width: V ? 960 : 1500 });
	const wrap = el("div", "", { display: "flex", flexWrap: "wrap", justifyContent: "center", gap: px(V ? 20 : 18), maxWidth: px(V ? 980 : 1500), marginTop: px(V ? 90 : 70) });
	root.appendChild(wrap);
	const items = [["window", "Dual-pane Explorer · F3"], ["terminal", "Docked terminal · F4"], ["box", "Git emblems"], ["window", "Quick Preview"], ["disk", "Copy To · Move To"], ["box", "Compress & extract"], ["window", "Templates"], ["terminal", "Dolphin service menus"], ["lock", "Permissions"], ["check", "SHA-256 checksums"], ["globe", "Tray & autostart"], ["window", "Catppuccin · Gruvbox · Dracula · Tokyo Night"]];
	const chips = items.map(([ic, l]) => {
		const c = el("div", "chip", { fontSize: px(V ? 28 : 26), padding: px(V ? 16 : 14) + " " + px(V ? 26 : 24) });
		c.innerHTML = icon(ic, V ? 30 : 28) + `<span>${l}</span>`;
		wrap.appendChild(c);
		return c;
	});
	const c0 = 1.1;
	chips.forEach((_, i) => { if (i % 2 === 0) api.cue("pop", c0 + i * 0.1 + 0.05, { g: 0.45, f: 1 + (i % 5) * 0.1 }); });
	return (t) => {
		revealHead(hb, t, 0.1);
		chips.forEach((c, i) => rise(c, P(t, c0 + i * 0.1, 0.9, E.outCubic), { y: 30, blur: 10, s: 0.12 }));
	};
};

S.local = (root, api) => {
	root.classList.add("center");
	const size = V ? 112 : 132;
	const a = el("div", "", { position: "absolute", left: 0, width: px(W), top: 0, height: px(H), display: "flex", flexDirection: "column", alignItems: "center", justifyContent: "center", gap: px(V ? 20 : 8) });
	const nos = ["No account.", "No telemetry.", "No central server."].map((s) => {
		const d = el("div", "h1", { fontSize: px(size) }, V ? s.replace("central ", "central<br>") : s);
		a.appendChild(d);
		return d;
	});
	const b = el("div", "", { position: "absolute", left: 0, width: px(W), top: 0, height: px(H), display: "flex", flexDirection: "column", alignItems: "center", justifyContent: "center", gap: px(28) });
	const yw = [];
	const hl = el("div", "", {});
	b.appendChild(hl);
	yw.push(...line(hl, "Local-first.", "h1 grad", V ? 150 : 170));
	yw.push(...line(hl, "Yours.", "h1", V ? 150 : 170));
	const sub = el("div", "body", { fontSize: px(V ? 34 : 32), maxWidth: px(V ? 900 : 1200), marginTop: px(20) }, "Indexing, thumbnails and search run on your machine.<br>Back up the whole library by copying one folder.");
	b.appendChild(sub);
	root.append(a, b);
	const T = [0.3, 0.95, 1.6];
	T.forEach((x, i) => api.cue("thud", x + 0.02, { f: 1 + i * 0.12 }));
	const swap = Math.max(2.7, api.D * 0.48);
	api.cue("whoosh", swap - 0.1, { g: 0.7, d: 1.0 });
	api.cue("hit", swap + 0.25, { g: 0.55 });
	return (t) => {
		const out = P(t, swap, 0.8, E.inOutCubic);
		nos.forEach((n, i) => {
			rise(n, P(t, T[i], 0.9), { y: 40, blur: 20, s: 0.06 });
			if (out > 0) {
				n.style.opacity = (1 - out) * n.style.opacity;
				n.style.filter = `blur(${out * 24}px)`;
				n.style.transform += ` translateY(${-out * 60}px)`;
			}
		});
		stagger(yw, t, swap + 0.25, 0.14, 1.2, { y: 70, blur: 26, s: 0.05 });
		rise(sub, P(t, swap + 0.8, 1.1), { y: 20, blur: 10 });
	};
};

S.identity = (root, api) => {
	const hb = headBlock(root, { eyebrow: "Content identity", lines: [["It knows what"], ["a file is.", true]], size: V ? 100 : 92, width: V ? 940 : 760, body: "Every file gets a BLAKE3 content identity. The same photo on your laptop and on Google Drive is recognized as one file.", bodySize: V ? 32 : 28 });
	abs(hb.box, V ? 70 : 140, V ? 200 : 210);
	const card = (where, ic) => {
		const c = el("div", "", { position: "absolute", width: px(V ? 430 : 420), height: px(132), borderRadius: px(22), background: "linear-gradient(180deg,#1b1830,#121022)", border: "1px solid rgba(255,255,255,.09)", display: "flex", alignItems: "center", gap: px(22), padding: px(18), boxShadow: "0 30px 80px rgba(0,0,0,.55)" });
		c.innerHTML = `<div style="width:${px(96)};height:${px(96)};border-radius:${px(14)};flex:none;background:radial-gradient(circle at 30% 30%,#fcd5a5,#f97316 35%,#7c3aed 80%);"></div>
			<div style="display:flex;flex-direction:column;gap:${px(8)}"><div style="font-size:${px(28)};font-weight:600;color:#f5f3ff">IMG_2041.jpg</div>
			<div style="display:flex;align-items:center;gap:${px(10)};font-size:${px(22)};color:#a19fb8">${icon(ic, 24, "#a19fb8")}<span>${where}</span></div></div>`;
		root.appendChild(c);
		return c;
	};
	const ca = card("Laptop", "laptop"), cb = card("Google Drive", "cloud");
	const badge = el("div", "chip", { position: "absolute", fontSize: px(24), padding: px(10) + " " + px(22), color: "#bbf7d0", borderColor: "rgba(34,197,94,.35)", background: "rgba(34,197,94,.10)" });
	badge.innerHTML = icon("check", 24, "#86efac", 2.2) + "<span>Same file, recognized</span>";
	root.appendChild(badge);
	const hash = el("div", "mono", { position: "absolute", fontSize: px(V ? 27 : 25), color: "#c4b5fd", whiteSpace: "nowrap", letterSpacing: ".04em" });
	const hashText = "blake3  4f1a9c2e 77b0d3e8 a9150c6f 2b4de871";
	root.appendChild(hash);
	const stat = el("div", "", { position: "absolute", display: "flex", alignItems: "baseline", gap: px(18) });
	stat.innerHTML = `<span class="h1 grad" style="font-size:${px(V ? 110 : 96)}"><span class="n">0</span> KB</span><span class="body" style="font-size:${px(V ? 30 : 26)};max-width:${px(380)}">sampled to identify<br>a 10 GB file</span>`;
	root.appendChild(stat);
	const num = stat.querySelector(".n");
	const vcx = V ? W / 2 : 1390, vcy = V ? 1130 : 430;
	const cw = V ? 430 : 420;
	for (let i = 0; i < hashText.length; i += 3) api.cue("type", 0.9 + i * 0.028, { g: 0.22 });
	api.cue("whoosh", 1.9, { g: 0.5, d: 0.9 });
	api.cue("merge", 2.75);
	return (t) => {
		revealHead(hb, t, 0.1);
		const n = Math.floor(clamp((t - 0.9) / (hashText.length * 0.028)) * hashText.length);
		hash.textContent = hashText.slice(0, n) + (t > 0.85 && Math.floor(t * 2.4) % 2 === 0 ? "▍" : " ");
		hash.style.left = px(vcx - hash.offsetWidth / 2);
		hash.style.top = px(vcy + (V ? 210 : 200));
		set(hash, P(t, 0.8, 0.4));
		const ap = P(t, 0.4, 1.2, E.outQuint);
		const m = P(t, 1.9, 0.9, E.inOutCubic);
		const spread = mix(V ? 0 : 250, 0, m);
		const vy = V ? mix(170, 0, m) : 0;
		const placeCard = (c, side) => {
			const x = vcx - cw / 2 + side * spread;
			const y = vcy - 66 + side * vy;
			c.style.left = px(x + (1 - ap) * side * 200);
			c.style.top = px(y);
			c.style.opacity = ap;
		};
		placeCard(ca, -1);
		placeCard(cb, 1);
		ca.style.transform = `rotate(${-3 * (1 - m)}deg)`;
		cb.style.transform = `rotate(${3 * (1 - m)}deg) translateZ(0)`;
		cb.style.opacity = ap * (1 - P(t, 2.6, 0.4));
		const bk = P(t, 2.75, 0.9, E.outBack);
		badge.style.left = px(vcx - badge.offsetWidth / 2);
		badge.style.top = px(vcy - 66 - 70);
		badge.style.opacity = clamp(bk * 1.5);
		badge.style.transform = `scale(${0.7 + 0.3 * bk})`;
		const glow = Math.sin(clamp((t - 2.6) / 1.2) * Math.PI);
		ca.style.boxShadow = `0 30px 80px rgba(0,0,0,.55), 0 0 ${px(90 * glow)} rgba(34,197,94,${0.35 * glow})`;
		counter(num, t, 2.2, 1.6, 58);
		abs(stat, V ? 70 : 140, V ? 1600 : 800);
		rise(stat, P(t, 2.1, 1.0), { y: 24 });
	};
};

S.devices = (root, api) => {
	const hb = headBlock(root, { eyebrow: "Devices", lines: [["Pair once."], ["Your library"], ["follows you.", true]], size: V ? 100 : 90, width: V ? 940 : 720 });
	abs(hb.box, V ? 70 : 140, V ? 200 : 230);
	const words = ["orbit", "maple", "falcon", "ember", "quartz", "harbor", "lumen", "cedar", "nova", "willow", "atlas", "river"];
	const grid = el("div", "", { position: "absolute", display: "grid", gridTemplateColumns: `repeat(${V ? 3 : 3}, 1fr)`, gap: px(16) });
	const gx = V ? 70 : 960, gy = V ? 780 : 260;
	abs(grid, gx, gy, V ? 940 : 820);
	const cells = words.map((w, i) => {
		const c = el("div", "mono", { display: "flex", gap: px(16), alignItems: "baseline", padding: px(20) + " " + px(24), borderRadius: px(16), background: "rgba(139,92,246,.07)", border: "1px solid rgba(167,139,250,.18)", fontSize: px(V ? 32 : 30), color: "#ede9fe" });
		c.innerHTML = `<span style="color:#6b6886;font-size:.75em">${String(i + 1).padStart(2, "0")}</span><span>${w}</span>`;
		grid.appendChild(c);
		return c;
	});
	root.appendChild(grid);
	const facts = bulletList(["End-to-end encrypted over Iroh/QUIC", "Ed25519 signatures on both sides", "No server of ours in the middle"], V ? 32 : 27, 18);
	abs(facts.box, V ? 70 : 960, V ? 1390 : 720);
	root.appendChild(facts.box);
	words.forEach((_, i) => { if (i % 2 === 0) api.cue("type", 0.8 + i * 0.075, { g: 0.35, f: 1 + i * 0.03 }); });
	api.cue("swell", 1.9, { g: 0.5 });
	return (t) => {
		revealHead(hb, t, 0.1);
		cells.forEach((c, i) => rise(c, P(t, 0.8 + i * 0.075, 0.8, E.outCubic), { y: 20, blur: 10, s: 0.1 }));
		const lockGlow = Math.sin(clamp((t - 1.9) / 1.4) * Math.PI);
		grid.style.filter = lockGlow > 0 ? `drop-shadow(0 0 ${30 * lockGlow}px rgba(139,92,246,.6))` : "none";
		stagger(facts.rows, t, 2.1, 0.2, 0.9, { x: -20, y: 0, blur: 8 });
	};
};

S.cloud = (root, api) => {
	const hb = headBlock(root, { eyebrow: "Cloud", lines: [["Clouds as"], ["plain volumes.", true]], size: V ? 100 : 92, width: V ? 940 : 760, body: "Copy, move, edit and sync them like local folders. Credentials stay encrypted with your library key.", bodySize: V ? 32 : 28 });
	abs(hb.box, V ? 70 : 140, V ? 200 : 210);
	const term = el("div", "", { position: "absolute", width: px(V ? 940 : 820), borderRadius: px(20), background: "linear-gradient(180deg,#14121f,#0d0c16)", border: "1px solid rgba(255,255,255,.09)", padding: px(34) + " " + px(38), boxShadow: "0 40px 120px rgba(0,0,0,.6)", display: "flex", flexDirection: "column", gap: px(22) });
	abs(term, V ? 70 : 960, V ? 820 : 230);
	const uris = [["gdrive://", "Photos/2026/Lisbon/"], ["s3://", "backups/archive/"], ["onedrive://", "Documents/Contracts/"]];
	const rows = uris.map(() => {
		const r = el("div", "mono", { fontSize: px(V ? 34 : 32), whiteSpace: "nowrap" });
		term.appendChild(r);
		return r;
	});
	root.appendChild(term);
	const provs = ["Google Drive", "Google Photos", "Amazon S3", "Dropbox", "OneDrive", "Azure Blob", "Google Cloud Storage"];
	const pw = el("div", "", { position: "absolute", display: "flex", flexWrap: "wrap", gap: px(14), width: px(V ? 940 : 820) });
	abs(pw, V ? 70 : 960, V ? 1230 : 590);
	const pc = provs.map((p) => {
		const c = el("div", "chip", { fontSize: px(V ? 27 : 24), padding: px(11) + " " + px(22) });
		c.innerHTML = icon("cloud", 24) + `<span>${p}</span>`;
		pw.appendChild(c);
		return c;
	});
	root.appendChild(pw);
	const stat = el("div", "", { position: "absolute", display: "flex", alignItems: "baseline", gap: px(18) });
	stat.innerHTML = `<span class="h1 grad" style="font-size:${px(V ? 96 : 84)}">40+</span><span class="body" style="font-size:${px(V ? 30 : 26)}">services through OpenDAL</span>`;
	abs(stat, V ? 70 : 960, V ? 1520 : 850);
	root.appendChild(stat);
	const t0 = 0.7, cps = 0.03;
	let acc = t0;
	const starts = uris.map(([a, b]) => { const s = acc; acc += (a.length + b.length) * cps + 0.25; return s; });
	starts.forEach((s, i) => { const len = uris[i][0].length + uris[i][1].length; for (let c = 0; c < len; c += 3) api.cue("type", s + c * cps, { g: 0.2 }); });
	provs.forEach((_, i) => { if (i % 2 === 0) api.cue("pop", acc + i * 0.08, { g: 0.35, f: 1.1 + i * 0.05 }); });
	return (t) => {
		revealHead(hb, t, 0.1);
		rise(term, P(t, 0.3, 1.1), { y: 40, blur: 12 });
		rows.forEach((r, i) => {
			const [a, b] = uris[i];
			const n = Math.floor(clamp((t - starts[i]) / ((a.length + b.length) * cps)) * (a.length + b.length));
			const sa = a.slice(0, n), sb = b.slice(0, Math.max(0, n - a.length));
			const caret = n > 0 && n < a.length + b.length ? '<span style="color:#a78bfa">▍</span>' : "";
			r.innerHTML = `<span style="color:#6b6886">$ </span><span style="color:#c4b5fd">${sa}</span><span style="color:#e9e5ff">${sb}</span>${caret}`;
		});
		pc.forEach((c, i) => rise(c, P(t, acc + i * 0.08, 0.8, E.outCubic), { y: 18, blur: 8, s: 0.1 }));
		rise(stat, P(t, acc + 0.6, 1.0), { y: 24 });
	};
};

S.arch = (root, api) => {
	const hb = headBlock(root, { eyebrow: "Under the hood", lines: V ? [["One daemon."], ["Many thin clients.", true]] : [["One daemon. Many thin clients.", true]], size: V ? 92 : 80, align: "center", width: W - 100 });
	abs(hb.box, 50, V ? 200 : 80, W - 100);
	const svg = document.createElementNS("http://www.w3.org/2000/svg", "svg");
	Object.assign(svg.style, { position: "absolute", left: 0, top: 0, width: px(W), height: px(H), overflow: "visible" });
	root.appendChild(svg);
	const box = (label, ic, x, y, w, h, strong) => {
		const b = el("div", "", { position: "absolute", left: px(x - w / 2), top: px(y - h / 2), width: px(w), height: px(h), borderRadius: px(18), display: "flex", alignItems: "center", justifyContent: "center", gap: px(14), fontSize: px(strong ? 36 : 25), fontWeight: strong ? 600 : 500, fontFamily: strong ? "JBM" : "Inter", color: strong ? "#f5f3ff" : "#ddd6fe", background: strong ? "linear-gradient(180deg, rgba(139,92,246,.32), rgba(67,56,202,.22))" : "rgba(255,255,255,.035)", border: `1px solid ${strong ? "rgba(196,181,253,.55)" : "rgba(255,255,255,.1)"}` });
		b.innerHTML = (ic ? icon(ic, strong ? 36 : 28) : "") + `<span>${label}</span>`;
		root.appendChild(b);
		return b;
	};
	const clients = [["Desktop app", "window"], ["wing CLI", "terminal"], ["Web · wing-server", "globe"], ["Mobile", "phone"]];
	const stor = [["Local disks", "ssd"], ["Removable drives", "usb"], ["Cloud volumes", "cloud"], ["Paired devices", "laptop"]];
	const pos = (i, row) => {
		if (V) { const col = i % 2, r = Math.floor(i / 2); return [W / 2 + (col ? 235 : -235), (row === 0 ? 560 : 1420) + r * 130]; }
		return [W / 2 + (i - 1.5) * 420, row === 0 ? 360 : 900];
	};
	const bw = V ? 430 : 370, bh = V ? 104 : 96;
	const dy = V ? 990 : 630;
	const daemon = box("wing-daemon", "disk", W / 2, dy, V ? 620 : 640, V ? 150 : 140, true);
	const layers = el("div", "mono", { position: "absolute", width: px(W), left: 0, top: px(dy + (V ? 100 : 90)), textAlign: "center", fontSize: px(V ? 22 : 20), color: "#8f8aa8", letterSpacing: ".06em" }, "CQRS · resumable jobs · indexer · P2P · SQLite");
	root.appendChild(layers);
	const mk = (list, row) => list.map(([l, ic], i) => {
		const [x, y] = pos(i, row);
		const b = box(l, ic, x, y, bw, bh, false);
		const ln = document.createElementNS("http://www.w3.org/2000/svg", "path");
		const y1 = row === 0 ? y + bh / 2 : y - bh / 2;
		const y2 = row === 0 ? dy - (V ? 75 : 70) : dy + (V ? 75 : 70);
		const x2 = W / 2 + (x - W / 2) * 0.35;
		const d = `M${x} ${y1} C ${x} ${(y1 + y2) / 2}, ${x2} ${(y1 + y2) / 2}, ${x2} ${y2}`;
		ln.setAttribute("d", d); ln.setAttribute("fill", "none"); ln.setAttribute("stroke", "rgba(167,139,250,.4)"); ln.setAttribute("stroke-width", "1.5");
		svg.appendChild(ln);
		const L = ln.getTotalLength();
		ln.setAttribute("stroke-dasharray", L); ln.setAttribute("stroke-dashoffset", L);
		const dot = document.createElementNS("http://www.w3.org/2000/svg", "circle");
		dot.setAttribute("r", 4.5); dot.setAttribute("fill", "#ede9fe"); dot.style.filter = "drop-shadow(0 0 6px #a78bfa)";
		svg.appendChild(dot);
		return { b, ln, L, dot, row };
	});
	const top = mk(clients, 0), bot = mk(stor, 1);
	api.cue("swell", 0.6, { g: 0.6 });
	[...top, ...bot].forEach((_, i) => { if (i % 2 === 0) api.cue("pop", 1.2 + i * 0.12, { g: 0.35, f: 1 + i * 0.05 }); });
	return (t) => {
		revealHead(hb, t, 0.1);
		const dk = P(t, 0.6, 1.0, E.outBack);
		daemon.style.opacity = clamp(dk * 1.4);
		daemon.style.transform = `scale(${0.75 + 0.25 * dk})`;
		daemon.style.boxShadow = `0 0 ${px(70 + 20 * Math.sin(t * 2.5))} rgba(139,92,246,.42)`;
		rise(layers, P(t, 1.0, 0.9), { y: 12, blur: 6 });
		[...top, ...bot].forEach((n, i) => {
			const k = P(t, 1.2 + i * 0.12, 0.9, E.outCubic);
			rise(n.b, k, { y: n.row ? 26 : -26, blur: 10 });
			n.ln.setAttribute("stroke-dashoffset", (n.L * (1 - P(t, 1.35 + i * 0.12, 0.8, E.outCubic))).toFixed(1));
			const ph = ((t - 2.4 - i * 0.21) / 1.3);
			const on = ph > 0;
			const f = ph - Math.floor(ph);
			const q = n.row === 0 ? f : 1 - f;
			const p = n.ln.getPointAtLength(n.L * E.inOutCubic(q));
			n.dot.setAttribute("cx", p.x); n.dot.setAttribute("cy", p.y);
			n.dot.style.opacity = on ? Math.sin(f * Math.PI) : 0;
		});
	};
};

S.platforms = (root, api) => {
	const hb = headBlock(root, { eyebrow: "Platforms", lines: [["Runs where"], ["you work.", true]], size: V ? 104 : 96, width: V ? 940 : 640 });
	abs(hb.box, V ? 70 : 140, V ? 200 : 380);
	const rows = [["Linux", "Supported", "ok", "AppImage, .deb and AUR"], ["Windows 10 / 11", "Beta", "warn", ".exe and .msi installers"], ["Server", "Supported", "ok", "Docker for x86_64 and ARM64"], ["macOS", "From source", "dimp", "Signed builds planned"], ["Android", "Preview", "dimp", "Preview APKs with each release"]];
	const list = el("div", "", { position: "absolute", display: "flex", flexDirection: "column", width: px(V ? 940 : 900) });
	abs(list, V ? 70 : 880, V ? 700 : 240);
	const rs = rows.map(([os, st, tone, d]) => {
		const r = el("div", "", { display: "flex", alignItems: "center", justifyContent: "space-between", padding: px(V ? 30 : 26) + " " + px(6), borderBottom: "1px solid rgba(255,255,255,.08)" });
		r.innerHTML = `<div style="display:flex;flex-direction:column;gap:${px(6)}"><span class="h2" style="font-size:${px(V ? 44 : 40)}">${os}</span><span class="body" style="font-size:${px(V ? 26 : 22)}">${d}</span></div><span class="pill ${tone}" style="font-size:${px(V ? 22 : 19)}">${st}</span>`;
		list.appendChild(r);
		return r;
	});
	root.appendChild(list);
	const foot = el("div", "eyebrow", { position: "absolute", fontSize: px(V ? 24 : 20), color: "#8f8aa8" }, "Open source · Apache-2.0");
	abs(foot, V ? 70 : 140, V ? 1650 : 800);
	root.appendChild(foot);
	rows.forEach((_, i) => api.cue("tick", 0.8 + i * 0.16, { g: 0.4, f: 1 + i * 0.08 }));
	return (t) => {
		revealHead(hb, t, 0.1);
		stagger(rs, t, 0.8, 0.16, 0.9, { x: 40, y: 0, blur: 10 });
		rise(foot, P(t, 1.8, 0.9), { y: 12, blur: 6 });
	};
};

S.stack = (root, api) => {
	root.classList.add("center");
	const eb = el("div", "eyebrow", { fontSize: px(V ? 26 : 22), marginBottom: px(50) }, "Built on a modern, auditable stack");
	root.appendChild(eb);
	const wrap = el("div", "", { display: "flex", flexWrap: "wrap", justifyContent: "center", alignItems: "baseline", columnGap: px(V ? 40 : 56), rowGap: px(V ? 24 : 18), maxWidth: px(V ? 960 : 1700) });
	root.appendChild(wrap);
	const names = ["Rust", "Tokio", "SQLite", "Iroh / QUIC", "BLAKE3", "OpenDAL", "Tauri 2", "React 19"];
	const ns = names.map((n, i) => {
		const d = el("div", "h2" + (i % 3 === 0 ? " grad" : ""), { fontSize: px(V ? 88 : 96), color: i % 3 === 0 ? undefined : "#e9e5ff" }, n);
		wrap.appendChild(d);
		return d;
	});
	names.forEach((_, i) => api.cue("tick", 0.5 + i * 0.12, { g: 0.32, f: 0.9 + i * 0.07 }));
	return (t) => {
		rise(eb, P(t, 0.1, 0.9), { y: 14, blur: 6 });
		stagger(ns, t, 0.5, 0.12, 1.0, { y: 40, blur: 18, s: 0.06 });
		wrap.style.transform = `scale(${1 + 0.03 * clamp(t / api.D)})`;
	};
};

S.end = (root, api) => {
	const L = V ? 200 : 170;
	const lg = makeLogo(L);
	root.appendChild(lg.node);
	const glow = el("div", "", { position: "absolute", width: px(900), height: px(900), borderRadius: "50%", background: "radial-gradient(circle, rgba(139,92,246,.32), rgba(67,56,202,.1) 40%, transparent 68%)" });
	root.insertBefore(glow, lg.node);
	const col = el("div", "", { position: "absolute", left: 0, width: px(W), display: "flex", flexDirection: "column", alignItems: "center", textAlign: "center", gap: px(26) });
	root.appendChild(col);
	const h = el("div", "", {});
	col.appendChild(h);
	const ws = [];
	if (V) { ws.push(...line(h, "Give your drives", "h1", 112)); ws.push(...line(h, "wings.", "h1 grad", 140)); }
	else {
		ws.push(...line(h, "Give your drives wings.", "h1", 128));
		ws[ws.length - 1].classList.add("grad");
	}
	const sub = el("div", "body", { fontSize: px(V ? 32 : 30) }, "Free and open source. No account. No telemetry.");
	const url = el("div", "mono", { marginTop: px(30), fontSize: px(V ? 44 : 40), color: "#f5f3ff", padding: px(16) + " " + px(36), borderRadius: "999px", border: "1px solid rgba(196,181,253,.35)", background: "rgba(139,92,246,.1)" }, "wingdrive.org");
	const gh = el("div", "mono", { fontSize: px(V ? 24 : 22), color: "#8f8aa8", letterSpacing: ".04em" }, "github.com/bernardopg/wingdrive");
	col.append(sub, url, gh);
	api.cue("riser", 0.0, { d: 0.55, g: 0.5 });
	api.cue("hit", 0.55, { g: 0.8 });
	api.cue("shimmer", 1.55, { g: 0.8 });
	api.cue("resolve", 1.0);
	return (t) => {
		const ly = V ? 520 : 150;
		Object.assign(lg.node.style, { left: px(W / 2 - L / 2), top: px(ly) });
		lg.set({
			draw: P(t, 0.0, 0.8, E.inOutCubic), drawFade: 1 - 0.85 * P(t, 1.0, 0.8),
			sq: P(t, 0.35, 0.7, E.outCubic),
			f: [0, 1, 2].map((i) => P(t, 0.45 + i * 0.12, 1.1)),
			shine: t < 1.5 ? -1 : clamp((t - 1.5) / 0.9),
		});
		lg.node.style.transform = `scale(${0.9 + 0.1 * P(t, 0, 1.6, E.outCubic)})`;
		Object.assign(glow.style, { left: px(W / 2 - 450), top: px(ly + L / 2 - 450), opacity: P(t, 0.5, 0.5) * (1 - 0.4 * P(t, 1, 2)) });
		col.style.top = px(ly + L + (V ? 110 : 80));
		stagger(ws, t, 0.9, 0.1, 1.2, { y: 50, blur: 20 });
		rise(sub, P(t, 1.6, 1.0), { y: 20, blur: 8 });
		rise(url, P(t, 1.9, 1.0, E.outBack), { y: 20, blur: 8, s: 0.1 });
		rise(gh, P(t, 2.2, 1.0), { y: 14, blur: 6 });
	};
};

// One gradient across a whole line instead of one per word or letter.
function fixGrad(root) {
	for (const sp of root.querySelectorAll(".w.grad")) {
		const line = sp.parentElement;
		const lr = line.getBoundingClientRect(), r = sp.getBoundingClientRect();
		const spans = [...line.querySelectorAll(".w.grad")].map((x) => x.getBoundingClientRect());
		const x0 = Math.min(...spans.map((b) => b.left)), x1 = Math.max(...spans.map((b) => b.right));
		sp.style.backgroundSize = `${x1 - x0}px 100%`;
		sp.style.backgroundPosition = `${x0 - r.left}px 0`;
		void lr;
	}
}

// ---------------------------------------------------------------- cuts
// [scene, real seconds, speed]
const CUTS = {
	youtube: [
		["intro", 7.0, 1], ["statement", 5.0, 1], ["hero", 6.2, 1], ["library", 7.6, 1], ["search", 6.6, 1],
		["views", 8.4, 1], ["undo", 6.0, 1], ["toolbox", 6.4, 1], ["local", 7.4, 1], ["identity", 7.4, 1],
		["devices", 6.8, 1], ["cloud", 7.4, 1], ["arch", 7.0, 1], ["platforms", 5.8, 1], ["stack", 4.6, 1], ["end", 7.5, 1],
	],
	github: [
		["intro", 5.4, 1.2], ["statement", 3.6, 1.25], ["hero", 4.4, 1.2], ["search", 4.2, 1.25], ["views", 5.8, 1.15],
		["local", 4.8, 1.25], ["cloud", 4.6, 1.3], ["end", 5.6, 1.1],
	],
	shorts: [
		["intro", 5.8, 1.15], ["statement", 4.0, 1.2], ["hero", 4.6, 1.15], ["library", 5.6, 1.2], ["search", 4.8, 1.2],
		["views", 6.2, 1.15], ["undo", 4.6, 1.2], ["local", 5.4, 1.2], ["identity", 5.6, 1.2], ["cloud", 5.2, 1.25], ["end", 6.0, 1.1],
	],
};

const scenesRoot = document.getElementById("scenes");
const fx = document.getElementById("fx");
const fade = el("div", "layer", { background: "#000" });
fx.appendChild(fade);
const RUN = [];
const CUES = [];
let TOTAL = 0;

async function init() {
	await Promise.all([
		document.fonts.load('600 100px "SG"'), document.fonts.load('400 30px "Inter"'), document.fonts.load('600 30px "Inter"'),
		document.fonts.load('500 20px "JBM"'),
	]);
	await document.fonts.ready;
	let start = 0;
	for (const [name, dur, speed] of CUTS[CUT]) {
		const root = el("div", "scene");
		scenesRoot.appendChild(root);
		const s0 = start;
		const api = {
			D: dur * speed,
			cue: (type, t, o = {}) => CUES.push(Object.assign({ type, t: s0 + t / speed, scene: name }, o, o.d ? { d: o.d / speed } : {})),
		};
		root.style.display = "block";
		const render = S[name](root, api);
		const disp = root.classList.contains("center") ? "flex" : "block";
		root.style.display = disp;
		render(api.D * 0.999);
		fixGrad(root);
		root.style.display = "none";
		RUN.push({ name, root, start, dur, speed, render, disp });
		CUES.push({ type: "scene", t: start, scene: name, d: dur });
		start += dur - OV;
	}
	TOTAL = start + OV;
	await Promise.all([...document.images].map((i) => i.decode().catch(() => {})));
	CUES.sort((a, b) => a.t - b.t);
	window.TOTAL = TOTAL;
	window.CUES = CUES;
	window.W = W; window.H = H;
}

function seek(t, tb = t) {
	renderBg(tb);
	for (const s of RUN) {
		const lt = t - s.start;
		if (lt < 0 || lt >= s.dur) { s.root.style.display = "none"; continue; }
		s.root.style.display = s.disp;
		s.render(lt * s.speed);
		const outK = s === RUN[RUN.length - 1] ? 0 : P(lt, s.dur - OV - 0.05, OV + 0.05, E.inOutCubic);
		s.root.style.opacity = 1 - outK;
		s.root.style.transform = outK > 0 ? `scale(${1 - 0.035 * outK})` : "none";
		s.root.style.filter = outK > 0.001 ? `blur(${(outK * 14).toFixed(2)}px)` : "none";
	}
	fade.style.opacity = Math.max(1 - P(t, 0, 0.7, E.outCubic), P(t, TOTAL - 1.1, 1.1, E.inOutCubic));
}

window.seek = seek;
window.__ready = init();
