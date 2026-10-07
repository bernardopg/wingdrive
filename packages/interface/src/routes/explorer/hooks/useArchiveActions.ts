import {toast} from '@wingdrive/primitives';
import type {ArchiveFormat, File, WingPath} from '@wingdrive/ts-client';
import {useCallback} from 'react';

import {usePlatform} from '../../../contexts/PlatformContext';
import {useLibraryMutation} from '../../../contexts/WingDriveContext';
import {useRefetchFileListings} from '../../../hooks/useRefetchFileListings';
import {useWaitForJob} from '../../../hooks/useWaitForJob';
import {physicalPath} from '../fileCapabilities';
import {archiveBaseName} from './archiveName';

// Archives can take minutes; the toast still reports the outcome later.
const ARCHIVE_TIMEOUT_MS = 60 * 60 * 1000;

function parentOf(file: File): WingPath | null {
	if (!('Physical' in file.wing_path)) return null;
	const {device_slug, path} = file.wing_path.Physical;
	const slash = path.lastIndexOf('/');
	return {Physical: {device_slug, path: slash > 0 ? path.slice(0, slash) : '/'}};
}

function diskName(file: File): string {
	const path = physicalPath(file);
	return path ? path.slice(path.lastIndexOf('/') + 1) : file.name;
}

/** Compress and extract through archive jobs, refreshing listings when they finish. */
export function useArchiveActions() {
	const compressMutation = useLibraryMutation('archive.compress');
	const extractMutation = useLibraryMutation('archive.extract');
	const waitForJob = useWaitForJob();
	const refetchListings = useRefetchFileListings();
	const platform = usePlatform();

	const run = useCallback(
		async (label: string, dispatch: () => Promise<{id: string}>) => {
			// Progress shows in the job manager; the toasts bracket the run.
			toast.info(`${label}...`);
			try {
				const {result} = await waitForJob(dispatch, ARCHIVE_TIMEOUT_MS);
				if (result.status === 'failed') toast.error(`${label} failed: ${result.error}`);
				else if (result.status === 'cancelled') toast.info(`${label} cancelled`);
				else toast.success(`${label} finished`);
			} catch (error) {
				toast.error(`${label} failed: ${error}`);
			}
			refetchListings();
		},
		[waitForJob, refetchListings]
	);

	const compress = useCallback(
		(files: File[], format: ArchiveFormat) => {
			const destination = files[0] && parentOf(files[0]);
			if (!destination) return;
			void run('Compressing', () =>
				compressMutation.mutateAsync({
					sources: files.map((f) => f.wing_path),
					destination,
					name: archiveBaseName(files.map(diskName)),
					format
				})
			);
		},
		[compressMutation, run]
	);

	const extract = useCallback(
		(file: File, destination?: WingPath) => {
			const target = destination ?? parentOf(file);
			if (!target) return;
			void run('Extracting', () =>
				extractMutation.mutateAsync({archive: file.wing_path, destination: target})
			);
		},
		[extractMutation, run]
	);

	const extractTo = useCallback(
		async (file: File) => {
			if (!platform.openDirectoryPickerDialog || !('Physical' in file.wing_path)) return;
			const picked = await platform.openDirectoryPickerDialog({title: 'Extract to'});
			const directory = Array.isArray(picked) ? picked[0] : picked;
			if (!directory) return;
			extract(file, {
				Physical: {device_slug: file.wing_path.Physical.device_slug, path: directory}
			});
		},
		[platform, extract]
	);

	return {compress, extract, extractTo, canPickDirectory: !!platform.openDirectoryPickerDialog};
}
