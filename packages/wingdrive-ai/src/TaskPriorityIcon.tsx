import { ArrowDown, ArrowUp, Equals, Warning } from "@phosphor-icons/react";
import clsx from "clsx";

import type { TaskPriority } from "./types";

const config: Record<
	TaskPriority,
	{ icon: React.ElementType; weight: "fill" | "bold"; color: string }
> = {
	critical: { icon: Warning, weight: "fill", color: "text-red-400" },
	high: { icon: ArrowUp, weight: "bold", color: "text-amber-400" },
	medium: { icon: Equals, weight: "bold", color: "text-ink-dull" },
	low: { icon: ArrowDown, weight: "bold", color: "text-ink-faint" },
};

export interface TaskPriorityIconProps {
	priority: TaskPriority;
	size?: number;
	className?: string;
}

export function TaskPriorityIcon({ priority, size = 14, className }: TaskPriorityIconProps) {
	const { icon: Icon, weight, color } = config[priority];
	return <Icon size={size} weight={weight} className={clsx(color, className)} />;
}
