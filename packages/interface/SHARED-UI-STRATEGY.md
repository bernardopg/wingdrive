# WingDrive shared UI

WingDrive keeps its shared design system in this monorepo. This makes desktop,
web, and mobile development independent from external package registries and
repositories.

## Workspaces

- `packages/wingdrive-tokens` owns themes and design tokens.
- `packages/wingdrive-primitives` owns reusable UI primitives.
- `packages/wingdrive-ai` owns optional agent interface components.
- `packages/interface` owns WingDrive product views and business logic.

Use `@wingdrive/*` workspace imports. Do not add runtime or build dependencies
on the original SpaceUI repository. The vendored packages retain their original
MIT notices in their `LICENSE` files.

Keep product-specific components in `packages/interface` until at least two
maintained WingDrive surfaces need the same component. This avoids creating
packages for speculative reuse.
