import {DeviceMobile, Eject, Globe, Plus} from '@phosphor-icons/react';
import {Dialog, dialogManager, Input, Label, toast, useDialog} from '@wingdrive/primitives';
import {useQuery, useQueryClient} from '@tanstack/react-query';
import {useForm} from 'react-hook-form';
import {useNavigate} from 'react-router-dom';
import {useTranslation} from '../../i18n';

import {usePlatform, type NetworkMount} from '../../contexts/PlatformContext';
import {useOpenInNewTabAction} from '../../routes/explorer/OpenFileProvider';
import {explorerUrlForDirectory} from '../TabManager/LaunchRequestSync';

const MOUNTS_KEY = ['gvfs', 'mounts'];
const VOLUMES_KEY = ['gvfs', 'mountable'];

interface ConnectForm {
	uri: string;
	password: string;
}

type Connect = (uri: string, password?: string) => Promise<void>;

// Dialogs render outside the router and query providers, so the section
// passes in the work that needs them.
function openConnectDialog(connect: Connect) {
	dialogManager.create((props) => <ConnectDialog {...props} connect={connect} />);
}

function ConnectDialog(props: {id: number; connect: Connect}) {
	const {t} = useTranslation('navigation');
	const dialog = useDialog(props);
	const form = useForm<ConnectForm>({defaultValues: {uri: '', password: ''}});

	const onSubmit = form.handleSubmit(async ({uri, password}) => {
		try {
			await props.connect(uri, password || undefined);
			dialog.state.open = false;
		} catch (error) {
			form.setError('uri', {message: String(error)});
		}
	});

	return (
		<Dialog
			form={form}
			dialog={dialog}
			title={t('network.connectTitle')}
			onSubmit={onSubmit}
			ctaLabel={t('network.connect')}
			loading={form.formState.isSubmitting}
		>
			<div className="space-y-4">
				<div>
					<Label>{t('network.address')}</Label>
					<Input
						{...form.register('uri', {required: true})}
						placeholder="sftp://user@server/  ·  smb://server/share  ·  ftp://host/"
						autoFocus
					/>
					<p className="text-ink-faint mt-1 text-xs">
						{t('network.addressHint')}
					</p>
					{form.formState.errors.uri?.message && (
						<p className="mt-1 text-xs text-red-400">{form.formState.errors.uri.message}</p>
					)}
				</div>
				<div>
					<Label>{t('network.password')}</Label>
					<Input {...form.register('password')} type="password" placeholder={t('network.passwordPlaceholder')} />
				</div>
			</div>
		</Dialog>
	);
}

function iconFor(mount: NetworkMount) {
	return ['mtp', 'gphoto2', 'afc'].includes(mount.kind) ? DeviceMobile : Globe;
}

/** Network locations and devices from gvfs, shown under Volumes on Linux. */
export function NetworkSection() {
	const {t} = useTranslation('navigation');
	const platform = usePlatform();
	const navigate = useNavigate();
	const queryClient = useQueryClient();
	const supported = !!platform.listNetworkMounts;

	// gvfs has no change feed here; polling keeps plugged phones and dropped
	// connections current without a file watcher on the FUSE root.
	const mounts = useQuery({
		queryKey: MOUNTS_KEY,
		queryFn: () => platform.listNetworkMounts!(),
		enabled: supported,
		refetchInterval: 5000
	});
	const mountable = useQuery({
		queryKey: VOLUMES_KEY,
		queryFn: () => platform.listMountableVolumes!(),
		enabled: supported && !!platform.listMountableVolumes,
		refetchInterval: 10000
	});
	const openInNewTab = useOpenInNewTabAction();

	if (!supported) return null;

	const refresh = () => {
		void queryClient.invalidateQueries({queryKey: ['gvfs']});
	};

	const mountVolume = async (uri: string, name: string) => {
		try {
			const mount = await platform.mountNetworkLocation!(uri);
			refresh();
			navigate(explorerUrlForDirectory(mount.path));
		} catch (error) {
			toast.error(t('network.mountFailed', {name, error: String(error)}));
		}
	};

	const connect: Connect = async (uri, password) => {
		const mount = await platform.mountNetworkLocation!(uri, password);
		refresh();
		navigate(explorerUrlForDirectory(mount.path));
	};

	const unmount = async (mount: NetworkMount) => {
		try {
			await platform.unmountNetworkLocation!(mount.path);
			refresh();
			// A tab left on the unmounted folder would keep its stale listing.
			if (decodeURIComponent(window.location.search).includes(mount.path)) navigate('/');
		} catch (error) {
			toast.error(t('network.unmountFailed', {name: mount.name, error: String(error)}));
		}
	};

	const rowClass =
		'group flex w-full items-center gap-2 rounded-md px-2 py-1 text-sm font-medium text-sidebar-inkDull hover:bg-sidebar-selected/40 hover:text-sidebar-ink';

	return (
		<div className="mt-1 space-y-0.5">
			{(mounts.data ?? []).map((mount) => {
				const Icon = iconFor(mount);
				return (
					<div key={mount.path} className={rowClass}>
						<button
							type="button"
							className="flex min-w-0 flex-1 items-center gap-2 text-left"
							onClick={(e) => {
								const route = explorerUrlForDirectory(mount.path);
								if (e.ctrlKey || e.metaKey) openInNewTab(route);
								else navigate(route);
							}}
							onAuxClick={(e) => {
								if (e.button !== 1) return;
								e.preventDefault();
								openInNewTab(explorerUrlForDirectory(mount.path));
							}}
						>
							<Icon className="size-4 shrink-0" weight="bold" />
							<span className="truncate">{mount.name}</span>
						</button>
						<button
							type="button"
							aria-label={t('network.unmountNamed', {name: mount.name})}
							title={t('network.unmount')}
							className="text-ink-faint hover:text-ink opacity-0 group-hover:opacity-100"
							onClick={() => void unmount(mount)}
						>
							<Eject className="size-3.5" weight="bold" />
						</button>
					</div>
				);
			})}
			{(mountable.data ?? []).map((volume) => (
				<button
					key={volume.uri}
					type="button"
					className={rowClass}
					title={t('network.mountNamed', {name: volume.name})}
					onClick={() => void mountVolume(volume.uri, volume.name)}
				>
					<DeviceMobile className="size-4 shrink-0 opacity-60" weight="bold" />
					<span className="truncate opacity-70">{volume.name}</span>
				</button>
			))}
			<button type="button" className={rowClass} onClick={() => openConnectDialog(connect)}>
				<Plus className="size-4 shrink-0" weight="bold" />
				<span className="truncate">{t('network.connectToServer')}</span>
			</button>
		</div>
	);
}
