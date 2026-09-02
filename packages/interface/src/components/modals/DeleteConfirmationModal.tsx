import { useEffect, useRef } from "react";
import { useForm } from "react-hook-form";
import { Trash, Warning } from "@phosphor-icons/react";
import {
	Dialog,
	dialogManager,
	useDialog,
	type UseDialogProps,
} from "@wingdrive/primitives";
import type { File as FileType } from "@sd/ts-client";
import { File, FileStack } from "../../routes/explorer/File";

interface DeleteConfirmationDialogProps {
	id: number;
	files: FileType[];
	permanent: boolean;
	onConfirm: () => Promise<void> | void;
	onCancelled?: () => void;
}

export function useDeleteConfirmationDialog() {
	return (options: Omit<DeleteConfirmationDialogProps, "id">) => {
		return dialogManager.create((props: UseDialogProps) => (
			<DeleteConfirmationDialog
				{...(props as DeleteConfirmationDialogProps)}
				{...options}
			/>
		));
	};
}

function DeleteConfirmationDialog(props: DeleteConfirmationDialogProps) {
	const dialog = useDialog(props);
	const form = useForm();

	// The primitive only invokes onCancelled from the footer button, so ESC and
	// click-outside would leave the caller's promise pending forever.
	const settled = useRef(false);
	useEffect(
		() => () => {
			if (!settled.current) props.onCancelled?.();
		},
		// eslint-disable-next-line react-hooks/exhaustive-deps
		[],
	);

	const count = props.files.length;
	const isSingle = count === 1;
	const label = props.permanent ? "Permanently delete" : "Delete";
	const suffix = props.permanent ? " This cannot be undone." : "";
	const message = isSingle
		? `${label} "${props.files[0].name}"?${suffix}`
		: `${label} ${count} items?${suffix}`;

	const handleConfirm = async () => {
		settled.current = true;
		await props.onConfirm();
	};

	const handleCancel = () => {
		settled.current = true;
		props.onCancelled?.();
	};

	return (
		<Dialog
			dialog={dialog}
			form={form}
			title={isSingle ? "Delete file" : `Delete ${count} items`}
			icon={
				<Trash
					size={20}
					weight="fill"
					className="text-red-500"
				/>
			}
			ctaLabel={label}
			ctaDanger
			closeLabel="Cancel"
			onSubmit={form.handleSubmit(handleConfirm)}
			onCancelled={handleCancel}
			formClassName="!min-w-[380px] !max-w-[380px]"
		>
			<div className="flex flex-col gap-4 py-4">
				{/* File thumbnails */}
				<div className="flex items-center justify-center py-2">
					{isSingle ? (
						<File.Thumb file={props.files[0]} size={64} />
					) : (
						<FileStack files={props.files.slice(0, 3)} size={64} />
					)}
				</div>

				{/* Warning */}
				<div className="flex items-start gap-2 rounded-md border border-red-500/20 bg-red-500/10 p-3">
					<Warning
						className="mt-0.5 size-5 flex-shrink-0 text-red-500"
						weight="fill"
					/>
					<div className="flex-1">
						<div className="text-sm font-medium text-ink">
							{message}
						</div>
						<div className="mt-0.5 text-xs text-ink-dull">
							{isSingle
								? props.files[0].kind === "Directory"
									? "The folder and all its contents will be moved to trash."
									: "The file will be moved to trash."
								: "All selected items will be moved to trash."}
						</div>
						{props.permanent && (
							<div className="mt-0.5 text-xs text-red-400">
								Permanent deletion cannot be undone.
							</div>
						)}
					</div>
				</div>
			</div>
		</Dialog>
	);
}