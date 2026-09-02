import {
	CheckCircle,
	Circle,
	CircleDashed,
	CircleHalf,
	Clock,
} from "@phosphor-icons/react";
import clsx from "clsx";

import type { TaskStatus } from "./types";

const config: Record<
	TaskStatus,
	{ icon: React.ElementType; weight: "fill" | "bold"; color: string }
> = {
	done: { icon: CheckCircle, weight: "fill", color: "text-emerald-400" },
	in_progress: { icon: CircleHalf, weight: "fill", color: "text-violet-400" },
	ready: { icon: Circle, weight: "fill", color: "text-accent" },
	pending_approval: { icon: Clock, weight: "fill", color: "text-amber-400" },
	backlog: { icon: CircleDashed, weight: "bold", color: "text-ink-faint" },
};

export interface TaskStatusIconProps {
	status: TaskStatus;
	size?: number;
	className?: string;
}

export function TaskStatusIcon({ status, size = 16, className }: TaskStatusIconProps) {
	const { icon: Icon, weight, color } = config[status];
	return (
		<Icon
			size={size}
			weight={weight}
			className={clsx(color, className)}
		/>
	);
}
