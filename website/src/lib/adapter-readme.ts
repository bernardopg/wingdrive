import MarkdownIt from "markdown-it";
import { REGISTRY, webUrl, type StoreAdapter } from "../i18n/adapters";

const sectionKeys = ["setup", "data", "privacy"] as const;
type SectionKey = (typeof sectionKeys)[number];

export function adapterSource({ source }: StoreAdapter) {
	if (!/^[0-9a-f]{40}$/i.test(source.commit)) return;
	const repository = source.type === "local" ? REGISTRY : webUrl(source.repository);
	const match = repository?.match(/^https?:\/\/github\.com\/([\w.-]+\/[\w.-]+?)\/?$/i);
	if (!match) return;

	const segments = (source.path ?? "").split("/").filter(Boolean);
	if (segments.some((part) => part === "." || part === "..")) return;
	const path = segments.map(encodeURIComponent).join("/");
	const slug = match[1].replace(/\.git$/i, "");
	const root = `https://github.com/${slug}`;
	const directory = `${source.commit}${path ? `/${path}` : ""}`;
	return {
		tree: `${root}/tree/${directory}`,
		commit: `${root}/commit/${source.commit}`,
		readme: `${root}/blob/${directory}/README.md`,
		raw: `https://raw.githubusercontent.com/${slug}/${directory}/README.md`,
	};
}

const markdown = new MarkdownIt({ html: false }).disable("image");
const validateLink = markdown.validateLink.bind(markdown);
markdown.validateLink = (url) => validateLink(url) && Boolean(webUrl(url, REGISTRY));
markdown.renderer.rules.link_open = (tokens, i, options, env, renderer) => {
	const token = tokens[i];
	const href = token.attrGet("href");
	const base = typeof env?.readmeUrl === "string" ? env.readmeUrl : REGISTRY;
	token.attrSet("href", typeof href === "string" ? webUrl(href, base) ?? "" : "");
	token.attrSet("rel", "noopener noreferrer");
	return renderer.renderToken(tokens, i, options);
};

export function renderReadme(markdownText: string, readmeUrl: string) {
	const env = { readmeUrl };
	const tokens = markdown.parse(markdownText, env);
	const headings = tokens.flatMap((token, i) => {
		if (token.type !== "heading_open" || token.level !== 0 || !["h1", "h2"].includes(token.tag)) {
			return [];
		}
		return [{ i, key: tokens[i + 1].content.trim().toLowerCase(), tag: token.tag }];
	});

	return headings.flatMap(({ i, key, tag }, index) => {
		const section = sectionKeys.find((name) => name === key);
		if (!section || tag !== "h2") return [];
		const end = headings[index + 1]?.i ?? tokens.length;
		const html = markdown.renderer.render(tokens.slice(i + 3, end), markdown.options, env);
		return html.trim() ? [{ section, html }] : [];
	});
}

const readmes = new Map<string, Promise<{ section: SectionKey; html: string }[]>>();

export function adapterReadme(source: NonNullable<ReturnType<typeof adapterSource>>) {
	const cached = readmes.get(source.raw);
	if (cached) return cached;
	const pending = loadReadme(source);
	readmes.set(source.raw, pending);
	return pending;
}

async function loadReadme(source: NonNullable<ReturnType<typeof adapterSource>>) {
	try {
		const res = await fetch(source.raw, { signal: AbortSignal.timeout(10_000) });
		if (res.ok) return renderReadme(await res.text(), source.readme);
	} catch {
		// A link to the published README remains available when the fetch fails.
	}
	return [];
}
