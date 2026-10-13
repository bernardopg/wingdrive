import { Textbox } from "@phosphor-icons/react";
import {
	Dialog,
	dialogManager,
	Input,
	Label,
	toast,
	useDialog,
	type UseDialogProps,
} from "@wingdrive/primitives";
import { fileDisplayName, type File } from "@wingdrive/ts-client";
import { useState } from "react";
import { useForm } from "react-hook-form";
import { useLibraryMutation } from "../../contexts/WingDriveContext";
import { usePlatform } from "../../contexts/PlatformContext";
import { useRefetchFileListings } from "../../hooks/useRefetchFileListings";
import { useUndo } from "../../hooks/useUndo";
import { useWaitForJob } from "../../hooks/useWaitForJob";
import { planBatchRename, type BatchRenameRule } from "./batchRename";

interface BatchRenameDialogProps {
	id: number;
	files: File[];
}

export function useBatchRenameDialog() {
	return (files: File[]) =>
		dialogManager.create((props: UseDialogProps) => (
			<BatchRenameDialog {...(props as BatchRenameDialogProps)} files={files} />
		));
}

function parentOf(path: string): string {
	return path.slice(0, Math.max(path.lastIndexOf("/"), path.lastIndexOf("\\")) + 1);
}

function BatchRenameDialog(props: BatchRenameDialogProps) {
	const dialog = useDialog(props);
	const form = useForm();
	const platform = usePlatform();
	const renameFile = useLibraryMutation("files.rename");
	const waitForJob = useWaitForJob();
	const refetchListings = useRefetchFileListings();
	const undo = useUndo();
	const [rule, setRule] = useState<BatchRenameRule>({
		find: "",
		replace: "",
		template: "{name}",
		start: 1,
		pad: 0,
	});

	const plan = planBatchRename(props.files, rule);
	const changes = plan
		.map((entry, index) => ({ ...entry, file: props.files[index] }))
		.filter((entry) => entry.newName !== fileDisplayName(entry.file));
	const blocked = changes.some((entry) => entry.error !== null);

	const set = <K extends keyof BatchRenameRule>(key: K, value: BatchRenameRule[K]) =>
		setRule((prev) => ({ ...prev, [key]: value }));

	const apply = async () => {
		const reversals: (() => Promise<void>)[] = [];
		let failed = 0;
		// Sequential, so a later rename never races an earlier one in the same folder.
		for (const { file, newName } of changes) {
			try {
				const { result } = await waitForJob(() =>
					renameFile.mutateAsync({ target: file.wing_path, new_name: newName }),
				);
				if (result.status !== "completed") throw new Error("rename did not complete");
				if (
					file.is_local &&
					"Physical" in file.wing_path &&
					platform.fileIdentity &&
					platform.undoMove
				) {
					const oldPath = file.wing_path.Physical.path;
					const newPath = parentOf(oldPath) + newName;
					const expected = await platform.fileIdentity(newPath);
					const move = platform.undoMove;
					reversals.push(() => move(newPath, oldPath, expected));
				}
			} catch {
				failed += 1;
			}
		}
		refetchListings();
		if (reversals.length > 0) {
			undo.record("batch rename", async () => {
				for (const reverse of reversals.reverse()) await reverse();
			});
		}
		if (failed > 0) toast.error(`${failed} of ${changes.length} renames failed`);
		else toast.success(`Renamed ${changes.length} items`);
		dialogManager.setState(props.id, { open: false });
	};

	return (
		<Dialog
			dialog={dialog}
			form={form}
			title={`Rename ${props.files.length} items`}
			icon={<Textbox size={20} weight="fill" className="text-accent" />}
			ctaLabel={changes.length > 0 ? `Rename ${changes.length}` : "Rename"}
			submitDisabled={changes.length === 0 || blocked}
			closeLabel="Cancel"
			onSubmit={form.handleSubmit(apply)}
			formClassName="!min-w-[520px] !max-w-[520px]"
		>
			<div className="flex flex-col gap-3 py-3">
				<div className="grid grid-cols-2 gap-3">
					<Field label="Find">
						<Input value={rule.find} onChange={(e) => set("find", e.target.value)} />
					</Field>
					<Field label="Replace with">
						<Input value={rule.replace} onChange={(e) => set("replace", e.target.value)} />
					</Field>
				</div>
				<Field label="Name pattern ({name} = name, {n} = number)">
					<Input value={rule.template} onChange={(e) => set("template", e.target.value)} />
				</Field>
				<div className="grid grid-cols-2 gap-3">
					<Field label="Start at">
						<Input
							type="number"
							value={rule.start}
							onChange={(e) => set("start", Number(e.target.value) || 0)}
						/>
					</Field>
					<Field label="Digits">
						<Input
							type="number"
							min={0}
							max={8}
							value={rule.pad}
							onChange={(e) => set("pad", Math.min(8, Math.max(0, Number(e.target.value) || 0)))}
						/>
					</Field>
				</div>
				<div className="border-app-line max-h-56 overflow-y-auto rounded-lg border">
					{plan.map((entry, index) => {
						const file = props.files[index];
						return (
							<div
								key={file.id}
								className="border-app-line flex items-center gap-2 border-b px-3 py-1.5 text-xs last:border-b-0"
							>
								<span className="text-ink-dull min-w-0 flex-1 truncate">
									{fileDisplayName(file)}
								</span>
								<span className="text-ink-faint">→</span>
								<span
									className={
										entry.error
											? "text-status-error min-w-0 flex-1 truncate"
											: "text-ink min-w-0 flex-1 truncate"
									}
									title={entry.error ?? undefined}
								>
									{entry.newName}
								</span>
							</div>
						);
					})}
				</div>
			</div>
		</Dialog>
	);
}

function Field({ label, children }: { label: string; children: React.ReactNode }) {
	return (
		<div className="flex flex-col gap-1">
			<Label>{label}</Label>
			{children}
		</div>
	);
}
