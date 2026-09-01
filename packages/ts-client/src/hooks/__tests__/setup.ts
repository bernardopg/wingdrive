/**
 * Test setup for Bun test runner
 * Provides DOM environment for React Testing Library
 */

import { JSDOM } from 'jsdom';

const dom = new JSDOM('<!DOCTYPE html><html><body></body></html>', {
	url: 'http://localhost',
});

Object.defineProperties(globalThis, {
	document: { configurable: true, value: dom.window.document },
	window: { configurable: true, value: dom.window },
	navigator: { configurable: true, value: dom.window.navigator },
});
