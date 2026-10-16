import { SelectionAll } from "@phosphor-icons/react";
import {
	Dialog,
	dialogManager,
	Input,
	Label,
	useDialog,
	type UseDialogProps,
} from "@wingdrive/primitives";
import type { File } from "@wingdrive/ts-client";
import { useState } from "react";
import { useForm } from "react-hook-form";
import { filesMatching } from "../../routes/explorer/hooks/selectionPattern";
import { useTranslation } from "../../i18n";

interface SelectByPatternDialogProps {
	id: number;
	files: File[];
	onSelect: (files: File[]) => void;
}

/** Opens the "Select by Pattern" dialog; `onSelect` receives the matching files. */
export function useSelectByPatternDialog() {
	return (files: File[], onSelect: (files: File[]) => void) =>
		dialogManager.create((props: UseDialogProps) => (
			<SelectByPatternDialog
				{...(props as SelectByPatternDialogProps)}
				files={files}
				onSelect={onSelect}
			/>
		));
}

function SelectByPatternDialog(props: SelectByPatternDialogProps) {
	const { t } = useTranslation("dialogs");
	const dialog = useDialog(props);
	const form = useForm();
	const [pattern, setPattern] = useState("");
	const matches = filesMatching(props.files, pattern);

	const apply = () => {
		props.onSelect(matches);
		dialogManager.setState(props.id, { open: false });
	};

	return (
		<Dialog
			dialog={dialog}
			form={form}
			title={t("selectByPattern.title")}
			icon={<SelectionAll size={20} weight="fill" className="text-accent" />}
			ctaLabel={t("selectByPattern.select", { count: matches.length })}
			submitDisabled={matches.length === 0}
			closeLabel={t("common.cancel")}
			onSubmit={form.handleSubmit(apply)}
			formClassName="!min-w-[420px] !max-w-[420px]"
		>
			<div className="flex flex-col gap-1.5 py-3">
				<Label>{t("selectByPattern.pattern")}</Label>
				<Input
					autoFocus
					value={pattern}
					placeholder="*.jpg; *.png"
					onChange={(e) => setPattern(e.target.value)}
				/>
				<span className="text-ink-faint text-xs">
					{t("selectByPattern.hint", { count: matches.length, total: props.files.length })}
				</span>
			</div>
		</Dialog>
	);
}
