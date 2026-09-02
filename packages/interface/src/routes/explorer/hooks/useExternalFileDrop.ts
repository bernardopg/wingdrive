import type {SdPath} from '@sd/ts-client';
import {toast} from '@wingdrive/primitives';
import {useEffect, useRef} from 'react';
import {useFileOperationDialog} from '../../../components/modals/FileOperationModal';
import {usePlatform} from '../../../contexts/PlatformContext';
import {
	resolveExternalDrop,
	type ExternalDropRejectionReason
} from './resolveExternalDrop';

const REJECTION_MESSAGES: Record<ExternalDropRejectionReason, string> = {
	'no-files': 'No importable files were dropped.',
	'no-destination': 'Open a physical folder before dropping files.',
	'not-physical':
		'Files from outside WingDrive can only be dropped into a physical folder.'
};

/**
 * Imports files dragged from the operating system into the current Explorer folder.
 *
 * A drop without a physical destination is rejected with an explicit toast
 * instead of being silently swallowed. Valid drops open the file operation
 * dialog so conflicts are resolved before the copy job starts; job progress
 * and per-file failures are then reported through the Job Manager.
 */
export function useExternalFileDrop(destination: SdPath | null) {
	const platform = usePlatform();
	const openFileOperation = useFileOperationDialog();

	const destinationRef = useRef(destination);
	const openFileOperationRef = useRef(openFileOperation);

	useEffect(() => {
		destinationRef.current = destination;
	}, [destination]);

	useEffect(() => {
		openFileOperationRef.current = openFileOperation;
	}, [openFileOperation]);

	useEffect(() => {
		const subscribe = platform.onExternalFileDrop;
		if (!subscribe) return;

		let cancelled = false;
		let unlisten: (() => void) | undefined;
		// `over` events carry no paths, so remember the ones from `enter`
		let hoveredPaths: string[] = [];

		const handleDrop = (paths: string[]) => {
			const decision = resolveExternalDrop(destinationRef.current, paths);
			if (decision.kind === 'reject') {
				toast.warning({
					title: 'Cannot Import Here',
					body: REJECTION_MESSAGES[decision.reason]
				});
				return;
			}
			openFileOperationRef.current({
				operation: 'copy',
				sources: decision.sources,
				destination: decision.destination
			});
		};

		subscribe
			.call(platform, (event) => {
				switch (event.type) {
					case 'enter':
						hoveredPaths = event.paths;
						break;
					case 'leave':
						hoveredPaths = [];
						break;
					case 'drop': {
						const paths = event.paths.length
							? event.paths
							: hoveredPaths;
						hoveredPaths = [];
						handleDrop(paths);
						break;
					}
				}
			})
			.then((fn) => {
				if (cancelled) fn();
				else unlisten = fn;
			})
			.catch((error) => {
				if (cancelled) return;
				console.error(
					'Failed to subscribe to external file drops',
					error
				);
			});

		return () => {
			cancelled = true;
			unlisten?.();
		};
		// Subscribe once per window; the destination is read through a ref so
		// navigation does not resubscribe mid-drag.
		// eslint-disable-next-line react-hooks/exhaustive-deps
	}, []);
}
