import { defineConfig } from "astro/config";
import sitemap from "@astrojs/sitemap";

export default defineConfig({
	site: "https://wingdrive.org",
	trailingSlash: "ignore",
	build: { format: "directory" },
	i18n: {
		defaultLocale: "en",
		locales: ["en", "pt"],
		routing: { prefixDefaultLocale: false },
	},
	integrations: [
		sitemap({
			i18n: { defaultLocale: "en", locales: { en: "en", pt: "pt-BR" } },
		}),
	],
	markdown: {
		shikiConfig: { theme: "tokyo-night" },
	},
});
