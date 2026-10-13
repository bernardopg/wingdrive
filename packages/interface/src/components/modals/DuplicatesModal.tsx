import { CircleNotch, Copy } from "@phosphor-icons/react";
import {
	Dialog,
	dialogManager,
	toast,
	useDialog,
	type UseDialogProps,
} from "@wingdrive/primitives";
import type { DuplicateSet, WingPath } from "@wingdrive/ts-client";
import { useEffect, useState } from "react";
import { useForm } from "react-hook-form";
import {
	useCoreMutation,
	useLibraryMutation,
} from "../../contexts/WingDriveContext";
import { useRefetchFileListings } from "../../hooks/useRefetchFileListings";
import { useUndo } from "../../hooks/useUndo";
import { useWaitForJob } from "../../hooks/useWaitForJob";
import { formatBytes } from "../../routes/explorer/utils";

interface DuplicatesDialogProps {
	id: number;
	/** Absolute local folder to scan */
	folder: string;
}

// Hashing every byte of a large folder can take a while.
const SCAN_TIMEOUT_MS = 30 * 60 * 1000;

export function useDuplicatesDialog() {
	return (folder: string) =>
		dialogManager.create((props: UseDialogProps) => (
			<DuplicatesDialog {...(props as DuplicatesDialogProps)} folder={folder} />
		));
}

function localPath(path: WingPath): string {
	return "Physical" in path ? path.Physical.path : "";
}

/** Keep the copy with the shortest path; it is usually the original. */
function defaultRemovals(groups: DuplicateSet[]): Set<string> {
	const removals = new Set<string>();
	for (const group of groups) {
		const paths = group.paths.map(localPath);
		const keep = paths.reduce((a, b) => (b.length < a.length ? b : a));
		for (const path of paths) if (path !== keep) removals.add(path);
	}
	return removals;
}

function DuplicatesDialog(props: DuplicatesDialogProps) {
	const dialog = useDialog(props);
	const form = useForm();
	const detect = useLibraryMutation("files.duplicate_detection");
	const remove = useLibraryMutation("files.delete");
	const restore = useCoreMutation("trash.restore");
	const waitForJob = useWaitForJob();
	const refetchListings = useRefetchFileListings();
	const undo = useUndo();
	const [groups, setGroups] = useState<DuplicateSet[] | null>(null);
	const [error, setError] = useState<string | null>(null);
	const [removals, setRemovals] = useState<Set<string>>(new Set());

	useEffect(() => {
		let cancelled = false;
		waitForJob(
			() =>
				detect.mutateAsync({
					paths: [props.folder],
					algorithm: "content_hash",
					threshold: 1,
				}),
			SCAN_TIMEOUT_MS,
		)
			.then(({ result }) => {
				if (cancelled) return;
				if (result.status === "completed" && result.output.type === "DuplicateDetection") {
					const found = result.output.data.groups ?? [];
					setGroups(found);
					setRemovals(defaultRemovals(found));
				} else {
					setError(result.status === "failed" ? result.error : `Scan ${result.status}`);
				}
			})
			.catch((err) => !cancelled && setError(String(err)));
		return () => {
			cancelled = true;
		};
		// Scan once per dialog.
		// eslint-disable-next-line react-hooks/exhaustive-deps
	}, []);

	const toggle = (group: DuplicateSet, path: string) => {
		setRemovals((prev) => {
			const next = new Set(prev);
			if (next.has(path)) {
				next.delete(path);
			} else {
				// Never let every copy of a set be removed.
				const others = group.paths.map(localPath).filter((p) => p !== path);
				if (others.every((p) => next.has(p))) return prev;
				next.add(path);
			}
			return next;
		});
	};

	const targets = groups
		? groups.flatMap((group) => group.paths.filter((p) => removals.has(localPath(p))))
		: [];
	const reclaimed = groups
		? groups.reduce(
				(sum, group) =>
					sum + group.size * group.paths.filter((p) => removals.has(localPath(p))).length,
				0,
			)
		: 0;

	const apply = async () => {
		const trashed = targets.map(localPath);
		try {
			const { result } = await waitForJob(() =>
				remove.mutateAsync({ targets: { paths: targets }, permanent: false, recursive: false }),
			);
			if (result.status !== "completed") throw new Error(`delete ${result.status}`);
			undo.record("remove duplicates", async () => {
				await restore.mutateAsync({ ids: [], original_paths: trashed });
			});
			toast.success(`Moved ${trashed.length} duplicates to trash`);
		} catch (err) {
			toast.error(`Could not remove duplicates: ${err}`);
		}
		refetchListings();
		dialogManager.setState(props.id, { open: false });
	};

	return (
		<Dialog
			dialog={dialog}
			form={form}
			title="Duplicate files"
			icon={<Copy size={20} weight="fill" className="text-accent" />}
			ctaLabel={
				targets.length > 0
					? `Move ${targets.length} to Trash (${formatBytes(reclaimed)})`
					: "Move to Trash"
			}
			submitDisabled={targets.length === 0 || remove.isPending}
			closeLabel="Close"
			onSubmit={form.handleSubmit(apply)}
			formClassName="!min-w-[600px] !max-w-[600px]"
		>
			<div className="flex max-h-[60vh] flex-col gap-3 overflow-y-auto py-3">
				<p className="text-ink-dull truncate text-xs">{props.folder}</p>
				{error ? (
					<p className="text-status-error text-sm">{error}</p>
				) : groups === null ? (
					<div className="text-ink-dull flex items-center gap-2 py-6 text-sm">
						<CircleNotch className="animate-spin" size={16} />
						Comparing file contents…
					</div>
				) : groups.length === 0 ? (
					<p className="text-ink-dull py-6 text-sm">No duplicates found.</p>
				) : (
					groups.map((group) => (
						<div key={localPath(group.paths[0])} className="border-app-line rounded-lg border">
							<div className="text-ink-faint border-app-line border-b px-3 py-1.5 text-xs">
								{group.paths.length} copies · {formatBytes(group.size)} each
							</div>
							{group.paths.map((wingPath) => {
								const path = localPath(wingPath);
								const marked = removals.has(path);
								return (
									<label
										key={path}
										className="hover:bg-app-hover flex cursor-pointer items-center gap-2 px-3 py-1.5 text-xs"
									>
										<input
											type="checkbox"
											checked={marked}
											onChange={() => toggle(group, path)}
										/>
										<span
											className={
												marked ? "text-ink-dull truncate line-through" : "text-ink truncate"
											}
										>
											{path}
										</span>
										{!marked && <span className="text-accent ml-auto shrink-0">keep</span>}
									</label>
								);
							})}
						</div>
					))
				)}
			</div>
		</Dialog>
	);
}
