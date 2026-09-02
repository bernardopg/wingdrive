"use client";

import {
	useVirtualizer,
	type Virtualizer,
	type VirtualizerOptions,
} from "@tanstack/react-virtual";

type PartialKeys<T, K extends keyof T> = Omit<T, K> & Partial<Pick<T, K>>;

export type EndAnchoredVirtualizerOptions<
	TScrollElement extends Element,
	TItemElement extends Element,
> = PartialKeys<
	VirtualizerOptions<TScrollElement, TItemElement>,
	"observeElementRect" | "observeElementOffset" | "scrollToFn"
>;

export function useEndAnchoredVirtualizer<
	TScrollElement extends Element,
	TItemElement extends Element = Element,
>(
	options: EndAnchoredVirtualizerOptions<TScrollElement, TItemElement>,
): Virtualizer<TScrollElement, TItemElement> {
	return useVirtualizer({
		anchorTo: "end",
		followOnAppend: true,
		...options,
	});
}
