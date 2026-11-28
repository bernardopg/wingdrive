import { CaretDown, CaretRight, Warning } from "@phosphor-icons/react";
import type { SourceSearchHit } from "@wingdrive/ts-client";
import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { useAdapterIcons } from "../../hooks/useAdapterIcons";
import { uiLocale, useTranslation } from "../../i18n";
import { SnippetText } from "./SnippetText";
import { SourceTypeIcon } from "./SourceTypeIcon";
import { groupHitsBySource, sourceItemLink } from "./sourceSearch";

/** Hits each source shows before "Show all" opens the source. */
export const HITS_PER_SOURCE = 5;

function formatDate(iso: string): string {
	const date = new Date(iso);
	return Number.isNaN(date.getTime())
		? iso
		: date.toLocaleDateString(uiLocale(), { year: "numeric", month: "short", day: "numeric" });
}

interface SourceSearchResultsProps {
	query: string;
	hits: readonly SourceSearchHit[];
}

/**
 * Items of the library's sources that match the Explorer search, grouped by source.
 * Opening a hit shows it selected in its source, with the same words searched.
 */
export function SourceSearchResults({ query, hits }: SourceSearchResultsProps) {
	const { t } = useTranslation("explorer");
	const navigate = useNavigate();
	const { getIcon } = useAdapterIcons();
	const [open, setOpen] = useState(true);

	if (hits.length === 0) return null;
	const groups = groupHitsBySource(hits, HITS_PER_SOURCE);

	return (
		<section
			aria-label={t("sourceResults.title")}
			className="border-sidebar-line/30 max-h-[45%] shrink-0 overflow-y-auto border-b"
		>
			<button
				type="button"
				aria-expanded={open}
				onClick={() => setOpen((value) => !value)}
				className="text-ink-dull hover:text-ink flex w-full items-center gap-1.5 px-4 py-2 text-left text-xs font-medium transition-colors"
			>
				{open ? <CaretDown className="size-3" weight="bold" /> : <CaretRight className="size-3" weight="bold" />}
				{t("sourceResults.heading", { count: hits.length })}
			</button>
			{open && (
				<div className="space-y-3 px-3 pb-3">
					{groups.map((group) => (
						<div key={group.sourceId}>
							<div className="flex items-center gap-2 px-1 pb-1">
								<SourceTypeIcon type={group.dataType} svg={getIcon(group.adapterId)} size="sm" />
								<span className="text-ink min-w-0 flex-1 truncate text-xs font-semibold">
									{group.sourceName}
								</span>
								<button
									type="button"
									onClick={() => navigate(sourceItemLink(group.sourceId, query))}
									className="text-accent hover:text-accent-deep shrink-0 text-xs transition-colors"
								>
									{group.hidden > 0
										? t("sourceResults.showAllMore", { count: group.hidden })
										: t("sourceResults.openSource")}
								</button>
							</div>
							<ul>
								{group.hits.map((hit) => (
									<li key={hit.id}>
										<button
											type="button"
											onClick={() => navigate(sourceItemLink(hit.source_id, query, hit.id))}
											className="hover:bg-app-hover flex w-full min-w-0 flex-col gap-0.5 rounded-lg px-3 py-2 text-left transition-colors"
										>
											<span className="flex min-w-0 items-baseline gap-2">
												<span className="text-ink min-w-0 flex-1 truncate text-sm">{hit.title}</span>
												{hit.flagged && (
													<span
														role="img"
														title={t("sourceResults.flagged")}
														aria-label={t("sourceResults.flagged")}
														className="shrink-0 self-center text-yellow-500"
													>
														<Warning className="size-3.5" weight="fill" />
													</span>
												)}
												{hit.date && (
													<span className="text-ink-faint shrink-0 text-[11px]">{formatDate(hit.date)}</span>
												)}
											</span>
											{hit.subtitle && <span className="text-ink-faint truncate text-xs">{hit.subtitle}</span>}
											{(hit.snippet ?? hit.preview) && (
												<span className="text-ink-faint/80 line-clamp-2 text-xs">
													{hit.snippet ? <SnippetText snippet={hit.snippet} /> : hit.preview}
												</span>
											)}
										</button>
									</li>
								))}
							</ul>
						</div>
					))}
				</div>
			)}
		</section>
	);
}
