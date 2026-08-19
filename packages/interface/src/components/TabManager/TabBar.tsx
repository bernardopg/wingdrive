import {
	horizontalListSortingStrategy,
	SortableContext,
	useSortable,
} from '@dnd-kit/sortable';
import { CSS } from '@dnd-kit/utilities';
import {
	Plus,
	X,
	Copy,
	ArrowBendUpLeft,
	ArrowSquareOut,
} from '@phosphor-icons/react';
import {
	TabBar as TabBarPrimitive,
	TabBarItem,
} from '@spacedrive/primitives';
import clsx from 'clsx';
import { useCallback, useMemo } from 'react';
import type { Tab } from '.';
import { useTabManager } from './useTabManager';
import { useContextMenu } from '../../hooks/useContextMenu';

interface SortableTabProps {
	tab: Tab;
	isActive: boolean;
	onSwitch: (tabId: string) => void;
	onClose: (tabId: string) => void;
	onCloseOthers: (tabId: string) => void;
	onCloseToRight: (tabId: string) => void;
	onReopen: () => void;
	canReopen: boolean;
	isLastTab: boolean;
	isRightmost: boolean;
}

function SortableTab({
	tab,
	isActive,
	onSwitch,
	onClose,
	onCloseOthers,
	onCloseToRight,
	onReopen,
	canReopen,
	isLastTab,
	isRightmost,
}: SortableTabProps) {
	const {
		attributes,
		listeners,
		setNodeRef,
		transform,
		transition,
		isDragging,
	} = useSortable({
		id: tab.id,
		data: {
			type: 'tab',
			tabId: tab.id,
		},
	});

	const style = {
		transform: CSS.Transform.toString(transform),
		transition,
	};

	// Tab context menu: right-click for close variants and reopen
	const contextMenu = useContextMenu({
		items: [
			{
				icon: X,
				label: 'Close Tab',
				keybindId: 'global.closeTab',
				onClick: () => onClose(tab.id),
				disabled: isLastTab,
			},
			{
				icon: Copy,
				label: 'Close Other Tabs',
				onClick: () => onCloseOthers(tab.id),
				disabled: isLastTab,
			},
			{
				icon: ArrowSquareOut,
				label: 'Close Tabs to the Right',
				onClick: () => onCloseToRight(tab.id),
				disabled: isRightmost,
			},
			{ type: 'separator' },
			{
				icon: ArrowBendUpLeft,
				label: 'Reopen Closed Tab',
				keybindId: 'global.reopenTab',
				onClick: onReopen,
				disabled: !canReopen,
			},
		],
	});

	const handleContextMenu = useCallback(
		async (e: React.MouseEvent) => {
			e.preventDefault();
			e.stopPropagation();
			await contextMenu.show(e);
		},
		[contextMenu],
	);

	return (
		<TabBarItem
			ref={setNodeRef}
			style={style}
			{...attributes}
			{...listeners}
			label={tab.title}
			active={isActive}
			onClose={() => onClose(tab.id)}
			closeIcon={<X size={10} weight="bold" />}
			onClick={() => onSwitch(tab.id)}
			onContextMenu={handleContextMenu}
			className={clsx(isDragging && 'z-50 opacity-50')}
		/>
	);
}

export function TabBar() {
	const {
		tabs,
		activeTabId,
		switchTab,
		closeTab,
		createTab,
		reopenTab,
		hasClosedTabs,
	} = useTabManager();

	// Ensure activeTabId exists in tabs array, fallback to first tab
	// Memoize to prevent unnecessary rerenders during rapid state updates
	const safeActiveTabId = useMemo(() => {
		return tabs.find((t) => t.id === activeTabId)?.id ?? tabs[0]?.id;
	}, [tabs, activeTabId]);

	// Don't show tab bar if only one tab
	if (tabs.length <= 1) {
		return null;
	}

	const closeOthers = (keepId: string) => {
		for (const t of tabs) {
			if (t.id !== keepId) closeTab(t.id);
		}
	};

	const closeToRight = (fromId: string) => {
		const fromIndex = tabs.findIndex((t) => t.id === fromId);
		if (fromIndex === -1) return;
		for (const t of tabs.slice(fromIndex + 1)) {
			closeTab(t.id);
		}
	};

	return (
		<TabBarPrimitive
			trailing={
				<button
					onClick={() => createTab()}
					className="hover:bg-app-hover text-ink-dull hover:text-ink flex size-7 shrink-0 items-center justify-center rounded-full transition-colors"
					title="New tab (⌘T)"
				>
					<Plus size={14} weight="bold" />
				</button>
			}
		>
			<SortableContext
				items={tabs.map((tab) => tab.id)}
				strategy={horizontalListSortingStrategy}
			>
				{tabs.map((tab, index) => (
					<SortableTab
						key={tab.id}
						tab={tab}
						isActive={tab.id === safeActiveTabId}
						onSwitch={switchTab}
						onClose={closeTab}
						onCloseOthers={closeOthers}
						onCloseToRight={closeToRight}
						onReopen={reopenTab}
						canReopen={hasClosedTabs}
						isLastTab={tabs.length === 1}
						isRightmost={index === tabs.length - 1}
					/>
				))}
			</SortableContext>
		</TabBarPrimitive>
	);
}