"""Sound design and score for the WingDrive promo, synthesized from the cue list.

    .venv/bin/python audio.py out/<cut>.cues.json out/<cut>-audio.wav

Every sound is generated here (no samples): a lush pad, sub bass, soft pluck
arpeggio and kick for the score, and whooshes, hits, ticks, key presses and
shimmers placed on the cues the animation engine exported.
"""

import json
import sys

import numpy as np
from scipy import signal
from scipy.io import wavfile

SR = 48000
RNG = np.random.default_rng(7)
BPM = 96
BEAT = 60 / BPM


def midi(n):
	return 440.0 * 2 ** ((n - 69) / 12)


def env_adsr(n, a, d, s, r, sustain_len):
	"""Linear ADSR of n samples; times in seconds."""
	out = np.zeros(n)
	A, D, R = int(a * SR), int(d * SR), int(r * SR)
	S = max(0, int(sustain_len * SR))
	i = 0
	seg = min(A, n - i)
	out[i:i + seg] = np.linspace(0, 1, A, endpoint=False)[:seg]
	i += seg
	seg = min(D, n - i)
	out[i:i + seg] = np.linspace(1, s, D, endpoint=False)[:seg]
	i += seg
	seg = min(S, n - i)
	out[i:i + seg] = s
	i += seg
	seg = min(R, n - i)
	out[i:i + seg] = np.linspace(s, 0, R)[:seg]
	return out


def sos_bp(lo, hi, order=2):
	return signal.butter(order, [lo, hi], btype="band", fs=SR, output="sos")


def lp(x, fc, order=2):
	return signal.sosfilt(signal.butter(order, fc, fs=SR, output="sos"), x)


def hp(x, fc, order=2):
	return signal.sosfilt(signal.butter(order, fc, btype="high", fs=SR, output="sos"), x)


def sweep_bp(x, centers, q=1.2, block=512):
	"""Band-pass with a moving center (one value per block)."""
	out = np.zeros_like(x)
	zi = None
	for b in range(0, len(x), block):
		c = centers[min(b // block, len(centers) - 1)]
		lo, hi = c / (1 + 1 / q), c * (1 + 1 / q)
		hi = min(hi, SR / 2 * 0.95)
		sos = sos_bp(max(20, lo), hi)
		if zi is None:
			zi = signal.sosfilt_zi(sos) * 0
		y, zi = signal.sosfilt(sos, x[b:b + block], zi=zi)
		out[b:b + block] = y
	return out


def place(bus, t, mono_or_stereo, gain=1.0, pan=0.0):
	"""Mix a mono (panned) or stereo buffer into bus at time t."""
	i = int(t * SR)
	if i >= bus.shape[0]:
		return
	x = mono_or_stereo
	if x.ndim == 1:
		l, r = np.cos((pan + 1) * np.pi / 4), np.sin((pan + 1) * np.pi / 4)
		x = np.stack([x * l, x * r], axis=1) * np.sqrt(2)
	n = min(len(x), bus.shape[0] - i)
	bus[i:i + n] += x[:n] * gain


def reverb_ir(seconds=3.2, bright=6000, seed=3):
	rng = np.random.default_rng(seed)
	n = int(seconds * SR)
	t = np.arange(n) / SR
	decay = np.exp(-6.9 * t / seconds)
	ir = np.stack([rng.standard_normal(n), rng.standard_normal(n)], axis=1) * decay[:, None]
	for c in range(2):
		ir[:, c] = lp(ir[:, c], bright)
	pre = int(0.022 * SR)
	ir = np.vstack([np.zeros((pre, 2)), ir])
	return ir / np.sqrt((ir ** 2).sum() / 2)


def convolve(x, ir):
	return np.stack([signal.fftconvolve(x[:, c], ir[:, c])[: len(x)] for c in range(2)], axis=1)


# ------------------------------------------------------------------ instruments

def pad_voice(freq, dur, bright=1400.0):
	n = int(dur * SR)
	t = np.arange(n) / SR
	out = np.zeros((n, 2))
	for det, panv in ((-7, -0.7), (0, 0.0), (6, 0.7)):
		f = freq * 2 ** (det / 1200)
		vib = 1 + 0.0018 * np.sin(2 * np.pi * 0.21 * t + det)
		ph = 2 * np.pi * np.cumsum(f * vib) / SR
		w = np.zeros(n)
		for h in range(1, 9):
			if f * h > bright * 2.2:
				break
			w += np.sin(h * ph + h * 0.7) / h ** 1.35 * np.exp(-max(0, f * h - bright) / bright)
		l, r = np.cos((panv + 1) * np.pi / 4), np.sin((panv + 1) * np.pi / 4)
		out[:, 0] += w * l
		out[:, 1] += w * r
	return out / 3


def pluck(freq, decay=0.9, bright=1.0):
	n = int((decay * 2.2) * SR)
	t = np.arange(n) / SR
	w = np.zeros(n)
	for h in range(1, 7):
		w += np.sin(2 * np.pi * freq * h * t) * np.exp(-t * (3.0 + h * 2.4 / bright) / decay) / h ** 1.6
	w *= np.minimum(1, t / 0.004)
	return lp(w, 5200)


def kick(g=1.0):
	n = int(0.5 * SR)
	t = np.arange(n) / SR
	f = 46 + 70 * np.exp(-t * 28)
	ph = 2 * np.pi * np.cumsum(f) / SR
	w = np.sin(ph) * np.exp(-t * 7.5)
	w += 0.15 * lp(RNG.standard_normal(n), 1800) * np.exp(-t * 120)
	return w * g


def hat():
	n = int(0.09 * SR)
	t = np.arange(n) / SR
	return hp(RNG.standard_normal(n), 7000) * np.exp(-t * 55) * 0.5


# ------------------------------------------------------------------ sound effects

def sfx_whoosh(d=1.0, f=1.0):
	n = int(d * SR)
	x = RNG.standard_normal(n)
	u = np.linspace(0, 1, n)
	shape = np.sin(np.pi * u) ** 2.2
	centers = 350 * f + 2600 * f * np.sin(np.pi * np.linspace(0, 1, n // 512 + 1)) ** 1.5
	y = sweep_bp(x, centers, q=1.6) * shape
	pan = np.linspace(-0.6, 0.6, n)
	l, r = np.cos((pan + 1) * np.pi / 4), np.sin((pan + 1) * np.pi / 4)
	return np.stack([y * l, y * r], axis=1) * 1.6


def sfx_swish(f=1.0):
	n = int(0.38 * SR)
	x = RNG.standard_normal(n)
	u = np.linspace(0, 1, n)
	shape = (u ** 0.6) * np.exp(-u * 5)
	centers = 1800 * f + 5200 * f * np.linspace(0, 1, n // 512 + 1)
	return sweep_bp(x, centers, q=2.0) * shape * 2.4


def sfx_riser(d=1.3):
	n = int(d * SR)
	u = np.linspace(0, 1, n)
	x = RNG.standard_normal(n)
	centers = 200 + 5200 * np.linspace(0, 1, n // 512 + 1) ** 2.2
	noise = sweep_bp(x, centers, q=2.5) * u ** 2.5
	f = 90 + 520 * u ** 2
	tone = np.sin(2 * np.pi * np.cumsum(f) / SR) * u ** 3 * 0.25
	y = (noise * 0.9 + tone)
	y[-int(0.012 * SR):] *= np.linspace(1, 0, int(0.012 * SR))
	return y


def sfx_hit(g=1.0):
	n = int(3.2 * SR)
	t = np.arange(n) / SR
	f = 34 + 52 * np.exp(-t * 9)
	boom = np.sin(2 * np.pi * np.cumsum(f) / SR) * np.exp(-t * 1.6)
	body = lp(RNG.standard_normal(n), 900) * np.exp(-t * 14) * 0.5
	bell = np.zeros(n)
	for ratio, amp, dec in ((1, 1, 1.4), (2.0, 0.45, 1.8), (2.76, 0.32, 2.4), (4.07, 0.18, 3.2), (5.4, 0.1, 4.0)):
		bell += amp * np.sin(2 * np.pi * 587.33 * ratio * t) * np.exp(-t * dec)
	bell *= np.minimum(1, t / 0.003) * 0.16
	return (boom * 0.95 + body + bell) * g


def sfx_tick(f=1.0, g=1.0):
	n = int(0.09 * SR)
	t = np.arange(n) / SR
	w = np.sin(2 * np.pi * 2300 * f * t) * np.exp(-t * 75) * 0.5
	w += hp(RNG.standard_normal(n), 4000) * np.exp(-t * 400) * 0.25
	return w * g


def sfx_pop(f=1.0):
	n = int(0.16 * SR)
	t = np.arange(n) / SR
	fr = 620 * f * (1 + 0.6 * np.exp(-t * 40))
	return np.sin(2 * np.pi * np.cumsum(fr) / SR) * np.exp(-t * 32) * np.minimum(1, t / 0.002) * 0.6


def sfx_key(f=1.0):
	n = int(0.25 * SR)
	t = np.arange(n) / SR
	thump = np.sin(2 * np.pi * (140 * f) * t) * np.exp(-t * 30)
	click = lp(hp(RNG.standard_normal(n), 1500), 7000) * np.exp(-t * 180)
	return thump * 0.7 + click * 0.5


def sfx_type():
	n = int(0.03 * SR)
	t = np.arange(n) / SR
	return hp(RNG.standard_normal(n), 2500 + RNG.uniform(0, 2500)) * np.exp(-t * 300) * 0.5


def sfx_thud(f=1.0):
	n = int(1.2 * SR)
	t = np.arange(n) / SR
	fr = 62 * f + 60 * np.exp(-t * 18)
	w = np.sin(2 * np.pi * np.cumsum(fr) / SR) * np.exp(-t * 4.2)
	w += lp(RNG.standard_normal(n), 600) * np.exp(-t * 30) * 0.3
	return w


def sfx_shimmer(g=1.0):
	n = int(2.4 * SR)
	t = np.arange(n) / SR
	out = np.zeros((n, 2))
	notes = [86, 90, 93, 97, 98, 102]
	for i, m in enumerate(notes):
		st = int(i * 0.07 * SR)
		tt = t[: n - st]
		w = np.sin(2 * np.pi * midi(m) * tt) * np.exp(-tt * 3.2) * np.minimum(1, tt / 0.004)
		w += 0.3 * np.sin(2 * np.pi * midi(m) * 2.01 * tt) * np.exp(-tt * 5)
		p = -0.7 + 1.4 * i / (len(notes) - 1)
		out[st:, 0] += w * np.cos((p + 1) * np.pi / 4)
		out[st:, 1] += w * np.sin((p + 1) * np.pi / 4)
	return out * 0.09 * g


def sfx_swell(g=1.0):
	n = int(1.6 * SR)
	u = np.linspace(0, 1, n)
	shape = np.sin(np.pi * np.clip(u * 1.25, 0, 1)) ** 2
	x = RNG.standard_normal(n)
	centers = 600 + 2400 * np.sin(np.pi * np.linspace(0, 1, n // 512 + 1))
	return sweep_bp(x, centers, q=1.4) * shape * g


def sfx_merge():
	n = int(1.8 * SR)
	t = np.arange(n) / SR
	w = np.zeros(n)
	for m, st in ((81, 0.0), (88, 0.09)):
		s = int(st * SR)
		tt = t[: n - s]
		w[s:] += np.sin(2 * np.pi * midi(m) * tt) * np.exp(-tt * 3.0) * np.minimum(1, tt / 0.003)
	return w * 0.2


# ------------------------------------------------------------------ score

def render(cues_path, out_path):
	data = json.load(open(cues_path))
	total = data["total"]
	cues = data["cues"]
	n = int((total + 0.2) * SR)
	music = np.zeros((n, 2))
	pluck_bus = np.zeros((n, 2))
	drums = np.zeros((n, 2))
	sfx = np.zeros((n, 2))
	sfx_dry = np.zeros((n, 2))
	sub_bus = np.zeros((n, 2))

	scenes = [c for c in cues if c["type"] == "scene"]
	start_of = {}
	for c in scenes:
		start_of.setdefault(c["scene"], c["t"])
	t_intro_hit = next((c["t"] for c in cues if c["type"] == "hit"), 1.3)
	t_end = start_of.get("end", total - 6)
	t_local = start_of.get("local")
	local_end = None
	if t_local is not None:
		idx = [c["scene"] for c in scenes].index("local")
		local_end = scenes[idx + 1]["t"] if idx + 1 < len(scenes) else None

	# Chords: Dmaj9, Bm11, Gmaj9(#11), Asus — voicings in MIDI.
	chords = [
		[50, 57, 62, 66, 69, 76],
		[47, 54, 61, 62, 66, 73],
		[43, 50, 57, 59, 66, 73],
		[45, 52, 59, 62, 64, 71],
	]
	bars2 = BEAT * 8  # two bars per chord
	# The progression starts on the intro hit so the first chord lands with the logo.
	t0 = t_intro_hit
	k = 0
	t = t0
	while t < t_end + 0.01:
		ch = chords[k % 4]
		dur = min(bars2, t_end - t) + 2.2
		for m in ch:
			v = pad_voice(midi(m), dur, bright=1100 if m < 55 else 1500)
			e = env_adsr(len(v), 1.4, 0.8, 0.85, 1.8, dur - 4.0)
			place(music, t, v * e[:, None], gain=0.055 if m >= 55 else 0.07)
		k += 1
		t += bars2
	# Opening drone before the hit, rising out of silence.
	dr = pad_voice(midi(38), t0 + 0.6, bright=700) + pad_voice(midi(45), t0 + 0.6, bright=900)
	dr *= np.linspace(0, 1, len(dr))[:, None] ** 1.5
	place(music, 0.0, dr, gain=0.06)
	# Final chord: Dmaj9 with an added high E, long tail.
	fin = [38, 50, 57, 62, 66, 69, 76, 81]
	fdur = total - t_end + 0.2
	for m in fin:
		v = pad_voice(midi(m), fdur, bright=1600)
		e = env_adsr(len(v), 0.9, 1.2, 0.8, 2.6, fdur - 4.7)
		place(music, t_end + 0.5, v * e[:, None], gain=0.06)

	# Sub bass on chord roots, pulsing in eighths with a sidechain feel.
	t_sub = start_of.get("statement", t0 + 4)
	t = t0
	k = 0
	while t < t_end:
		root = chords[k % 4][0] - 12
		for b in range(16):
			tb = t + b * BEAT / 2
			if tb < t_sub or tb >= t_end:
				continue
			if local_end and t_local and t_local + 0.3 <= tb < local_end - 2.0:
				continue
			nn = int(BEAT / 2 * SR)
			tt = np.arange(nn) / SR
			w = np.sin(2 * np.pi * midi(root) * tt) + 0.18 * np.sin(4 * np.pi * midi(root) * tt)
			w *= np.minimum(1, tt / 0.03) * (1 - tt / (BEAT / 2)) ** 0.8
			place(sub_bus, tb, w, gain=0.06 if b % 2 == 0 else 0.032)
		k += 1
		t += bars2

	# Pluck arpeggio and light drums through the feature section.
	t_feat = start_of.get("hero", t0 + 8)
	t_feat_end = start_of.get("stack", start_of.get("end", t_end))
	pattern = [0, 2, 4, 3, 5, 4, 2, 3]
	t = t0
	k = 0
	while t < t_end:
		ch = chords[k % 4]
		tones = sorted(m + 12 for m in ch[2:])
		for b in range(16):
			tb = t + b * BEAT / 2
			if not (t_feat <= tb < t_feat_end):
				continue
			if local_end and t_local and t_local + 0.3 <= tb < local_end - 2.0:
				continue
			m = tones[pattern[b % 8] % len(tones)]
			vel = 0.75 + 0.25 * ((b % 4) == 0)
			pan = 0.45 if b % 2 else -0.45
			place(pluck_bus, tb, pluck(midi(m), decay=0.7), gain=0.05 * vel, pan=pan)
			if b % 4 == 0:
				place(drums, tb, kick(), gain=0.24 if b % 8 == 0 else 0.15)
			if b % 2 == 1:
				place(drums, tb, hat(), gain=0.045, pan=0.25)
		k += 1
		t += bars2

	# Sound effects on the cues.
	for c in cues:
		ty, t = c["type"], c["t"]
		g = c.get("g", 1.0)
		f = c.get("f", 1.0)
		d = c.get("d", 1.0)
		if ty == "whoosh":
			place(sfx, t - d * 0.45, sfx_whoosh(d, f), gain=0.10 * g)
		elif ty == "swish":
			place(sfx, t - 0.05, sfx_swish(f), gain=0.07 * g, pan=-0.2)
		elif ty == "riser":
			place(sfx, t, sfx_riser(d), gain=0.10 * g)
		elif ty == "hit":
			place(sfx, t, sfx_hit(), gain=0.42 * g)
		elif ty == "tick":
			place(sfx_dry, t, sfx_tick(f), gain=0.10 * g, pan=0.15)
		elif ty == "pop":
			place(sfx_dry, t, sfx_pop(f), gain=0.10 * g, pan=RNG.uniform(-0.4, 0.4))
		elif ty == "key":
			place(sfx_dry, t, sfx_key(f), gain=0.26 * g)
		elif ty == "type":
			place(sfx_dry, t, sfx_type(), gain=0.07 * g, pan=RNG.uniform(-0.25, 0.25))
		elif ty == "thud":
			place(sfx, t, sfx_thud(f), gain=0.30 * g)
		elif ty == "shimmer":
			place(sfx, t, sfx_shimmer(g), gain=1.0)
		elif ty == "swell":
			place(sfx, t - 0.5, sfx_swell(g), gain=0.05)
		elif ty == "merge":
			place(sfx, t, sfx_merge(), gain=1.0)

	ir_big = reverb_ir(3.4, 5200)
	ir_small = reverb_ir(1.6, 7000, seed=5)
	mix = music * 0.9 + convolve(music, ir_big) * 0.32
	for c in range(2):
		sub_bus[:, c] = lp(sub_bus[:, c], 140, order=4)
	mix += sub_bus
	mix += pluck_bus * 0.8 + convolve(pluck_bus, ir_big) * 0.45
	mix += drums * 0.85 + convolve(drums, ir_small) * 0.10
	mix += sfx * 0.85 + convolve(sfx, ir_big) * 0.30
	mix += sfx_dry + convolve(sfx_dry, ir_small) * 0.18
	for c in range(2):
		mix[:, c] = hp(mix[:, c], 24, order=2)
	# Fade at the very end and a soft limiter.
	fade = int(1.4 * SR)
	mix[-fade:] *= np.linspace(1, 0, fade)[:, None] ** 1.5
	mix = np.tanh(mix * 1.6) / 1.6
	peak = np.abs(mix).max()
	mix = mix / peak * 0.89
	wavfile.write(out_path, SR, mix.astype(np.float32))
	print(f"wrote {out_path} {total:.2f}s peak={peak:.3f}")


if __name__ == "__main__":
	render(sys.argv[1], sys.argv[2])
