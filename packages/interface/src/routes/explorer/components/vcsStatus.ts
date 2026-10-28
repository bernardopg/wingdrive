import type {File, VcsFileStatus, VcsStatusOutput} from '@wingdrive/ts-client';

/**
 * Git state of a listed entry, or `null` when it matches the last commit.
 *
 * The daemon keys entries by file name, so this reads the last component of the
 * physical path rather than `name`, which leaves the extension out.
 */
export function vcsStatusOf(
	status: VcsStatusOutput | undefined,
	file: File
): VcsFileStatus | null {
	if (!status?.repository_root) return null;
	if (!('Physical' in file.wing_path)) return null;

	const name = file.wing_path.Physical.path.split(/[\\/]/).filter(Boolean).pop();
	if (!name) return null;

	return status.entries[name] ?? status.inherited_status ?? null;
}
