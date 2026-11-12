import type {
	FilePickerFilter,
	FilePickerRequest,
	FilePickerSelection
} from '../../contexts/PlatformContext';

/** A picker window's label starts with this; the daemon side names them so. */
export const FILE_PICKER_WINDOW_PREFIX = 'file-chooser-';

export function isFilePickerWindow(label: string | undefined): boolean {
	return !!label && label.startsWith(FILE_PICKER_WINDOW_PREFIX);
}

/**
 * A shell glob as a case-insensitive regular expression over a file name.
 * Browsers send `*.PNG` and `*.png` alike, so case is ignored.
 */
export function globToRegExp(glob: string): RegExp {
	let source = '';
	for (let i = 0; i < glob.length; i++) {
		const char = glob[i]!;
		if (char === '*') source += '.*';
		else if (char === '?') source += '.';
		else if (char === '[') {
			const end = glob.indexOf(']', i + 1);
			if (end === -1) {
				source += '\\[';
				continue;
			}
			let body = glob.slice(i + 1, end);
			if (body.startsWith('!')) body = '^' + body.slice(1);
			source += '[' + body.replace(/\\/g, '\\\\') + ']';
			i = end;
		} else source += char.replace(/[.+^${}()|\\/]/g, '\\$&');
	}
	return new RegExp(`^${source}$`, 'i');
}

/** Whether a file name passes a filter. No filter, or one without globs, passes everything. */
export function matchesFilter(name: string, filter: FilePickerFilter | undefined): boolean {
	if (!filter || filter.globs.length === 0) return true;
	return filter.globs.some((glob) => globToRegExp(glob).test(name));
}

/** An entry the user picked; `path` is unset for an entry with no local path. */
export interface PickedEntry {
	name: string;
	path: string | undefined;
	isDirectory: boolean;
}

/** Translation keys under `explorer:filePicker.errors`. */
export type PickerProblem =
	| 'nothingSelected'
	| 'onlyOne'
	| 'notFolder'
	| 'notFile'
	| 'notLocal'
	| 'filterMismatch'
	| 'emptyName'
	| 'badName';

export type PickerResult =
	| {ok: true; selection: FilePickerSelection}
	| {ok: false; problem: PickerProblem};

export interface PickerState {
	picked: PickedEntry[];
	/** The folder the Explorer shows, when it is local */
	currentFolder: string | undefined;
	name: string;
	filter: number | null;
	choices: Record<string, string>;
}

export function joinPath(folder: string, name: string): string {
	return folder.endsWith('/') ? folder + name : `${folder}/${name}`;
}

/** Checks a save name: one path component that is not `.` or `..`. */
export function nameProblem(name: string): PickerProblem | null {
	const trimmed = name.trim();
	if (trimmed === '') return 'emptyName';
	if (trimmed.includes('/') || trimmed === '.' || trimmed === '..') return 'badName';
	return null;
}

function choiceList(choices: Record<string, string>): [string, string][] {
	return Object.entries(choices);
}

/**
 * What the confirm button would return for the current state.
 *
 * Open in folder mode takes the selected folders, or the folder on screen
 * when nothing is selected. Saves go to the folder on screen.
 */
export function buildSelection(request: FilePickerRequest, state: PickerState): PickerResult {
	const {options} = request;
	const choices = choiceList(state.choices);
	if (request.mode === 'open') {
		const picked =
			options.directory && state.picked.length === 0 && state.currentFolder
				? [{name: '', path: state.currentFolder, isDirectory: true}]
				: state.picked;
		if (picked.length === 0) return {ok: false, problem: 'nothingSelected'};
		if (picked.length > 1 && !options.multiple) return {ok: false, problem: 'onlyOne'};
		if (picked.some((entry) => !entry.path)) return {ok: false, problem: 'notLocal'};
		if (options.directory && picked.some((entry) => !entry.isDirectory))
			return {ok: false, problem: 'notFolder'};
		if (!options.directory && picked.some((entry) => entry.isDirectory))
			return {ok: false, problem: 'notFile'};
		const filter = state.filter === null ? undefined : options.filters[state.filter];
		if (!options.directory && picked.some((entry) => !matchesFilter(entry.name, filter)))
			return {ok: false, problem: 'filterMismatch'};
		return {
			ok: true,
			selection: {
				kind: 'open',
				paths: picked.map((entry) => entry.path!),
				choices,
				current_filter: state.filter
			}
		};
	}
	if (!state.currentFolder) return {ok: false, problem: 'notLocal'};
	if (request.mode === 'save_files') {
		return {
			ok: true,
			selection: {kind: 'save_files', folder: state.currentFolder, choices}
		};
	}
	const problem = nameProblem(state.name);
	if (problem) return {ok: false, problem};
	return {
		ok: true,
		selection: {
			kind: 'save',
			path: joinPath(state.currentFolder, state.name.trim()),
			overwrite: false,
			choices,
			current_filter: state.filter
		}
	};
}

/** The choices' starting values, keyed by choice id. */
export function initialChoices(request: FilePickerRequest): Record<string, string> {
	return Object.fromEntries(
		request.options.choices.map((choice) => [
			choice.id,
			choice.initial || (choice.options[0]?.id ?? 'false')
		])
	);
}

/** The filter shown first: the app's current filter, else the first one. */
export function initialFilter(request: FilePickerRequest): number | null {
	if (request.options.filters.length === 0) return null;
	return request.options.current_filter ?? 0;
}

/**
 * The accept label without GTK mnemonic underscores (`_Open` to `Open`,
 * `__` to a literal underscore).
 */
export function acceptLabel(label: string | null): string | null {
	if (!label) return null;
	return label.replace(/__|_/g, (match) => (match === '__' ? '_' : ''));
}
