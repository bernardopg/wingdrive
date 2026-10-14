import { useNavigate } from "react-router-dom";
import { useAdapterIcons } from "../../hooks/useAdapterIcons";
import { SourceTypeIcon } from "./SourceTypeIcon";
import { SourceStatusBadge } from "./SourceStatusBadge";
import { i18n, useTranslation } from "../../i18n";
import {uiLocale} from "../../i18n";

interface SourceCardProps {
	source: {
		id: string;
		name: string;
		data_type: string;
		adapter_id: string;
		status: string;
		item_count: number;
		last_synced: string | null;
	};
}

function formatRelative(iso: string): string {
	const date = new Date(iso);
	const now = new Date();
	const diffMs = now.getTime() - date.getTime();
	const diffMin = Math.floor(diffMs / 60000);
	const diffHr = Math.floor(diffMin / 60);
	const diffDay = Math.floor(diffHr / 24);

	if (diffMin < 1) return i18n.t("dialogs:sources.justNow");
	if (diffMin < 60) return i18n.t("dialogs:sources.minutesAgo", { minutes: diffMin });
	if (diffHr < 24) return i18n.t("dialogs:sources.hoursAgo", { hours: diffHr });
	if (diffDay < 30) return i18n.t("dialogs:sources.daysAgo", { days: diffDay });
	return date.toLocaleDateString(uiLocale());
}

export function SourceCard({ source }: SourceCardProps) {
	const { t } = useTranslation("dialogs");
	const navigate = useNavigate();
	const { getIcon } = useAdapterIcons();

	return (
		<button
			onClick={() => navigate(`/sources/${source.id}`)}
			className="border-app-line bg-app-box hover:border-app-line/80 hover:bg-app-hover group relative rounded-lg border p-4 text-left transition-all"
		>
			<div className="mb-3 flex items-center gap-3">
				<SourceTypeIcon type={source.data_type} svg={getIcon(source.adapter_id)} size="md" />
				<div className="min-w-0 flex-1">
					<h3 className="text-ink truncate text-sm font-medium">
						{source.name}
					</h3>
					<p className="text-ink-faint text-xs">{source.adapter_id}</p>
				</div>
				<SourceStatusBadge status={source.status} />
			</div>

			<div className="text-ink-faint flex items-center justify-between text-xs">
				<span>
					{t("sources.itemCount", { count: source.item_count, formattedCount: source.item_count.toLocaleString() })}
				</span>
				<span>
					{source.last_synced
						? formatRelative(source.last_synced)
						: t("sources.neverSynced")}
				</span>
			</div>
		</button>
	);
}
