import {toast} from '@wingdrive/primitives';
import type {File, IntegrationAccountInfo} from '@wingdrive/ts-client';
import {useCallback} from 'react';

import {useLibraryMutation, useLibraryQuery} from '../../../contexts/WingDriveContext';
import {useWaitForJob} from '../../../hooks/useWaitForJob';
import {useTranslation} from '../../../i18n';

/** Uploads of large videos can run for a long time. */
const UPLOAD_TIMEOUT_MS = 6 * 60 * 60 * 1000;

/** Extensions Google Photos accepts; the daemon applies the same list. */
const PHOTOS_EXTENSIONS = new Set([
	'jpg', 'jpeg', 'png', 'gif', 'webp', 'heic', 'heif', 'avif', 'bmp', 'ico', 'tif', 'tiff',
	'raw', 'dng', 'cr2', 'cr3', 'nef', 'arw', 'orf', 'rw2', 'raf', 'srw',
	'mp4', 'mov', 'm4v', '3gp', '3g2', 'avi', 'mkv', 'mpg', 'mpeg', 'mts', 'm2ts', 'wmv', 'asf',
	'divx', 'mmv', 'mod', 'tod', 'webm'
]);

/** Whether Google Photos accepts a file with this name. */
export function isPhotosMedia(name: string): boolean {
	const dot = name.lastIndexOf('.');
	return dot > 0 && PHOTOS_EXTENSIONS.has(name.slice(dot + 1).toLowerCase());
}

/** Whether every file can go to Google Photos: local photos, videos or folders of them. */
export function canUploadToPhotos(files: File[]): boolean {
	return (
		files.length > 0 &&
		files.every(
			(f) =>
				'Physical' in f.wing_path &&
				f.is_local &&
				!f.archive &&
				(f.kind === 'Directory' || isPhotosMedia(f.name))
		)
	);
}

/** Connected accounts that granted Google Photos, and uploading files to one. */
export function useGooglePhotosUpload() {
	const {t} = useTranslation('explorer');
	const upload = useLibraryMutation('integrations.google.photos.upload');
	const waitForJob = useWaitForJob();
	const {data} = useLibraryQuery({type: 'integrations.accounts.list', input: {}});
	const accounts: IntegrationAccountInfo[] = (data ?? []).filter((a) => a.services.includes('photos'));

	const uploadTo = useCallback(
		async (account: IntegrationAccountInfo, files: File[]) => {
			toast.info(t('googlePhotos.uploading', {count: files.length, email: account.email}));
			try {
				const {result} = await waitForJob(
					() =>
						upload.mutateAsync({
							account_id: account.id,
							paths: files.map((f) => f.wing_path),
							album_id: null,
							new_album_title: null
						}),
					UPLOAD_TIMEOUT_MS
				);
				if (result.status === 'failed') {
					toast.error(t('googlePhotos.failed', {error: String(result.error)}));
				} else if (result.status === 'completed') {
					toast.success(t('googlePhotos.uploaded', {email: account.email}));
				}
			} catch (error) {
				toast.error(t('googlePhotos.failed', {error: error instanceof Error ? error.message : String(error)}));
			}
		},
		[t, upload, waitForJob]
	);

	return {accounts, uploadTo};
}
