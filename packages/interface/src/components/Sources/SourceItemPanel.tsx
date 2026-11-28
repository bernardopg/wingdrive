import { X } from "@phosphor-icons/react";
import { CircleButton } from "@wingdrive/primitives";
import { useLibraryQuery } from "../../contexts/WingDriveContext";
import { uiLocale, useTranslation } from "../../i18n";

/** Characters of an item's text the panel renders; a whole coding session can be far longer. */
export const ITEM_TEXT_LIMIT = 20_000;

function formatDateTime(iso: string): string {
	const date = new Date(iso);
	return Number.isNaN(date.getTime())
		? iso
		: date.toLocaleString(uiLocale(), { dateStyle: "long", timeStyle: "short" });
}

interface SourceItemPanelProps {
	sourceId: string;
	itemId: string;
	onClose: () => void;
}

/** The item a person opened in a source: its title, details and whole text. */
export function SourceItemPanel({ sourceId, itemId, onClose }: SourceItemPanelProps) {
	const { t } = useTranslation("navigation");
	const { data: item, isLoading, error } = useLibraryQuery({
		type: "sources.get_item",
		input: { source_id: sourceId, item_id: itemId },
	});
	const text = item?.preview ?? "";
	const shown = text.length > ITEM_TEXT_LIMIT ? text.slice(0, ITEM_TEXT_LIMIT) : text;

	return (
		<aside
			aria-label={t("sources.itemPanel")}
			className="border-app-line/40 bg-app-box/40 flex w-[42%] min-w-[280px] max-w-2xl flex-col border-l"
		>
			<div className="border-app-line/30 flex items-start gap-2 border-b px-5 py-4">
				<div className="min-w-0 flex-1">
					{item ? (
						<>
							<h2 className="text-ink text-base font-semibold break-words">{item.title}</h2>
							{item.subtitle && <p className="text-ink-dull mt-1 text-sm break-words">{item.subtitle}</p>}
						</>
					) : (
						<p className="text-ink-faint text-sm">
							{isLoading
								? t("common.loading")
								: error
									? t("sources.itemLoadFailed", { error: String(error) })
									: t("sources.itemMissing")}
						</p>
					)}
				</div>
				<CircleButton icon={X} onClick={onClose} title={t("sources.closeItem")} />
			</div>
			{item && (
				<div className="min-h-0 flex-1 overflow-y-auto px-5 py-4">
					<dl className="text-ink-faint mb-4 grid grid-cols-[auto_1fr] gap-x-4 gap-y-1 text-xs">
						{item.date && (
							<>
								<dt>{t("sources.itemDate")}</dt>
								<dd className="text-ink-dull">{formatDateTime(item.date)}</dd>
							</>
						)}
						<dt>{t("sources.itemExternalId")}</dt>
						<dd className="text-ink-dull break-all font-mono">{item.external_id}</dd>
					</dl>
					{shown ? (
						<p className="text-ink whitespace-pre-wrap break-words text-sm leading-relaxed">{shown}</p>
					) : (
						<p className="text-ink-faint text-sm">{t("sources.itemNoText")}</p>
					)}
					{shown.length < text.length && (
						<p className="text-ink-faint mt-3 text-xs">
							{t("sources.itemTextCut", { count: ITEM_TEXT_LIMIT })}
						</p>
					)}
				</div>
			)}
		</aside>
	);
}
