# WingDrive UI packages

The UI migration is complete. The maintained frontend imports shared code from
the local `@wingdrive/primitives`, `@wingdrive/tokens`, and `@wingdrive/ai`
workspaces.

These packages are part of this repository and use the root Bun lockfile. No
SpaceUI clone or package publication is required to install, type-check, build,
or release WingDrive.

Original MIT copyright notices remain in each vendored package. New WingDrive
changes are developed and reviewed here.

Run these checks after changing the shared packages:

```bash
bun install --frozen-lockfile
bun run typecheck
bun test packages/interface/src/routes/explorer/hooks/resolveExternalDrop.test.ts
```
