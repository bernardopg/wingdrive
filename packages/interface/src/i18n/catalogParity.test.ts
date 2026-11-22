import {describe, expect, it} from 'bun:test';
import {readdirSync, readFileSync} from 'node:fs';
import path from 'node:path';

const localesDir = path.join(import.meta.dir, 'locales');
const languages = readdirSync(localesDir).sort();
const namespaces = readdirSync(path.join(localesDir, 'en'))
	.filter((f) => f.endsWith('.json'))
	.sort();

function flatten(value: unknown, prefix = ''): Record<string, string> {
	if (typeof value === 'string') return {[prefix]: value};
	if (Array.isArray(value)) {
		const out: Record<string, string> = {};
		value.forEach((child, i) => Object.assign(out, flatten(child, `${prefix}[${i}]`)));
		return out;
	}
	if (!value || typeof value !== 'object') {
		throw new Error(`Unsupported value at ${prefix || '<root>'}`);
	}
	const out: Record<string, string> = {};
	for (const [key, child] of Object.entries(value as Record<string, unknown>)) {
		const next = prefix ? `${prefix}.${key}` : key;
		Object.assign(out, flatten(child, next));
	}
	return out;
}

function readCatalog(lang: string, ns: string) {
	const file = path.join(localesDir, lang, ns);
	return flatten(JSON.parse(readFileSync(file, 'utf8')));
}

describe('i18n catalog parity', () => {
	it('ships the expected language folders', () => {
		expect(languages).toEqual(['en', 'pt']);
	});

	for (const ns of namespaces) {
		it(`${ns} has identical keys in en and pt`, () => {
			const en = readCatalog('en', ns);
			const pt = readCatalog('pt', ns);
			expect(Object.keys(pt).sort()).toEqual(Object.keys(en).sort());
			for (const [key, value] of Object.entries(en)) {
				expect(value.trim().length > 0).toBe(true);
				expect(pt[key]?.trim().length > 0).toBe(true);
			}
		});
	}
});
