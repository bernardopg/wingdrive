import {
	horizontalListSortingStrategy,
	SortableContext,
	useSortable,
} from '@dnd-kit/sortable';
import { CSS } from '@dnd-kit/utilities';
import {
	Plus,
	X,
	XCircle,
	ArrowBendUpLeft,
	ArrowLineRight,
} from '@phosphor-icons/react';
import {
	TabBar as TabBarPrimitive,
	TabBarItem,
} from '@wingdrive/primitives';
import clsx from 'clsx';
import { useCallback, useMemo } from 'react';
import type { Tab } from '.';
import { useTabManager } from './useTabManager';
import { useContextMenu } from '../../hooks/useContextMenu';
import { getKeybind } from '../../util/keybinds/registry';
import {
	getComboForPlatform,
	getCurrentPlatform,
	toDisplayString,
} from '../../util/keybinds/platform';

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
				icon: XCircle,
				label: 'Close Other Tabs',
				onClick: () => onCloseOthers(tab.id),
				disabled: isLastTab,
			},
			{
				icon: ArrowLineRight,
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

	// The primitive forwards ref/style to its inner button but keeps className on
	// its wrapper, so sortable measured the wrong box. Own the drag node here.
	return (
		<div
			ref={setNodeRef}
			style={style}
			{...attributes}
			{...listeners}
			className={clsx(
				'flex min-w-0 flex-1',
				isDragging && 'z-50 opacity-50',
			)}
		>
			<TabBarItem
				label={tab.title}
				active={isActive}
				/* The window keeps at least one tab, so hide the affordance
				   instead of offering a click that does nothing. */
				closable={!isLastTab}
				onClose={() => onClose(tab.id)}
				closeIcon={<X size={10} weight="bold" />}
				onClick={() => onSwitch(tab.id)}
				onContextMenu={handleContextMenu}
			/>
		</div>
	);
}

/** Platform-correct shortcut hint, so Linux/Windows don't get macOS glyphs. */
function keybindHint(id: Parameters<typeof getKeybind>[0]): string {
	const keybind = getKeybind(id);
	if (!keybind) return '';
	const platform = getCurrentPlatform();
	return toDisplayString(
		getComboForPlatform(keybind.combo, platform),
		platform,
	);
}

export function TabBar() {
	const {
		tabs,
		activeTabId,
		switchTab,
		closeTab,
		closeTabs,
		createTab,
		reopenTab,
		hasClosedTabs,
	} = useTabManager();

	// Ensure activeTabId exists in tabs array, fallback to first tab
	// Memoize to prevent unnecessary rerenders during rapid state updates
	const safeActiveTabId = useMemo(() => {
		return tabs.find((t) => t.id === activeTabId)?.id ?? tabs[0]?.id;
	}, [tabs, activeTabId]);

	const closeOthers = (keepId: string) => {
		closeTabs(tabs.filter((t) => t.id !== keepId).map((t) => t.id));
	};

	const closeToRight = (fromId: string) => {
		const fromIndex = tabs.findIndex((t) => t.id === fromId);
		if (fromIndex === -1) return;
		closeTabs(tabs.slice(fromIndex + 1).map((t) => t.id));
	};

	return (
		<TabBarPrimitive
			trailing={
				<button
					onClick={() => createTab()}
					className="hover:bg-app-hover text-ink-dull hover:text-ink flex size-7 shrink-0 items-center justify-center rounded-full transition-colors"
					title={`New tab (${keybindHint('global.newTab')})`}
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
						isLastTab={tabs.length <= 1}
						isRightmost={index === tabs.length - 1}
					/>
				))}
			</SortableContext>
		</TabBarPrimitive>
	);
}