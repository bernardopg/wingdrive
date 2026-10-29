import type {Icon} from '@phosphor-icons/react';
import {toast} from '@wingdrive/primitives';
import {useQuery} from '@tanstack/react-query';

import {usePlatform, type ContextAction} from '../../../contexts/PlatformContext';
import type {ContextMenuItem} from '../../../hooks/useContextMenu';
import {useTranslation} from '../../../i18n';

/**
 * Context menu submenus for the user's service menu actions.
 *
 * Each menu file's `X-KDE-Submenu` becomes its own submenu, as in Dolphin;
 * actions without one share a single submenu named `fallbackLabel`. Groups
 * keep the order the actions came in.
 */
export function contextActionMenus(
	actions: ContextAction[],
	fallbackLabel: string,
	icon: Icon,
	run: (action: ContextAction) => void
): ContextMenuItem[] {
	const groups = new Map<string, ContextAction[]>();
	for (const action of actions) {
		const label = action.submenu ?? fallbackLabel;
		groups.set(label, [...(groups.get(label) ?? []), action]);
	}
	return [...groups].map(([label, members]) => ({
		type: 'submenu' as const,
		icon,
		label,
		submenu: members.map((action) => ({
			label: action.name,
			iconUrl: action.icon ?? undefined,
			onClick: () => run(action)
		}))
	}));
}

/** The service menu actions that apply to `paths`, and a way to run one. */
export function useContextActions(paths: string[]) {
	const {t} = useTranslation('explorer');
	const platform = usePlatform();
	const {data} = useQuery({
		queryKey: ['contextActions', ...paths],
		queryFn: () => platform.getContextActions!(paths),
		enabled: paths.length > 0 && !!platform.getContextActions,
		staleTime: 10_000
	});

	const run = (action: ContextAction) => {
		platform
			.runContextAction?.(action.id, paths)
			.catch((error) => toast.error(t('errors.contextActionFailed', {name: action.name, error})));
	};

	return {actions: data ?? [], run};
}
