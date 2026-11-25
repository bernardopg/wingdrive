import {useLocation} from "react-router-dom";
import {useSidebarNavigate} from "../TabManager/useSidebarNavigate";
import { ArrowSquareUp, Database } from "@phosphor-icons/react";
import { useCoreQuery, useLibraryQuery } from "../../contexts/WingDriveContext";
import { useAdapterIcons } from "../../hooks/useAdapterIcons";
import { SortableDynamicList } from "./SortableDynamicList";
import { GroupHeader } from "./GroupHeader";
import { useTranslation } from "../../i18n";

interface SourcesGroupProps {
  groupId: string;
	isCollapsed: boolean;
	onToggle: () => void;
	sortableAttributes?: any;
	sortableListeners?: any;
}

export function SourcesGroup({
  groupId,
	isCollapsed,
	onToggle,
	sortableAttributes,
	sortableListeners,
}: SourcesGroupProps) {
	const { t } = useTranslation("navigation");
	const navigate = useSidebarNavigate();
	const location = useLocation();
	const { getIcon } = useAdapterIcons();

	const { data: sources } = useLibraryQuery({
		type: "sources.list",
		input: { data_type: null },
	});

	const sourcesList = sources ?? [];

	// Adapter updates and revocations, from the store's cached index.
	const { data: store } = useCoreQuery({ type: "adapters.store.list", input: {} });
	const updates = store?.updates ?? 0;

	return (
		<div>
			<GroupHeader
				label={t("sidebar.sources")}
				isCollapsed={isCollapsed}
				onToggle={onToggle}
				sortableAttributes={sortableAttributes}
				sortableListeners={sortableListeners}
			/>

			{!isCollapsed && (
				<div className="space-y-0.5">
					<SortableDynamicList groupId={groupId} items={sourcesList} label={(source) => source.name}>
          {(source) => {
						const isActive = location.pathname === `/sources/${source.id}`;
						return (
							<button
								key={source.id}
								onClick={() => navigate(`/sources/${source.id}`)}
								className={`flex w-full items-center gap-2 rounded-md px-2 py-1 text-left text-sm font-medium ${
									isActive
										? "bg-sidebar-selected text-sidebar-ink"
										: "text-sidebar-ink-dull hover:text-sidebar-ink"
								}`}
							>
								{getIcon(source.adapter_id) ? (
									<img
										alt=""
										src={`data:image/svg+xml;charset=utf-8,${encodeURIComponent(getIcon(source.adapter_id)!)}`}
										className={`size-4 shrink-0 object-contain ${
											isActive
												? "opacity-100"
												: "opacity-60 grayscale"
										}`}
									/>
								) : (
									<Database
										className="size-4 shrink-0"
										weight={isActive ? "fill" : "bold"}
									/>
								)}
								<span className="truncate">{source.name}</span>
							</button>
						);
					}}
					</SortableDynamicList>
					{updates > 0 && (
						<button
							onClick={() => navigate("/sources/adapters")}
							className="flex w-full items-center gap-2 rounded-md px-2 py-1 text-left text-xs font-medium text-blue-400 hover:text-blue-300"
						>
							<ArrowSquareUp className="size-4 shrink-0" weight="bold" />
							<span className="truncate">
								{t("store.updatesAvailable", { count: updates })}
							</span>
						</button>
					)}
				</div>
			)}
		</div>
	);
}
