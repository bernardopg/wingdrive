# WingDrive media kit

Logos, colors, type and social images for writing about WingDrive. Everything here is
vector or generated from vectors; the wordmark is converted to outlines, so the SVGs need no
installed font.

## Logo

The mark is a wing of three stacked feathers on a rounded night tile. The feathers run from
violet `#8B5CF6` to indigo `#4338CA`, at 100 %, 72 % and 44 % opacity.

| File | Use |
| --- | --- |
| `logo/wingdrive-icon.svg` | App icon: feathers on the night tile. The default. |
| `logo/wingdrive-icon-light.svg` | App icon on a light tile, for light layouts. |
| `logo/wingdrive-mark.svg` | Feathers only, transparent background. |
| `logo/wingdrive-mark-white.svg`, `-black.svg` | One-color marks for photos, print and dark or light fills. |
| `logo/wingdrive-lockup-horizontal-dark.svg`, `-light.svg` | Icon and wordmark side by side. |
| `logo/wingdrive-lockup-stacked-dark.svg`, `-light.svg` | Icon above the wordmark, for square spaces. |
| `logo/wingdrive-wordmark-*.svg` | Wordmark alone: white, black, or two-tone for dark and light. |
| `logo/png/` | Every logo as PNG, 256 to 2400 px wide. |

- "Wing" takes the text color; "Drive" takes the accent (`#A78BFA` on dark, `#6D28D9` on light).
- Keep clear space of at least half the icon's width around any lockup.
- Do not recolor the feathers, rotate the mark, add effects, or set the wordmark in another font.
- Write the name as one word with two capitals: **WingDrive**.

## Colors

`palette.svg` / `palette.png`, values in `colors.json`.

| Name | Hex | Use |
| --- | --- | --- |
| Violet | `#8B5CF6` | Primary accent, feather highlight |
| Indigo | `#4338CA` | Feather shadow, gradient end |
| Lilac | `#C4B5FD` | Highlights on dark, gradient text start |
| Lavender | `#EDE9FE` | Text on dark |
| Night | `#0F0A1F` | App icon tile |
| Ink | `#05040B` | Video and page background |
| Mist | `#A19FB8` | Body text on dark |
| Paper | `#FAF9FF` | Light background |

The signature gradient for headlines runs left to right through Lavender, Lilac, Violet and
`#6366F1`.

## Type

- **Space Grotesk** SemiBold (600), tight tracking (-0.03 em): wordmark and headlines.
- **Inter** Regular and Medium: body text.
- **JetBrains Mono** Medium, uppercase, wide tracking (0.24 em): eyebrows, paths and code.

All three are open fonts (SIL OFL) on Google Fonts and Fontsource.

## Social images

| File | Size | Use |
| --- | --- | --- |
| `social/github-social-preview-1280x640.png` | 1280 × 640 | GitHub repository social preview |
| `social/youtube-thumbnail-1280x720.png` | 1280 × 720 | YouTube video thumbnail |
| `social/youtube-banner-2560x1440.png` | 2560 × 1440 | YouTube channel art (content inside the 1546 × 423 safe area) |
| `social/vertical-poster-1080x1920.png` | 1080 × 1920 | Shorts cover, stories |

## Videos

| Cut | Format | Download |
| --- | --- | --- |
| Film | 1920 × 1080, 60 fps, 35 s, under 10 MB | [wingdrive-film.mp4](https://wingdrive.org/video/wingdrive-film.mp4) |
| Tour | 1920 × 1080, 60 fps, 1:39 | [wingdrive-tour.mp4](https://wingdrive.org/video/wingdrive-tour.mp4) |
| Shorts | 1080 × 1920, 60 fps, 52 s | [wingdrive-shorts.mp4](https://wingdrive.org/video/wingdrive-shorts.mp4) |

The files live in `website/public/video/`. Audio is mixed to -14 LUFS, -1.5 dBTP. Every frame
and sound is generated from code in `website/raw-assets/promo/`; `./build.sh <cut>` renders a
master into `out/export/`.
