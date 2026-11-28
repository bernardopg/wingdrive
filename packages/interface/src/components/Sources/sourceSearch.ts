import type { SourceSearchHit } from "@wingdrive/ts-client";

/** U+0002 and U+0003 wrap each matched term in a hit's snippet (see `sources.search`). */
const MATCH_START = "\u0002";
const MATCH_END = "\u0003";

export interface SnippetPart {
	text: string;
	match: boolean;
}

/** Splits a hit's snippet into plain text and matched terms, dropping the markers. */
export function splitSnippet(snippet: string): SnippetPart[] {
	const parts: SnippetPart[] = [];
	let match = false;
	let text = "";
	for (const char of snippet) {
		if (char === MATCH_START || char === MATCH_END) {
			if (text) parts.push({ text, match });
			text = "";
			match = char === MATCH_START;
		} else {
			text += char;
		}
	}
	if (text) parts.push({ text, match });
	return parts;
}

/** Hits of one source, in the order the search ranked them. */
export interface SourceHitGroup {
	sourceId: string;
	sourceName: string;
	adapterId: string;
	dataType: string;
	hits: SourceSearchHit[];
	/** Hits of this source left out of `hits` by the per-source cap. */
	hidden: number;
}

/**
 * Groups hits by source. Sources keep the rank of their best hit, so the source with the best
 * match comes first, and each shows at most `perSource` hits.
 */
export function groupHitsBySource(hits: readonly SourceSearchHit[], perSource: number): SourceHitGroup[] {
	const groups = new Map<string, SourceHitGroup>();
	for (const hit of hits) {
		let group = groups.get(hit.source_id);
		if (!group) {
			group = {
				sourceId: hit.source_id,
				sourceName: hit.source_name,
				adapterId: hit.adapter_id,
				dataType: hit.data_type,
				hits: [],
				hidden: 0,
			};
			groups.set(hit.source_id, group);
		}
		if (group.hits.length < perSource) group.hits.push(hit);
		else group.hidden += 1;
	}
	return [...groups.values()];
}

/** Where a source opens with `query` searched and, when given, one item selected. */
export function sourceItemLink(sourceId: string, query: string, itemId?: string): string {
	const params = new URLSearchParams();
	if (query.trim()) params.set("q", query);
	if (itemId) params.set("item", itemId);
	const search = params.toString();
	return `/sources/${encodeURIComponent(sourceId)}${search ? `?${search}` : ""}`;
}

/** Whether `query` has a word the source index can match; punctuation alone matches nothing. */
export function hasSearchableWord(query: string): boolean {
	return /[\p{L}\p{N}]/u.test(query);
}
