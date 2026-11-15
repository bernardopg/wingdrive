import {DotsSixVertical} from '@phosphor-icons/react';
import {SortableContext, useSortable, verticalListSortingStrategy} from '@dnd-kit/sortable';
import {CSS} from '@dnd-kit/utilities';
import {useSidebarStore} from '@wingdrive/ts-client';
import type {ReactNode} from 'react';
import {useTranslation} from '../../i18n';
import {orderDynamicItems} from './reorderItems';

export function SortableDynamicList<T extends {id: string}>({groupId, items, label, children}: {
	groupId: string;
	items: T[];
	label: (item: T) => string;
	children: (item: T, index: number) => ReactNode;
}) {
	const saved = useSidebarStore((state) => state.dynamicItemOrder[groupId]);
	const ordered = orderDynamicItems(items, saved ?? []);
	const ids = ordered.map((item) => item.id);
	return (
		<SortableContext items={ids.map((id) => `dynamic:${groupId}:${id}`)} strategy={verticalListSortingStrategy}>
			{ordered.map((item, index) => (
				<DynamicRow key={item.id} groupId={groupId} itemId={item.id} ids={ids} label={label(item)}>
					{children(item, index)}
				</DynamicRow>
			))}
		</SortableContext>
	);
}

function DynamicRow({groupId, itemId, ids, label, children}: {
	groupId: string; itemId: string; ids: string[]; label: string; children: ReactNode;
}) {
	const {t} = useTranslation('navigation');
	const {attributes, listeners, setNodeRef, setActivatorNodeRef, transform, transition, isDragging} = useSortable({
		id: `dynamic:${groupId}:${itemId}`,
		data: {type: 'sidebar-dynamic', groupId, itemId, ids, label}
	});
	return (
		<div ref={setNodeRef} style={{transform: CSS.Transform.toString(transform), transition}} className={`group flex items-center ${isDragging ? 'relative z-50 opacity-50' : ''}`}>
			<div className="min-w-0 flex-1">{children}</div>
			<button ref={setActivatorNodeRef} type="button" {...attributes} {...listeners}
				aria-label={t('sidebar.reorderItem', {name: label})}
				className="cursor-grab rounded-md p-1 text-sidebar-ink-faint opacity-50 transition-opacity hover:text-sidebar-ink hover:opacity-100 focus-visible:opacity-100 focus-visible:outline focus-visible:outline-2 focus-visible:outline-accent active:cursor-grabbing">
				<DotsSixVertical size={14} aria-hidden="true" />
			</button>
		</div>
	);
}
