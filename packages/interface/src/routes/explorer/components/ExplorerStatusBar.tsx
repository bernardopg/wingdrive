import type {Volume, WingPath} from '@wingdrive/ts-client';
import {useMemo} from 'react';
import {useNormalizedQuery} from '../../../contexts/WingDriveContext';
import {useTranslation} from '../../../i18n';
import {useExplorer} from '../context';
import {useSelection} from '../SelectionContext';
import {formatBytes} from '../utils';
import {summarizeFiles, volumeForPath} from './statusBar';

interface ExplorerStatusBarProps {
	/** Folder whose volume reports free space; null in views without one. */
	path: WingPath | null;
}

/** Item counts, the selection and the drive's free space, under the view. */
export function ExplorerStatusBar({path}: ExplorerStatusBarProps) {
	const {t} = useTranslation('explorer');
	const {currentFiles, devices, unfilteredFileCount} = useExplorer();
	const {selectedFiles} = useSelection();

	const physical = path && 'Physical' in path ? path.Physical : null;
	const deviceId = useMemo(() => {
		if (!physical) return null;
		for (const device of devices.values()) {
			if (device.slug === physical.device_slug) return device.id;
		}
		return null;
	}, [devices, physical]);

	const {data: volumesData} = useNormalizedQuery({
		query: 'volumes.list',
		input: {filter: 'All'},
		resourceType: 'volume',
		enabled: !!deviceId
	});
	const volume = useMemo(() => {
		const volumes = (volumesData as {volumes: Volume[]} | undefined)?.volumes;
		if (!volumes || !deviceId || !physical) return null;
		return volumeForPath(volumes, deviceId, physical.path);
	}, [volumesData, deviceId, physical]);

	const listing = useMemo(() => summarizeFiles(currentFiles), [currentFiles]);
	const selection = useMemo(() => summarizeFiles(selectedFiles), [selectedFiles]);

	const parts: string[] = [];
	if (selectedFiles.length > 0) {
		parts.push(t('statusBar.selected', {count: selectedFiles.length}));
		if (selection.bytes > 0) parts.push(formatBytes(selection.bytes));
	} else if (unfilteredFileCount !== null) {
		parts.push(
			t('statusBar.filtered', {shown: currentFiles.length, total: unfilteredFileCount})
		);
	} else {
		if (listing.folders > 0) parts.push(t('statusBar.folders', {count: listing.folders}));
		if (listing.files > 0 || listing.folders === 0) {
			parts.push(t('statusBar.files', {count: listing.files}));
		}
	}

	return (
		<div
			role="status"
			className="flex h-6 shrink-0 items-center justify-between gap-4 border-t border-app-line/50 px-3 text-xs tabular-nums text-ink-dull"
		>
			<span className="truncate">{parts.join(' · ')}</span>
			{volume && (
				<span className="shrink-0" title={volume.name}>
					{t('statusBar.free', {size: formatBytes(volume.available_space)})}
				</span>
			)}
		</div>
	);
}
