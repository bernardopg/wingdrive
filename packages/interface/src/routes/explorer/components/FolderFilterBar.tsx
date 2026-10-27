import {Funnel, X} from '@phosphor-icons/react';
import {useRef} from 'react';
import {useKeybind} from '../../../hooks/useKeybind';
import {useTranslation} from '../../../i18n';
import {useExplorer} from '../context';

interface FolderFilterBarProps {
	/** Items the filter shows, and the listing size before it. */
	shown: number;
	total: number;
}

/** Narrows the folder being browsed by name, as you type, without a search. */
export function FolderFilterBar({shown, total}: FolderFilterBarProps) {
	const {t} = useTranslation('explorer');
	const {folderFilter, setFolderFilter} = useExplorer();
	const inputRef = useRef<HTMLInputElement>(null);

	useKeybind('explorer.filterFolder', () => {
		if (folderFilter === null) {
			setFolderFilter('');
		} else {
			inputRef.current?.focus();
			inputRef.current?.select();
		}
	});

	if (folderFilter === null) return null;

	return (
		<div className="mx-3 mb-1.5 flex h-8 items-center gap-2 rounded-lg border border-app-line bg-app-box px-2.5">
			<Funnel className="size-4 shrink-0 text-ink-faint" />
			<input
				ref={inputRef}
				autoFocus
				type="text"
				value={folderFilter}
				onChange={(e) => setFolderFilter(e.target.value)}
				onKeyDown={(e) => {
					if (e.key === 'Escape') {
						e.preventDefault();
						e.stopPropagation();
						setFolderFilter(null);
					}
				}}
				placeholder={t('filterBar.placeholder')}
				aria-label={t('filterBar.placeholder')}
				className="min-w-0 flex-1 border-0 bg-transparent text-sm text-ink outline-none ring-0 placeholder:text-ink-faint focus:outline-none focus:ring-0"
			/>
			{folderFilter.trim() && (
				<span className="shrink-0 text-xs tabular-nums text-ink-dull">
					{t('filterBar.matches', {shown, total})}
				</span>
			)}
			<button
				type="button"
				onClick={() => setFolderFilter(null)}
				title={t('filterBar.clear')}
				aria-label={t('filterBar.clear')}
				className="shrink-0 rounded-md p-0.5 text-ink-faint transition-colors hover:bg-app-hover hover:text-ink"
			>
				<X className="size-3.5" />
			</button>
		</div>
	);
}
