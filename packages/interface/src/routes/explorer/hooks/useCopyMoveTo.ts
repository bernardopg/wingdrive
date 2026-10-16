import type {File} from '@wingdrive/ts-client';
import {useCallback} from 'react';

import {useFileOperationDialog} from '../../../components/modals/FileOperationModal';
import {usePlatform} from '../../../contexts/PlatformContext';
import {useTranslation} from '../../../i18n';
import {canPickDestination, pickedDestination} from './copyMoveDestination';

/** "Copy To…" and "Move To…": pick a folder, then run the usual copy or move dialog. */
export function useCopyMoveTo() {
	const {t} = useTranslation('explorer');
	const platform = usePlatform();
	const openFileOperation = useFileOperationDialog();

	const run = useCallback(
		async (files: File[], operation: 'copy' | 'move') => {
			if (!platform.openDirectoryPickerDialog || !canPickDestination(files)) return;
			const picked = await platform.openDirectoryPickerDialog({
				title: t(operation === 'copy' ? 'contextMenu.copyTo' : 'contextMenu.moveTo')
			});
			const destination = pickedDestination(files, picked);
			if (!destination) return;
			openFileOperation({operation, sources: files.map((f) => f.wing_path), destination});
		},
		[platform, openFileOperation, t]
	);

	return {
		copyTo: (files: File[]) => run(files, 'copy'),
		moveTo: (files: File[]) => run(files, 'move'),
		canPick: (files: File[]) => !!platform.openDirectoryPickerDialog && canPickDestination(files)
	};
}
