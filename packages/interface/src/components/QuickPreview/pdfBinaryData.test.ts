import {describe, expect, test} from 'bun:test';

import {bytesFromDataUrl} from './pdfBinaryData';

describe('bytesFromDataUrl', () => {
	test('decodes base64 data URLs byte for byte', () => {
		expect([...bytesFromDataUrl('data:application/wasm;base64,AGFzbQ==')]).toEqual([
			0, 97, 115, 109
		]);
		expect([...bytesFromDataUrl('data:font/ttf;base64,')]).toEqual([]);
	});

	test('rejects URLs that are not base64 data', () => {
		expect(() => bytesFromDataUrl('data:text/plain,hello')).toThrow();
		expect(() => bytesFromDataUrl('/assets/font.ttf')).toThrow();
	});
});
