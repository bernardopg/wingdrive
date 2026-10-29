import {describe, expect, test} from 'bun:test';
import type {File, VcsStatusOutput} from '@wingdrive/ts-client';

import {vcsStatusOf} from './vcsStatus';

const file = (path: string) =>
	({wing_path: {Physical: {device_slug: 'local', path}}}) as unknown as File;

const status = (overrides: Partial<VcsStatusOutput> = {}): VcsStatusOutput => ({
	repository_root: '/repo',
	inherited_status: null,
	entries: {},
	...overrides
});

describe('vcsStatusOf', () => {
	test('reads the entry by file name, extension included', () => {
		const output = status({entries: {'main.rs': 'modified', src: 'untracked'}});

		expect(vcsStatusOf(output, file('/repo/main.rs'))).toBe('modified');
		expect(vcsStatusOf(output, file('/repo/src/'))).toBe('untracked');
		expect(vcsStatusOf(output, file('/repo/README.md'))).toBeNull();
	});

	test('falls back to the state the whole folder inherits', () => {
		const output = status({inherited_status: 'untracked', entries: {'a.txt': 'added'}});

		expect(vcsStatusOf(output, file('/repo/new/b.txt'))).toBe('untracked');
		expect(vcsStatusOf(output, file('/repo/new/a.txt'))).toBe('added');
	});

	test('shows nothing outside a working tree or for virtual paths', () => {
		expect(vcsStatusOf(undefined, file('/repo/main.rs'))).toBeNull();
		expect(
			vcsStatusOf(status({repository_root: null, entries: {'main.rs': 'modified'}}), file('/repo/main.rs'))
		).toBeNull();
		expect(
			vcsStatusOf(status({entries: {x: 'modified'}}), {wing_path: {Content: {content_id: 'x'}}} as unknown as File)
		).toBeNull();
	});
});
