"use client";

import { type Virtualizer } from "@tanstack/react-virtual";
import clsx from "clsx";
import {
	type ReactNode,
	type Ref,
	useEffect,
	useImperativeHandle,
	useRef,
} from "react";

import { useEndAnchoredVirtualizer } from "./useEndAnchoredVirtualizer";

export interface VirtualListHandle {
	virtualizer: Virtualizer<HTMLDivElement, Element> | null;
	scrollToEnd: (opts?: { behavior?: ScrollBehavior }) => void;
	scrollToIndex: (
		index: number,
		opts?: { align?: "start" | "center" | "end" | "auto"; behavior?: ScrollBehavior },
	) => void;
	isAtEnd: () => boolean;
	getDistanceFromEnd: () => number;
}

export interface VirtualListProps<T> {
	items: ReadonlyArray<T>;
	getItemKey: (index: number) => string | number;
	estimateSize: (index: number) => number;
	renderItem: (item: T, index: number) => ReactNode;
	overscan?: number;
	anchorTo?: "start" | "end";
	followOnAppend?: boolean | ScrollBehavior;
	scrollEndThreshold?: number;
	onReachStart?: () => void;
	handleRef?: Ref<VirtualListHandle>;
	className?: string;
	innerClassName?: string;
}

export function VirtualList<T>({
	items,
	getItemKey,
	estimateSize,
	renderItem,
	overscan = 8,
	anchorTo = "end",
	followOnAppend = true,
	scrollEndThreshold,
	onReachStart,
	handleRef,
	className,
	innerClassName,
}: VirtualListProps<T>) {
	const scrollRef = useRef<HTMLDivElement>(null);

	const virtualizer = useEndAnchoredVirtualizer<HTMLDivElement, Element>({
		count: items.length,
		getScrollElement: () => scrollRef.current,
		estimateSize,
		getItemKey,
		overscan,
		anchorTo,
		followOnAppend,
		...(scrollEndThreshold !== undefined && { scrollEndThreshold }),
	});

	useImperativeHandle(
		handleRef,
		() => ({
			virtualizer,
			scrollToEnd: (opts) => virtualizer.scrollToEnd(opts ?? {}),
			scrollToIndex: (index, opts) => virtualizer.scrollToIndex(index, opts),
			isAtEnd: () => virtualizer.isAtEnd(),
			getDistanceFromEnd: () => virtualizer.getDistanceFromEnd(),
		}),
		[virtualizer],
	);

	const range = virtualizer.range;
	const startIndex = range?.startIndex ?? -1;
	const reachedStartRef = useRef(false);

	useEffect(() => {
		if (!onReachStart) return;
		if (startIndex === 0 && items.length > 0) {
			if (!reachedStartRef.current) {
				reachedStartRef.current = true;
				onReachStart();
			}
		} else {
			reachedStartRef.current = false;
		}
	}, [startIndex, items.length, onReachStart]);

	const virtualItems = virtualizer.getVirtualItems();

	return (
		<div
			ref={scrollRef}
			className={clsx("relative h-full w-full overflow-auto", className)}
		>
			<div
				className={clsx("relative w-full", innerClassName)}
				style={{ height: `${virtualizer.getTotalSize()}px` }}
			>
				{virtualItems.map((virtual) => (
					<div
						key={virtual.key}
						data-index={virtual.index}
						ref={virtualizer.measureElement}
						className="absolute left-0 top-0 w-full"
						style={{ transform: `translateY(${virtual.start}px)` }}
					>
						{renderItem(items[virtual.index]!, virtual.index)}
					</div>
				))}
			</div>
		</div>
	);
}
