'use client';

import {
	JumpToEndButton,
	useEndAnchoredVirtualizer,
} from '@wingdrive/primitives';
import clsx from 'clsx';
import {
	type ReactNode,
	type Ref,
	useEffect,
	useImperativeHandle,
	useRef,
} from 'react';

export interface ChatMessageListHandle {
	scrollToEnd: (opts?: {behavior?: ScrollBehavior}) => void;
	scrollToIndex: (
		index: number,
		opts?: {
			align?: 'start' | 'center' | 'end' | 'auto';
			behavior?: ScrollBehavior;
		}
	) => void;
	isAtEnd: () => boolean;
	getDistanceFromEnd: () => number;
}

export interface ChatMessageListProps<T> {
	messages: ReadonlyArray<T>;
	getMessageKey: (index: number) => string | number;
	renderMessage: (message: T, index: number) => ReactNode;
	estimateMessageSize: (index: number) => number;
	overscan?: number;
	onLoadOlder?: () => void;
	hasMoreHistory?: boolean;
	isLoadingOlder?: boolean;
	showJumpToLatest?: boolean;
	jumpToLatestLabel?: ReactNode;
	emptyState?: ReactNode;
	handleRef?: Ref<ChatMessageListHandle>;
	className?: string;
}

export function ChatMessageList<T>({
	messages,
	getMessageKey,
	renderMessage,
	estimateMessageSize,
	overscan = 8,
	onLoadOlder,
	hasMoreHistory = false,
	isLoadingOlder = false,
	showJumpToLatest = true,
	jumpToLatestLabel,
	emptyState,
	handleRef,
	className
}: ChatMessageListProps<T>) {
	const scrollRef = useRef<HTMLDivElement>(null);

	const virtualizer = useEndAnchoredVirtualizer<HTMLDivElement>({
		count: messages.length,
		getScrollElement: () => scrollRef.current,
		estimateSize: estimateMessageSize,
		getItemKey: getMessageKey,
		overscan,
		followOnAppend: 'smooth'
	});

	useImperativeHandle(
		handleRef,
		() => ({
			scrollToEnd: (opts) => virtualizer.scrollToEnd(opts ?? {}),
			scrollToIndex: (index, opts) =>
				virtualizer.scrollToIndex(index, opts),
			isAtEnd: () => virtualizer.isAtEnd(),
			getDistanceFromEnd: () => virtualizer.getDistanceFromEnd()
		}),
		[virtualizer]
	);

	const startIndex = virtualizer.range?.startIndex ?? -1;
	const reachedStartRef = useRef(false);

	useEffect(() => {
		if (!onLoadOlder || !hasMoreHistory) return;
		if (startIndex === 0 && messages.length > 0) {
			if (!reachedStartRef.current) {
				reachedStartRef.current = true;
				onLoadOlder();
			}
		} else {
			reachedStartRef.current = false;
		}
	}, [startIndex, messages.length, onLoadOlder, hasMoreHistory]);

	if (messages.length === 0 && emptyState) {
		return (
			<div
				className={clsx(
					'flex h-full w-full items-center justify-center',
					className
				)}
			>
				{emptyState}
			</div>
		);
	}

	const virtualItems = virtualizer.getVirtualItems();

	return (
		<div className={clsx('relative h-full w-full', className)}>
			<div ref={scrollRef} className="relative h-full w-full overflow-auto">
				<div
					className="relative w-full"
					style={{height: `${virtualizer.getTotalSize()}px`}}
				>
					{virtualItems.map((virtual) => (
						<div
							key={virtual.key}
							data-index={virtual.index}
							ref={virtualizer.measureElement}
							className="absolute left-0 top-0 w-full"
							style={{transform: `translateY(${virtual.start}px)`}}
						>
							{renderMessage(messages[virtual.index]!, virtual.index)}
						</div>
					))}
				</div>
			</div>
			{isLoadingOlder ? (
				<div className="pointer-events-none absolute inset-x-0 top-0 flex justify-center py-2">
					<div className="rounded-full bg-app-box/80 px-3 py-1 text-xs text-ink-faint shadow-md">
						Loading older messages…
					</div>
				</div>
			) : null}
			{showJumpToLatest ? (
				<JumpToEndButton virtualizer={virtualizer}>
					{jumpToLatestLabel}
				</JumpToEndButton>
			) : null}
		</div>
	);
}
