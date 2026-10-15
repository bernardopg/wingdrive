import {LockKey} from '@phosphor-icons/react';
import {toast} from '@wingdrive/primitives';
import type {WingPath} from '@wingdrive/ts-client';
import {useState} from 'react';

import {useLibraryMutation, useLibraryQuery} from '../../../contexts/WingDriveContext';
import {InfoRow, Section} from '../Inspector';
import {formatMode, parseMode} from './permissionMode';
import {useTranslation} from '../../../i18n';

const CLASSES = [
	{label: 'owner', shift: 6},
	{label: 'group', shift: 3},
	{label: 'others', shift: 0}
] as const;

const BITS = [
	{label: 'read', bit: 4},
	{label: 'write', bit: 2},
	{label: 'execute', bit: 1}
] as const;

/** Unix mode bits and group for a local file or folder. */
export function PermissionsSection({path}: {path: WingPath}) {
const {t} = useTranslation('inspector');
	const isLocal = 'Physical' in path;
	const query = useLibraryQuery(
		{type: 'files.permissions', input: {path}},
		{enabled: isLocal, retry: false}
	);
	const setPermissions = useLibraryMutation('files.setPermissions');
	const setGroup = useLibraryMutation('files.setGroup');
	// Text being typed in the octal field; null shows the current mode.
	const [octalDraft, setOctalDraft] = useState<string | null>(null);
	const [recursive, setRecursive] = useState(false);

	const data = query.data;

	if (!isLocal || query.isError || !data) return null;
	const editable = data.is_owner;

	const applyMode = async (mode: number) => {
		try {
			const result = await setPermissions.mutateAsync({path, mode, recursive});
			if (result.failed.length > 0) {
				toast.error(
					t('permissions.partialFailure', {changed: result.changed, failed: result.failed.length, error: result.failed[0]})
				);
			}
		} catch (error) {
			toast.error(t('permissions.changeFailed', {error: String(error)}));
		}
		await query.refetch();
	};

	const applyGroup = async (group: string) => {
		try {
			await setGroup.mutateAsync({path, group});
		} catch (error) {
			toast.error(t('permissions.groupFailed', {error: String(error)}));
		}
		await query.refetch();
	};

	return (
<Section title={t('inspector.permissions.title')} icon={LockKey}>
<InfoRow label={t('inspector.permissions.owner')} value={data.owner} />
			<div className="flex items-center justify-between gap-2 py-0.5">
<span className="text-ink-dull text-xs">{t('inspector.permissions.group')}</span>
				{editable && data.available_groups.length > 1 ? (
					<select
						className="bg-app-box border-app-line text-ink rounded border px-1 py-0.5 text-xs"
						value={data.group}
						onChange={(e) => void applyGroup(e.target.value)}
aria-label={t('inspector.permissions.group')}
					>
						{[...new Set([data.group, ...data.available_groups])].map((group) => (
							<option key={group} value={group}>
								{group}
							</option>
						))}
					</select>
				) : (
					<span className="text-ink text-xs">{data.group}</span>
				)}
			</div>

			<table className="mt-2 w-full text-xs">
				<thead>
					<tr className="text-ink-dull">
						<th className="text-left font-normal" />
						{BITS.map(({label}) => (
							<th key={label} className="font-normal">
								{t(`inspector.permissions.${label}`)}
							</th>
						))}
					</tr>
				</thead>
				<tbody>
					{CLASSES.map(({label, shift}) => (
						<tr key={label}>
							<td className="text-ink-dull py-0.5">{t(`inspector.permissions.${label}`)}</td>
							{BITS.map(({label: bitLabel, bit}) => {
								const mask = bit << shift;
								return (
									<td key={bitLabel} className="text-center">
										<input
											type="checkbox"
											className="accent-accent"
											aria-label={`${t(`inspector.permissions.${label}`)} ${t(`inspector.permissions.${bitLabel}`)}`}
											checked={(data.mode & mask) !== 0}
											disabled={!editable || setPermissions.isPending}
											onChange={(e) =>
												void applyMode(
													e.target.checked ? data.mode | mask : data.mode & ~mask
												)
											}
										/>
									</td>
								);
							})}
						</tr>
					))}
				</tbody>
			</table>

			<div className="mt-2 flex items-center justify-between gap-2">
<span className="text-ink-dull text-xs">{t('inspector.permissions.octal')}</span>
				<input
					className="bg-app-box border-app-line text-ink w-16 rounded border px-1 py-0.5 text-right font-mono text-xs"
					value={octalDraft ?? formatMode(data.mode)}
					disabled={!editable}
aria-label={t('inspector.permissions.octalMode')}
					onChange={(e) => setOctalDraft(e.target.value)}
					onKeyDown={(e) => {
						if (e.key === 'Enter') (e.target as HTMLInputElement).blur();
					}}
					onBlur={() => {
						const mode = octalDraft === null ? null : parseMode(octalDraft);
						setOctalDraft(null);
						if (mode !== null && mode !== data.mode) void applyMode(mode);
					}}
				/>
			</div>

			{data.is_dir && editable && (
				<label className="text-ink-dull mt-2 flex items-center gap-2 text-xs">
					<input
						type="checkbox"
						className="accent-accent"
						checked={recursive}
						onChange={(e) => setRecursive(e.target.checked)}
					/>
					{t("inspector.permissions.applyRecursive")}
				</label>
			)}
			{!editable && (
				<p className="text-ink-faint mt-2 text-xs">{t('inspector.permissions.onlyOwnerCanChange')}</p>
			)}
		</Section>
	);
}
