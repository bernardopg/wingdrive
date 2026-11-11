# wingdrive.org

The WingDrive website: an [Astro](https://astro.build) static site in English (`/`) and Brazilian
Portuguese (`/pt/`), with a canvas sparkle layer over the hero art, a Three.js library scene and
SVG animations. It is a standalone Bun project with its own `bun.lock`, outside the root workspace.

```bash
cd website && bun install && bun run dev   # http://localhost:4321
bun run build                                  # static output in website/dist/
```

## Content

- Marketing copy for both languages: `src/i18n/strings.ts`.
- Docs: `src/content/docs/en/*.md` and `src/content/docs/pt/*.md`, same file names.
- The changelog page renders the root `CHANGELOG.md` (copied into `src/data/` on every dev and
  build run, not committed).
- The download page reads the latest GitHub Release at build time.

## Assets

- `raw-assets/gen/`: generated art, the source of every size in `public/img/art/`.
- App screenshots come from `../docs/public/Wing*.webp`.
- `raw-assets/blender/winged_drive.py`: the Blender scene behind `public/video/winged-drive.*`
  (`blender -b -P winged_drive.py -- <out-dir> anim`, then encode the frames with ffmpeg).
- `bun run assets` rebuilds the image sizes, the hero sparkle points
  (`src/data/hero-points.json`), favicons, manifest icons and `public/og.png`.

## Deploy

`.github/workflows/website.yml` builds every pull request that touches `website/` and deploys
`main` to Hostinger over SSH (rsync into `domains/wingdrive.org/public_html`). It also redeploys
after each `WingDrive Release` run and daily, so new releases reach the download page and the
changelog with no manual step. Credentials live only in the `website` GitHub environment.
`public/.htaccess` forces HTTPS, sets the 404 page and cache headers.
