# NOTICE

## Origin

WingDrive is a community continuation fork of [Spacedrive](https://github.com/spacedriveapp/spacedrive).

- Upstream repository: `https://github.com/spacedriveapp/spacedrive`
- Fork point: commit `6dfeccf2113039e35f2ce735f945e70dc3e4ea45`
- Fork maintainer: bernardopg

The full upstream commit history is preserved in this repository. Authorship of
every original commit remains with its author.

## Vendored UI Packages

`packages/wingdrive-ai`, `packages/wingdrive-primitives`, and
`packages/wingdrive-tokens` are derived from the MIT-licensed SpaceUI packages
originally published by Spacedrive. Their original copyright and MIT license
are preserved in each package directory.

## Reused visual assets

WingDrive uses its own wing mark for app branding. Generic file-kind and device
icons in `packages/assets/icons` remain from upstream under the package's
GPL-3.0-only license. Generic UI glyphs remain under their existing terms.
Third-party service logos identify their integrations. These are kept because
replacing them does not improve file recognition. The GPL text is included in
`packaging/licenses/assets-GPL-3.0.txt` and distributed with the desktop bundle.

## Copyright

Copyright 2026 Spacedrive Technology Inc.

The original Spacedrive source code is the copyright of Spacedrive Technology
Inc. and its contributors. Modifications made in this fork are the copyright of
their respective authors and are licensed under the same terms.

Copyright notices provided in or with the original Software have not been
removed.

## License

This project is licensed under the **Apache License, Version 2.0**. See
[LICENSE](./LICENSE) for the full text.

Spacedrive Technology Inc. relicensed Spacedrive to Apache-2.0 in upstream
commit `bcc124765c809e115669e770b918625942621bce` ("Relicense to Apache-2.0",
2026-08-21). That commit's only parent is the WingDrive fork point
`6dfeccf2113039e35f2ce735f945e70dc3e4ea45`, so the exact tree WingDrive was
forked from was published by its copyright holder under Apache-2.0. WingDrive
merges that commit to record this provenance.

Earlier WingDrive releases (up to `v2.0.0-alpha.7`) were distributed under
FSL-1.1-ALv2. Upstream states that its releases prior to 2026-03-24 remain
available under AGPL-3.0.

Exceptions:

- `packages/assets/icons` remain under the package's GPL-3.0-only license, as
  described in "Reused visual assets".
- `packages/wingdrive-ai`, `packages/wingdrive-primitives`, and
  `packages/wingdrive-tokens` remain under MIT.
- `crates/sdk` is dual-licensed MIT OR Apache-2.0.

## Trademarks

"Spacedrive" is a trademark of Spacedrive Technology Inc. Section 6 of the
Apache License grants no permission to use it beyond identifying the origin of
the Work.

This fork is named **WingDrive** and is **not affiliated with, endorsed by, or
supported by Spacedrive Technology Inc.** Do not report WingDrive issues to the
upstream project.

## Contributions

Contributions to this repository are accepted under Apache-2.0, per section 5
of the license. By opening a pull request you agree that your contribution is
licensed under those terms, including the patent grant in section 3.
