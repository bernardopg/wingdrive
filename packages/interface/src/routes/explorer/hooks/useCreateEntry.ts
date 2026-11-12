import {toast} from '@wingdrive/primitives';
import type {File, TemplateFile} from '@wingdrive/ts-client';
import {useCallback} from 'react';

import {useLibraryMutation} from '../../../contexts/WingDriveContext';
import {useRefetchFileListings} from '../../../hooks/useRefetchFileListings';
import {useUndo} from '../../../hooks/useUndo';
import {useExplorer} from '../context';
import {physicalPath} from '../fileCapabilities';
import {requestReveal} from '../pendingReveal';
import {useTranslation} from '../../../i18n';

/** First of `base`, `base 2`, ... (before the extension) that is not taken. */
export function nextFreeName(base: string, existing: Iterable<string>): string {
	const taken = new Set(existing);
	if (!taken.has(base)) return base;
	// Only a short alphanumeric suffix counts as an extension, so "a.txt (link)" has none.
	const match = /\.[A-Za-z0-9]{1,10}$/.exec(base);
	const dot = match && match.index > 0 ? match.index : base.length;
	const [stem, ext] = [base.slice(0, dot), base.slice(dot)];
	let n = 2;
	while (taken.has(`${stem} ${n}${ext}`)) n++;
	return `${stem} ${n}${ext}`;
}

/** The on-disk name, with extension; `File.name` omits the extension. */
function diskName(file: File): string {
	const path = physicalPath(file);
	return path ? path.slice(path.lastIndexOf('/') + 1) : file.name;
}

/**
 * Creates empty files, copies of templates and symbolic links in the
 * operational directory. The new entry is selected and, for files, renamed
 * in place once the listing shows it.
 */
export function useCreateEntry() {
	const {t} = useTranslation('explorer');
	const {operationalPath, currentFiles} = useExplorer();
	const createFile = useLibraryMutation('files.createFile');
	const createFromTemplate = useLibraryMutation('files.createFromTemplate');
	const createSymlink = useLibraryMutation('files.createSymlink');
	const refetchListings = useRefetchFileListings();
	const undo = useUndo();

	const newFile = useCallback(async () => {
		if (!operationalPath) return;
		const name = nextFreeName(
		t('create.fileBaseName'),
			currentFiles.map(diskName)
		);
		try {
			const {path, undo_id} = await createFile.mutateAsync({parent: operationalPath, name});
			if ('Physical' in path) requestReveal(path.Physical.path, {rename: true});
			undo.recordJournaled(t('undo.newFile'), [undo_id]);
			refetchListings();
		} catch (error) {
			toast.error(t('errors.failedToCreateFile', {error}));
		}
	}, [operationalPath, currentFiles, createFile, undo, refetchListings, t]);

	const newFromTemplate = useCallback(
		async (template: TemplateFile) => {
			if (!operationalPath) return;
			const name = nextFreeName(template.name, currentFiles.map(diskName));
			try {
				const {path, undo_id} = await createFromTemplate.mutateAsync({
					template: template.path,
					parent: operationalPath,
					name
				});
				if ('Physical' in path) requestReveal(path.Physical.path, {rename: true});
				// Undo removes the copy only while it still matches what was created.
				undo.recordJournaled(t('undo.newFile'), [undo_id]);
				refetchListings();
			} catch (error) {
				toast.error(t('errors.failedToCreateFile', {error}));
			}
		},
		[operationalPath, currentFiles, createFromTemplate, refetchListings, undo, t]
	);

	const newLink = useCallback(
		async (target: File) => {
			if (!operationalPath) return;
			const name = nextFreeName(
				`${diskName(target)} (link)`,
				currentFiles.map(diskName)
			);
			try {
				const {path, undo_id} = await createSymlink.mutateAsync({
					target: target.wing_path,
					parent: operationalPath,
					name
				});
				if ('Physical' in path) requestReveal(path.Physical.path);
				undo.recordJournaled(t('undo.newLink'), [undo_id]);
				refetchListings();
			} catch (error) {
			toast.error(t('errors.failedToCreateLink', {error}));
			}
		},
	[operationalPath, currentFiles, createSymlink, undo, refetchListings, t]
	);

	return {newFile, newFromTemplate, newLink};
}
