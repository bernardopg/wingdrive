import {toast} from '@wingdrive/primitives';
import type {WingPath} from '@wingdrive/ts-client';

import {usePlatform} from '../../../contexts/PlatformContext';
import {useWingDriveClient} from '../../../contexts/WingDriveContext';
import {useTranslation} from '../../../i18n';

/** Opens a local folder in a new app window; undefined when unsupported. */
export function useOpenInNewWindow() {
	const {t} = useTranslation('explorer');
	const platform = usePlatform();
	const client = useWingDriveClient();
	if (!platform.showWindow) return undefined;
	return async (path: WingPath | null | undefined) => {
		const libraryId = client.getCurrentLibraryId();
		if (!path || !('Physical' in path) || !libraryId) return;
		try {
			await platform.showWindow!({
				type: 'Explorer',
				library_id: libraryId,
				path: path.Physical.path
			});
		} catch (error) {
			toast.error(t('errors.couldNotOpenWindow', {error}));
		}
	};
}
