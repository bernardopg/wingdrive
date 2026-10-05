import {describe, expect, it} from 'bun:test';
import {effectiveViewMode} from './effectiveViewMode';

describe('effectiveViewMode', () => {
	it('keeps released views', () => {
		expect(effectiveViewMode('column', false)).toBe('column');
		expect(effectiveViewMode('media', false)).toBe('media');
	});

	it('hides the knowledge prototype in production', () => {
		expect(effectiveViewMode('knowledge', false)).toBe('grid');
		expect(effectiveViewMode('knowledge', true)).toBe('knowledge');
	});
});
