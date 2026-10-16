import {
	ArrowsClockwise,
	Calendar,
	ChatCircle,
	ClockCounterClockwise,
	Cube,
	DotsThree,
	Eye,
	FilmStrip,
	Fingerprint,
	HardDrive,
	Hash,
	Image,
	Info,
	MagnifyingGlass,
	MapPin,
	Microphone,
	Paperclip,
	PaperPlaneRight,
	ShareNetwork,
	Sparkle,
	Star,
	Tag as TagIcon,
	TextAa,
	Timer,
	VideoCamera
} from '@phosphor-icons/react';
import {getIcon} from '@wingdrive/assets/util';
import {useIsDarkTheme} from '../../../hooks/useTheme';
import type {File, WingPath} from '@wingdrive/ts-client';
import {fileDisplaySize, getContentKind} from '@wingdrive/ts-client';
import {toast} from '@wingdrive/primitives';
import clsx from 'clsx';
import {useEffect, useState} from 'react';
import {useJobsContext} from '../../../components/JobManager/hooks/JobsContext';
import {TagSelectorButton} from '../../../components/Tags';
import {usePlatform} from '../../../contexts/PlatformContext';
import {useServer} from '../../../contexts/ServerContext';
import {
	getDeviceIcon,
	useLibraryMutation,
	useNormalizedQuery
} from '../../../contexts/WingDriveContext';
import {useContextMenu} from '../../../hooks/useContextMenu';
import {useRefetchTagQueries} from '../../../hooks/useRefetchTagQueries';
import {useOptionalExplorer} from '../../../routes/explorer';
import {File as FileComponent} from '../../../routes/explorer/File';
import {formatBytes} from '../../../routes/explorer/utils';
import {Divider, InfoRow, Section, TabContent, Tabs, Tag} from '../Inspector';
import {ChecksumSection} from './ChecksumSection';
import {PermissionsSection} from './PermissionsSection';
import {LocationMap} from '../LocationMap';
import {revealLabel} from '../../../util/keybinds/platform';
import {useTranslation} from '../../../i18n';
import {uiLocale} from '../../../i18n';

interface FileInspectorProps {
	file: File;
}

export function FileInspector({file}: FileInspectorProps) {
const {t} = useTranslation('inspector');
	const [activeTab, setActiveTab] = useState('overview');
	const isDev = import.meta.env.DEV;

	// Extract parent directory for pathScope to enable reactive sidecar updates
	const getParentPath = (): WingPath | undefined => {
		if (!file.wing_path) return undefined;

		if ('Physical' in file.wing_path) {
			const fullPath = file.wing_path.Physical.path;
			const lastSlash = fullPath.lastIndexOf('/');
			if (lastSlash === -1) return undefined;

			return {
				Physical: {
					...file.wing_path.Physical,
					path: fullPath.substring(0, lastSlash)
				}
			};
		}

		if ('Cloud' in file.wing_path) {
			const fullPath = file.wing_path.Cloud.path;
			const lastSlash = fullPath.lastIndexOf('/');
			if (lastSlash === -1) return undefined;

			return {
				Cloud: {
					...file.wing_path.Cloud,
					path: fullPath.substring(0, lastSlash)
				}
			};
		}

		return undefined;
	};

	const fileQuery = useNormalizedQuery<{file_id: string}, File>({
		query: 'files.by_id',
		input: {file_id: file?.id || ''},
		resourceType: 'file',
		resourceId: file?.id,
		pathScope: getParentPath(),
		includeDescendants: false,
		enabled: !!file?.id
	});

	const fileData = fileQuery.data || file;

	const tabs = [
{id: 'overview', label: t('inspector.tabs.overview'), icon: Info},
{id: 'sidecars', label: t('inspector.tabs.sidecars'), icon: Image},
{id: 'instances', label: t('inspector.tabs.instances'), icon: MapPin},
		...(isDev
? [{id: 'chat', label: t('inspector.tabs.chat'), icon: ChatCircle, badge: 3}]
			: []),
		...(isDev
? [{id: 'activity', label: t('inspector.tabs.activity'), icon: ClockCounterClockwise}]
			: []),
{id: 'details', label: t('inspector.tabs.more'), icon: DotsThree}
	];

	return (
		<>
			{/* Tabs */}
			<Tabs tabs={tabs} activeTab={activeTab} onChange={setActiveTab} />

			{/* Tab Content */}
			<div className="mt-2.5 flex flex-1 flex-col overflow-hidden">
				<TabContent id="overview" activeTab={activeTab}>
					<OverviewTab file={fileData} />
				</TabContent>

				<TabContent id="sidecars" activeTab={activeTab}>
					<SidecarsTab file={fileData} />
				</TabContent>

				<TabContent id="instances" activeTab={activeTab}>
					<InstancesTab file={fileData} />
				</TabContent>

				{isDev && (
					<TabContent id="chat" activeTab={activeTab}>
						<ChatTab />
					</TabContent>
				)}

				{isDev && (
					<TabContent id="activity" activeTab={activeTab}>
						<ActivityTab />
					</TabContent>
				)}

				<TabContent id="details" activeTab={activeTab}>
					<DetailsTab file={fileData} />
				</TabContent>
			</div>
		</>
	);
}

// Quick Actions Component - Favorite, Share & Jobs buttons
function FileQuickActions({file}: {file: File}) {
const {t} = useTranslation('inspector');
	const platform = usePlatform();
	// Null in the pop-out inspector window, which has no explorer to overlay.
	const explorer = useOptionalExplorer();
	const [isFavorite, setIsFavorite] = useState(file.favorite);
	const setFavorite = useLibraryMutation('metadata.set_favorite');

	useEffect(() => setIsFavorite(file.favorite), [file.id, file.favorite]);

	const handleFavorite = async () => {
		const next = !isFavorite;
		setIsFavorite(next);
		try {
			await setFavorite.mutateAsync({
				entry_uuid: file.id,
				is_favorite: next
			});
		} catch (error) {
			setIsFavorite(!next);
			toast.error({
				title: t('inspector.file.favoriteUpdateFailed'),
				body: String(error)
			});
		}
	};

	// AI Processing mutations
	const extractText = useLibraryMutation('media.ocr.extract');
	const transcribeAudio = useLibraryMutation('media.speech.transcribe');
	const generateSplat = useLibraryMutation('media.splat.generate');
	const regenerateThumbnail = useLibraryMutation(
		'media.thumbnail.regenerate'
	);
	const generateThumbstrip = useLibraryMutation('media.thumbstrip.generate');
	const generateProxy = useLibraryMutation('media.proxy.generate');

	// Check content kind for available actions
	const isImage = getContentKind(file) === 'image';
	const isVideo = getContentKind(file) === 'video';
	const isAudio = getContentKind(file) === 'audio';
	const showJobsButton = isImage || isVideo || isAudio;

	// The share sheet takes a path on this machine. Another device's path
	// would name a different file here, or none.
	const physicalPath =
		file.is_local && file.wing_path && 'Physical' in file.wing_path
			? file.wing_path.Physical.path
			: null;
	const canShare = !!physicalPath && !!platform.shareFiles;

	const handleShare = async () => {
		if (!physicalPath || !platform.shareFiles) {
			toast.error({
				title: t('inspector.file.cannotShare'),
				body: t('inspector.file.cannotShareBody')
			});
			return;
		}

		try {
			await platform.shareFiles([physicalPath]);
		} catch (err) {
			console.error('Failed to share file:', err);
			toast.error({
				title: t('inspector.file.shareFailed'),
				body: String(err)
			});
		}
	};

	// Jobs context menu
	const jobsMenu = useContextMenu({
		items: [
			// Image actions
			{
				icon: TextAa,
				label: t('inspector.file.extractTextOcr'),
				onClick: () => {
					extractText.mutate({
						entry_uuid: file.id,
						languages: ['eng'],
						force: false
					});
				},
				condition: () => isImage
			},
			{
				icon: Cube,
				label: t('inspector.file.generateSplat'),
				onClick: () => {
					generateSplat.mutate({
						entry_uuid: file.id,
						model_path: null
					});
				},
				condition: () => isImage
			},
			// Video/Audio actions
			{
				icon: Microphone,
				label: t('inspector.file.generateSubtitles'),
				onClick: () => {
					transcribeAudio.mutate({
						entry_uuid: file.id,
						model: 'base',
						language: null
					});
				},
				condition: () => isVideo || isAudio
			},
			// Video-only actions
			{
				icon: FilmStrip,
				label: t('inspector.file.generateThumbstrip'),
				onClick: () => {
					generateThumbstrip.mutate({
						entry_uuid: file.id,
						variants: ['thumbstrip_preview', 'thumbstrip_detailed'],
						force: false
					});
				},
				condition: () => isVideo
			},
			{
				icon: VideoCamera,
				label: t('inspector.file.generateScrubbingProxy'),
				onClick: () => {
					generateProxy.mutate({
						entry_uuid: file.id,
						resolution: 'scrubbing',
						force: false,
						use_hardware_accel: true
					});
				},
				condition: () => isVideo
			},
			{type: 'separator' as const},
			// Common actions
			{
				icon: ArrowsClockwise,
				label: t('inspector.file.regenerateThumbnails'),
				onClick: () => {
					regenerateThumbnail.mutate({
						entry_uuid: file.id,
						variants: ['grid@1x', 'grid@2x', 'detail@1x'],
						force: true
					});
				},
				condition: () => isImage || isVideo
			}
		]
	});

	return (
		<div className="flex items-center gap-1.5">
			<button
				type="button"
				onClick={handleFavorite}
				disabled={setFavorite.isPending}
				className={clsx(
					'border-sidebar-line/30 bg-sidebar-box/20 hover:bg-sidebar-box/30 flex size-7 items-center justify-center rounded-full border transition-all active:scale-95 disabled:cursor-wait disabled:opacity-60',
					isFavorite
						? 'text-yellow-400'
						: 'text-sidebar-ink-dull hover:text-sidebar-ink'
				)}
					title={
						isFavorite ? t('inspector.preview.removeFavorite') : t('inspector.preview.addFavorite')
					}
					aria-label={
						isFavorite ? t('inspector.preview.removeFavorite') : t('inspector.preview.addFavorite')
					}
			>
				<Star size={14} weight={isFavorite ? 'fill' : 'regular'} />
			</button>

			{/* Quick Preview Button — spacebar isn't available on touch devices */}
			{explorer && (
				<button
					type="button"
					onClick={() => explorer.openQuickPreview(file.id)}
					className="border-sidebar-line/30 bg-sidebar-box/20 text-sidebar-ink-dull hover:bg-sidebar-box/30 hover:text-sidebar-ink flex size-7 items-center justify-center rounded-full border transition-all active:scale-95"
title={t('inspector.file.quickPreview')}
				>
					<Eye size={14} weight="bold" />
				</button>
			)}

			{/* Share Button */}
			{canShare && (
				<button
					type="button"
					onClick={handleShare}
					className="border-sidebar-line/30 bg-sidebar-box/20 text-sidebar-ink-dull hover:bg-sidebar-box/30 hover:text-sidebar-ink flex size-7 items-center justify-center rounded-full border transition-all active:scale-95"
title={t('inspector.file.share')}
				>
					<ShareNetwork size={14} weight="bold" />
				</button>
			)}

			{/* Jobs Button */}
			{showJobsButton && (
				<button
					type="button"
					onClick={(e) => jobsMenu.show(e)}
					className="border-sidebar-line/30 bg-sidebar-box/20 text-sidebar-ink-dull hover:bg-sidebar-box/30 hover:text-sidebar-ink flex size-7 items-center justify-center rounded-full border transition-all active:scale-95"
title={t('inspector.file.processingJobs')}
				>
					<Sparkle size={14} weight="bold" />
				</button>
			)}
		</div>
	);
}

// Media Metadata Card Component - iOS Photos style
function MediaMetadataCard({file}: {file: File}) {
	const imageData = file.image_media_data;
	const videoData = file.video_media_data;
	const displaySize = fileDisplaySize(file);

	if (!imageData && !videoData) return null;

	// Format date/time like iOS: "Monday • Jan 19, 2026 • 08:00"
	const formatMediaDate = (dateStr: string | null | undefined) => {
		if (!dateStr) return null;
		const date = new Date(dateStr);
		const weekday = date.toLocaleDateString(uiLocale(), {weekday: 'long'});
		const monthDay = date.toLocaleDateString(uiLocale(), {
			month: 'short',
			day: 'numeric',
			year: 'numeric'
		});
		const time = date.toLocaleTimeString(uiLocale(), {
			hour: '2-digit',
			minute: '2-digit',
			hour12: false
		});
		return `${weekday} • ${monthDay} • ${time}`;
	};

	// Calculate megapixels
	const getMegapixels = (width: number, height: number) => {
		const mp = (width * height) / 1000000;
		return mp >= 1 ? `${mp.toFixed(1)} MP` : `${(mp * 1000).toFixed(0)} KP`;
	};

	// Get format from extension
	const getFormat = () => {
		const ext = file.extension?.toUpperCase();
		if (!ext) return null;
		// Map common extensions to display names
		const formatMap: Record<string, string> = {
			HEIC: 'HEIF',
			HEIF: 'HEIF',
			JPG: 'JPEG',
			JPEG: 'JPEG',
			PNG: 'PNG',
			WEBP: 'WebP',
			RAW: 'RAW',
			DNG: 'DNG',
			CR2: 'RAW',
			NEF: 'RAW',
			ARW: 'RAW',
			MOV: 'MOV',
			MP4: 'MP4',
			M4V: 'M4V',
			WEBM: 'WebM',
			MKV: 'MKV',
			AVI: 'AVI'
		};
		return formatMap[ext] || ext;
	};

	// Get camera display name
	const getCameraDisplay = () => {
		if (imageData?.camera_make && imageData?.camera_model) {
			return `${imageData.camera_make} ${imageData.camera_model}`;
		}
		if (imageData?.camera_model) return imageData.camera_model;
		if (imageData?.camera_make) return imageData.camera_make;
		return null;
	};

	// Get lens/camera info line (e.g., "Main Camera — 24 mm ƒ1.78")
	const getLensInfo = () => {
		const parts: string[] = [];

		if (imageData?.lens_model) {
			parts.push(imageData.lens_model);
		} else if (imageData?.focal_length) {
			// If no lens model, just show focal length
			parts.push(imageData.focal_length);
		}

		if (imageData?.aperture) {
			// Format aperture with ƒ symbol
			const aperture = imageData.aperture.replace(/^f\/?/i, 'ƒ');
			if (parts.length > 0) {
				parts[parts.length - 1] += ` ${aperture}`;
			} else {
				parts.push(aperture);
			}
		}

		return parts.length > 0 ? parts.join(' — ') : null;
	};

	// Get resolution line (e.g., "12 MP • 3024 × 4032 • 2.1 MB")
	const getResolutionLine = () => {
		const parts: string[] = [];
		const width = imageData?.width || videoData?.width;
		const height = imageData?.height || videoData?.height;

		if (width && height) {
			parts.push(getMegapixels(width, height));
			parts.push(`${width} × ${height}`);
		}

		if (displaySize) {
			parts.push(formatBytes(displaySize));
		}

		return parts.length > 0 ? parts.join(' • ') : null;
	};

	// Get technical specs (ISO, focal length, exposure, aperture, shutter)
	const getTechSpecs = () => {
		if (!imageData) return [];

		const specs: {label: string; value: string}[] = [];

		if (imageData.iso) {
			specs.push({label: 'ISO', value: String(imageData.iso)});
		}

		if (imageData.focal_length) {
			specs.push({label: '', value: imageData.focal_length});
		}

		// Exposure value (not typically in our data, but could be calculated)
		// For now skip this

		if (imageData.aperture) {
			const aperture = imageData.aperture.replace(/^f\/?/i, 'ƒ');
			specs.push({label: '', value: aperture});
		}

		if (imageData.shutter_speed) {
			specs.push({label: '', value: imageData.shutter_speed});
		}

		return specs;
	};

	// Get video-specific info
	const getVideoInfo = () => {
		if (!videoData) return null;

		const parts: string[] = [];

		if (videoData.codec) {
			parts.push(videoData.codec.toUpperCase());
		}

		if (videoData.fps_num && videoData.fps_den && videoData.fps_den !== 0) {
			const fps = Math.round(videoData.fps_num / videoData.fps_den);
			parts.push(`${fps} fps`);
		}

		if (videoData.bit_rate) {
			const mbps = (videoData.bit_rate / 1000000).toFixed(1);
			parts.push(`${mbps} Mbps`);
		}

		return parts.length > 0 ? parts.join(' • ') : null;
	};

	// Get duration for video
	const getDuration = () => {
		if (!videoData?.duration_seconds) return null;
		const duration = videoData.duration_seconds;
		const minutes = Math.floor(duration / 60);
		const seconds = Math.floor(duration % 60);
		return `${minutes}:${String(seconds).padStart(2, '0')}`;
	};

	const mediaDate = formatMediaDate(
		imageData?.date_taken?.toString() ||
			videoData?.date_captured?.toString()
	);
	const cameraDisplay = getCameraDisplay();
	const format = getFormat();
	const lensInfo = getLensInfo();
	const resolutionLine = getResolutionLine();
	const techSpecs = getTechSpecs();
	const hasLocation = !!(imageData?.latitude && imageData?.longitude);
	const videoInfo = getVideoInfo();
	const duration = getDuration();
	const colorProfile = imageData?.color_profile;

	return (
		<div className="space-y-3">
			{/* Date/Time Header */}
			{mediaDate && (
				<div className="px-2">
					<div className="text-sidebar-ink text-sm font-semibold">
						{mediaDate}
					</div>
					<div className="text-sidebar-ink-dull mt-0.5 flex items-center gap-1.5 text-xs">
						<Image size={12} weight="bold" />
						<span>{file.name}</span>
					</div>
				</div>
			)}

			{/* Camera Info Card */}
			{(cameraDisplay || resolutionLine || videoInfo) && (
				<div className="bg-app-box/60 border-app-line/50 mx-2 overflow-hidden rounded-xl border">
					{/* Camera Header */}
					{cameraDisplay && (
						<div className="border-app-line/30 flex items-center justify-between border-b px-3 py-2.5">
							<span className="text-sidebar-ink text-sm font-medium">
								{cameraDisplay}
							</span>
							<div className="flex items-center gap-1.5">
								{format && (
									<span className="bg-app-box border-app-line text-sidebar-ink-dull rounded-md border px-1.5 py-0.5 text-[10px] font-semibold">
										{format}
									</span>
								)}
								{hasLocation && (
									<span className="text-sidebar-ink-dull">
										<MapPin size={14} weight="fill" />
									</span>
								)}
							</div>
						</div>
					)}

					{/* Camera/Lens Details */}
					<div className="space-y-1 px-3 py-2.5">
						{lensInfo && (
							<div className="text-sidebar-ink-dull text-xs">
								{lensInfo}
							</div>
						)}
						{resolutionLine && (
							<div className="text-sidebar-ink-dull text-xs">
								{resolutionLine}
							</div>
						)}
						{videoInfo && (
							<div className="text-sidebar-ink-dull text-xs">
								{videoInfo}
							</div>
						)}
						{duration && (
							<div className="text-sidebar-ink-dull flex items-center gap-1 text-xs">
								<Timer size={12} weight="bold" />
								<span>{duration}</span>
							</div>
						)}
					</div>

					{/* Technical Specs Row */}
					{techSpecs.length > 0 && (
						<div className="border-app-line/30 flex items-center justify-between border-t px-3 py-2">
							{techSpecs.map((spec) => (
								<div
									key={`${spec.label}-${spec.value}`}
									className="text-sidebar-ink-dull text-center text-[11px]"
								>
									{spec.label && (
										<span className="mr-0.5 font-medium">
											{spec.label}
										</span>
									)}
									<span>{spec.value}</span>
								</div>
							))}
							{colorProfile && (
								<span className="bg-app-box border-app-line text-sidebar-ink-dull rounded-md border px-1.5 py-0.5 text-[10px] font-medium uppercase">
									{colorProfile}
								</span>
							)}
						</div>
					)}
				</div>
			)}

			{/* Location Map */}
			{hasLocation &&
				imageData &&
				imageData.latitude &&
				imageData.longitude && (
					<LocationMap
						latitude={imageData.latitude}
						longitude={imageData.longitude}
						className="bg-app-box/60 border-app-line/50 mx-2 overflow-hidden rounded-xl border"
					/>
				)}
		</div>
	);
}

function OverviewTab({file}: {file: File}) {
const {t} = useTranslation('inspector');
	const displaySize = fileDisplaySize(file);
	const formatDate = (dateStr: string) => {
		const date = new Date(dateStr);
		return date.toLocaleDateString(uiLocale(), {
			month: 'short',
			day: 'numeric',
			year: 'numeric'
		});
	};

	// Tag mutations — refetch queries on success to update the UI
	const refetchTagQueries = useRefetchTagQueries();
	const applyTag = useLibraryMutation('tags.apply', {
		onSuccess: refetchTagQueries
	});
	const unapplyTag = useLibraryMutation('tags.unapply', {
		onSuccess: refetchTagQueries
	});

	// AI Processing mutations
	const extractText = useLibraryMutation('media.ocr.extract');
	const transcribeAudio = useLibraryMutation('media.speech.transcribe');
	const generateSplat = useLibraryMutation('media.splat.generate');
	const regenerateThumbnail = useLibraryMutation(
		'media.thumbnail.regenerate'
	);
	const generateThumbstrip = useLibraryMutation('media.thumbstrip.generate');
	const generateProxy = useLibraryMutation('media.proxy.generate');

	// Job tracking for long-running operations
	const {jobs} = useJobsContext();
	const isSpeechJobRunning = jobs.some(
		(job) =>
			job.name === 'speech_to_text' &&
			(job.status === 'running' || job.status === 'queued')
	);

	// Check content kind for available actions
	const isImage = getContentKind(file) === 'image';
	const isVideo = getContentKind(file) === 'video';
	const isAudio = getContentKind(file) === 'audio';
	const hasText = file?.content_identity?.text_content;
	const isEphemeral = !file.content_identity;

	const contentKind = getContentKind(file);
	const rawKind =
		contentKind && contentKind !== 'unknown'
			? contentKind
			: file.kind === 'File'
				? file.extension || 'File'
				: file.kind;
	// Extensions are shown as-is; known kinds get their translated name
	const fileKind = t(`kinds.${rawKind}`, {defaultValue: rawKind});

	return (
		<div className="no-scrollbar mask-fade-out flex flex-col space-y-5 overflow-x-hidden overflow-y-scroll pb-10">
			{/* Thumbnail */}
			<div className="flex w-full justify-center px-4">
				<FileComponent.Thumb
					file={file}
					size={200}
					iconScale={0.6}
					className="w-full max-w-full"
				/>
			</div>

			{/* File name & Quick Actions */}
			<div className="px-2">
				<h4 className="text-sidebar-ink truncate text-center text-sm font-semibold">
					{file.name}
					{file.extension ? `.${file.extension}` : ''}
				</h4>
				<p className="text-sidebar-ink-dull mt-1 text-center text-xs">
					{fileKind}
				</p>
				{/* Quick Actions - Favorite, Share & Jobs */}
				<div className="mt-2.5">
					<FileQuickActions file={file} />
				</div>
			</div>

			{/* Media Metadata Card - iOS Photos style */}
			{(isImage || isVideo) &&
				(file.image_media_data || file.video_media_data) && (
					<>
						<Divider />
						<MediaMetadataCard file={file} />
					</>
				)}

			<Divider />

			{/* Details */}
<Section title={t('inspector.file.details')} icon={Info}>
{displaySize !== null && (
	<InfoRow label={t('inspector.file.size')} value={formatBytes(displaySize)} />
)}
				{file.kind === 'File' && file.extension && (
<InfoRow label={t('inspector.file.extension')} value={file.extension} />
				)}
<InfoRow label={t('inspector.file.kind')} value={fileKind} />
			</Section>

			{/* Dates */}
<Section title={t('inspector.file.dates')} icon={Calendar}>
				{/* Show capture date for media files */}
				{file.video_media_data?.date_captured && (
					<InfoRow
label={t('inspector.file.captured')}
						value={formatDate(file.video_media_data.date_captured)}
					/>
				)}
				{file.image_media_data?.date_taken && (
					<InfoRow
label={t('inspector.file.taken')}
						value={formatDate(file.image_media_data.date_taken)}
					/>
				)}
<InfoRow label={t('inspector.file.created')} value={formatDate(file.created_at)} />
				<InfoRow
label={t('inspector.file.modified')}
					value={formatDate(file.modified_at)}
				/>
				{file.accessed_at && (
					<InfoRow
label={t('inspector.file.accessed')}
						value={formatDate(file.accessed_at)}
					/>
				)}
			</Section>

			{/* Image Metadata */}
			{file.image_media_data && (
<Section title={t('inspector.file.imageInfo')} icon={Image}>
					<InfoRow
label={t('inspector.file.dimensions')}
						value={`${file.image_media_data.width} × ${file.image_media_data.height}`}
					/>
					{file.image_media_data.camera_make && (
						<InfoRow
label={t('inspector.file.camera')}
							value={`${file.image_media_data.camera_make} ${file.image_media_data.camera_model || ''}`}
						/>
					)}
					{file.image_media_data.lens_model && (
						<InfoRow
label={t('inspector.file.lens')}
							value={file.image_media_data.lens_model}
						/>
					)}
					{file.image_media_data.iso && (
						<InfoRow
label={t('inspector.file.iso')}
							value={String(file.image_media_data.iso)}
						/>
					)}
					{file.image_media_data.focal_length && (
						<InfoRow
label={t('inspector.file.focalLength')}
							value={file.image_media_data.focal_length}
						/>
					)}
					{file.image_media_data.aperture && (
						<InfoRow
label={t('inspector.file.aperture')}
							value={file.image_media_data.aperture}
						/>
					)}
					{file.image_media_data.shutter_speed && (
						<InfoRow
label={t('inspector.file.shutterSpeed')}
							value={file.image_media_data.shutter_speed}
						/>
					)}
				</Section>
			)}

			{/* Video Metadata */}
			{file.video_media_data && (
<Section title={t('inspector.file.videoInfo')} icon={Image}>
					<InfoRow
label={t('inspector.file.resolution')}
						value={`${file.video_media_data.width} × ${file.video_media_data.height}`}
					/>
					{file.video_media_data.duration_seconds && (
						<InfoRow
label={t('inspector.file.duration')}
							value={`${Math.floor(file.video_media_data.duration_seconds / 60)}:${String(Math.floor(file.video_media_data.duration_seconds % 60)).padStart(2, '0')}`}
						/>
					)}
					{file.video_media_data.codec && (
						<InfoRow
label={t('inspector.file.codec')}
							value={file.video_media_data.codec}
						/>
					)}
					{file.video_media_data.bit_rate && (
						<InfoRow
label={t('inspector.file.bitrate')}
							value={`${Math.round(file.video_media_data.bit_rate / 1000000)} Mbps`}
						/>
					)}
					{file.video_media_data.fps_num &&
						file.video_media_data.fps_den &&
						file.video_media_data.fps_den !== 0 && (
							<InfoRow
label={t('inspector.file.frameRate')}
								value={`${Math.round((file.video_media_data.fps_num / file.video_media_data.fps_den) * 100) / 100} fps`}
							/>
						)}
					{file.video_media_data.audio_codec && (
						<InfoRow
							label={t('inspector.file.audio')}
							value={`${file.video_media_data.audio_codec} · ${file.video_media_data.audio_channels || ''}`}
						/>
					)}
				</Section>
			)}

			{/* Audio Metadata */}
			{file.audio_media_data && (
<Section title={t('inspector.file.audioInfo')} icon={Image}>
					{file.audio_media_data.artist && (
						<InfoRow
							label={t('inspector.file.artist')}
							value={file.audio_media_data.artist}
						/>
					)}
					{file.audio_media_data.album && (
						<InfoRow
							label={t('inspector.file.album')}
							value={file.audio_media_data.album}
						/>
					)}
					{file.audio_media_data.title && (
						<InfoRow
							label={t('inspector.file.title')}
							value={file.audio_media_data.title}
						/>
					)}
					{file.audio_media_data.duration_seconds && (
						<InfoRow
							label={t('inspector.file.duration')}
							value={`${Math.floor(file.audio_media_data.duration_seconds / 60)}:${String(Math.floor(file.audio_media_data.duration_seconds % 60)).padStart(2, '0')}`}
						/>
					)}
					{file.audio_media_data.codec && (
						<InfoRow
							label={t('inspector.file.codec')}
							value={file.audio_media_data.codec}
						/>
					)}
					{file.audio_media_data.bit_rate && (
						<InfoRow
							label={t('inspector.file.bitrate')}
							value={`${Math.round(file.audio_media_data.bit_rate / 1000)} kbps`}
						/>
					)}
					{file.audio_media_data.genre && (
						<InfoRow
							label={t('inspector.file.genre')}
							value={file.audio_media_data.genre}
						/>
					)}
					{file.audio_media_data.year && (
						<InfoRow
							label={t('inspector.file.year')}
							value={String(file.audio_media_data.year)}
						/>
					)}
				</Section>
			)}

			{/* Storage */}
<Section title={t('inspector.file.storage')} icon={HardDrive}>
				<InfoRow
label={t('inspector.file.path')}
					value={
						'Physical' in file.wing_path
							? String(file.wing_path.Physical.path)
							: 'Cloud' in file.wing_path
								? String(file.wing_path.Cloud.path)
								: t('inspector.location.content')
					}
				/>
<InfoRow label={t('inspector.file.local')} value={file.is_local ? t('inspector.file.yes') : t('inspector.file.no')} />
			</Section>

			{file.is_local && <PermissionsSection path={file.wing_path} />}

			{file.is_local && file.kind === 'File' && 'Physical' in file.wing_path && (
				<ChecksumSection key={file.id} path={file.wing_path} />
			)}

			{/* Tags */}
<Section title={t('inspector.file.tags')} icon={TagIcon}>
				{isEphemeral ? (
					<p className="text-sidebar-ink-dull text-xs italic">
{t('inspector.file.tagsAfterIndexing')}
					</p>
				) : (
					<div className="flex flex-wrap gap-1.5">
						{file.tags &&
							file.tags.length > 0 &&
							file.tags.map((tag) => (
								<Tag
									key={tag.id}
									color={tag.color || '#3B82F6'}
									size="sm"
									onRemove={async () => {
										try {
											await unapplyTag.mutateAsync({
												entry_ids: [file.id],
												tag_ids: [tag.id]
											});
										} catch (err) {
											console.error(
												'Failed to remove tag:',
												err
											);
											toast.error(
												t('inspector.file.removeTagFailed', { error: String(err) })
											);
										}
									}}
								>
									{tag.canonical_name}
								</Tag>
							))}

						{/* Add Tag Button */}
						<TagSelectorButton
							onSelect={async (tag) => {
								await applyTag.mutateAsync({
									targets: file.content_identity?.uuid
										? {
												type: 'Content',
												ids: [
													file.content_identity.uuid
												]
											}
										: {
												type: 'EntryUuid',
												ids: [file.id]
											},
									tag_ids: [tag.id],
									source: 'User',
									confidence: 1.0,
									applied_context: null,
									instance_attributes: null
								});
							}}
							contextTags={file.tags || []}
							fileId={file.id}
							contentId={file.content_identity?.uuid}
							trigger={
								<button className="bg-app-box hover:bg-app-hover border-app-line text-ink-dull hover:text-ink rounded-full border px-2 py-0.5 text-xs font-medium transition-colors">
										{t('inspector.file.addTags')}
								</button>
							}
						/>
					</div>
				)}
			</Section>

			{/* AI Processing */}
			{(isImage || isVideo || isAudio) && (
<Section title={t('inspector.file.aiProcessing')} icon={Sparkle}>
					<div className="flex flex-col gap-2">
						{/* OCR for images */}
						{isImage && (
							<button
								onClick={() => {
									console.log(
										'Extract text clicked for file:',
										file.id
									);
									extractText.mutate(
										{
											entry_uuid: file.id,
											languages: ['eng'],
											force: false
										},
										{
											onSuccess: (data) => {
												console.log(
													'OCR success:',
													data
												);
											},
											onError: (error) => {
												console.error(
													'OCR error:',
													error
												);
											}
										}
									);
								}}
								disabled={extractText.isPending}
								className={clsx(
									'flex items-center gap-2 rounded-md px-3 py-2 text-sm font-medium transition-colors',
									'bg-app-box hover:bg-app-hover border-app-line border',
									extractText.isPending &&
										'cursor-not-allowed opacity-50'
								)}
							>
								<TextAa size={4} weight="bold" />
								<span>
									{extractText.isPending
										? t('inspector.file.extracting')
										: t('inspector.file.extractTextOcr')}
								</span>
							</button>
						)}

						{/* Gaussian Splat for images */}
						{isImage && (
							<button
								onClick={() => {
									console.log(
										'Generate splat clicked for file:',
										file.id
									);
									generateSplat.mutate(
										{
											entry_uuid: file.id,
											model_path: null
										},
										{
											onSuccess: (data) => {
												console.log(
													'Splat generation success:',
													data
												);
											},
											onError: (error) => {
												console.error(
													'Splat generation error:',
													error
												);
											}
										}
									);
								}}
								disabled={generateSplat.isPending}
								className={clsx(
									'flex items-center gap-2 rounded-md px-3 py-2 text-sm font-medium transition-colors',
									'bg-app-box hover:bg-app-hover border-app-line border',
									generateSplat.isPending &&
										'cursor-not-allowed opacity-50'
								)}
							>
								<Cube size={4} weight="bold" />
								<span>
									{generateSplat.isPending
										? t('inspector.file.generating')
										: t('inspector.file.generateSplat')}
								</span>
							</button>
						)}

						{/* Speech-to-text for audio/video */}
						{(isVideo || isAudio) && (
							<button
								onClick={() => {
									console.log(
										'Transcribe clicked for file:',
										file.id
									);
									transcribeAudio.mutate(
										{
											entry_uuid: file.id,
											model: 'base',
											language: null
										},
										{
											onSuccess: (data) => {
												console.log(
													'Transcription success:',
													data
												);
											},
											onError: (error) => {
												console.error(
													'Transcription error:',
													error
												);

												// Check if it's a feature-disabled error
												const errorMessage =
													error instanceof Error
														? error.message
														: String(error);
												if (
													errorMessage.includes(
														'feature is not enabled'
													) ||
													errorMessage.includes(
														'--features ffmpeg'
													)
												) {
													toast.error({
														title: t('inspector.file.featureUnavailable'),
														body: t('inspector.file.speechRequiresFfmpeg')
													});
												} else {
													toast.error({
														title: t('inspector.file.transcriptionFailed'),
														body: errorMessage
													});
												}
											}
										}
									);
								}}
								disabled={
									transcribeAudio.isPending ||
									isSpeechJobRunning
								}
								className={clsx(
									'flex items-center gap-2 rounded-md px-3 py-2 text-sm font-medium transition-colors',
									'bg-app-box hover:bg-app-hover border-app-line border',
									(transcribeAudio.isPending ||
										isSpeechJobRunning) &&
										'cursor-not-allowed opacity-50'
								)}
							>
								<Microphone size={4} weight="bold" />
								<span>
									{transcribeAudio.isPending ||
									isSpeechJobRunning
										? t('inspector.file.transcribing')
										: t('inspector.file.generateSubtitles')}
								</span>
							</button>
						)}

						{/* Regenerate thumbnails */}
						{(isImage || isVideo) && (
							<button
								onClick={() => {
									console.log(
										'Regenerate thumbnails clicked for file:',
										file.id
									);
									regenerateThumbnail.mutate(
										{
											entry_uuid: file.id,
											variants: [
												'grid@1x',
												'grid@2x',
												'detail@1x'
											],
											force: true
										},
										{
											onSuccess: (data) => {
												console.log(
													'Thumbnail regeneration success:',
													data
												);
											},
											onError: (error) => {
												console.error(
													'Thumbnail regeneration error:',
													error
												);

												const errorMessage =
													error instanceof Error
														? error.message
														: String(error);
												if (
													errorMessage.includes(
														'feature is not enabled'
													) ||
													errorMessage.includes(
														'--features ffmpeg'
													) ||
													errorMessage.includes(
														'FFmpeg feature'
													)
												) {
													toast.error({
														title: t('inspector.file.featureUnavailable'),
														body: t('inspector.file.thumbnailRequiresFfmpeg')
													});
												} else {
													toast.error({
														title: t('inspector.file.thumbnailFailed'),
														body: errorMessage
													});
												}
											}
										}
									);
								}}
								disabled={regenerateThumbnail.isPending}
								className={clsx(
									'flex items-center gap-2 rounded-md px-3 py-2 text-sm font-medium transition-colors',
									'bg-app-box hover:bg-app-hover border-app-line border',
									regenerateThumbnail.isPending &&
										'cursor-not-allowed opacity-50'
								)}
							>
								<ArrowsClockwise size={4} weight="bold" />
								<span>
									{regenerateThumbnail.isPending
										? t('inspector.file.generating')
										: t('inspector.file.regenerateThumbnails')}
								</span>
							</button>
						)}

						{/* Generate thumbstrip (for videos) */}
						{isVideo && (
							<button
								onClick={() => {
									console.log(
										'Generate thumbstrip clicked for file:',
										file.id
									);
									generateThumbstrip.mutate(
										{
											entry_uuid: file.id,
											variants: [
												'thumbstrip_preview',
												'thumbstrip_detailed'
											],
											force: false
										},
										{
											onSuccess: (data) => {
												console.log(
													'Thumbstrip generation success:',
													data
												);
											},
											onError: (error) => {
												console.error(
													'Thumbstrip generation error:',
													error
												);

												const errorMessage =
													error instanceof Error
														? error.message
														: String(error);
												if (
													errorMessage.includes(
														'feature is not enabled'
													) ||
													errorMessage.includes(
														'--features ffmpeg'
													)
												) {
													toast.error({
														title: t('inspector.file.featureUnavailable'),
														body: t('inspector.file.thumbstripRequiresFfmpeg')
													});
												} else {
													toast.error({
														title: t('inspector.file.thumbstripFailed'),
														body: errorMessage
													});
												}
											}
										}
									);
								}}
								disabled={generateThumbstrip.isPending}
								className={clsx(
									'flex items-center gap-2 rounded-md px-3 py-2 text-sm font-medium transition-colors',
									'bg-app-box hover:bg-app-hover border-app-line border',
									generateThumbstrip.isPending &&
										'cursor-not-allowed opacity-50'
								)}
							>
								<FilmStrip size={4} weight="bold" />
								<span>
									{generateThumbstrip.isPending
										? t('inspector.file.generating')
										: t('inspector.file.generateThumbstrip')}
								</span>
							</button>
						)}

						{/* Generate proxy (for videos) */}
						{isVideo && (
							<button
								onClick={() => {
									console.log(
										'Generate proxy clicked for file:',
										file.id
									);
									generateProxy.mutate(
										{
											entry_uuid: file.id,
											resolution: 'scrubbing', // Fast scrubbing proxy
											force: false,
											use_hardware_accel: true
										},
										{
											onSuccess: (data) => {
												console.log(
													'Proxy generation success:',
													data
												);
											},
											onError: (error) => {
												console.error(
													'Proxy generation error:',
													error
												);
											}
										}
									);
								}}
								disabled={generateProxy.isPending}
								className={clsx(
									'flex items-center gap-2 rounded-md px-3 py-2 text-sm font-medium transition-colors',
									'bg-app-box hover:bg-app-hover border-app-line border',
									generateProxy.isPending &&
										'cursor-not-allowed opacity-50'
								)}
							>
								<VideoCamera size={4} weight="bold" />
								<span>
									{generateProxy.isPending
										? t('inspector.file.encoding')
										: t('inspector.file.generateScrubbingProxy')}
								</span>
							</button>
						)}

						{/* Show extracted text if available */}
						{hasText && (
							<div className="bg-app-box/40 border-app-line/50 mt-2 rounded-lg border p-3">
								<div className="mb-2 flex items-center gap-2">
									<span className="text-accent">
										<TextAa size={16} weight="bold" />
									</span>
									<span className="text-sidebar-ink text-xs font-medium">
										{t("ui.extractedText")}
									</span>
								</div>
								<pre className="text-sidebar-ink-dull no-scrollbar max-h-40 overflow-y-auto whitespace-pre-wrap text-xs">
									{file.content_identity?.text_content}
								</pre>
							</div>
						)}
					</div>
				</Section>
			)}
		</div>
	);
}

function SidecarsTab({file}: {file: File}) {
const {t} = useTranslation('inspector');
	const sidecars = file.sidecars || [];
	const platform = usePlatform();
	const {buildSidecarUrl, libraryId} = useServer();

	// Helper to get sidecar URL
	const getSidecarUrl = (sidecar: any) => {
		if (!file.content_identity) return null;

		return buildSidecarUrl(
			file.content_identity.uuid,
			sidecar.kind,
			sidecar.variant,
			sidecar.format
		);
	};

	return (
		<div className="no-scrollbar mask-fade-out flex flex-col space-y-4 overflow-x-hidden overflow-y-scroll px-2 pb-10 pt-2">
			<p className="text-sidebar-ink-dull text-xs">
				{t('inspector.file.derivativeFiles')}
			</p>

			{sidecars.length === 0 ? (
				<div className="text-sidebar-ink-dull flex items-center justify-center py-8 text-xs">
					{t('inspector.file.noSidecars')}
				</div>
			) : (
				<div className="space-y-2">
					{sidecars.map((sidecar, i) => (
						<SidecarItem
							key={i}
							sidecar={sidecar}
							file={file}
							sidecarUrl={getSidecarUrl(sidecar)}
							platform={platform}
							libraryId={libraryId}
						/>
					))}
				</div>
			)}
		</div>
	);
}

function SidecarItem({
	sidecar,
	file,
	sidecarUrl,
	platform,
	libraryId
}: {
	sidecar: any;
	file: File;
	sidecarUrl: string | null;
	platform: ReturnType<typeof usePlatform>;
	libraryId: string | null;
}) {
	const isDark = useIsDarkTheme();
	const isImage =
		(sidecar.kind === 'thumb' || sidecar.kind === 'thumbstrip') &&
		(sidecar.format === 'webp' ||
			sidecar.format === 'jpg' ||
			sidecar.format === 'png');

	// Get appropriate WingDrive icon based on sidecar format/kind
	const getSidecarIcon = () => {
		const format = String(sidecar.format).toLowerCase();

		// PLY files (3D mesh) use Mesh icon
		if (format === 'ply') {
			return getIcon('Mesh', isDark);
		}

		// Text files use Text icon
		if (format === 'text' || format === 'txt' || format === 'srt') {
			return getIcon('Text', isDark);
		}

		// Thumbs/thumbstrips use Image icon
		if (sidecar.kind === 'thumb' || sidecar.kind === 'thumbstrip') {
			return getIcon('Image', isDark);
		}

		// Default to Document icon
		return getIcon('Document', isDark);
	};

	const sidecarIcon = getSidecarIcon();

	const contextMenu = useContextMenu({
		items: [
			{
				icon: MagnifyingGlass,
				label: revealLabel(),
				onClick: async () => {
					if (
						platform.getSidecarPath &&
						platform.revealFile &&
						file.content_identity &&
						libraryId
					) {
						try {
							// Convert "text" format to "txt" extension (matches actual file on disk)
							const format =
								sidecar.format === 'text'
									? 'txt'
									: sidecar.format;
							const sidecarPath = await platform.getSidecarPath(
								libraryId,
								file.content_identity.uuid,
								sidecar.kind,
								sidecar.variant,
								format
							);

							await platform.revealFile(sidecarPath);
						} catch (err) {
							console.error('Failed to reveal sidecar:', err);
						}
					}
				},
				condition: () =>
					!!platform.getSidecarPath &&
					!!platform.revealFile &&
					!!file.content_identity &&
					!!libraryId
			}
		]
	});

	const handleContextMenu = async (e: React.MouseEvent) => {
		e.preventDefault();
		e.stopPropagation();
		await contextMenu.show(e);
	};

	return (
		<div
			onContextMenu={handleContextMenu}
			className="bg-app-box/40 border-app-line/50 hover:bg-app-box/60 flex cursor-default items-start gap-3 rounded-lg border p-2.5 transition-colors"
		>
			{/* Preview thumbnail for image sidecars */}
			{isImage && sidecarUrl ? (
				<div className="bg-app-box border-app-line size-12 shrink-0 overflow-hidden rounded border">
					<img
						src={sidecarUrl}
						alt={`${sidecar.variant} preview`}
						className="h-full w-full object-cover"
						onError={(e) => {
							// Fallback to icon on error
							e.currentTarget.style.display = 'none';
							if (e.currentTarget.nextElementSibling) {
								(
									e.currentTarget
										.nextElementSibling as HTMLElement
								).style.display = 'flex';
							}
						}}
					/>
					<div className="hidden h-full w-full items-center justify-center">
						<img
							src={sidecarIcon}
							alt=""
							className="size-6 object-contain"
						/>
					</div>
				</div>
			) : (
				<div className="bg-app-box border-app-line flex size-12 shrink-0 items-center justify-center rounded border">
					<img
						src={sidecarIcon}
						alt=""
						className="size-6 object-contain"
					/>
				</div>
			)}

			<div className="min-w-0 flex-1">
				<div className="text-sidebar-ink text-xs font-medium">
					{String(sidecar.kind)}
				</div>
				<div className="text-sidebar-ink-dull text-[11px]">
					{String(sidecar.variant)} · {formatBytes(sidecar.size)}
				</div>
				<div className="text-sidebar-ink-dull/70 mt-0.5 text-[10px]">
					{String(sidecar.format).toUpperCase()}
				</div>
			</div>
			{/* <span
				className={clsx(
					"text-[10px] font-semibold px-2 py-0.5 rounded-full shrink-0",
					sidecar.status === "ready" && "bg-accent/20 text-accent",
					sidecar.status === "pending" &&
						"bg-sidebar-ink-dull/20 text-sidebar-ink-dull",
				)}
			>
				{String(sidecar.status)}
			</span> */}
		</div>
	);
}

function InstancesTab({file}: {file: File}) {
	const {t} = useTranslation('inspector');
	// Query for alternate instances with full File data
	const instancesQuery = useNormalizedQuery<
		{entry_uuid: string},
		{instances: File[]; total_count: number}
	>({
		query: 'files.alternate_instances',
		input: {entry_uuid: file?.id || ''},
		resourceType: 'file',
		enabled: !!file?.id && !!file?.content_identity
	});

	const instances =
		(
			instancesQuery.data as
				{instances: File[]; total_count: number} | undefined
		)?.instances || [];

	// Query devices to get proper names and icons
	const devicesQuery = useNormalizedQuery<any, any[]>({
		query: 'devices.list',
		input: {
			include_offline: true,
			include_details: false,
			show_paired: true
		},
		resourceType: 'device'
	});

	const devices = devicesQuery.data || [];

	// Group instances by device_slug
	const instancesByDevice = instances.reduce(
		(acc, instance) => {
			let deviceSlug = 'unknown';
			if ('Physical' in instance.wing_path) {
				deviceSlug = instance.wing_path.Physical.device_slug;
			} else if ('Cloud' in instance.wing_path) {
				deviceSlug = 'cloud';
			}

			if (!acc[deviceSlug]) {
				acc[deviceSlug] = [];
			}
			acc[deviceSlug].push(instance);
			return acc;
		},
		{} as Record<string, File[]>
	);

	const getDeviceName = (deviceSlug: string) => {
		const device = devices.find((d) => d.slug === deviceSlug);
		return device?.name || deviceSlug;
	};

	const getDeviceInfo = (deviceSlug: string) => {
		return devices.find((d) => d.slug === deviceSlug);
	};

	if (instancesQuery.isLoading) {
		return (
			<div className="text-sidebar-ink-dull flex items-center justify-center py-8 text-xs">
				{t("ui.loadingInstances")}
			</div>
		);
	}

	if (!file.content_identity) {
		return (
			<div className="no-scrollbar mask-fade-out flex flex-col space-y-4 overflow-x-hidden overflow-y-scroll px-2 pb-10 pt-2">
				<p className="text-sidebar-ink-dull text-xs">
					This file has not been content-hashed yet. Instances will
					appear after indexing completes.
				</p>
			</div>
		);
	}

	return (
		<div className="no-scrollbar mask-fade-out flex flex-col space-y-5 overflow-x-hidden overflow-y-scroll px-2 pb-10 pt-2">
			<p className="text-sidebar-ink-dull text-xs">
				{t("ui.allCopiesOfThisFileAcross")}
			</p>

			{instances.length === 0 || instances.length === 1 ? (
				<div className="text-sidebar-ink-dull flex items-center justify-center py-8 text-xs">
					{t("ui.noAlternateInstancesFound")}
				</div>
			) : (
				<div className="space-y-4">
					{Object.entries(instancesByDevice).map(
						([deviceSlug, deviceInstances]: [string, File[]]) => {
							const deviceInfo = getDeviceInfo(deviceSlug);
							const deviceName = getDeviceName(deviceSlug);

							return (
								<div key={deviceSlug} className="space-y-1">
									{/* Device Header */}
									<div className="flex items-center gap-2 px-2">
										<img
											src={getDeviceIcon(deviceInfo)}
											className="size-4 shrink-0"
											alt=""
										/>
										<span className="text-sidebar-ink text-xs font-semibold">
											{deviceName}
										</span>
										<div className="flex-1" />
										<div className="bg-app-box border-app-line text-sidebar-ink-dull flex size-5 items-center justify-center rounded-full border text-[10px] font-semibold">
											{deviceInstances.length}
										</div>
									</div>

									{/* List of instances */}
									<div className="space-y-0.5">
										{deviceInstances.map((instance, i) => (
											<InstanceRow
												key={i}
												instance={instance}
											/>
										))}
									</div>
								</div>
							);
						}
					)}
				</div>
			)}
		</div>
	);
}

function InstanceRow({instance}: {instance: File}) {
	const {t} = useTranslation('inspector');
	const displaySize = fileDisplaySize(instance);
	const getPathDisplay = (sdPath: typeof instance.wing_path) => {
		if ('Physical' in sdPath) {
			return sdPath.Physical.path;
		} else if ('Cloud' in sdPath) {
			return sdPath.Cloud.path;
		} else {
			return t('inspector.location.content');
		}
	};

	const formatDate = (dateStr: string) => {
		const date = new Date(dateStr);
		return date.toLocaleDateString(uiLocale(), {
			month: 'short',
			day: 'numeric'
		});
	};

	return (
		<div
			className="hover:bg-app-box/40 flex cursor-default items-center gap-2 rounded-md px-2 py-1.5 transition-colors"
			title={getPathDisplay(instance.wing_path)}
		>
			{/* Thumbnail */}
			<div className="flex-shrink-0 [&_*]:!rounded-[3px]">
				<FileComponent.Thumb file={instance} size={20} />
			</div>

			{/* File info */}
			<div className="flex min-w-0 flex-1 items-center gap-2">
				<span className="text-sidebar-ink truncate text-xs">
					{instance.name}
					{instance.extension && `.${instance.extension}`}
				</span>
			</div>

			{/* Metadata */}
			<div className="flex shrink-0 items-center gap-2">
				{/* Tags */}
				{instance.tags && instance.tags.length > 0 && (
					<div
						className="flex items-center gap-0.5"
						title={instance.tags
							.map((t) => t.canonical_name)
							.join(', ')}
					>
						{instance.tags.slice(0, 3).map((tag) => (
							<div
								key={tag.id}
								className="size-1.5 rounded-full"
								style={{
									backgroundColor: tag.color || '#3B82F6'
								}}
							/>
						))}
						{instance.tags.length > 3 && (
							<span className="text-ink-faint text-[9px] font-medium">
								+{instance.tags.length - 3}
							</span>
						)}
					</div>
				)}

				{/* Modified date */}
				<span className="text-sidebar-ink-dull text-[10px]">
					{formatDate(instance.modified_at)}
				</span>

				{/* Size */}
				{displaySize !== null && (
					<span className="text-sidebar-ink-dull min-w-[50px] text-right text-[10px]">
						{formatBytes(displaySize)}
					</span>
				)}

				{/* Local indicator */}
				<div
					className={clsx(
						'size-1.5 rounded-full',
						instance.is_local
							? 'bg-accent'
							: 'bg-sidebar-ink-dull/40'
					)}
					title={instance.is_local ? t('inspector.file.availableLocally') : t('inspector.file.remote')}
				/>
			</div>
		</div>
	);
}

function ChatTab() {
const {t} = useTranslation('inspector');
	const [message, setMessage] = useState('');

	const messages = [
		{
			id: 1,
			sender: t('sampleChat.sarah'),
			avatar: 'S',
			content: t('sampleChat.question'),
			time: t('sampleChat.time1'),
			isUser: false
		},
		{
			id: 2,
			sender: t('sampleChat.you'),
			avatar: 'J',
			content: t('sampleChat.synced'),
			time: t('sampleChat.time2'),
			isUser: true
		},
		{
			id: 3,
			sender: t('sampleChat.assistant'),
			avatar: '',
			content:
				t('sampleChat.suggestCollection'),
			time: t('sampleChat.time3'),
			isUser: false,
			isAI: true,
			unread: true
		},
		{
			id: 4,
			sender: t('sampleChat.sarah'),
			avatar: 'S',
			content: t('sampleChat.thanks'),
			time: t('sampleChat.time4'),
			isUser: false,
			unread: true
		},
		{
			id: 5,
			sender: t('sampleChat.alex'),
			avatar: 'A',
			content: t('sampleChat.tagged'),
			time: t('sampleChat.time5'),
			isUser: false,
			unread: true
		}
	];

	return (
		<div className="flex h-full flex-col">
			{/* Messages */}
			<div className="flex-1 space-y-3 overflow-y-auto px-2 pt-2">
				{messages.map((msg) => (
					<div
						key={msg.id}
						className={clsx(
							'flex gap-2',
							msg.isUser ? 'flex-row-reverse' : 'flex-row'
						)}
					>
						{/* Avatar */}
						<div
							className={clsx(
								'flex size-6 shrink-0 items-center justify-center rounded-full text-[10px] font-bold',
								msg.isAI
									? 'bg-accent/20 text-accent'
									: msg.isUser
										? 'bg-sidebar-selected text-sidebar-ink'
										: 'bg-app-box text-sidebar-ink-dull'
							)}
						>
							{msg.avatar}
						</div>

						{/* Message bubble */}
						<div
							className={clsx(
								'flex max-w-[75%] flex-col',
								msg.isUser ? 'items-end' : 'items-start'
							)}
						>
							<div
								className={clsx(
									'rounded-lg px-2.5 py-1.5',
									msg.isAI
										? 'bg-accent/10 border-accent/20 border'
										: msg.isUser
											? 'bg-sidebar-selected/60'
											: 'bg-app-box/60',
									msg.unread && 'ring-accent/50 ring-1'
								)}
							>
								{!msg.isUser && (
									<div
										className={clsx(
											'mb-0.5 text-[10px] font-semibold',
											msg.isAI
												? 'text-accent'
												: 'text-sidebar-ink-dull'
										)}
									>
										{msg.sender}
									</div>
								)}
								<p className="text-sidebar-ink text-xs leading-relaxed">
									{msg.content}
								</p>
							</div>
							<span className="text-sidebar-ink-dull mt-0.5 px-1 text-[10px]">
								{msg.time}
							</span>
						</div>
					</div>
				))}
			</div>

			{/* Input */}
			<div className="border-sidebar-line space-y-2 border-t p-2">
				<div className="flex items-end gap-1.5">
					<button
						className="hover:bg-sidebar-selected text-sidebar-ink-dull hover:text-sidebar-ink rounded-lg p-1.5 transition-colors"
						title={t('inspector.preview.attachFile')}
					>
						<Paperclip size={4} weight="bold" />
					</button>

					<div className="bg-app-box border-app-line flex flex-1 items-center gap-1.5 rounded-lg border px-2 py-1.5">
						<input
							type="text"
							value={message}
							onChange={(e) => setMessage(e.target.value)}
							placeholder={t('inspector.preview.typeMessage')}
							className="text-sidebar-ink placeholder:text-sidebar-ink-dull flex-1 bg-transparent text-xs outline-none"
						/>
					</div>

					<button
						className="bg-accent hover:bg-accent/90 rounded-lg p-1.5 text-white transition-colors"
						title={t('inspector.preview.sendMessage')}
					>
						<PaperPlaneRight size={4} weight="bold" />
					</button>
				</div>

				<div className="flex gap-1">
					<button className="text-sidebar-ink-dull hover:text-sidebar-ink bg-app-box/40 hover:bg-app-box/60 flex items-center gap-1 rounded-md px-2 py-1 text-[10px] font-medium transition-colors">
						<Sparkle size={3} weight="bold" />
						Ask AI
					</button>
					<button className="text-sidebar-ink-dull hover:text-sidebar-ink bg-app-box/40 hover:bg-app-box/60 rounded-md px-2 py-1 text-[10px] font-medium transition-colors">
						{t("ui.shareFile")}
					</button>
					<button className="text-sidebar-ink-dull hover:text-sidebar-ink bg-app-box/40 hover:bg-app-box/60 rounded-md px-2 py-1 text-[10px] font-medium transition-colors">
						{t("ui.createTask")}
					</button>
				</div>
			</div>
		</div>
	);
}

function ActivityTab() {
	const {t} = useTranslation('inspector');
	const activity = [
		{action: t('sampleChat.syncedToNas'), time: t('inspector.location.minutesAgo', { count: 2 }), device: 'MacBook Pro'},
		{action: t('sampleChat.uploadedToS3'), time: t('inspector.location.hoursAgo', { count: 1 }), device: 'MacBook Pro'},
		{
			action: t('sampleChat.thumbnailGenerated'),
			time: t('inspector.location.hoursAgo', { count: 2 }),
			device: 'MacBook Pro'
		},
		{action: t('sampleChat.taggedTravel'), time: t('inspector.location.hoursAgo', { count: 3 }), device: 'iPhone'},
		{action: t('sampleChat.created'), time: t('inspector.location.addedDate'), device: 'iPhone'}
	];

	return (
		<div className="no-scrollbar mask-fade-out flex flex-col space-y-4 overflow-x-hidden overflow-y-scroll px-2 pb-10 pt-2">
			<p className="text-sidebar-ink-dull text-xs">
				{t('sampleChat.history')}
			</p>

			<div className="space-y-0.5">
				{activity.map((item, i) => (
					<div
						key={i}
						className="hover:bg-app-box/40 flex items-start gap-3 rounded-lg p-2 transition-colors"
					>
						<span className="text-sidebar-ink-dull mt-0.5 shrink-0">
							<ClockCounterClockwise size={16} weight="bold" />
						</span>
						<div className="min-w-0 flex-1">
							<div className="text-sidebar-ink text-xs">
								{item.action}
							</div>
							<div className="text-sidebar-ink-dull mt-0.5 text-[11px]">
								{item.time} · {item.device}
							</div>
						</div>
					</div>
				))}
			</div>
		</div>
	);
}

function DetailsTab({file}: {file: File}) {
const {t} = useTranslation('inspector');
	return (
		<div className="no-scrollbar mask-fade-out flex flex-col space-y-5 overflow-x-hidden overflow-y-scroll pb-10">
			{/* Content Identity */}
			{file.content_identity && (
				<Section title={t('inspector.file.contentIdentity')} icon={Fingerprint}>
					<InfoRow
						label={t('inspector.file.contentHash')}
						value={
							String(file.content_identity.content_hash).slice(
								0,
								16
							) + '...'
						}
						mono
					/>
					{file.content_identity.integrity_hash && (
						<InfoRow
							label={t('inspector.file.integrityHash')}
							value={
								String(
									file.content_identity.integrity_hash
								).slice(0, 16) + '...'
							}
							mono
						/>
					)}
					{file.content_identity.mime_type_id !== null && (
						<InfoRow
							label={t('inspector.file.mimeTypeId')}
							value={String(file.content_identity.mime_type_id)}
						/>
					)}
				</Section>
			)}

			{/* Metadata */}
			<Section title={t('inspector.file.metadata')} icon={Hash}>
				<InfoRow
					label={t('inspector.file.fileId')}
					value={String(file.id).slice(0, 8) + '...'}
					mono
				/>
				<InfoRow
					label={t('inspector.file.contentKind')}
					value={String(file.content_kind || t('inspector.location.unknown'))}
				/>
				{file.extension && (
					<InfoRow label={t('inspector.file.extension')} value={String(file.extension)} />
				)}
			</Section>

			{/* System */}
			<Section title={t('inspector.file.system')} icon={DotsThree}>
				<InfoRow label={t('inspector.file.entryKind')} value={t(`kinds.${file.kind}`, {defaultValue: file.kind})} />
				<InfoRow label={t('inspector.file.local')} value={file.is_local ? t('inspector.file.yes') : t('inspector.file.no')} />
				<InfoRow
					label={t('inspector.file.instancesCount')}
					value={String((file.alternate_paths?.length || 0) + 1)}
				/>
			</Section>
		</div>
	);
}
