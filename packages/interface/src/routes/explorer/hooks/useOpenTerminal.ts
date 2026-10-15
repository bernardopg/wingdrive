import {toast} from '@wingdrive/primitives';
import type {WingPath} from '@wingdrive/ts-client';

import {usePlatform} from '../../../contexts/PlatformContext';
import {useTranslation} from '../../../i18n';

/** Opens the user's terminal in a local folder; undefined when unsupported. */
export function useOpenTerminal() {
	const {t} = useTranslation('explorer');
	const platform = usePlatform();
	if (!platform.openTerminal) return undefined;
	return async (path: WingPath | string | null | undefined) => {
		const directory =
			typeof path === 'string'
				? path
				: path && 'Physical' in path
					? path.Physical.path
					: undefined;
		if (!directory) return;
		try {
			await platform.openTerminal!(directory);
		} catch (error) {
			toast.error(t('errors.couldNotOpenTerminal', {error}));
		}
	};
}
