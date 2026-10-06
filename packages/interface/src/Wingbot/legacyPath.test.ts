import {describe, expect, it} from 'bun:test';
import {legacyWingbotPath} from './legacyPath';

describe('legacyWingbotPath', () => {
	it('maps the old root and nested routes', () => {
		expect(legacyWingbotPath('/spacebot')).toBe('/wingbot');
		expect(legacyWingbotPath('/spacebot/chat/conversation/abc')).toBe(
			'/wingbot/chat/conversation/abc'
		);
	});

	it('leaves other paths alone', () => {
		expect(legacyWingbotPath('/spacebots')).toBe('/spacebots');
		expect(legacyWingbotPath('/explorer/spacebot')).toBe('/explorer/spacebot');
	});
});
