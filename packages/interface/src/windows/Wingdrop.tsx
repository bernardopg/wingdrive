import {motion} from 'framer-motion';
import {CheckCircle, PaperPlaneTilt, Spinner, Users, WarningCircle, X} from '@phosphor-icons/react';
import {useMemo, useState} from 'react';
import {CircleButton} from '@wingdrive/primitives';
import type {Device} from '@wingdrive/ts-client';
import {usePlatform} from '../contexts/PlatformContext';
import {useWingdrop} from '../routes/explorer/hooks/useWingdrop';
import {useTranslation} from '../i18n';

interface WingdropProps {
	onClose?: () => void;
}

type SendState =
	| {status: 'sending'}
	| {status: 'completed'}
	| {status: 'cancelled'}
	| {status: 'failed'; error: string};

function initials(name: string) {
	return (
		name
			.split(/[\s-]+/)
			.filter(Boolean)
			.slice(0, 2)
			.map((part) => part[0]?.toUpperCase())
			.join('') || '?'
	);
}

/**
 * Wingdrop window: the paired devices, online first. Choosing an online
 * device opens a file picker and offers the chosen files to it.
 */
export function Wingdrop({onClose}: WingdropProps) {
	const {t} = useTranslation('dialogs');
	const platform = usePlatform();
	const {paired, sendPaths} = useWingdrop();
	const [states, setStates] = useState<Record<string, SendState>>({});

	const devices = useMemo(
		() => [...paired].sort((a, b) => Number(b.is_connected) - Number(a.is_connected)),
		[paired]
	);

	// Generate star positions
	const stars = useMemo(
		() =>
			Array.from({length: 50}, (_, i) => ({
				id: i,
				x: Math.random() * 100,
				y: Math.random() * 100,
				size: Math.random() * 2 + 1,
				duration: Math.random() * 3 + 2
			})),
		[]
	);
	const sending = Object.values(states).some((s) => s.status === 'sending');

	const choose = async (device: Device) => {
		if (!device.is_connected || !platform.openFilePickerDialog) return;
		if (states[device.id]?.status === 'sending') return;
		const picked = await platform.openFilePickerDialog({
			title: t('wingdrop.chooseFiles', {name: device.name}),
			multiple: true
		});
		const files = (Array.isArray(picked) ? picked : picked ? [picked] : []).filter(Boolean);
		if (files.length === 0) return;
		setStates((s) => ({...s, [device.id]: {status: 'sending'}}));
		const outcome = await sendPaths(
			device,
			// The daemon resolves the `local` slug to this device.
			files.map((path) => ({Physical: {device_slug: 'local', path}}))
		);
		setStates((s) => ({...s, [device.id]: outcome}));
	};

	const caption = (device: Device) => {
		const state = states[device.id];
		if (state?.status === 'sending') return t('wingdrop.sending');
		if (state?.status === 'completed') return t('wingdrop.delivered');
		if (state?.status === 'cancelled') return t('wingdrop.notDelivered');
		if (state?.status === 'failed') return state.error;
		return t(device.is_connected ? 'wingdrop.online' : 'wingdrop.offline');
	};

	return (
		<div className="border-app-line relative flex h-full w-full flex-col overflow-hidden rounded-2xl border bg-black">
			{/* Animated Stars Background */}
			<div className="absolute inset-0">
				{stars.map((star) => (
					<motion.div
						key={star.id}
						className="absolute rounded-full bg-white"
						style={{
							left: `${star.x}%`,
							top: `${star.y}%`,
							width: star.size,
							height: star.size
						}}
						animate={{opacity: [0.2, 1, 0.2], scale: [1, 1.2, 1]}}
						transition={{
							duration: star.duration / (sending ? 3 : 1),
							repeat: Infinity,
							ease: 'easeInOut'
						}}
					/>
				))}
			</div>

			{/* Top Bar */}
			<div className="border-app-line/30 bg-app/80 relative z-10 flex items-center justify-between border-b p-3 backdrop-blur-xl">
				<CircleButton icon={X} onClick={onClose}>
					{t('common.close')}
				</CircleButton>
				<CircleButton icon={Users}>
					{t('wingdrop.deviceCount', {count: devices.length})}
				</CircleButton>
			</div>

			{/* Content */}
			<div className="relative z-10 flex flex-1 items-center justify-center p-8">
				{devices.length === 0 ? (
					<motion.div initial={{opacity: 0, y: 20}} animate={{opacity: 1, y: 0}} className="text-center">
						<div className="mb-4 flex justify-center">
							<div className="bg-sidebar-box/40 rounded-full p-6 backdrop-blur-lg">
								<Users className="text-sidebar-ink-faint size-12" />
							</div>
						</div>
						<h2 className="text-sidebar-ink mb-2 text-lg font-semibold">{t('wingdrop.noDevices')}</h2>
						<p className="text-sidebar-ink-faint max-w-xs text-sm">{t('wingdrop.pairFirst')}</p>
					</motion.div>
				) : (
					<div className="flex flex-col items-center gap-6">
						<p className="text-sidebar-ink-dull text-sm">{t('wingdrop.pickDevice')}</p>
						<div className="grid max-w-4xl grid-cols-2 gap-6 sm:grid-cols-3 lg:grid-cols-4">
							{devices.map((device, index) => {
								const state = states[device.id];
								return (
									<motion.button
										key={device.id}
										type="button"
										disabled={!device.is_connected || state?.status === 'sending'}
										initial={{opacity: 0, scale: 0.8}}
										animate={{opacity: 1, scale: 1}}
										transition={{duration: 0.3, delay: index * 0.08, ease: [0.25, 1, 0.5, 1]}}
										onClick={() => void choose(device)}
										className="border-sidebar-line/30 bg-sidebar-box/40 hover:enabled:border-accent/50 hover:enabled:bg-sidebar-box/60 group relative flex w-40 flex-col items-center gap-3 rounded-xl border p-4 backdrop-blur-lg transition-all disabled:cursor-default disabled:opacity-60"
									>
										<div className="relative">
											<div className="bg-accent/20 text-accent flex size-16 items-center justify-center rounded-full text-xl font-bold">
												{state?.status === 'sending' ? (
													<Spinner className="size-7 animate-spin" />
												) : state?.status === 'completed' ? (
													<CheckCircle className="size-8" weight="fill" />
												) : state?.status === 'failed' ? (
													<WarningCircle className="size-8 text-status-error" weight="fill" />
												) : (
													initials(device.name)
												)}
											</div>
											<div
												className={
													device.is_connected
														? 'border-sidebar-box absolute -bottom-1 -right-1 size-4 rounded-full border-2 bg-status-success'
														: 'border-sidebar-box bg-sidebar-ink-faint absolute -bottom-1 -right-1 size-4 rounded-full border-2'
												}
											/>
										</div>
										<div className="w-full text-center">
											<p className="text-sidebar-ink group-hover:group-enabled:text-accent truncate text-sm font-medium">
												{device.name}
											</p>
											<p
												className={
													state?.status === 'failed'
														? 'line-clamp-2 text-xs text-status-error'
														: 'text-sidebar-ink-faint truncate text-xs'
												}
												title={caption(device)}
											>
												{caption(device)}
											</p>
										</div>
										{device.is_connected && !state && (
											<PaperPlaneTilt className="text-sidebar-ink-faint group-hover:text-accent absolute right-2 top-2 size-4" />
										)}
									</motion.button>
								);
							})}
						</div>
					</div>
				)}
			</div>
		</div>
	);
}
