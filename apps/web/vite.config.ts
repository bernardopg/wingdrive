import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";
import path from "path";
import fs from "fs";

const spaceui = path.resolve(import.meta.dirname, "../../../spaceui/packages");
const hasSpaceui = fs.existsSync(spaceui);
const spacebot = path.resolve(import.meta.dirname, "../../../spacebot/packages");
const hasSpacebot = fs.existsSync(spacebot);

export default defineConfig({
	plugins: [react(), tailwindcss()],
	resolve: {
		dedupe: ["react", "react-dom"],
		alias: [
			{
				find: /^react$/,
				replacement: path.resolve(import.meta.dirname, "./node_modules/react/index.js"),
			},
			{
				find: /^react\/jsx-runtime$/,
				replacement: path.resolve(import.meta.dirname, "./node_modules/react/jsx-runtime.js"),
			},
			{
				find: /^react\/jsx-dev-runtime$/,
				replacement: path.resolve(import.meta.dirname, "./node_modules/react/jsx-dev-runtime.js"),
			},
			{
				find: /^react-dom$/,
				replacement: path.resolve(import.meta.dirname, "./node_modules/react-dom/index.js"),
			},
			{
				find: /^react-dom\/client$/,
				replacement: path.resolve(import.meta.dirname, "./node_modules/react-dom/client.js"),
			},
			// SpaceUI — resolve to source for HMR when available locally
			...(hasSpaceui
				? [
						{
							find: /^@wingdrive\/tokens\/css\/themes\/(.+)$/,
							replacement: `${spaceui}/tokens/src/css/themes/$1.css`,
						},
						{
							find: /^@wingdrive\/tokens\/theme$/,
							replacement: `${spaceui}/tokens/src/css/theme.css`,
						},
						{
							find: /^@wingdrive\/tokens\/css$/,
							replacement: `${spaceui}/tokens/src/css/base.css`,
						},
						{
							find: /^@wingdrive\/tokens$/,
							replacement: `${spaceui}/tokens`,
						},
						{
							find: /^@wingdrive\/ai$/,
							replacement: `${spaceui}/ai/src/index.ts`,
						},
						{
							find: /^@wingdrive\/primitives$/,
							replacement: `${spaceui}/primitives/src/index.ts`,
						},
					]
				: []),
			// Spacebot lives in a separate private repo. Without it, reuse the
			// desktop app's stub; an external specifier left a bare import that
			// the browser cannot resolve, so the web UI rendered a blank page.
			{
				find: /^@spacebot\/api-client$/,
				replacement: hasSpacebot
					? `${spacebot}/api-client/src`
					: path.resolve(
							import.meta.dirname,
							"../tauri/src/stubs/spacebot-api-client.ts",
						),
			},
			{
				find: "@wingdrive/interface",
				replacement: path.resolve(import.meta.dirname, "../../packages/interface/src"),
			},
			{
				find: "@wingdrive/ts-client",
				replacement: path.resolve(import.meta.dirname, "../../packages/ts-client/src"),
			},
			{
				find: "openapi-fetch",
				replacement: path.resolve(
					import.meta.dirname,
					"../../packages/interface/node_modules/openapi-fetch/dist/index.mjs",
				),
			},
		],
	},
	server: {
		port: 3000,
		fs: {
			allow: [
				path.resolve(import.meta.dirname, "../../.."),
				...(hasSpaceui ? [spaceui] : []),
			],
		},
		proxy: {
			// Proxy RPC requests to server
			"/rpc": {
				target: "http://localhost:8080",
				changeOrigin: true,
			},
		},
	},
	optimizeDeps: {
		exclude: ["@wingdrive/ai", "@wingdrive/primitives", "@wingdrive/tokens"],
	},
	build: {
		outDir: "dist",
		emptyOutDir: true,
		sourcemap: true,
	},
});
