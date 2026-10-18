import {PaperPlaneTilt} from '@phosphor-icons/react';
import {Dialog, dialogManager, toast, useDialog, type UseDialogProps} from '@wingdrive/primitives';
import type {Event, WingdropOffer} from '@wingdrive/ts-client';
import {useEffect, useRef} from 'react';
import {useForm} from 'react-hook-form';

import {usePlatform} from '../../contexts/PlatformContext';
import {useCoreMutation, useCoreQuery, useWingDriveClient} from '../../contexts/WingDriveContext';
import {useTranslation} from '../../i18n';
import {formatBytes} from '../../routes/explorer/utils';

/** Drops answered or ended elsewhere; their prompt closes without declining. */
const settled = new Set<string>();
/** Closes the open prompt of a drop. */
const closers = new Map<string, () => void>();

/**
 * Shows a prompt for every Wingdrop offer this device receives, and a toast
 * when an accepted drop finishes. Mounted once, next to the app shell.
 */
export function WingdropOffers() {
	const {t} = useTranslation('dialogs');
	const client = useWingDriveClient();
	const platform = usePlatform();
	const pending = useCoreQuery({type: 'network.wingdrop.pending', input: null});
	const shown = useRef(new Set<string>());
	const known = useRef(new Map<string, WingdropOffer>());

	const open = (offer: WingdropOffer) => {
		known.current.set(offer.id, offer);
		if (shown.current.has(offer.id)) return;
		shown.current.add(offer.id);
		void dialogManager.create((props: UseDialogProps) => (
			<WingdropOfferDialog {...props} offer={offer} />
		));
	};

	// Offers that arrived before this window opened.
	useEffect(() => {
		pending.data?.forEach(open);
		// eslint-disable-next-line react-hooks/exhaustive-deps
	}, [pending.data]);

	useEffect(() => {
		if (!client) return;
		let unsubscribe: (() => void) | undefined;
		let cancelled = false;

		const close = (dropId: string) => {
			settled.add(dropId);
			closers.get(dropId)?.();
		};

		const handle = (event: Event) => {
			if (typeof event !== 'object') return;
			if ('WingdropOffered' in event) {
				open(event.WingdropOffered.offer);
				return;
			}
			if (!('WingdropUpdated' in event)) return;
			const {drop_id, status} = event.WingdropUpdated;
			const offer = known.current.get(drop_id);
			const from = offer?.from_device_name ?? '';
			switch (status.status) {
				case 'accepted':
					close(drop_id);
					break;
				case 'declined':
					close(drop_id);
					break;
				case 'expired':
					close(drop_id);
					toast.info(t('wingdrop.expired', {name: from}));
					break;
				case 'cancelled':
					close(drop_id);
					toast.info(t('wingdrop.cancelled', {name: from}));
					break;
				case 'completed': {
					const [first] = status.paths;
					toast.success(
						t('wingdrop.received', {count: offer?.file_count ?? status.paths.length, name: from}),
						first && platform.revealFile
							? {
									action: {
										label: t('wingdrop.showInFolder'),
										onClick: () => void platform.revealFile?.(first)
									}
								}
							: undefined
					);
					break;
				}
			}
		};

		client
			.subscribeFiltered({event_types: ['WingdropOffered', 'WingdropUpdated']}, handle)
			.then((unsub) => {
				if (cancelled) unsub();
				else unsubscribe = unsub;
			});
		return () => {
			cancelled = true;
			unsubscribe?.();
		};
		// eslint-disable-next-line react-hooks/exhaustive-deps
	}, [client]);

	return null;
}

function WingdropOfferDialog(props: UseDialogProps & {offer: WingdropOffer}) {
	const {t} = useTranslation('dialogs');
	const dialog = useDialog(props);
	const form = useForm();
	const respond = useCoreMutation('network.wingdrop.respond');
	const answered = useRef(false);
	const {offer} = props;

	const answer = async (accept: boolean) => {
		if (answered.current || settled.has(offer.id)) return;
		answered.current = true;
		settled.add(offer.id);
		try {
			await respond.mutateAsync({drop_id: offer.id, accept});
		} catch (error) {
			toast.error(t('wingdrop.answerFailed', {error: error instanceof Error ? error.message : String(error)}));
		}
	};

	// Closing the prompt any other way (Escape, clicking outside) declines it.
	useEffect(
		() => {
			closers.set(offer.id, () => dialogManager.setState(props.id, {open: false}));
			return () => {
				closers.delete(offer.id);
				void answer(false);
			};
		},
		// eslint-disable-next-line react-hooks/exhaustive-deps
		[]
	);

	const shownItems = offer.items.slice(0, 5);
	return (
		<Dialog
			dialog={dialog}
			form={form}
			title={t('wingdrop.title', {name: offer.from_device_name})}
			icon={<PaperPlaneTilt size={20} weight="fill" className="text-accent" />}
			ctaLabel={t('wingdrop.accept')}
			closeLabel={t('wingdrop.decline')}
			onSubmit={form.handleSubmit(async () => {
				await answer(true);
				dialogManager.setState(props.id, {open: false});
			})}
			onCancelled={() => void answer(false)}
			formClassName="!min-w-[380px] !max-w-[420px]"
		>
			<div className="flex flex-col gap-3 py-3 text-sm">
				<p className="text-ink">
					{t('wingdrop.summary', {
						count: Number(offer.file_count),
						size: formatBytes(Number(offer.total_bytes))
					})}
				</p>
				<ul className="bg-app-box border-app-line space-y-1 rounded-md border p-2">
					{shownItems.map((item) => (
						<li key={item.name} className="text-ink-dull flex justify-between gap-3 truncate">
							<span className="truncate">
								{item.name}
								{item.is_dir ? '/' : ''}
							</span>
							<span className="shrink-0">{formatBytes(Number(item.size))}</span>
						</li>
					))}
					{offer.items.length > shownItems.length && (
						<li className="text-ink-faint">
							{t('wingdrop.more', {count: offer.items.length - shownItems.length})}
						</li>
					)}
				</ul>
			</div>
		</Dialog>
	);
}
