import clsx from "clsx";
import {uiLocale} from "../../i18n";
import { SnippetText } from "./SnippetText";

interface SourceDataRowProps {
	title: string;
	preview?: string | null;
	subtitle?: string | null;
	date?: string | null;
	/** Text around a search match, shown instead of the preview (see `sources.search`). */
	snippet?: string | null;
	selected?: boolean;
	/** Makes the row a button that opens the item. */
	onSelect?: () => void;
}

function formatDate(iso: string): string {
	try {
		const d = new Date(iso);
		if (isNaN(d.getTime())) return iso;
		return d.toLocaleDateString(uiLocale(), {
			year: "numeric",
			month: "short",
			day: "numeric",
		});
	} catch {
		return iso;
	}
}

function cleanPreview(text: string): string {
	return text.replace(/[\r\n\t]+/g, " ").replace(/\s{2,}/g, " ").trim();
}

export function SourceDataRow({
	title,
	preview,
	subtitle,
	date,
	snippet,
	selected = false,
	onSelect,
}: SourceDataRowProps) {
	const content = (
		<>
			{/* Title row */}
			<span className="flex min-w-0 items-baseline gap-2">
				<span className="text-ink min-w-0 flex-1 truncate text-sm">
					{title}
				</span>
				{date && (
					<span className="text-ink-faint shrink-0 text-[11px]">
						{formatDate(date)}
					</span>
				)}
			</span>
			{subtitle && (
				<span className="text-ink-faint truncate text-xs">{subtitle}</span>
			)}
			{snippet ? (
				<span className="text-ink-faint/80 truncate text-xs">
					<SnippetText snippet={snippet} />
				</span>
			) : (
				preview && (
					<span className="text-ink-faint/70 truncate text-xs">
						{cleanPreview(preview)}
					</span>
				)
			)}
		</>
	);
	const className = clsx(
		"flex w-full min-w-0 flex-col gap-0.5 overflow-hidden rounded-lg px-3 py-2.5 text-left transition-colors",
		selected ? "bg-app-selected" : "hover:bg-app-hover",
	);

	if (onSelect) {
		return (
			<button type="button" aria-current={selected || undefined} onClick={onSelect} className={className}>
				{content}
			</button>
		);
	}
	return <div className={className}>{content}</div>;
}
