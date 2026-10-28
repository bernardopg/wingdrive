import {useDndContext} from '@dnd-kit/core';
import {
	horizontalListSortingStrategy,
	SortableContext,
	useSortable
} from '@dnd-kit/sortable';
import {CSS} from '@dnd-kit/utilities';
import {
	ArrowBendUpLeft,
	ArrowLineRight,
	Plus,
	X,
	XCircle
} from '@phosphor-icons/react';
import {TabBarItem, TabBar as TabBarPrimitive} from '@wingdrive/primitives';
import clsx from 'clsx';
import {AnimatePresence, motion} from 'framer-motion';
import {useCallback, useEffect, useMemo} from 'react';
import type {Tab} from '.';
import {folderOfRoute} from '../../routes/explorer/wingPathParam';
import {useContextMenu} from '../../hooks/useContextMenu';
import {
	getComboForPlatform,
	getCurrentPlatform,
	toDisplayString
} from '../../util/keybinds/platform';
import {getKeybind} from '../../util/keybinds/registry';
import {useTabManager} from './useTabManager';
import {useTranslation} from '../../i18n';

const TAB_SWITCH_DELAY_MS = 400;

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
	isRightmost
}: SortableTabProps) {
const {t} = useTranslation('dialogs');
	const {
		attributes,
		listeners,
		setNodeRef,
		transform,
		transition,
		isDragging,
		isOver
	} = useSortable({
		id: tab.id,
		data: {
			type: 'tab',
			tabId: tab.id,
			// Files dropped on a tab go into the folder it shows.
			targetPath: folderOfRoute(tab.savedPath)
		}
	});

	// Holding dragged files over another tab switches to it, so the drop can
	// land in one of its subfolders. Short enough to feel immediate, long
	// enough not to flip tabs while the pointer only crosses the bar.
	const {active} = useDndContext();
	const fileOverOtherTab =
		isOver && !isActive && active?.data.current?.type === 'explorer-file';
	useEffect(() => {
		if (!fileOverOtherTab) return;
		const timer = setTimeout(() => onSwitch(tab.id), TAB_SWITCH_DELAY_MS);
		return () => clearTimeout(timer);
	}, [fileOverOtherTab, onSwitch, tab.id]);

	const style = {
		transform: CSS.Transform.toString(transform),
		transition
	};

	// Tab context menu: right-click for close variants and reopen
	const contextMenu = useContextMenu({
		items: [
			{
				icon: X,
 label: t('tabs.closeTab'),
				keybindId: 'global.closeTab',
				onClick: () => onClose(tab.id),
				disabled: isLastTab
			},
			{
				icon: XCircle,
 label: t('tabs.closeOtherTabs'),
				onClick: () => onCloseOthers(tab.id),
				disabled: isLastTab
			},
			{
				icon: ArrowLineRight,
 label: t('tabs.closeTabsToRight'),
				onClick: () => onCloseToRight(tab.id),
				disabled: isRightmost
			},
			{type: 'separator'},
			{
				icon: ArrowBendUpLeft,
 label: t('tabs.reopenClosedTab'),
				keybindId: 'global.reopenTab',
				onClick: onReopen,
				disabled: !canReopen
			}
		]
	});

	const handleContextMenu = useCallback(
		async (e: React.MouseEvent) => {
			e.preventDefault();
			e.stopPropagation();
			await contextMenu.show(e);
		},
		[contextMenu]
	);

	// The primitive forwards ref/style to its inner button but keeps className on
	// its wrapper, so sortable measured the wrong box. Own the drag node here.
	return (
		<motion.div
			initial={{opacity: 0}}
			animate={{opacity: 1}}
			exit={{opacity: 0, width: 0}}
			transition={{duration: 0.15}}
			ref={setNodeRef}
			data-tab-id={tab.id}
			data-active={isActive}
			style={style}
			{...attributes}
			{...listeners}
			className={clsx(
				'flex min-w-0 flex-1 rounded-md',
				isDragging && 'z-50 opacity-50',
				fileOverOtherTab && 'ring-2 ring-inset ring-accent'
			)}
		>
			<TabBarItem
				label={tab.title === 'Overview' ? t('navigation:items.overview') : tab.title}
				active={isActive}
				/* The window keeps at least one tab, so hide the affordance
				   instead of offering a click that does nothing. */
				closable={!isLastTab}
				onClose={() => onClose(tab.id)}
				closeIcon={<X size={10} weight="bold" />}
				onClick={() => onSwitch(tab.id)}
				onAuxClick={(e) => {
					if (e.button !== 1) return;
					e.preventDefault();
					e.stopPropagation();
					onClose(tab.id);
				}}
				onContextMenu={handleContextMenu}
			/>
		</motion.div>
	);
}

/** Platform-correct shortcut hint, so Linux/Windows don't get macOS glyphs. */
function keybindHint(id: Parameters<typeof getKeybind>[0]): string {
	const keybind = getKeybind(id);
	if (!keybind) return '';
	const platform = getCurrentPlatform();
	return toDisplayString(
		getComboForPlatform(keybind.combo, platform),
		platform
	);
}

export function TabBar() {
const {t} = useTranslation('dialogs');
	const {
		tabs,
		activeTabId,
		switchTab,
		closeTab,
		closeTabs,
		createTab,
		reopenTab,
		hasClosedTabs
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
title={t('tabs.newTab', {shortcut: keybindHint('global.newTab')})}
				>
					<Plus size={14} weight="bold" />
				</button>
			}
		>
			<SortableContext
				items={tabs.map((tab) => tab.id)}
				strategy={horizontalListSortingStrategy}
			>
				<AnimatePresence initial={false}>
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
				</AnimatePresence>
			</SortableContext>
		</TabBarPrimitive>
	);
}
