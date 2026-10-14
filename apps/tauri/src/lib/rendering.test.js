import {readFileSync} from 'node:fs';
import {describe, expect, mock, test} from 'bun:test';
import {configureRendering} from './rendering';

describe('desktop rendering policy', () => {
	test('reduces backdrop compositing in the Linux main window', () => {
		const toggleAttribute = mock(() => true);
		configureRendering({toggleAttribute}, 'Linux x86_64', 'main');
		expect(toggleAttribute).toHaveBeenCalledWith(
			'data-reduced-transparency',
			true
		);
	});

	for (const platform of ['MacIntel', 'Win32']) {
		test(`preserves effects on ${platform}`, () => {
			const toggleAttribute = mock(() => false);
			configureRendering({toggleAttribute}, platform, 'main');
			expect(toggleAttribute).toHaveBeenCalledWith(
				'data-reduced-transparency',
				false
			);
		});
	}

	for (const label of [
		'voice-overlay',
		'quick-preview',
		'settings',
		'drag-overlay'
	]) {
		test(`preserves effects in the ${label} window`, () => {
			const toggleAttribute = mock(() => false);
			configureRendering({toggleAttribute}, 'Linux x86_64', label);
			expect(toggleAttribute).toHaveBeenCalledWith(
				'data-reduced-transparency',
				false
			);
		});
	}

	test('keeps the CSS override opt-in and limits it to backdrop filters', () => {
		const css = readFileSync(
			new URL('../rendering.css', import.meta.url),
			'utf8'
		);
		expect(css).toContain(':root[data-reduced-transparency] #root *');
		expect(css).toMatch(/backdrop-filter:\s*none\s*!important/);
		expect(css).toMatch(/-webkit-backdrop-filter:\s*none\s*!important/);
		expect(css).not.toMatch(
			/(?:^|[;{])\s*(?:animation|transition|filter)\s*:/m
		);
	});
});
