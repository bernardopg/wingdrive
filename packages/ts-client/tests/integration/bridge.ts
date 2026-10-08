/**
 * Gate for the Rust-driven bridge tests in this directory.
 *
 * These tests need a live daemon with an indexed library, which only the Rust
 * harnesses create (core/tests/typescript_bridge_test.rs,
 * typescript_search_bridge_test.rs, ephemeral_bridge_test.rs). The harness writes
 * a config file and passes its path in BRIDGE_CONFIG_PATH before spawning
 * `bun test`. A plain `bun test packages/ts-client` has no daemon, so the suites
 * are skipped with a warning instead of failing on the missing env var.
 */

import { beforeAll, describe } from "bun:test";

export const BRIDGE_CONFIG_PATH = process.env.BRIDGE_CONFIG_PATH;

if (!BRIDGE_CONFIG_PATH) {
	console.warn(
		"[ts-client] BRIDGE_CONFIG_PATH not set, skipping daemon bridge tests. Run them through the Rust harness, e.g. `cargo test -p wing-core --test typescript_bridge_test -- --nocapture` (see tests/integration/README.md).",
	);
}

/** `describe` that runs only when a Rust harness provided a bridge config. */
export const describeBridge = describe.skipIf(!BRIDGE_CONFIG_PATH);

/** `beforeAll` that runs only when a Rust harness provided a bridge config. */
export function beforeAllBridge(fn: () => Promise<void>) {
	beforeAll(async () => {
		if (BRIDGE_CONFIG_PATH) await fn();
	});
}
