import {toast} from '@wingdrive/primitives';
import {useCallback} from 'react';

import {usePlatform, type AdminOperation} from '../contexts/PlatformContext';
import {useTranslation} from '../i18n';
import {useRefetchFileListings} from './useRefetchFileListings';

/** Whether an error message says the user lacks the rights for the change. */
export function isPermissionError(message: string): boolean {
	return /permission denied|operation not permitted|os error (1|13)\b|EACCES|EPERM/i.test(message);
}

/**
 * Reports a failed file operation, offering to repeat it as administrator
 * when the failure was a permission error and the platform supports it.
 */
export function useAdminRetry() {
	const {t} = useTranslation('common');
	const platform = usePlatform();
	const refetchListings = useRefetchFileListings();

	const runAsAdmin = useCallback(
		async (operation: AdminOperation) => {
			if (!platform.runAsAdmin) return false;
			try {
				await platform.runAsAdmin(operation);
				refetchListings();
				toast.success(t('admin.done'));
				return true;
			} catch (error) {
				toast.error(t('admin.failed', {error: String(error)}));
				return false;
			}
		},
		[platform, refetchListings, t]
	);

	/** Whether `message` can be answered with an administrator retry. */
	const canRetry = useCallback(
		(message: string) => !!platform.runAsAdmin && isPermissionError(message),
		[platform]
	);

	const report = useCallback(
		(message: string, operation: AdminOperation | null, detail: string = message) => {
			if (operation && canRetry(detail)) {
				toast.error(message, {
					action: {label: t('admin.retry'), onClick: () => void runAsAdmin(operation)}
				});
			} else {
				toast.error(message);
			}
		},
		[canRetry, runAsAdmin, t]
	);

	return {report, canRetry, runAsAdmin};
}
