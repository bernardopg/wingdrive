import {GearSix, Plus} from '@phosphor-icons/react';
import type {Space} from '@wingdrive/ts-client';
import {DropdownMenu, SelectPill} from '@wingdrive/primitives';
import clsx from 'clsx';
import { useTranslation } from '../../i18n';
import {useCreateSpaceDialog} from './CreateSpaceModal';

interface SpaceSwitcherProps {
	spaces: Space[] | undefined;
	currentSpace: Space | undefined;
	onSwitch: (spaceId: string) => void;
	onOpenSettings: () => void;
}

// The core creates the default space with this English name; show it translated.
const DEFAULT_SPACE_NAME = 'All Devices';

export function SpaceSwitcher({
	spaces,
	currentSpace,
	onSwitch,
	onOpenSettings
}: SpaceSwitcherProps) {
	const { t } = useTranslation('navigation');
	const spaceName = (space: Space) =>
		space.name === DEFAULT_SPACE_NAME ? t('groups.allDevices') : space.name;
	const createSpaceDialog = useCreateSpaceDialog;

	return (
		<DropdownMenu.Root>
			<DropdownMenu.Trigger asChild>
				<SelectPill variant="sidebar" size="lg" className="shrink-0">
					<div
						className="size-2 rounded-full"
						style={{backgroundColor: currentSpace?.color || '#666'}}
					/>
					<span className="flex-1 truncate text-left">
						{currentSpace ? spaceName(currentSpace) : t('space.select')}
					</span>
				</SelectPill>
			</DropdownMenu.Trigger>
			<DropdownMenu.Content className="min-w-[var(--radix-dropdown-menu-trigger-width)] p-1">
				{spaces && spaces.length > 1
					? spaces.map((space) => (
							<DropdownMenu.Item
								key={space.id}
								onClick={() => onSwitch(space.id)}
								className={clsx(
									'rounded-md px-2 py-1 text-sm',
									space.id === currentSpace?.id
										? 'bg-accent text-white'
										: 'text-sidebar-ink hover:bg-sidebar-selected'
								)}
							>
								<div className="flex items-center gap-2">
									<div
										className="size-2 rounded-full"
										style={{backgroundColor: space.color}}
									/>
									<span>{spaceName(space)}</span>
								</div>
							</DropdownMenu.Item>
						))
					: null}
				{spaces && spaces.length > 1 && (
					<DropdownMenu.Separator className="border-sidebar-line my-1" />
				)}
				<DropdownMenu.Item
					onClick={() => createSpaceDialog()}
					className="hover:bg-sidebar-selected text-sidebar-ink rounded-md px-2 py-1 text-sm font-medium"
				>
					<Plus className="mr-2 size-4" weight="bold" />
					{t('space.new')}
				</DropdownMenu.Item>
				<DropdownMenu.Item
					onClick={onOpenSettings}
					className="hover:bg-sidebar-selected text-sidebar-ink rounded-md px-2 py-1 text-sm font-medium"
				>
					<GearSix className="mr-2 size-4" weight="bold" />
					{t('space.settings')}
				</DropdownMenu.Item>
			</DropdownMenu.Content>
		</DropdownMenu.Root>
	);
}
