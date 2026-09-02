import {CaretDown} from "@phosphor-icons/react";
import {clsx} from "clsx";
import {forwardRef} from "react";

export interface SelectTriggerButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
	placeholder?: string;
}

const SelectTriggerButton = forwardRef<
	HTMLButtonElement,
	SelectTriggerButtonProps
>(({className, children, placeholder, ...props}, ref) => {
	return (
		<button
			ref={ref}
			type="button"
			className={clsx(
				"flex h-9 min-w-[220px] items-center gap-2 rounded-full border border-app-line bg-app-box px-4 text-left text-sm font-medium text-ink-dull transition-colors",
				"hover:bg-app-hover hover:text-ink",
				"disabled:cursor-not-allowed disabled:opacity-60",
				className,
			)}
			{...props}
		>
			<span className="flex-1 truncate text-left">
				{children || placeholder}
			</span>
			<CaretDown className="size-3.5 shrink-0" weight="bold" />
		</button>
	);
});

SelectTriggerButton.displayName = "SelectTriggerButton";

export {SelectTriggerButton};
