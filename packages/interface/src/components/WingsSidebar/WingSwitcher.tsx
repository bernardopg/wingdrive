import {GearSix, Plus} from '@phosphor-icons/react';
import type {Wing} from '@wingdrive/ts-client';
import {DropdownMenu, SelectPill} from '@wingdrive/primitives';
import clsx from 'clsx';
import { useTranslation } from '../../i18n';
import {useCreateWingDialog} from './CreateWingModal';

interface WingSwitcherProps {
	wings: Wing[] | undefined;
	currentWing: Wing | undefined;
	onSwitch: (wingId: string) => void;
	onOpenSettings: () => void;
}

// The core creates the default wing with this English name; show it translated.
const DEFAULT_WING_NAME = 'All Devices';

export function WingSwitcher({
	wings,
	currentWing,
	onSwitch,
	onOpenSettings
}: WingSwitcherProps) {
	const { t } = useTranslation('navigation');
	const wingName = (wing: Wing) =>
		wing.name === DEFAULT_WING_NAME ? t('groups.allDevices') : wing.name;
	const createWingDialog = useCreateWingDialog;

	return (
		<DropdownMenu.Root>
			<DropdownMenu.Trigger asChild>
				<SelectPill variant="sidebar" size="lg" className="shrink-0">
					<div
						className="size-2 rounded-full"
						style={{backgroundColor: currentWing?.color || '#666'}}
					/>
					<span className="flex-1 truncate text-left">
						{currentWing ? wingName(currentWing) : t('wing.select')}
					</span>
				</SelectPill>
			</DropdownMenu.Trigger>
			<DropdownMenu.Content className="min-w-[var(--radix-dropdown-menu-trigger-width)] p-1">
				{wings && wings.length > 1
					? wings.map((wing) => (
							<DropdownMenu.Item
								key={wing.id}
								onClick={() => onSwitch(wing.id)}
								className={clsx(
									'rounded-md px-2 py-1 text-sm',
									wing.id === currentWing?.id
										? 'bg-accent text-white'
										: 'text-sidebar-ink hover:bg-sidebar-selected'
								)}
							>
								<div className="flex items-center gap-2">
									<div
										className="size-2 rounded-full"
										style={{backgroundColor: wing.color}}
									/>
									<span>{wingName(wing)}</span>
								</div>
							</DropdownMenu.Item>
						))
					: null}
				{wings && wings.length > 1 && (
					<DropdownMenu.Separator className="border-sidebar-line my-1" />
				)}
				<DropdownMenu.Item
					onClick={() => createWingDialog()}
					className="hover:bg-sidebar-selected text-sidebar-ink rounded-md px-2 py-1 text-sm font-medium"
				>
					<Plus className="mr-2 size-4" weight="bold" />
					{t('wing.new')}
				</DropdownMenu.Item>
				<DropdownMenu.Item
					onClick={onOpenSettings}
					className="hover:bg-sidebar-selected text-sidebar-ink rounded-md px-2 py-1 text-sm font-medium"
				>
					<GearSix className="mr-2 size-4" weight="bold" />
					{t('wing.settings')}
				</DropdownMenu.Item>
			</DropdownMenu.Content>
		</DropdownMenu.Root>
	);
}
