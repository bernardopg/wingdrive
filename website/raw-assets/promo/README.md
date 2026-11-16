# WingDrive promo videos

Source of the promo videos and the media kit in `/media-kit`. Every frame and every sound is
generated from code, so a cut can be re-rendered after a UI or copy change.

- `promo.html` + `engine.js`: the animation. Each frame is a pure function of time
  (`window.seek(t)`), and each scene builds its DOM once and returns a `render(t)`.
  Three cuts share the scenes: `youtube` (16:9, full tour), `github` (16:9, short) and
  `shorts` (9:16). Open `promo.html?cut=youtube` in a browser and call `seek(12)` to look
  at any instant.
- `render.mjs`: drives headless Chrome through `playwright-core`. Every output frame
  averages 4 sub-frames spread over 60 % of the frame (real motion blur); frames where
  nothing moves reuse one capture. Workers render frame ranges in parallel into lossless
  segments.
- `audio.py`: the score (pad, sub, pluck arpeggio, light drums in D major at 96 BPM) and the
  sound effects, synthesized with NumPy/SciPy and placed on the cues `engine.js` exports.
- `build.sh <cut> [workers]`: frames, audio, two-pass loudness normalization to -14 LUFS /
  -1.5 dBTP, H.264 + AAC MP4 in `out/export/`. The `github` cut also gets a copy under
  10 MB for READMEs.
- `mediakit.py`: logo SVGs (wordmark converted to outlines with fontTools), palette and PNG
  exports into `/media-kit`. `social.mjs` renders `social.html` into `/media-kit/social`.

## Setup

```bash
cd website && bun install                 # Fontsource fonts
cd raw-assets/promo && bun install        # playwright-core
uv venv .venv && uv pip install --python .venv/bin/python numpy scipy fonttools brotli
./build.sh github                         # about 15 minutes; youtube about 45
```

Chrome is taken from `/usr/bin/google-chrome-stable`; set `CHROME` to use another binary.
Rendering at 1080p60 with motion blur takes about 0.3 s per frame per worker.
