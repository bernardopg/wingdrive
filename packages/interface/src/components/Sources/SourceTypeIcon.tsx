interface SourceTypeIconProps {
	type: string;
	svg?: string | null;
	size?: "sm" | "md" | "lg";
}

const SIZE_CLASSES = {
	sm: "h-5 w-5 p-0.5",
	md: "h-8 w-8 p-1.5",
	lg: "h-10 w-10 p-2",
};

const FALLBACK_LABELS: Record<string, string> = {
	email: "mail",
	file: "doc",
	note: "note",
	bookmark: "link",
	history: "time",
	markdown: "md",
	session: "term",
};

export function SourceTypeIcon({
	type: typeName,
	svg,
	size = "md",
}: SourceTypeIconProps) {
	if (svg) {
		// An <img> never runs scripts or loads resources from an SVG, so an adapter's icon
		// cannot reach the app even if it slipped past the registry's checks.
		return (
			<img
				alt=""
				src={`data:image/svg+xml;charset=utf-8,${encodeURIComponent(svg)}`}
				className={`shrink-0 rounded-lg object-contain ${SIZE_CLASSES[size]}`}
			/>
		);
	}

	const label = FALLBACK_LABELS[typeName] ?? typeName.slice(0, 4);

	return (
		<div
			className={`bg-accent/10 text-accent flex shrink-0 items-center justify-center rounded-lg font-mono text-[10px] font-medium uppercase ${SIZE_CLASSES[size]}`}
		>
			{label}
		</div>
	);
}
