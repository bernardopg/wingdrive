import {describe, expect, test} from 'bun:test';
import type {FilePickerRequest} from '../../contexts/PlatformContext';
import {
	acceptLabel,
	buildSelection,
	globToRegExp,
	initialChoices,
	initialFilter,
	isFilePickerWindow,
	joinPath,
	matchesFilter,
	nameProblem,
	type PickerState
} from './filePicker';

function request(overrides: Partial<FilePickerRequest> = {}, options: Partial<FilePickerRequest['options']> = {}): FilePickerRequest {
	return {
		id: '1',
		mode: 'open',
		app_id: 'org.example.App',
		parent_window: '',
		title: 'Open',
		start_folder: '/home/me',
		suggested_name: null,
		...overrides,
		options: {
			accept_label: null,
			modal: true,
			multiple: false,
			directory: false,
			filters: [],
			current_filter: null,
			choices: [],
			current_folder: null,
			current_name: null,
			current_file: null,
			files: [],
			...options
		}
	};
}

function state(overrides: Partial<PickerState> = {}): PickerState {
	return {picked: [], currentFolder: '/home/me', name: '', filter: null, choices: {}, ...overrides};
}

const png = {name: 'Photo.PNG', path: '/home/me/Photo.PNG', isDirectory: false};
const txt = {name: 'notes.txt', path: '/home/me/notes.txt', isDirectory: false};
const folder = {name: 'Docs', path: '/home/me/Docs', isDirectory: true};
const images = {name: 'Images', rules: [], globs: ['*.png', '*.jp[e]g']};

describe('globs', () => {
	test('match whole names, ignoring case', () => {
		expect(globToRegExp('*.png').test('a.PNG')).toBe(true);
		expect(globToRegExp('*.png').test('a.png.txt')).toBe(false);
		expect(globToRegExp('file?.txt').test('file1.txt')).toBe(true);
		expect(globToRegExp('[!a]*').test('abc')).toBe(false);
		expect(globToRegExp('[!a]*').test('bcd')).toBe(true);
		expect(globToRegExp('a+b(c).*').test('a+b(c).x')).toBe(true);
		expect(globToRegExp('[unclosed').test('[unclosed')).toBe(true);
	});

	test('filters without globs let everything through', () => {
		expect(matchesFilter('x.txt', undefined)).toBe(true);
		expect(matchesFilter('x.txt', {name: 'All', rules: [], globs: []})).toBe(true);
		expect(matchesFilter('x.jpeg', images)).toBe(true);
		expect(matchesFilter('x.txt', images)).toBe(false);
	});
});

describe('open', () => {
	test('returns the picked file paths', () => {
		expect(buildSelection(request(), state({picked: [png]}))).toEqual({
			ok: true,
			selection: {kind: 'open', paths: [png.path], choices: [], current_filter: null}
		});
	});

	test('follows multiple, directory and filters', () => {
		expect(buildSelection(request(), state())).toEqual({ok: false, problem: 'nothingSelected'});
		expect(buildSelection(request(), state({picked: [png, txt]}))).toEqual({ok: false, problem: 'onlyOne'});
		expect(buildSelection(request({}, {multiple: true}), state({picked: [png, txt]})).ok).toBe(true);
		expect(buildSelection(request(), state({picked: [folder]}))).toEqual({ok: false, problem: 'notFile'});

		const filtered = request({}, {filters: [images]});
		expect(buildSelection(filtered, state({picked: [txt], filter: 0}))).toEqual({
			ok: false,
			problem: 'filterMismatch'
		});
		const chosen = buildSelection(filtered, state({picked: [png], filter: 0}));
		expect(chosen.ok && chosen.selection.kind === 'open' && chosen.selection.current_filter).toBe(0);
	});

	test('folder mode takes folders, or the folder on screen', () => {
		const folders = request({}, {directory: true});
		expect(buildSelection(folders, state({picked: [png]}))).toEqual({ok: false, problem: 'notFolder'});
		expect(buildSelection(folders, state({picked: [folder]}))).toMatchObject({
			ok: true,
			selection: {paths: ['/home/me/Docs']}
		});
		expect(buildSelection(folders, state())).toMatchObject({ok: true, selection: {paths: ['/home/me']}});
		expect(buildSelection(folders, state({currentFolder: undefined}))).toEqual({
			ok: false,
			problem: 'nothingSelected'
		});
	});

	test('entries without a local path are refused', () => {
		const cloud = {name: 'a.png', path: undefined, isDirectory: false};
		expect(buildSelection(request(), state({picked: [cloud]}))).toEqual({ok: false, problem: 'notLocal'});
	});
});

describe('save', () => {
	const save = request({mode: 'save'});

	test('joins the name to the folder on screen', () => {
		expect(buildSelection(save, state({name: ' report.pdf ', choices: {enc: 'utf8'}}))).toEqual({
			ok: true,
			selection: {
				kind: 'save',
				path: '/home/me/report.pdf',
				overwrite: false,
				choices: [['enc', 'utf8']],
				current_filter: null
			}
		});
		expect(buildSelection(save, state({name: 'a', currentFolder: '/'}))).toMatchObject({
			selection: {path: '/a'}
		});
	});

	test('refuses bad names and remote folders', () => {
		expect(buildSelection(save, state({name: '  '}))).toEqual({ok: false, problem: 'emptyName'});
		expect(buildSelection(save, state({name: 'a/b'}))).toEqual({ok: false, problem: 'badName'});
		expect(nameProblem('..')).toBe('badName');
		expect(nameProblem('ok.txt')).toBeNull();
		expect(buildSelection(save, state({name: 'x', currentFolder: undefined}))).toEqual({
			ok: false,
			problem: 'notLocal'
		});
	});

	test('save_files returns the folder', () => {
		expect(buildSelection(request({mode: 'save_files'}), state())).toEqual({
			ok: true,
			selection: {kind: 'save_files', folder: '/home/me', choices: []}
		});
	});
});

describe('helpers', () => {
	test('initial values', () => {
		const withChoices = request({}, {
			choices: [
				{id: 'enc', label: 'Encoding', options: [{id: 'utf8', label: 'UTF-8'}], initial: ''},
				{id: 'flag', label: 'Flag', options: [], initial: 'true'},
				{id: 'box', label: 'Box', options: [], initial: ''}
			],
			filters: [images, images],
			current_filter: 1
		});
		expect(initialChoices(withChoices)).toEqual({enc: 'utf8', flag: 'true', box: 'false'});
		expect(initialFilter(withChoices)).toBe(1);
		expect(initialFilter(request())).toBeNull();
		expect(initialFilter(request({}, {filters: [images]}))).toBe(0);
	});

	test('labels, paths and window labels', () => {
		expect(acceptLabel('_Upload')).toBe('Upload');
		expect(acceptLabel('Save__as')).toBe('Save_as');
		expect(acceptLabel(null)).toBeNull();
		expect(joinPath('/home/me/', 'a')).toBe('/home/me/a');
		expect(isFilePickerWindow('file-chooser-3')).toBe(true);
		expect(isFilePickerWindow('main')).toBe(false);
		expect(isFilePickerWindow(undefined)).toBe(false);
	});
});
