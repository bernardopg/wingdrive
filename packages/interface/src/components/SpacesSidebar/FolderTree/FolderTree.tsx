import {
	ArrowsInLineVertical,
	ArrowsOutLineVertical,
	CaretRight,
	Folder,
	FolderOpen,
	HardDrives,
	House
} from '@phosphor-icons/react';
import type {Icon} from '@phosphor-icons/react';
import {useQuery} from '@tanstack/react-query';
import type {File, WingPath} from '@wingdrive/ts-client';
import {useNormalizedQuery, useSidebarStore} from '@wingdrive/ts-client';
import clsx from 'clsx';
import {useCallback, useEffect, useRef, useState} from 'react';
import {useNavigate, useSearchParams} from 'react-router-dom';

import {usePlatform} from '../../../contexts/PlatformContext';
import {useTranslation} from '../../../i18n';
import {useExplorer} from '../../../routes/explorer/context';
import {useOpenInNewTabAction} from '../../../routes/explorer/OpenFileProvider';
import {wingPathFromParam} from '../../../routes/explorer/wingPathParam';
import {explorerUrlForDirectory} from '../../TabManager/LaunchRequestSync';
import {GroupHeader} from '../GroupHeader';
import {ancestorsWithin, normalizeFolder, rootFor} from './folderTreePaths';

/** Sidebar collapse key; the section is not a space group stored by the daemon. */
const SECTION_ID = 'folder-tree';
/** Indentation per tree level, in pixels. */
const INDENT = 12;
/**
 * Most folders one "expand" click opens. Each one lists its directory, and
 * the levels below a home folder or the filesystem root hold thousands.
 */
const EXPAND_BATCH = 50;
/** Polls while a folder the indexer has not seen yet still lists empty. */
const WARMUP_POLLS = 6;

interface Root {
	path: string;
	label: string;
	icon: Icon;
}

/** The physical folder the active tab shows, from the Explorer URL. */
function useCurrentFolder(): string | null {
	const [params] = useSearchParams();
	const raw = params.get('path');
	const path = raw ? wingPathFromParam(raw) : null;
	return path && 'Physical' in path ? normalizeFolder(path.Physical.path) : null;
}

/**
 * Collapsible folder tree rooted at the home folder and the filesystem root,
 * as the Folders panel of Dolphin and the tree of Windows Explorer. Platforms
 * that cannot tell the home folder (the web UI) show the root alone. It opens
 * the branch of the folder the active tab shows and highlights it.
 */
export function FolderTree({sortableAttributes, sortableListeners}: {
	sortableAttributes?: React.HTMLAttributes<HTMLElement>;
	sortableListeners?: React.DOMAttributes<HTMLElement>;
} = {}) {
	const {t} = useTranslation('navigation');
	const platform = usePlatform();
	const {collapsedGroups, toggleGroup} = useSidebarStore();
	const isCollapsed = collapsedGroups.has(SECTION_ID);
	const current = useCurrentFolder();
	const [expanded, setExpanded] = useState<Set<string>>(() => new Set());

	const {data: home} = useQuery({
		queryKey: ['platform', 'homeDir'],
		queryFn: () => platform.homeDir!(),
		enabled: !!platform.homeDir,
		staleTime: Infinity
	});

	const roots: Root[] = [
		...(home ? [{path: normalizeFolder(home), label: t('folderTree.home'), icon: House}] : []),
		{path: '/', label: t('folderTree.computer'), icon: HardDrives}
	];
	const rootPaths = roots.map((root) => root.path);

	// Follow navigation: open every folder above the current one, once per change.
	const followRoot = current ? rootFor(current, rootPaths) : null;
	useEffect(() => {
		if (!current || !followRoot) return;
		const ancestors = ancestorsWithin(current, followRoot);
		if (ancestors.length === 0) return;
		setExpanded((previous) => {
			if (ancestors.every((path) => previous.has(path))) return previous;
			return new Set([...previous, ...ancestors]);
		});
	}, [current, followRoot]);

	const toggle = (path: string) =>
		setExpanded((previous) => {
			const next = new Set(previous);
			if (next.has(path)) next.delete(path);
			else next.add(path);
			return next;
		});

	// Folders on screen that may hold subfolders. Each click opens the
	// shallowest closed ones, capped at EXPAND_BATCH: a recursive
	// expand of the filesystem root would list every folder on the disk.
	// Counted, not a set: the home folder shows both as a root and under
	// Computer, and closing one copy must not forget the other.
	const visible = useRef(new Map<string, number>());
	const registerVisible = useCallback((path: string) => {
		const counts = visible.current;
		counts.set(path, (counts.get(path) ?? 0) + 1);
		return () => {
			const left = (counts.get(path) ?? 1) - 1;
			if (left > 0) counts.set(path, left);
			else counts.delete(path);
		};
	}, []);
	const expandVisible = () =>
		setExpanded((previous) => {
			const depth = (path: string) => path.split('/').length;
			const closed = [...visible.current.keys()]
				.filter((path) => !previous.has(path))
				.sort((a, b) => depth(a) - depth(b) || a.localeCompare(b));
			return new Set([...previous, ...closed.slice(0, EXPAND_BATCH)]);
		});
	const collapseAll = () => setExpanded(new Set());

	return (
		<div>
			<GroupHeader
				sortableAttributes={sortableAttributes}
				sortableListeners={sortableListeners}
				label={t('folderTree.title')}
				isCollapsed={isCollapsed}
				onToggle={() => toggleGroup(SECTION_ID)}
				actions={
					!isCollapsed && (
						<div className="flex items-center">
							<TreeAction
								label={t('folderTree.expandAll')}
								icon={ArrowsOutLineVertical}
								onClick={expandVisible}
							/>
							<TreeAction
								label={t('folderTree.collapseAll')}
								icon={ArrowsInLineVertical}
								onClick={collapseAll}
							/>
						</div>
					)
				}
			/>
			{!isCollapsed && (
				<div role="tree" aria-label={t('folderTree.title')} className="space-y-0.5">
					{roots.map((root) => (
						<FolderNode
							key={root.path}
							path={root.path}
							label={root.label}
							icon={root.icon}
							depth={0}
							expanded={expanded}
							onToggle={toggle}
							registerVisible={registerVisible}
							current={current}
							// The filesystem root lists the home folder's parent too; the
							// tree only follows a folder in the deepest root holding it.
							follows={followRoot === root.path}
						/>
					))}
				</div>
			)}
		</div>
	);
}

interface FolderNodeProps {
	path: string;
	label: string;
	icon?: Icon;
	depth: number;
	expanded: Set<string>;
	onToggle: (path: string) => void;
	/** Records the folder as on screen until the returned cleanup runs. */
	registerVisible: (path: string) => () => void;
	current: string | null;
	follows: boolean;
}

function TreeAction({label, icon: ActionIcon, onClick}: {label: string; icon: Icon; onClick: () => void}) {
	return (
		<button
			type="button"
			aria-label={label}
			title={label}
			className="text-sidebar-ink-faint hover:bg-sidebar-selected hover:text-sidebar-ink flex size-6 items-center justify-center rounded-md transition-colors"
			onClick={onClick}
		>
			<ActionIcon size={12} weight="bold" />
		</button>
	);
}

function FolderNode({
	path,
	label,
	icon,
	depth,
	expanded,
	onToggle,
	registerVisible,
	current,
	follows
}: FolderNodeProps) {
	const {t} = useTranslation('navigation');
	const navigate = useNavigate();
	const openInNewTab = useOpenInNewTabAction();
	const {viewSettings} = useExplorer();
	const isOpen = expanded.has(path);
	const isCurrent = follows && current === path;
	const wingPath: WingPath = {Physical: {device_slug: 'local', path}};

	const {data} = useNormalizedQuery({
		query: 'files.directory_listing',
		input: {
			path: wingPath,
			limit: null,
			include_hidden: viewSettings.showHiddenFiles,
			sort_by: 'name',
			folders_first: true,
			sort_direction: 'Asc'
		},
		resourceType: 'file',
		pathScope: wingPath,
		enabled: isOpen,
		// One subscription per open folder used up the daemon's 100
		// connections once "expand" opened a few levels. The tree refetches
		// when a folder reopens, on window focus, and after the explorer's own
		// file operations (useRefetchFileListings), which covers its use.
		live: false,
		refetchInterval: (query) =>
			query.state.data &&
			(query.state.data as {files: File[]}).files.length === 0 &&
			query.state.dataUpdateCount < WARMUP_POLLS
				? 750
				: false
	});
	const children = ((data as {files: File[]} | undefined)?.files ?? []).filter(
		(file) => file.kind === 'Directory' && 'Physical' in file.wing_path
	);
	const isLeaf = isOpen && !!data && children.length === 0;
	useEffect(() => (isLeaf ? undefined : registerVisible(path)), [isLeaf, path, registerVisible]);
	const RowIcon = icon ?? (isOpen ? FolderOpen : Folder);
	const url = explorerUrlForDirectory(path);

	return (
		<div role="treeitem" aria-expanded={isLeaf ? undefined : isOpen} aria-selected={isCurrent}>
			<div
				className={clsx(
					'group flex w-full cursor-default items-center gap-1 rounded-md py-1 pr-2 text-sm font-medium',
					isCurrent
						? 'bg-sidebar-selected text-sidebar-ink'
						: 'text-sidebar-inkDull hover:bg-sidebar-selected/40 hover:text-sidebar-ink'
				)}
				style={{paddingLeft: 4 + depth * INDENT}}
				onClick={() => navigate(url)}
				onAuxClick={(e) => {
					if (e.button !== 1) return;
					e.preventDefault();
					openInNewTab(url, label);
				}}
				title={path}
			>
				<button
					type="button"
					aria-label={isOpen ? t('folderTree.collapse') : t('folderTree.expand')}
					tabIndex={-1}
					className={clsx(
						'flex size-4 shrink-0 items-center justify-center rounded-sm hover:bg-sidebar-selected',
						isLeaf && 'invisible'
					)}
					onClick={(e) => {
						e.stopPropagation();
						onToggle(path);
					}}
				>
					<CaretRight
						size={10}
						weight="bold"
						className={clsx('transition-transform', isOpen && 'rotate-90')}
					/>
				</button>
				<RowIcon size={16} weight={isCurrent ? 'fill' : 'regular'} className="shrink-0" />
				<span className="truncate">{label}</span>
			</div>
			{isOpen && children.length > 0 && (
				<div role="group">
					{children.map((child) => {
						const childPath = normalizeFolder(
							'Physical' in child.wing_path ? child.wing_path.Physical.path : ''
						);
						return (
							<FolderNode
								key={childPath}
								path={childPath}
								// `name` drops what looks like an extension, which folders keep.
								label={childPath.slice(childPath.lastIndexOf('/') + 1)}
								depth={depth + 1}
								expanded={expanded}
								onToggle={onToggle}
								registerVisible={registerVisible}
								current={current}
								follows={follows}
							/>
						);
					})}
				</div>
			)}
		</div>
	);
}
