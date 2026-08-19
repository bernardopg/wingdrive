import { motion } from "framer-motion";
import { Camera, HardDrive, Plus, Tag, X } from "@phosphor-icons/react";
import type { File } from "@sd/ts-client";
import { getContentKind } from "@sd/ts-client";
import { TagPill, TagSelectorButton } from "../Tags";
import { formatBytes } from "../../routes/explorer/utils";
import {
	useLibraryMutation,
} from "../../contexts/SpacedriveContext";
import { useRefetchTagQueries } from "../../hooks/useRefetchTagQueries";
import { toast } from "@spacedrive/primitives";

interface MetadataPanelProps {
	file: File;
	onClose: () => void;
}

/**
 * Slide-in metadata panel for the fullscreen preview.
 *
 * Shows dimensions, media data (camera, duration, codecs), timestamps and
 * tags in a dark glass panel so details never distract from the preview.
 */
export function MetadataPanel({ file, onClose }: MetadataPanelProps) {
	const refetchTagQueries = useRefetchTagQueries();
	const applyTag = useLibraryMutation("tags.apply", {
		onSuccess: refetchTagQueries,
	});
	const unapplyTags = useLibraryMutation("tags.unapply", {
		onSuccess: refetchTagQueries,
	});

	const kind = getContentKind(file);
	const isImage = kind === "image" && file.image_media_data;
	const isVideo = kind === "video" && file.video_media_data;
	const isAudio = kind === "audio" && file.audio_media_data;

	const dimensionLine = isImage
		? `${file.image_media_data!.width} × ${file.image_media_data!.height}`
		: isVideo
			? `${file.video_media_data!.width} × ${file.video_media_data!.height}`
			: null;

	const duration =
		isVideo && file.video_media_data!.duration_seconds != null
			? formatDuration(file.video_media_data!.duration_seconds)
			: isAudio && file.audio_media_data!.duration_seconds != null
				? formatDuration(file.audio_media_data!.duration_seconds)
				: null;

	const camera = isImage
		? file.image_media_data!.camera_make || file.image_media_data!.camera_model
		: null;
	const exposure = isImage
		? [
				file.image_media_data!.focal_length &&
					`${file.image_media_data!.focal_length}mm`,
				file.image_media_data!.aperture &&
					`ƒ/${file.image_media_data!.aperture}`,
				file.image_media_data!.shutter_speed &&
					`${file.image_media_data!.shutter_speed}s`,
				file.image_media_data!.iso &&
					`ISO ${file.image_media_data!.iso}`,
			].filter(Boolean)
		: [];

	const videoCodec = isVideo ? file.video_media_data!.codec : null;
	const audioCodec = isAudio ? file.audio_media_data!.codec : null;

	const physicalPath =
		"Physical" in file.sd_path ? file.sd_path.Physical.path : null;

	return (
		<motion.aside
			initial={{ x: 40, opacity: 0 }}
			animate={{ x: 0, opacity: 1 }}
			exit={{ x: 40, opacity: 0 }}
			transition={{ duration: 0.18, ease: "easeOut" }}
			className="absolute right-0 top-0 z-20 flex h-full w-[300px] flex-col border-l border-white/10 bg-black/70 backdrop-blur-2xl"
		>
			{/* Header */}
			<div className="flex items-center justify-between border-b border-white/10 px-4 py-3">
				<div className="text-sm font-medium text-white/90">
					Details
				</div>
				<button
					onClick={onClose}
					className="rounded-md p-1.5 text-white/60 transition-colors hover:bg-white/10 hover:text-white"
				>
					<X size={14} weight="bold" />
				</button>
			</div>

			{/* Scrollable content */}
			<div className="flex-1 space-y-5 overflow-y-auto px-4 py-4 text-sm">
				{/* Tags */}
				<section className="space-y-2">
					<SectionLabel icon={<Tag size={13} weight="fill" />}>
						Tags
					</SectionLabel>
					<div className="flex flex-wrap items-center gap-1.5">
						{file.tags && file.tags.length > 0
							? file.tags.map((tag) => (
									<TagPill
										key={tag.id}
										color={tag.color || "#3B82F6"}
										size="xs"
										onRemove={async () => {
											try {
												await unapplyTags.mutateAsync({
													entry_ids: [file.id],
													tag_ids: [tag.id],
												});
											} catch (err) {
												toast.error(
													`Failed to remove tag: ${err}`,
												);
											}
										}}
									>
										{tag.canonical_name}
									</TagPill>
								))
							: null}
						<TagSelectorButton
							onSelect={async (tag) => {
								try {
									await applyTag.mutateAsync({
										targets: file.content_identity?.uuid
											? {
													type: "Content",
													ids: [
														file.content_identity.uuid,
													],
												}
											: {
													type: "EntryUuid",
													ids: [file.id],
												},
										tag_ids: [tag.id],
										source: "User",
										confidence: 1.0,
										applied_context: null,
										instance_attributes: null,
									});
								} catch (err) {
									toast.error(
										`Failed to add tag: ${err}`,
									);
								}
							}}
							contextTags={file.tags || []}
							fileId={file.id}
							contentId={file.content_identity?.uuid}
							trigger={
								<button className="flex items-center gap-1 rounded-full border border-white/15 bg-white/5 px-2 py-0.5 text-[11px] font-medium text-white/70 transition-colors hover:bg-white/10 hover:text-white">
									<Plus size={10} weight="bold" />
									Add tags
								</button>
							}
						/>
					</div>
				</section>

				{/* Media specifics */}
				{(dimensionLine || duration || videoCodec || audioCodec) && (
					<section className="space-y-2">
						<SectionLabel
							icon={<Camera size={13} weight="fill" />}
						>
							Media
						</SectionLabel>
						<dl className="space-y-1.5">
							{dimensionLine && (
								<Row label="Dimensions">
									{dimensionLine}
								</Row>
							)}
							{duration && <Row label="Duration">{duration}</Row>}
							{videoCodec && (
								<Row label="Codec">{videoCodec}</Row>
							)}
							{audioCodec && (
								<Row label="Codec">{audioCodec}</Row>
							)}
						</dl>
						{camera && (
							<div className="mt-2 flex items-center gap-2 text-white/70">
								<span className="text-xs">{camera}</span>
							</div>
						)}
						{exposure.length > 0 && (
							<div className="flex flex-wrap gap-1.5">
								{exposure.map((part) => (
									<span
										key={part}
										className="rounded-md bg-white/10 px-1.5 py-0.5 text-[11px] text-white/70"
									>
										{part}
									</span>
								))}
							</div>
						)}
					</section>
				)}

				{/* File facts */}
				<section className="space-y-2">
					<SectionLabel
						icon={<HardDrive size={13} weight="fill" />}
					>
						File
					</SectionLabel>
					<dl className="space-y-1.5">
						{file.size > 0 && (
							<Row label="Size">
								{formatBytes(file.size)}
							</Row>
						)}
						<Row label="Created">
							{formatDate(file.created_at)}
						</Row>
						<Row label="Modified">
							{formatDate(file.modified_at)}
						</Row>
						{file.content_identity?.content_hash && (
							<Row label="Content hash" mono>
								{shortHash(
									file.content_identity.content_hash,
								)}
							</Row>
						)}
					</dl>
				</section>

				{/* Location */}
				{physicalPath && (
					<section className="space-y-2">
						<SectionLabel
							icon={<HardDrive size={13} weight="fill" />}
						>
							Location
						</SectionLabel>
						<div className="break-all rounded-md bg-white/5 px-2.5 py-2 text-xs leading-relaxed text-white/60">
							{physicalPath}
						</div>
					</section>
				)}
			</div>
		</motion.aside>
	);
}

function SectionLabel({
	icon,
	children,
}: {
	icon: React.ReactNode;
	children: React.ReactNode;
}) {
	return (
		<div className="flex items-center gap-1.5 text-[11px] font-semibold uppercase tracking-wider text-white/45">
			{icon}
			{children}
		</div>
	);
}

function Row({
	label,
	children,
	mono,
}: {
	label: string;
	children: React.ReactNode;
	mono?: boolean;
}) {
	return (
		<div className="flex items-baseline justify-between gap-3">
			<dt className="flex-shrink-0 text-xs text-white/45">{label}</dt>
			<dd
				className={`truncate text-right text-xs text-white/85 ${mono ? "font-mono" : ""}`}
			>
				{children}
			</dd>
		</div>
	);
}

function formatDuration(totalSeconds: number): string {
	const minutes = Math.floor(totalSeconds / 60);
	const seconds = Math.round(totalSeconds % 60);
	return `${minutes}:${String(seconds).padStart(2, "0")}`;
}

function formatDate(date: string): string {
	return new Date(date).toLocaleDateString(undefined, {
		year: "numeric",
		month: "short",
		day: "numeric",
	});
}

function shortHash(hash: string): string {
	return hash.length > 12 ? `${hash.slice(0, 6)}…${hash.slice(-6)}` : hash;
}