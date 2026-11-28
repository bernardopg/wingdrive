import { useParams, useNavigate } from "react-router-dom";
import { useState, useEffect, useMemo, useRef } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";
import {
	ArrowLeft,
	ArrowsClockwise,
	DotsThree,
	Trash,
	ArrowSquareUp,
	PencilSimple,
} from "@phosphor-icons/react";
import {
	useCoreQuery,
	useLibraryQuery,
	useLibraryMutation,
} from "../../contexts/WingDriveContext";
import { useTabManager } from "../../components/TabManager/useTabManager";
import { TopBarPortal, TopBarItem } from "../../TopBar";
import { CircleButton, Popover, usePopover } from "@wingdrive/primitives";
import { ExpandableSearchButton } from "../explorer/components/ExpandableSearchButton";
import { SourcePathBar } from "../../components/Sources/SourcePathBar";
import { SourceDataRow } from "../../components/Sources/SourceDataRow";
import { useTranslation } from "../../i18n";
import type { SourceItem } from "@wingdrive/ts-client";
import { ConfigureForm, Modal } from "./Adapters";

const PAGE_SIZE = 100;

export function SourceDetail() {
	const { t } = useTranslation("navigation");
	const { sourceId } = useParams<{ sourceId: string }>();
	const navigate = useNavigate();
	const [showDeleteConfirm, setShowDeleteConfirm] = useState(false);
	const [editing, setEditing] = useState(false);
	const [searchValue, setSearchValue] = useState("");
	const scrollRef = useRef<HTMLDivElement>(null);

	// Infinite scroll state
	const [allItems, setAllItems] = useState<SourceItem[]>([]);
	const [offset, setOffset] = useState(0);
	const [hasMore, setHasMore] = useState(true);
	const [loadingMore, setLoadingMore] = useState(false);

	const {
		data: source,
		isLoading,
		error,
		refetch,
		isFetching,
	} = useLibraryQuery({
		type: "sources.get",
		input: { source_id: sourceId ?? "" },
	});

	// Initial page
	const { data: initialItems, isLoading: itemsLoading } = useLibraryQuery({
		type: "sources.list_items",
		input: { source_id: sourceId ?? "", limit: PAGE_SIZE, offset: 0 },
	});

	// Seed allItems from initial fetch
	useEffect(() => {
		if (initialItems) {
			setAllItems(initialItems);
			setOffset(initialItems.length);
			setHasMore(initialItems.length >= PAGE_SIZE);
		}
	}, [initialItems]);

	// Reset when sourceId changes
	useEffect(() => {
		setAllItems([]);
		setOffset(0);
		setHasMore(true);
	}, [sourceId]);

	const { data: store } = useCoreQuery({
		type: "adapters.store.list",
		input: {},
	});

	const syncMutation = useLibraryMutation("sources.sync");
	const deleteMutation = useLibraryMutation("sources.delete");

	const adapterHasUpdate =
		store?.adapters.find((a) => a.id === source?.adapter_id)?.update_available ??
		false;

	// Sync tab title with source name
	const { activeTabId, updateTabTitle } = useTabManager();
	useEffect(() => {
		if (source?.name) {
			updateTabTitle(activeTabId, source.name);
		}
	}, [source?.name, activeTabId, updateTabTitle]);

	// Filter items by search
	const filteredItems = useMemo(() => {
		if (!searchValue.trim()) return allItems;
		const q = searchValue.toLowerCase();
		return allItems.filter(
			(item) =>
				item.title.toLowerCase().includes(q) ||
				item.subtitle?.toLowerCase().includes(q) ||
				item.preview?.toLowerCase().includes(q),
		);
	}, [allItems, searchValue]);

	// Virtualizer
	const virtualizer = useVirtualizer({
		count: filteredItems.length,
		getScrollElement: () => scrollRef.current,
		estimateSize: () => 64,
		overscan: 20,
		// Avoid flushSync during render (React 19 compat)
		scrollToFn: (offset, { adjustments, behavior }, instance) => {
			const el = instance.scrollElement;
			if (!el) return;
			const top = offset + (adjustments ?? 0);
			el.scrollTo({ top, behavior });
		},
	});

	// Fetch next page
	const { data: nextPage } = useLibraryQuery(
		{
			type: "sources.list_items",
			input: {
				source_id: sourceId ?? "",
				limit: PAGE_SIZE,
				offset,
			},
		},
		{ enabled: loadingMore && offset > 0 },
	);

	useEffect(() => {
		if (nextPage && loadingMore) {
			setAllItems((prev) => [...prev, ...nextPage]);
			setOffset((prev) => prev + nextPage.length);
			setHasMore(nextPage.length >= PAGE_SIZE);
			setLoadingMore(false);
		}
	}, [nextPage, loadingMore]);

	// Trigger load more when virtualizer reaches near the end
	const lastVirtualItem =
		virtualizer.getVirtualItems()[
			virtualizer.getVirtualItems().length - 1
		];

	useEffect(() => {
		if (
			lastVirtualItem &&
			lastVirtualItem.index >= filteredItems.length - 10 &&
			hasMore &&
			!loadingMore &&
			!searchValue
		) {
			setLoadingMore(true);
		}
	}, [
		lastVirtualItem?.index,
		filteredItems.length,
		hasMore,
		loadingMore,
		searchValue,
	]);

	if (isLoading) {
		return (
			<div className="flex items-center justify-center py-20">
				<div className="text-ink-faint text-sm">{t("common.loading")}</div>
			</div>
		);
	}

	if (error || !source) {
		return (
			<div className="p-6">
				<div className="rounded-lg border border-red-400/20 p-4">
					<p className="text-sm text-red-400">
						{t("sources.sourceLoadFailed", {error: error ? String(error) : t("common.notFound")})}{" "}
						{error ? String(error) : t("common.notFound")}
					</p>
					{error && (
						<button
							type="button"
							disabled={isFetching}
							onClick={() => void refetch()}
							className="mt-2 text-xs text-ink underline disabled:opacity-50"
						>
							{isFetching ? t("common.retrying") : t("common.retry")}
						</button>
					)}
				</div>
			</div>
		);
	}

	return (
		<div className="flex h-full flex-col">
			<TopBarPortal
				left={
					<>
						<TopBarItem
							id="back"
							label={t("common.back")}
							priority="high"
						>
							<CircleButton
								icon={ArrowLeft}
								onClick={() => navigate("/sources")}
							/>
						</TopBarItem>
						<TopBarItem
							id="source-path"
							label={t("common.path")}
							priority="high"
						>
							<SourcePathBar
								sourceName={source.name}
								itemCount={source.item_count}
							/>
						</TopBarItem>
					</>
				}
				right={
					<>
						<TopBarItem
							id="search"
							label={t("common.search")}
							priority="high"
						>
							<ExpandableSearchButton
								placeholder={t("sources.searchItems")}
								value={searchValue}
								onChange={setSearchValue}
								onClear={() => setSearchValue("")}
							/>
						</TopBarItem>
						<TopBarItem
							id="sync"
							label={t("sources.sync")}
							priority="high"
						>
							<CircleButton
								icon={ArrowsClockwise}
								onClick={() =>
									syncMutation.mutate({
										source_id: source.id,
									})
								}
								title={t("sources.sync")}
								active={
									syncMutation.isPending ||
									source.status === "syncing"
								}
							/>
						</TopBarItem>
						<TopBarItem
							id="more-actions"
							label={t("common.more")}
							priority="normal"
						>
							<MoreActionsMenu
								adapterHasUpdate={adapterHasUpdate}
								onUpdate={() =>
									navigate(
										`/sources/adapters?adapter=${encodeURIComponent(source.adapter_id)}`,
									)
								}
								isUpdating={false}
								onEdit={() => setEditing(true)}
								onDelete={() => setShowDeleteConfirm(true)}
							/>
						</TopBarItem>
					</>
				}
			/>

			{/* Banners */}
			{syncMutation.error && (
				<div className="border-app-line/30 space-y-2 border-b px-6 py-3">
					{syncMutation.error && (
						<div className="rounded-lg border border-red-400/20 p-3">
							<p className="text-xs text-red-400">
								{t("sources.syncFailed", {error: String(syncMutation.error)})}
							</p>
							<button
								type="button"
								disabled={syncMutation.isPending}
								onClick={() =>
									syncMutation.mutate({ source_id: source.id })
								}
								className="mt-1 text-xs text-ink underline disabled:opacity-50"
							>
								{syncMutation.isPending ? t("common.retrying") : t("sources.retrySync")}
							</button>
						</div>
					)}
				</div>
			)}

			{/* Virtualized items list */}
			<div ref={scrollRef} className="flex-1 overflow-y-auto">
				{itemsLoading && (
					<div className="text-ink-faint py-12 text-center text-sm">
						{t("sources.loadingItems")}
					</div>
				)}

				{!itemsLoading && filteredItems.length === 0 && (
					<div className="text-ink-faint py-12 text-center text-sm">
						{searchValue
							? t("sources.noMatchingItems")
							: t("sources.noItems")}
					</div>
				)}

				{filteredItems.length > 0 && (
					<div
						style={{
							height: virtualizer.getTotalSize(),
							position: "relative",
						}}
					>
						{virtualizer.getVirtualItems().map((virtualRow) => {
							const item = filteredItems[virtualRow.index];
							if (!item) return null;
							return (
								<div
									key={item.id}
									style={{
										position: "absolute",
										top: 0,
										left: 0,
										right: 0,
										height: virtualRow.size,
										transform: `translateY(${virtualRow.start}px)`,
									}}
								>
									<SourceDataRow
										title={item.title}
										preview={item.preview}
										subtitle={item.subtitle}
										date={item.date}
									/>
								</div>
							);
						})}
					</div>
				)}

				{loadingMore && (
					<div className="text-ink-faint py-4 text-center text-xs">
						{t("common.loadingMore")}
					</div>
				)}
			</div>

			{/* Delete confirmation */}
			{editing && source && (
				<Modal onClose={() => setEditing(false)}>
					<h2 className="text-ink mb-4 text-base font-semibold">
						{t("sources.editTitle", { name: source.name })}
					</h2>
					<ConfigureForm
						adapterId={source.adapter_id}
						adapterName={source.name}
						sourceId={source.id}
						sourceName={source.name}
						onDone={() => {
							setEditing(false);
							void refetch();
						}}
					/>
				</Modal>
			)}

			{showDeleteConfirm && (
				<div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40 backdrop-blur-sm">
					<div className="border-app-line bg-app-box w-full max-w-sm rounded-xl border p-6 shadow-2xl">
						<h3 className="text-ink text-base font-semibold">
							{t("sources.deleteTitle")}
						</h3>
						<p className="text-ink-faint mt-2 text-sm">
							{t("sources.deleteConfirm", {name: source.name})}
						</p>
						{deleteMutation.error && (
							<p className="mt-2 text-xs text-red-400">
								{String(deleteMutation.error)}
							</p>
						)}
						<div className="mt-5 flex justify-end gap-2">
							<button
								onClick={() => setShowDeleteConfirm(false)}
								disabled={deleteMutation.isPending}
								className="border-app-line text-ink-faint hover:text-ink rounded-lg border px-3.5 py-1.5 text-sm font-medium transition-colors"
							>{t("common.cancel")}
							</button>
							<button
								onClick={() =>
									deleteMutation.mutate(
										{ source_id: source.id },
										{
											onSuccess: () =>
												navigate("/sources"),
										},
									)
								}
								disabled={deleteMutation.isPending}
								className="rounded-lg bg-red-500 px-3.5 py-1.5 text-sm font-medium text-white transition-colors hover:bg-red-500/80 disabled:opacity-50"
							>
								{deleteMutation.isPending
									? t("common.deleting")
									: t("common.delete")}
							</button>
						</div>
					</div>
				</div>
			)}
		</div>
	);
}

function MoreActionsMenu({
	adapterHasUpdate,
	onUpdate,
	isUpdating,
	onEdit,
	onDelete,
}: {
	adapterHasUpdate: boolean;
	onUpdate: () => void;
	isUpdating: boolean;
	onEdit: () => void;
	onDelete: () => void;
}) {
	const { t } = useTranslation("navigation");
	const popover = usePopover();

	return (
		<Popover.Root open={popover.open} onOpenChange={popover.setOpen}>
			<Popover.Trigger asChild>
				<CircleButton icon={DotsThree} title={t("common.moreActions")} />
			</Popover.Trigger>
			<Popover.Content
				side="bottom"
				align="end"
				sideOffset={8}
				className="!bg-app-box z-50 w-[200px] !rounded-lg !p-1"
			>
				{adapterHasUpdate && (
					<button
						onClick={() => {
							onUpdate();
							popover.setOpen(false);
						}}
						disabled={isUpdating}
						className="hover:bg-app-hover text-ink flex w-full items-center gap-2 rounded-md px-2.5 py-1.5 text-left text-sm disabled:opacity-40"
					>
						<ArrowSquareUp size={16} className="text-blue-400" />
						{isUpdating ? t("common.updating") : t("sources.updateAdapter")}
					</button>
				)}
				<button
					onClick={() => {
						onEdit();
						popover.setOpen(false);
					}}
					className="hover:bg-app-hover text-ink flex w-full items-center gap-2 rounded-md px-2.5 py-1.5 text-left text-sm"
				>
					<PencilSimple size={16} />
					{t("sources.edit")}
				</button>
				<button
					onClick={() => {
						onDelete();
						popover.setOpen(false);
					}}
					className="hover:bg-app-hover flex w-full items-center gap-2 rounded-md px-2.5 py-1.5 text-left text-sm text-red-400"
				>
									<Trash size={16} />
									{t("sources.delete")}
				</button>
			</Popover.Content>
		</Popover.Root>
	);
}
