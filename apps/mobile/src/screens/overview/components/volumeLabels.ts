// Labels for the volume rows of the Overview. Kept free of React Native imports so bun can test it.
// The labels are English catalog keys; LocalizedText translates them.

/** An enum the core sends either as a string or as `{ Other: string }`. */
type CoreEnum = string | { Other?: string } | null | undefined;

function enumValue(value: CoreEnum): string | null {
	if (!value) return null;
	if (typeof value === "string") return value;
	return value.Other ?? null;
}

/**
 * The file system chip, or null when it says nothing to the user.
 *
 * Android reports "android" for shared storage, a FUSE view whose real file system is hidden.
 */
export function fileSystemLabel(fileSystem: CoreEnum): string | null {
	const value = enumValue(fileSystem);
	if (!value) return null;
	const lower = value.toLowerCase();
	if (lower === "unknown" || lower === "android") return null;
	return value;
}

const VOLUME_TYPE_LABELS: Record<string, string> = {
	Primary: "Internal",
	UserData: "Internal",
	Secondary: "Internal",
	External: "External",
	Network: "Network",
	Cloud: "Cloud",
};

/** The volume type chip as a catalog key, or null for types the user does not need to see. */
export function volumeTypeLabel(volumeType: CoreEnum): string | null {
	const value = enumValue(volumeType);
	return value ? (VOLUME_TYPE_LABELS[value] ?? null) : null;
}

/** The volume types the library counts in its storage totals (`calculate_volume_capacity`). */
const COUNTED_TYPES = new Set(["Primary", "UserData", "External", "Secondary"]);

export interface CapacityVolume {
	fingerprint: string;
	volume_type?: CoreEnum;
	total_capacity?: number | null;
	available_space?: number | null;
	is_user_visible?: boolean | null;
}

/**
 * Total and available bytes for the Overview.
 *
 * The library counts tracked volumes only, so a phone whose storage is not tracked showed
 * "0 B" (MOB-017). When the library reports nothing, the detected volumes are counted instead.
 */
export function storageTotals(
	library: { total: number; available: number },
	volumes: readonly CapacityVolume[] | undefined,
): { total: number; available: number } {
	if (library.total > 0 || !volumes?.length) return library;
	const seen = new Set<string>();
	let total = 0;
	let available = 0;
	for (const volume of volumes) {
		const type = enumValue(volume.volume_type);
		if (!type || !COUNTED_TYPES.has(type)) continue;
		const capacity = volume.total_capacity ?? 0;
		if (volume.is_user_visible === false || capacity <= 0) continue;
		if (seen.has(volume.fingerprint)) continue;
		seen.add(volume.fingerprint);
		total += capacity;
		available += Math.max(0, volume.available_space ?? 0);
	}
	return { total, available };
}
