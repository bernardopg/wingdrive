import {ArrowCounterClockwise, ArrowLeft, Trash, X} from '@phosphor-icons/react';
import {getIcon} from '@wingdrive/assets/util';
import {Button, CircleButton, toast} from '@wingdrive/primitives';
import type {TrashEntry} from '@wingdrive/ts-client';
import clsx from 'clsx';
import {useState} from 'react';
import {useNavigate} from 'react-router-dom';
import {usePlatform} from '../../contexts/PlatformContext';
import {useCoreMutation, useCoreQuery} from '../../contexts/WingDriveContext';
import {TopBarItem, TopBarPortal} from '../../TopBar';
import {formatBytes, formatRelativeTime} from '../explorer/utils';

export function TrashView() {
	const navigate = useNavigate();
	const platform = usePlatform();
	const [selected, setSelected] = useState<Set<string>>(new Set());
	const {data, isLoading, refetch} = useCoreQuery({
		type: 'trash.list',
		input: {}
	});
	const restore = useCoreMutation('trash.restore');
	const purge = useCoreMutation('trash.purge');

	const items = data?.items ?? [];
	const targets = selected.size > 0 ? [...selected] : [];
	const busy = restore.isPending || purge.isPending;

	function toggle(id: string) {
		setSelected((prev) => {
			const next = new Set(prev);
			if (next.has(id)) next.delete(id);
			else next.add(id);
			return next;
		});
	}

	async function run(
		action: () => Promise<unknown>,
		done: string
	): Promise<void> {
		try {
			await action();
			toast.success(done);
		} catch (err) {
			toast.error(String(err));
		}
		setSelected(new Set());
		await refetch();
	}

	function restoreItems(ids: string[]) {
		void run(
			() => restore.mutateAsync({ids}),
			ids.length === 1 ? 'Item restored' : `${ids.length} items restored`
		);
	}

	function purgeItems(ids: string[], label: string) {
		platform.confirm(
			`${label} cannot be recovered. Continue?`,
			(ok) => {
				if (ok) {
					void run(
						() => purge.mutateAsync({ids}),
						`${ids.length} ${ids.length === 1 ? 'item' : 'items'} deleted`
					);
				}
			}
		);
	}

	return (
		<>
			<TopBarPortal
				left={
					<>
						<TopBarItem id="back" label="Back" priority="high">
							<CircleButton
								icon={ArrowLeft}
								onClick={() => navigate(-1)}
							/>
						</TopBarItem>
						<TopBarItem id="title" label="Title" priority="high">
							<h1 className="text-ink text-xl font-bold">Trash</h1>
						</TopBarItem>
					</>
				}
				right={
					<>
						<TopBarItem id="restore" label="Restore" priority="high">
							<Button
								variant="gray"
								size="sm"
								disabled={busy || targets.length === 0}
								onClick={() => restoreItems(targets)}
							>
								Restore
							</Button>
						</TopBarItem>
						<TopBarItem id="empty" label="Empty Trash" priority="high">
							<Button
								variant="colored"
								className="bg-status-error border-status-error"
								size="sm"
								disabled={busy || items.length === 0}
								onClick={() =>
									targets.length > 0
										? purgeItems(targets, 'The selected items')
										: purgeItems(
												items.map((item) => item.id),
												'Everything in the trash'
											)
								}
							>
								{targets.length > 0 ? 'Delete Forever' : 'Empty Trash'}
							</Button>
						</TopBarItem>
					</>
				}
			/>

			<div className="flex h-full flex-col overflow-y-auto px-6 py-4">
				{data && !data.supported ? (
					<EmptyState text="This system does not let WingDrive read its trash. Open it from your file manager instead." />
				) : isLoading ? null : items.length === 0 ? (
					<EmptyState text="The trash is empty." />
				) : (
					<ul className="flex flex-col gap-1">
						{items.map((item) => (
							<TrashRow
								key={item.id}
								item={item}
								selected={selected.has(item.id)}
								disabled={busy}
								onToggle={() => toggle(item.id)}
								onRestore={() => restoreItems([item.id])}
								onPurge={() => purgeItems([item.id], `"${item.name}"`)}
							/>
						))}
					</ul>
				)}
			</div>
		</>
	);
}

interface TrashRowProps {
	item: TrashEntry;
	selected: boolean;
	disabled: boolean;
	onToggle: () => void;
	onRestore: () => void;
	onPurge: () => void;
}

function TrashRow({
	item,
	selected,
	disabled,
	onToggle,
	onRestore,
	onPurge
}: TrashRowProps) {
	const extension = item.is_dir ? null : (item.name.split('.').pop() ?? null);
	const icon = getIcon(item.is_dir ? 'Folder' : 'Document', true, extension, item.is_dir);
	const parent = item.original_path.slice(
		0,
		item.original_path.length - item.name.length - 1
	);

	return (
		<li
			className={clsx(
				'group flex items-center gap-3 rounded-lg px-3 py-2',
				selected ? 'bg-accent/15' : 'hover:bg-app-hover'
			)}
		>
			<button
				type="button"
				aria-pressed={selected}
				aria-label={`Select ${item.name}`}
				className="flex min-w-0 flex-1 items-center gap-3 text-left"
				onClick={onToggle}
			>
				<img src={icon} alt="" className="size-8 shrink-0" />
				<div className="min-w-0 flex-1">
					<div className="text-ink truncate text-sm font-medium">
						{item.name}
					</div>
					<div className="text-ink-faint truncate text-xs">{parent || '/'}</div>
				</div>
				<div className="text-ink-dull w-20 shrink-0 text-right text-xs">
					{item.size !== null ? formatBytes(item.size) : '—'}
				</div>
				<div className="text-ink-dull w-28 shrink-0 text-right text-xs">
					{formatRelativeTime(new Date(item.deleted_at * 1000))}
				</div>
			</button>
			<div className="flex gap-1 opacity-0 transition-opacity group-focus-within:opacity-100 group-hover:opacity-100">
				<CircleButton
					icon={ArrowCounterClockwise}
					title="Restore"
					disabled={disabled}
					onClick={onRestore}
				/>
				<CircleButton
					icon={X}
					title="Delete forever"
					disabled={disabled}
					onClick={onPurge}
				/>
			</div>
		</li>
	);
}

function EmptyState({text}: {text: string}) {
	return (
		<div className="text-ink-dull flex flex-1 flex-col items-center justify-center gap-3 text-sm">
			<Trash size={40} weight="thin" />
			<p>{text}</p>
		</div>
	);
}
