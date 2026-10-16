import { FunnelSimple, X } from "@phosphor-icons/react";
import type { ContentKind } from "@wingdrive/ts-client";
import clsx from "clsx";
import { useState } from "react";
import { useExplorer } from "./context";
import type { SearchFilters, SearchScope } from "./context";
import {
	SEARCH_CONTENT_KINDS,
	SEARCH_DATE_PRESETS,
	SEARCH_SIZE_PRESETS,
} from "./hooks/searchQuery";
import { useExplorerFiles } from "./hooks/useExplorerFiles";
import { useTranslation } from "../../i18n";

/** Most extension pills shown; selected extensions always stay visible. */
const MAX_EXTENSION_PILLS = 8;

function toggle<T>(list: readonly T[], value: T): T[] {
	return list.includes(value) ? list.filter((v) => v !== value) : [...list, value];
}

export function SearchToolbar() {
	const { t } = useTranslation("explorer");
	const explorer = useExplorer();
	const { searchScopeFallback, totalFound, facets } = useExplorerFiles();
	const [filtersOpen, setFiltersOpen] = useState(false);

	if (explorer.mode.type !== "search") {
		return null;
	}

	const { scope } = explorer.mode;
	const filters = explorer.searchFilters;
	const activeKinds = filters.contentTypes ?? [];
	const activeExtensions = filters.fileTypes ?? [];
	const hiddenOn = filters.includeHidden ?? explorer.viewSettings.showHiddenFiles;
	const activeCount =
		activeKinds.length +
		activeExtensions.length +
		(filters.size ? 1 : 0) +
		(filters.modified ? 1 : 0) +
		(hiddenOn ? 1 : 0);

	const update = (next: Partial<SearchFilters>) =>
		explorer.setSearchFilters({ ...filters, ...next });

	const handleScopeChange = (newScope: SearchScope) => {
		if (explorer.mode.type === "search") {
			explorer.enterSearchMode(explorer.mode.query, newScope);
		}
	};

	const extensionOptions = [
		...activeExtensions,
		...Object.entries(facets?.file_types ?? {})
			.filter(([ext]) => !activeExtensions.includes(ext))
			.sort(([, a], [, b]) => (b ?? 0) - (a ?? 0))
			.map(([ext]) => ext),
	].slice(0, Math.max(MAX_EXTENSION_PILLS, activeExtensions.length));

	// A count describes the current match set, so it is shown only where adding the pill narrows it.
	const countFor = (map: { [key in string]?: number } | undefined, keys: string[], show: boolean) =>
		show && map ? keys.reduce((sum, key) => sum + (map[key] ?? 0), 0) : undefined;

	const modifiedDays = (days: number) =>
		new Date(Date.now() - days * 24 * 60 * 60 * 1000).toISOString();

	return (
		<div className="border-b border-sidebar-line/30 bg-sidebar-box/10">
			<div className="flex items-center gap-3 px-4 py-2">
				<div className="flex items-center gap-2">
					<span className="text-xs font-medium text-sidebar-ink-dull">
						{t("search.searchIn")}
					</span>
					<div className="flex items-center gap-1 rounded-lg bg-sidebar-box/30 p-0.5">
						<ScopeButton
							active={scope === "folder"}
							onClick={() => handleScopeChange("folder")}
						>
							{t("search.thisFolder")}
						</ScopeButton>
						<ScopeButton
							active={scope === "location"}
							onClick={() => handleScopeChange("location")}
						>
							{t("search.location")}
						</ScopeButton>
						<ScopeButton
							active={scope === "library"}
							onClick={() => handleScopeChange("library")}
						>
							{t("search.library")}
						</ScopeButton>
					</div>
				</div>

				<div className="h-4 w-px bg-sidebar-line/30" />

				<button
					type="button"
					aria-expanded={filtersOpen}
					onClick={() => setFiltersOpen((open) => !open)}
					className={clsx(
						"flex items-center gap-1.5 px-2 py-1 rounded-md",
						"text-xs font-medium text-sidebar-ink",
						"hover:bg-sidebar-selected/40 transition-colors",
						activeCount > 0 && "bg-sidebar-selected/40",
					)}
				>
					<FunnelSimple className="size-3.5" weight="bold" />
					{activeCount > 0 ? t("search.filtersCount", { count: activeCount }) : t("search.filters")}
				</button>

				{totalFound !== undefined && (
					<span className="text-xs text-sidebar-ink-dull" role="status">
						{t("search.results", { count: totalFound })}
					</span>
				)}

				{searchScopeFallback && (
					<span className="text-xs text-sidebar-ink-dull" role="status">
						{t(scope === "location" ? "search.fallbackLocation" : "search.fallbackFolder")}
					</span>
				)}

				<div className="flex-1" />

				<button
					type="button"
					onClick={explorer.exitSearchMode}
					className={clsx(
						"flex items-center gap-1.5 px-2 py-1 rounded-md",
						"text-xs font-medium text-sidebar-ink-dull",
						"hover:bg-sidebar-selected/40 hover:text-sidebar-ink transition-colors",
					)}
				>
					<X className="size-3.5" weight="bold" />
					{t("search.clear")}
				</button>
			</div>

			{filtersOpen && (
				<div className="flex flex-col gap-1.5 px-4 pb-2">
					<PillRow label={t("search.kind")}>
						{SEARCH_CONTENT_KINDS.map(({ kind, label }) => (
							<ScopeButton
								key={kind}
								active={activeKinds.includes(kind)}
								onClick={() =>
									update({ contentTypes: toggle<ContentKind>(activeKinds, kind) })
								}
								count={countFor(facets?.kinds, [kind], activeKinds.length === 0)}
							>
								{t(label)}
							</ScopeButton>
						))}
					</PillRow>

					{extensionOptions.length > 0 && (
						<PillRow label={t("search.extension")}>
							{extensionOptions.map((ext) => (
								<ScopeButton
									key={ext}
									active={activeExtensions.includes(ext)}
									onClick={() => update({ fileTypes: toggle(activeExtensions, ext) })}
									count={countFor(facets?.file_types, [ext], activeExtensions.length === 0)}
								>
									.{ext}
								</ScopeButton>
							))}
						</PillRow>
					)}

					<PillRow label={t("search.size")}>
						{SEARCH_SIZE_PRESETS.map(({ preset, label, facet }) => (
							<ScopeButton
								key={preset}
								active={filters.size === preset}
								onClick={() =>
									update({ size: filters.size === preset ? undefined : preset })
								}
								count={countFor(facets?.size_ranges, [facet], !filters.size)}
							>
								{t(label)}
							</ScopeButton>
						))}
					</PillRow>

					<PillRow label={t("search.modified")}>
						{SEARCH_DATE_PRESETS.map(({ preset, label, days, facets: keys }) => (
							<ScopeButton
								key={preset}
								active={filters.modified?.preset === preset}
								onClick={() =>
									update({
										modified:
											filters.modified?.preset === preset
												? undefined
												: { preset, since: modifiedDays(days) },
									})
								}
								count={countFor(facets?.date_ranges, keys, !filters.modified)}
							>
								{t(label)}
							</ScopeButton>
						))}
					</PillRow>

					<PillRow label={t("search.hidden")}>
						<ScopeButton
							active={hiddenOn}
							onClick={() => update({ includeHidden: !hiddenOn })}
						>
							{t("search.showHidden")}
						</ScopeButton>
						{activeCount > 0 && (
							<button
								type="button"
								onClick={() => explorer.setSearchFilters({})}
								className="px-2 py-1 text-xs text-sidebar-ink-dull hover:text-sidebar-ink"
							>
								{t("ui.clearFilters")}
							</button>
						)}
					</PillRow>
				</div>
			)}
		</div>
	);
}

interface PillRowProps {
	label: string;
	children: React.ReactNode;
}

function PillRow({ label, children }: PillRowProps) {
	return (
		<div className="flex flex-wrap items-center gap-1.5" role="group" aria-label={label}>
			<span className="w-20 shrink-0 text-xs font-medium text-sidebar-ink-dull">{label}</span>
			{children}
		</div>
	);
}

interface ScopeButtonProps {
	active: boolean;
	onClick: () => void;
	count?: number;
	children: React.ReactNode;
}

function ScopeButton({ active, onClick, count, children }: ScopeButtonProps) {
	return (
		<button
			type="button"
			aria-pressed={active}
			onClick={onClick}
			className={clsx(
				"px-3 py-1 rounded-md text-xs font-medium transition-all",
				active
					? "bg-accent text-white shadow-sm"
					: "text-sidebar-ink-dull hover:text-sidebar-ink hover:bg-sidebar-selected/30",
			)}
		>
			{children}
			{count !== undefined && <span className="ml-1.5 tabular-nums opacity-70">{count}</span>}
		</button>
	);
}
