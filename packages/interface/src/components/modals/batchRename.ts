export interface BatchRenameRule {
	/** Literal text to replace in the name stem; empty skips the step */
	find: string;
	replace: string;
	/** `{name}` is the stem after find/replace, `{n}` the running number */
	template: string;
	start: number;
	/** Zero-pad `{n}` to this many digits */
	pad: number;
}

export interface BatchRenameSource {
	name: string;
	extension: string | null;
}

export interface BatchRenamePlan {
	newName: string;
	/** Why this name cannot be applied, if it cannot */
	error: string | null;
}

/**
 * Compute the new name for every source, keeping extensions untouched.
 * A name that is empty, contains a path separator or repeats another new
 * name in the batch is flagged instead of applied.
 */
export function planBatchRename(
	sources: BatchRenameSource[],
	rule: BatchRenameRule
): BatchRenamePlan[] {
	const names = sources.map((source, index) => {
		const stem = rule.find
			? source.name.split(rule.find).join(rule.replace)
			: source.name;
		const number = String(rule.start + index).padStart(rule.pad, '0');
		const base = rule.template
			.split('{name}')
			.join(stem)
			.split('{n}')
			.join(number)
			.trim();
		return source.extension ? `${base}.${source.extension}` : base;
	});

	const seen = new Map<string, number>();
	for (const name of names) {
		seen.set(name.toLowerCase(), (seen.get(name.toLowerCase()) ?? 0) + 1);
	}

	return names.map((newName, index) => {
		const stemEmpty = sources[index].extension
			? newName === `.${sources[index].extension}`
			: newName === '';
		let error: string | null = null;
		if (stemEmpty) error = 'Name is empty';
		else if (/[\\/]/.test(newName)) error = 'Name contains a slash';
		else if ((seen.get(newName.toLowerCase()) ?? 0) > 1)
			error = 'Duplicates another new name';
		return {newName, error};
	});
}
