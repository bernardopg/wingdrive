import path from 'path';
import fs from 'fs';
import tailwindcss from '@tailwindcss/vite';
import react from '@vitejs/plugin-react';
import {defineConfig} from 'vite';
import commonjs from 'vite-plugin-commonjs';

const spaceui = path.resolve(import.meta.dirname, '../../../spaceui/packages');
const hasSpaceui = fs.existsSync(spaceui);
const spacebot = path.resolve(import.meta.dirname, '../../../spacebot/packages');
const hasSpacebot = fs.existsSync(spacebot);

export default defineConfig(() => ({
	plugins: [react(), tailwindcss(), commonjs()],

	resolve: {
		dedupe: ['react', 'react-dom'],
		alias: [
			{
				find: /^react$/,
				replacement: path.resolve(
					import.meta.dirname,
					'./node_modules/react/index.js'
				)
			},
			{
				find: /^react\/jsx-runtime$/,
				replacement: path.resolve(
					import.meta.dirname,
					'./node_modules/react/jsx-runtime.js'
				)
			},
			{
				find: /^react\/jsx-dev-runtime$/,
				replacement: path.resolve(
					import.meta.dirname,
					'./node_modules/react/jsx-dev-runtime.js'
				)
			},
			{
				find: /^react-dom$/,
				replacement: path.resolve(
					import.meta.dirname,
					'./node_modules/react-dom/index.js'
				)
			},
			{
				find: /^react-dom\/client$/,
				replacement: path.resolve(
					import.meta.dirname,
					'./node_modules/react-dom/client.js'
				)
			},
			{
				find: 'openapi-fetch',
				replacement: path.resolve(
					import.meta.dirname,
					'../../packages/interface/node_modules/openapi-fetch/dist/index.mjs'
				)
			},
			// SpaceUI — resolve to source for HMR when available locally
			...(hasSpaceui
				? [
						{
							find: /^@spacedrive\/tokens\/css\/themes\/(.+)$/,
							replacement: `${spaceui}/tokens/src/css/themes/$1.css`,
						},
						{
							find: /^@spacedrive\/tokens\/theme$/,
							replacement: `${spaceui}/tokens/src/css/theme.css`,
						},
						{
							find: /^@spacedrive\/tokens\/css$/,
							replacement: `${spaceui}/tokens/src/css/base.css`,
						},
						{
							find: /^@spacedrive\/tokens$/,
							replacement: `${spaceui}/tokens`,
						},
						{
							find: /^@spacedrive\/ai$/,
							replacement: `${spaceui}/ai/src/index.ts`,
						},
						{
							find: /^@spacedrive\/primitives$/,
							replacement: `${spaceui}/primitives/src/index.ts`,
						},
					]
				: []),
			// Spacebot lives in a separate private repo. Fall back to a tracked
			// in-tree stub so the desktop app still builds without it; leaving the
			// specifier external produced a bare import the webview cannot resolve.
			{
				find: /^@spacebot\/api-client$/,
				replacement: hasSpacebot
					? `${spacebot}/api-client/src`
					: path.resolve(import.meta.dirname, './src/stubs/spacebot-api-client.ts'),
			},
			{
				find: '@sd/interface',
				replacement: path.resolve(
					import.meta.dirname,
					'../../packages/interface/src'
				)
			},
			{
				find: '@sd/ts-client',
				replacement: path.resolve(
					import.meta.dirname,
					'../../packages/ts-client/src'
				)
			}
		]
	},

	optimizeDeps: {
		// Only skip pre-bundling when SpaceUI is checked out locally and aliased to
		// source for HMR. Excluding the published npm builds leaks their whole
		// transitive tree into the browser unbundled, and the CommonJS members of
		// that tree (style-to-js, debug via react-markdown) fail with "does not
		// provide an export named 'default'", which blanks the entire app.
		exclude: hasSpaceui
			? ['@spacedrive/ai', '@spacedrive/primitives', '@spacedrive/tokens']
			: []
	},

	clearScreen: false,
	server: {
		port: 1420,
		strictPort: true,
		fs: {
			allow: [
				path.resolve(import.meta.dirname, '../../..'),
				...(hasSpaceui ? [spaceui] : []),
			]
		},
		watch: {
			ignored: ['**/src-tauri/**']
		}
	},
	envPrefix: ['VITE_', 'TAURI_ENV_*'],
	build: {
		target: ['es2021', 'chrome100', 'safari13'],
		minify: !process.env.TAURI_ENV_DEBUG ? ('esbuild' as const) : false,
		sourcemap: !!process.env.TAURI_ENV_DEBUG
	}
}));
