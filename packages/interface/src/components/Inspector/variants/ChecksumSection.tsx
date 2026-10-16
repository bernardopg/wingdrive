import {Fingerprint} from '@phosphor-icons/react';
import {toast} from '@wingdrive/primitives';
import type {WingPath} from '@wingdrive/ts-client';
import {useState} from 'react';

import {useLibraryQuery} from '../../../contexts/WingDriveContext';
import {Section} from '../Inspector';
import {compareSha256} from './checksum';
import {useTranslation} from '../../../i18n';

/** SHA-256 of a local file, calculated only on request, with a check against a pasted hash. */
export function ChecksumSection({path}: {path: WingPath}) {
	const {t} = useTranslation('inspector');
	const [requested, setRequested] = useState(false);
	const [expected, setExpected] = useState('');
	const query = useLibraryQuery(
		{type: 'files.checksum', input: {path}},
		{enabled: requested, retry: false, staleTime: Infinity}
	);

	const digest = query.data?.sha256;
	const comparison = digest && expected.trim() ? compareSha256(digest, expected) : null;

	const copy = async () => {
		if (!digest) return;
		try {
			await navigator.clipboard.writeText(digest);
			toast.success(t('inspector.checksum.copied'));
		} catch (error) {
			toast.error(t('inspector.checksum.copyFailed', {error: String(error)}));
		}
	};

	return (
		<Section title={t('inspector.checksum.title')} icon={Fingerprint}>
			{!requested ? (
				<button
					type="button"
					className="bg-app-box border-app-line text-ink hover:bg-app-hover w-full rounded-md border px-2 py-1 text-xs transition-colors"
					onClick={() => setRequested(true)}
				>
					{t('inspector.checksum.calculate')}
				</button>
			) : query.isError ? (
				<p className="text-status-error text-xs">
					{t('inspector.checksum.failed', {error: String(query.error)})}
				</p>
			) : !digest ? (
				<p className="text-ink-dull text-xs">{t('inspector.checksum.calculating')}</p>
			) : (
				<>
					<div className="flex items-start gap-2">
						<code className="text-sidebar-ink min-w-0 flex-1 break-all font-mono text-[11px]" aria-label="SHA-256">
							{digest}
						</code>
						<button
							type="button"
							className="text-accent shrink-0 text-xs hover:underline"
							onClick={() => void copy()}
						>
							{t('inspector.checksum.copy')}
						</button>
					</div>
					<input
						className="bg-app-box border-app-line text-ink w-full rounded border px-1.5 py-1 font-mono text-[11px]"
						placeholder={t('inspector.checksum.comparePlaceholder')}
						aria-label={t('inspector.checksum.comparePlaceholder')}
						value={expected}
						onChange={(e) => setExpected(e.target.value)}
					/>
					{comparison && (
						<p
							role="status"
							className={
								comparison === 'match'
									? 'text-status-success text-xs'
									: comparison === 'mismatch'
										? 'text-status-error text-xs'
										: 'text-ink-dull text-xs'
							}
						>
							{t(`inspector.checksum.${comparison}`)}
						</p>
					)}
				</>
			)}
		</Section>
	);
}
