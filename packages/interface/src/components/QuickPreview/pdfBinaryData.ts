/**
 * Binary data pdf.js asks the page for while it parses a document.
 *
 * The worker requests standard fonts (the Symbol and Dingbats fonts, and the
 * others when a PDF relies on them without embedding) and the WebAssembly
 * decoders for JPEG 2000 and JBIG2 images, which scanned PDFs use. pdf.js
 * would `fetch` them from a base URL, but the app's CSP does not list its own
 * origin in `connect-src`. Each file is bundled as an inline module instead
 * and imported only when a document needs it.
 *
 * Built-in CMaps are not bundled: they cover CJK PDFs that do not embed their
 * fonts, and pdf.js renders such text without them, with fallback glyphs. ICC
 * profiles need the worker to fetch on its own, so pdf.js converts those
 * colours with its built-in approximation.
 */

type Loader = () => Promise<{default: string}>;

const STANDARD_FONTS: Record<string, Loader> = {
	'FoxitDingbats.pfb': () => import('pdfjs-dist/standard_fonts/FoxitDingbats.pfb?inline'),
	'FoxitFixed.pfb': () => import('pdfjs-dist/standard_fonts/FoxitFixed.pfb?inline'),
	'FoxitFixedBold.pfb': () => import('pdfjs-dist/standard_fonts/FoxitFixedBold.pfb?inline'),
	'FoxitFixedBoldItalic.pfb': () =>
		import('pdfjs-dist/standard_fonts/FoxitFixedBoldItalic.pfb?inline'),
	'FoxitFixedItalic.pfb': () => import('pdfjs-dist/standard_fonts/FoxitFixedItalic.pfb?inline'),
	'FoxitSerif.pfb': () => import('pdfjs-dist/standard_fonts/FoxitSerif.pfb?inline'),
	'FoxitSerifBold.pfb': () => import('pdfjs-dist/standard_fonts/FoxitSerifBold.pfb?inline'),
	'FoxitSerifBoldItalic.pfb': () =>
		import('pdfjs-dist/standard_fonts/FoxitSerifBoldItalic.pfb?inline'),
	'FoxitSerifItalic.pfb': () => import('pdfjs-dist/standard_fonts/FoxitSerifItalic.pfb?inline'),
	'FoxitSymbol.pfb': () => import('pdfjs-dist/standard_fonts/FoxitSymbol.pfb?inline'),
	'LiberationSans-Bold.ttf': () =>
		import('pdfjs-dist/standard_fonts/LiberationSans-Bold.ttf?inline'),
	'LiberationSans-BoldItalic.ttf': () =>
		import('pdfjs-dist/standard_fonts/LiberationSans-BoldItalic.ttf?inline'),
	'LiberationSans-Italic.ttf': () =>
		import('pdfjs-dist/standard_fonts/LiberationSans-Italic.ttf?inline'),
	'LiberationSans-Regular.ttf': () =>
		import('pdfjs-dist/standard_fonts/LiberationSans-Regular.ttf?inline')
};

const WASM: Record<string, Loader> = {
	'jbig2.wasm': () => import('pdfjs-dist/wasm/jbig2.wasm?inline'),
	'openjpeg.wasm': () => import('pdfjs-dist/wasm/openjpeg.wasm?inline')
};

const TABLES: Record<string, Record<string, Loader>> = {
	standardFontDataUrl: STANDARD_FONTS,
	wasmUrl: WASM
};

/** Base URL pdf.js requires to be set before it asks the factory for a kind. */
export const BUNDLED_DATA_BASE = 'bundled:/';

/** Decodes a `data:...;base64,` URL without `fetch`, which the CSP blocks for `data:`. */
export function bytesFromDataUrl(url: string): Uint8Array {
	const comma = url.indexOf(',');
	if (comma < 0 || !url.slice(0, comma).endsWith(';base64')) {
		throw new Error('Expected a base64 data URL');
	}
	const binary = atob(url.slice(comma + 1));
	const bytes = new Uint8Array(binary.length);
	for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
	return bytes;
}

/**
 * Stands in for pdf.js's `DOMBinaryDataFactory`, passed as the
 * `BinaryDataFactory` option of `getDocument` with `useWorkerFetch: false`.
 */
export class BundledBinaryDataFactory {
	async fetch({kind, filename}: {kind: string; filename: string}): Promise<Uint8Array> {
		const load = TABLES[kind]?.[filename];
		if (!load) throw new Error(`No bundled PDF data for ${kind} ${filename}`);
		const {default: url} = await load();
		return bytesFromDataUrl(url);
	}
}
