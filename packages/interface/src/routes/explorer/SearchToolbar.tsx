import { FunnelSimple, X } from "@phosphor-icons/react";
import type { ContentKind } from "@wingdrive/ts-client";
import clsx from "clsx";
import { useState } from "react";
import { useExplorer } from "./context";
import type { SearchScope } from "./context";
import { SEARCH_CONTENT_KINDS } from "./hooks/searchQuery";
import { useExplorerFiles } from "./hooks/useExplorerFiles";

export function SearchToolbar() {
	const explorer = useExplorer();
	const { searchScopeFallback } = useExplorerFiles();
	const [filtersOpen, setFiltersOpen] = useState(false);

	if (explorer.mode.type !== "search") {
		return null;
	}

	const { scope } = explorer.mode;
	const activeKinds = explorer.searchFilters.contentTypes ?? [];

	const handleScopeChange = (newScope: SearchScope) => {
		if (explorer.mode.type === "search") {
			explorer.enterSearchMode(explorer.mode.query, newScope);
		}
	};

	const toggleKind = (kind: ContentKind) => {
		const next = activeKinds.includes(kind)
			? activeKinds.filter((k) => k !== kind)
			: [...activeKinds, kind];
		explorer.setSearchFilters({ ...explorer.searchFilters, contentTypes: next });
	};

	return (
		<div className="border-b border-sidebar-line/30 bg-sidebar-box/10">
			<div className="flex items-center gap-3 px-4 py-2">
				<div className="flex items-center gap-2">
					<span className="text-xs font-medium text-sidebar-ink-dull">
						Search in:
					</span>
					<div className="flex items-center gap-1 rounded-lg bg-sidebar-box/30 p-0.5">
						<ScopeButton
							active={scope === "folder"}
							onClick={() => handleScopeChange("folder")}
						>
							This Folder
						</ScopeButton>
						<ScopeButton
							active={scope === "location"}
							onClick={() => handleScopeChange("location")}
						>
							Location
						</ScopeButton>
						<ScopeButton
							active={scope === "library"}
							onClick={() => handleScopeChange("library")}
						>
							Library
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
						activeKinds.length > 0 && "bg-sidebar-selected/40",
					)}
				>
					<FunnelSimple className="size-3.5" weight="bold" />
					{activeKinds.length > 0 ? `Filters (${activeKinds.length})` : "Filters"}
				</button>

				{searchScopeFallback && (
					<span className="text-xs text-sidebar-ink-dull" role="status">
						No {scope === "location" ? "location" : "folder"} here; searching the
						whole library.
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
					Clear Search
				</button>
			</div>

			{filtersOpen && (
				<div
					className="flex flex-wrap items-center gap-1.5 px-4 pb-2"
					role="group"
					aria-label="Filter by kind"
				>
					{SEARCH_CONTENT_KINDS.map(({ kind, label }) => (
						<ScopeButton
							key={kind}
							active={activeKinds.includes(kind)}
							onClick={() => toggleKind(kind)}
						>
							{label}
						</ScopeButton>
					))}
					{activeKinds.length > 0 && (
						<button
							type="button"
							onClick={() =>
								explorer.setSearchFilters({ ...explorer.searchFilters, contentTypes: [] })
							}
							className="px-2 py-1 text-xs text-sidebar-ink-dull hover:text-sidebar-ink"
						>
							Clear filters
						</button>
					)}
				</div>
			)}
		</div>
	);
}

interface ScopeButtonProps {
	active: boolean;
	onClick: () => void;
	children: React.ReactNode;
}

function ScopeButton({ active, onClick, children }: ScopeButtonProps) {
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
		</button>
	);
}
