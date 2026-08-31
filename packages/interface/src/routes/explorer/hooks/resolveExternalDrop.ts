import type {SdPath} from '@sd/ts-client';

/** Outcome of mapping an OS file drop onto the current Explorer destination */
export type ExternalDropRejectionReason =
	'no-files' | 'no-destination' | 'not-physical';

export type ExternalDropDecision =
	| {kind: 'copy'; destination: SdPath; sources: SdPath[]}
	| {kind: 'reject'; reason: ExternalDropRejectionReason};

/**
 * Decide what an operating-system file drop means for the current Explorer
 * folder. Only writable physical folders can receive imports; everything
 * else must be rejected explicitly so drops are never silently swallowed.
 */
export function resolveExternalDrop(
	destination: SdPath | null,
	paths: string[]
): ExternalDropDecision {
	if (paths.length === 0) {
		return {kind: 'reject', reason: 'no-files'};
	}
	if (!destination) {
		return {kind: 'reject', reason: 'no-destination'};
	}
	if (!('Physical' in destination)) {
		return {kind: 'reject', reason: 'not-physical'};
	}
	return {
		kind: 'copy',
		destination,
		sources: paths.map((path): SdPath => ({
			Physical: {device_slug: 'local', path}
		}))
	};
}
