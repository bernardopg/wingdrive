"use client";

import { ArrowDown } from "@phosphor-icons/react";
import { type Virtualizer } from "@tanstack/react-virtual";
import clsx from "clsx";
import { type ReactNode, useEffect, useState } from "react";

export interface JumpToEndButtonProps<
	TScrollElement extends Element = Element,
	TItemElement extends Element = Element,
> {
	virtualizer: Virtualizer<TScrollElement, TItemElement> | null | undefined;
	className?: string;
	children?: ReactNode;
	"aria-label"?: string;
}

export function JumpToEndButton<
	TScrollElement extends Element = Element,
	TItemElement extends Element = Element,
>({
	virtualizer,
	className,
	children,
	"aria-label": ariaLabel = "Jump to latest",
}: JumpToEndButtonProps<TScrollElement, TItemElement>) {
	const [visible, setVisible] = useState(false);

	useEffect(() => {
		if (!virtualizer) return;
		const el = virtualizer.scrollElement;
		if (!el) return;

		const update = () => setVisible(!virtualizer.isAtEnd());
		update();
		el.addEventListener("scroll", update, { passive: true });
		return () => el.removeEventListener("scroll", update);
	}, [virtualizer, virtualizer?.scrollElement]);

	if (!virtualizer || !visible) return null;

	return (
		<button
			type="button"
			onClick={() => virtualizer.scrollToEnd({ behavior: "smooth" })}
			aria-label={ariaLabel}
			className={clsx(
				"absolute bottom-4 left-1/2 z-10 flex -translate-x-1/2 items-center gap-1.5 rounded-full border border-app-line bg-app-box px-3 py-1.5 text-xs text-ink shadow-md transition-opacity hover:bg-app-selected",
				className,
			)}
		>
			{children ?? (
				<>
					<ArrowDown weight="bold" className="h-3 w-3" />
					<span>Jump to latest</span>
				</>
			)}
		</button>
	);
}
