import {toast} from '@wingdrive/primitives';
import type {Device, File, ListLibraryDevicesInput, WingPath} from '@wingdrive/ts-client';
import {useCallback} from 'react';

import {useLibraryMutation, useNormalizedQuery} from '../../../contexts/WingDriveContext';
import {useWaitForJob} from '../../../hooks/useWaitForJob';
import {useTranslation} from '../../../i18n';

// The receiver has five minutes to answer, then the transfer itself runs.
const WINGDROP_TIMEOUT_MS = 60 * 60 * 1000;

export type WingdropOutcome =
	| {status: 'completed'}
	| {status: 'cancelled'}
	| {status: 'failed'; error: string};

/** Paired devices, and sending files to one of them with Wingdrop. */
export function useWingdrop() {
	const {t} = useTranslation('explorer');
	const send = useLibraryMutation('network.wingdrop.send');
	const waitForJob = useWaitForJob();
	const {data: devices} = useNormalizedQuery<ListLibraryDevicesInput, Device[]>({
		query: 'devices.list',
		input: {include_offline: true, include_details: false, show_paired: true},
		resourceType: 'device'
	});
	const paired = (devices ?? []).filter((d) => !d.is_current && d.is_paired);
	const targets = paired.filter((d) => d.is_connected);

	/** Offers `paths` to `device` and resolves once the drop ends. */
	const sendPaths = useCallback(
		async (device: Device, paths: WingPath[]): Promise<WingdropOutcome> => {
			try {
				const {result} = await waitForJob(
					() => send.mutateAsync({device_id: device.id, paths}),
					WINGDROP_TIMEOUT_MS
				);
				if (result.status === 'failed') return {status: 'failed', error: String(result.error)};
				if (result.status === 'cancelled') return {status: 'cancelled'};
				return {status: 'completed'};
			} catch (error) {
				return {status: 'failed', error: error instanceof Error ? error.message : String(error)};
			}
		},
		[send, waitForJob]
	);

	/** Sends files from the explorer, reporting progress with toasts. */
	const sendTo = useCallback(
		async (device: Device, files: File[]) => {
			toast.info(t('wingdrop.waiting', {name: device.name}));
			const outcome = await sendPaths(
				device,
				files.map((f) => f.wing_path)
			);
			if (outcome.status === 'failed') toast.error(t('wingdrop.failed', {error: outcome.error}));
			else if (outcome.status === 'cancelled') toast.info(t('wingdrop.sendCancelled'));
			else toast.success(t('wingdrop.sent', {count: files.length, name: device.name}));
		},
		[sendPaths, t]
	);

	return {paired, targets, sendPaths, sendTo};
}
