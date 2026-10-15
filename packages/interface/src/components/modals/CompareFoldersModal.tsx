import { CircleNotch, GitDiff } from "@phosphor-icons/react";
import {
	Dialog,
	dialogManager,
	toast,
	useDialog,
	type UseDialogProps,
} from "@wingdrive/primitives";
import type { CompareEntry, WingPath } from "@wingdrive/ts-client";
import { useState } from "react";
import { useForm } from "react-hook-form";
import { useCoreQuery, useLibraryMutation } from "../../contexts/WingDriveContext";
import { useRefetchFileListings } from "../../hooks/useRefetchFileListings";
import { useWaitForJob } from "../../hooks/useWaitForJob";
import { formatBytes } from "../../routes/explorer/utils";
import { useTranslation } from "../../i18n";

interface CompareFoldersDialogProps {
	id: number;
	left: string;
	right: string;
}

export function useCompareFoldersDialog() {
	return (left: string, right: string) =>
		dialogManager.create((props: UseDialogProps) => (
			<CompareFoldersDialog
				{...(props as CompareFoldersDialogProps)}
				left={left}
				right={right}
			/>
		));
}

type Direction = "toRight" | "toLeft";

/** Which way an entry merges, or null when it cannot be merged safely. */
export function mergeDirection(entry: CompareEntry): Direction | null {
	if (entry.status === "OnlyLeft") return "toRight";
	if (entry.status === "OnlyRight") return "toLeft";
	const { left, right } = entry;
	// A file on one side and a folder on the other needs a human decision.
	if (!left || !right || left.is_dir || right.is_dir) return null;
	return (left.modified ?? 0) >= (right.modified ?? 0) ? "toRight" : "toLeft";
}

const local = (path: string): WingPath => ({
	Physical: { device_slug: "local", path },
});

function parentOf(relative: string): string {
	const cut = relative.lastIndexOf("/");
	return cut < 0 ? "" : relative.slice(0, cut);
}

function join(root: string, relative: string): string {
	return relative ? `${root.replace(/\/$/, "")}/${relative}` : root;
}

function CompareFoldersDialog(props: CompareFoldersDialogProps) {
	const { t } = useTranslation("dialogs");
	const dialog = useDialog(props);
	const form = useForm();
	const copy = useLibraryMutation("files.copy");
	const remove = useLibraryMutation("files.delete");
	const waitForJob = useWaitForJob();
	const refetchListings = useRefetchFileListings();
	const [skipped, setSkipped] = useState<Set<string>>(new Set());
	const [busy, setBusy] = useState(false);
	const { data, error, isLoading } = useCoreQuery({
		type: "files.compare_folders",
		input: { left: props.left, right: props.right },
	});

	const entries = data?.entries ?? [];
	const planned = entries.filter(
		(entry) => mergeDirection(entry) !== null && !skipped.has(entry.relative_path),
	);

	const toggle = (path: string) =>
		setSkipped((prev) => {
			const next = new Set(prev);
			if (next.has(path)) next.delete(path);
			else next.add(path);
			return next;
		});

	const apply = async () => {
		setBusy(true);
		try {
			// The older copy of a changed file goes to the trash first, so a
			// wrong guess about which side is newer can still be recovered.
			const replaced = planned
				.filter((entry) => entry.status === "Different")
				.map((entry) =>
					mergeDirection(entry) === "toRight"
						? join(props.right, entry.relative_path)
						: join(props.left, entry.relative_path),
				);
			if (replaced.length > 0) {
				const { result } = await waitForJob(() =>
					remove.mutateAsync({
						targets: { paths: replaced.map(local) },
						permanent: false,
						recursive: false,
					}),
				);
				if (result.status !== "completed") throw new Error(`trashing older copies ${result.status}`);
			}

			// One copy job per destination folder.
			const batches = new Map<string, string[]>();
			for (const entry of planned) {
				const toRight = mergeDirection(entry) === "toRight";
				const source = join(toRight ? props.left : props.right, entry.relative_path);
				const destination = join(
					toRight ? props.right : props.left,
					parentOf(entry.relative_path),
				);
				batches.set(destination, [...(batches.get(destination) ?? []), source]);
			}
			let failed = 0;
			for (const [destination, sources] of batches) {
				const { result } = await waitForJob(
					() =>
						copy.mutateAsync({
							sources: { paths: sources.map(local) },
							destination: local(destination),
							overwrite: false,
							verify_checksum: false,
							preserve_timestamps: true,
							move_files: false,
							copy_method: "Auto",
							on_conflict: "Skip",
						}),
					10 * 60 * 1000,
				);
				if (result.status !== "completed") failed += sources.length;
			}
			if (failed > 0) toast.error(t("compareFolders.mergeFailed", { count: failed }));
			else toast.success(t("compareFolders.merged", { count: planned.length }));
			dialogManager.setState(props.id, { open: false });
		} catch (err) {
			toast.error(t("compareFolders.mergeStopped", { error: err }));
		} finally {
			setBusy(false);
			refetchListings();
		}
	};

	const name = (path: string) => path.split("/").filter(Boolean).pop() ?? path;
	const label = (entry: CompareEntry): string => {
		if (entry.status === "OnlyLeft") return t("compareFolders.onlyIn", { name: name(props.left) });
		if (entry.status === "OnlyRight") return t("compareFolders.onlyIn", { name: name(props.right) });
		const direction = mergeDirection(entry);
		if (!direction) return t("compareFolders.fileVsFolder");
		return t("compareFolders.newerIn", { name: name(direction === "toRight" ? props.left : props.right) });
	};

	return (
		<Dialog
			dialog={dialog}
			form={form}
			title={t("compareFolders.title")}
			icon={<GitDiff size={20} weight="fill" className="text-accent" />}
			ctaLabel={planned.length > 0 ? t("compareFolders.mergeCount", { count: planned.length }) : t("compareFolders.merge")}
			submitDisabled={planned.length === 0 || busy}
			loading={busy}
			closeLabel={t("common.close")}
			onSubmit={form.handleSubmit(apply)}
			formClassName="!min-w-[640px] !max-w-[640px]"
		>
			<div className="flex max-h-[60vh] flex-col gap-2 overflow-y-auto py-3 text-xs">
				<div className="text-ink-dull grid grid-cols-2 gap-2">
					<span className="truncate">A: {props.left}</span>
					<span className="truncate">B: {props.right}</span>
				</div>
				{error ? (
					<p className="text-status-error text-sm">{String(error)}</p>
				) : isLoading || !data ? (
					<div className="text-ink-dull flex items-center gap-2 py-6 text-sm">
						<CircleNotch className="animate-spin" size={16} />
						{t("compareFolders.comparing")}
					</div>
				) : entries.length === 0 ? (
					<p className="text-ink-dull py-6 text-sm">
						{t("compareFolders.identicalFiles", { count: data.identical })}
					</p>
				) : (
					<>
						<p className="text-ink-faint">
							{entries.length} differences · {data.identical} identical
							{data.unreadable > 0 && ` · ${data.unreadable} unreadable`}. Merging copies
							each item toward the side that lacks it; for changed files the newer copy
							wins and the older one goes to the trash.
						</p>
						<div className="border-app-line rounded-lg border">
							{entries.map((entry) => {
								const mergeable = mergeDirection(entry) !== null;
								const side = entry.left ?? entry.right;
								return (
									<label
										key={entry.relative_path}
										className="border-app-line hover:bg-app-hover flex items-center gap-2 border-b px-3 py-1.5 last:border-b-0"
									>
										<input
											type="checkbox"
											disabled={!mergeable}
											checked={mergeable && !skipped.has(entry.relative_path)}
											onChange={() => toggle(entry.relative_path)}
										/>
										<span className="text-ink min-w-0 flex-1 truncate">
											{entry.relative_path}
											{side?.is_dir ? "/" : ""}
										</span>
										<span className="text-ink-faint w-16 shrink-0 text-right">
											{side && !side.is_dir ? formatBytes(side.size) : ""}
										</span>
										<span className="text-ink-dull w-32 shrink-0 truncate text-right">
											{label(entry)}
										</span>
									</label>
								);
							})}
						</div>
					</>
				)}
			</div>
		</Dialog>
	);
}
