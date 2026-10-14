import { useState } from 'react';
import { useTranslation } from '../../i18n';
import { Input, Label, dialogManager, useDialog, Dialog } from '@wingdrive/primitives';
import { useLibraryMutation } from '@wingdrive/ts-client';
import { useForm } from 'react-hook-form';
import type { GroupType } from '@wingdrive/ts-client';

interface FormData {
	groupName: string;
}

export function useAddGroupDialog(spaceId: string) {
	return dialogManager.create((props) => <AddGroupDialog {...props} spaceId={spaceId} />);
}

function AddGroupDialog(props: { id: number; spaceId: string }) {
	const { t } = useTranslation('navigation');
	const dialog = useDialog(props);
	const [groupType, setGroupType] = useState<GroupType>('Custom');

	const form = useForm<FormData>({
		defaultValues: { groupName: '' },
	});

	const addGroup = useLibraryMutation('spaces.add_group');

	const onSubmit = form.handleSubmit(async (data) => {
		await addGroup.mutateAsync({
			space_id: props.spaceId,
			name: data.groupName || getDefaultName(groupType, t),
			group_type: groupType,
		});
		form.reset();
		setGroupType('Custom');
		dialog.state.open = false;
	});

	return (
		<Dialog form={form} dialog={dialog} title={t('groups.add')} onSubmit={onSubmit} ctaLabel={t('common.create')}>
			<div className="space-y-4">
				<div>
					<Label>{t('groups.type')}</Label>
					<select
						value={typeof groupType === 'string' ? groupType : 'Custom'}
						onChange={(e) => setGroupType(e.target.value as GroupType)}
						className="w-full rounded-lg border border-app-line bg-app-input px-3 py-2 text-sm text-ink"
					>
						<option value="Devices">{t('groups.allDevices')}</option>
						<option value="Locations">{t('groups.allLocations')}</option>
						<option value="Tags">{t('groups.tags')}</option>
						<option value="Cloud">{t('groups.cloudStorage')}</option>
						<option value="Custom">{t('groups.custom')}</option>
					</select>
				</div>

				{groupType === 'Custom' && (
					<div>
						<Label>{t('groups.name')}</Label>
						<Input
							{...form.register('groupName')}
							placeholder={t('groups.namePlaceholder')}
						/>
					</div>
				)}
			</div>
		</Dialog>
	);
}

function getDefaultName(groupType: GroupType, t: (key: string) => string): string {
	if (groupType === 'Devices') return t('groups.devices');
	if (groupType === 'Locations') return t('groups.locations');
	if (groupType === 'Tags') return t('groups.tags');
	if (groupType === 'Cloud') return t('groups.cloud');
	if (groupType === 'Custom') return t('groups.customGroup');
	if (typeof groupType === 'object' && 'Device' in groupType) return t('groups.device');
	return t('groups.group');
}

