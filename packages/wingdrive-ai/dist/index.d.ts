import * as react from 'react';
import { ReactNode } from 'react';
import { SelectPillProps, usePopover } from '@wingdrive/primitives';

type ToolCallStatus = 'running' | 'completed' | 'error';
interface ToolCallPair {
    id: string;
    name: string;
    argsRaw: string;
    args: Record<string, unknown> | null;
    resultRaw: string | null;
    result: Record<string, unknown> | null;
    status: ToolCallStatus;
    /** Human-readable summary provided by live opencode parts */
    title?: string | null;
}
type ActionContent = {
    type: 'text';
    text: string;
} | {
    type: 'tool_call';
    id: string;
    name: string;
    args: string;
};
type TranscriptStep = {
    type: 'action';
    content: ActionContent[];
} | {
    type: 'user_text';
    text: string;
} | {
    type: 'system_text';
    text: string;
} | {
    type: 'tool_result';
    call_id: string;
    name: string;
    text: string;
};
type TranscriptItem = {
    kind: 'text';
    text: string;
} | {
    kind: 'tool';
    pair: ToolCallPair;
};
type TaskStatus = "pending_approval" | "backlog" | "ready" | "in_progress" | "done";
type TaskPriority = "critical" | "high" | "medium" | "low";
interface Subtask {
    title: string;
    completed: boolean;
}
interface Task {
    id: string;
    task_number: number;
    title: string;
    description?: string | null;
    status: TaskStatus;
    priority: TaskPriority;
    owner_agent_id: string;
    assigned_agent_id: string;
    subtasks: Subtask[];
    metadata: unknown;
    worker_id?: string | null;
    created_by: string;
    created_at: string;
    updated_at: string;
    completed_at?: string | null;
}
declare const TASK_STATUS_ORDER: TaskStatus[];
declare const TASK_STATUS_LABEL: Record<TaskStatus, string>;
declare const TASK_PRIORITY_LABEL: Record<TaskPriority, string>;
interface TaskInfo {
    id: string;
    title: string;
    status: string;
    priority: string;
    assignees: string[];
    conversation_id?: string;
}
interface MemoryInfo {
    id: string;
    type: string;
    content: string;
    source?: string;
    edges?: Array<{
        target: string;
        relation: string;
    }>;
}
interface CronJobInfo {
    id: string;
    name: string;
    schedule: string;
    last_run?: string;
    next_run?: string;
    status: string;
}
interface AgentInfo {
    id: string;
    name: string;
    detail: string;
    status?: string;
}
interface ModelOption {
    id: string;
    name: string;
    provider: string;
    context_window?: number;
    capabilities?: string[];
}
declare function tryParseJson(text: string): Record<string, unknown> | null;
declare function isErrorResult(text: string, parsed: Record<string, unknown> | null): boolean;
/**
 * Walk a flat TranscriptStep[] and pair each tool_call with its tool_result
 * via call_id matching, plus emit standalone text steps. Returns an ordered
 * list of renderable items: text blocks and paired tool calls.
 */
declare function pairTranscriptSteps(steps: TranscriptStep[]): TranscriptItem[];

declare function ToolCall({ pair }: {
    pair: ToolCallPair;
}): react.JSX.Element;

interface MarkdownProps {
    content: string;
    className?: string;
}
declare const Markdown: react.ForwardRefExoticComponent<MarkdownProps & react.RefAttributes<HTMLDivElement>>;

interface MessageBubbleProps {
    content: string;
    isUser: boolean;
    isStreaming?: boolean;
    onCopy?: (content: string) => void;
}
declare function MessageBubble({ content, isUser, isStreaming, onCopy }: MessageBubbleProps): react.JSX.Element;

interface InlineWorkerCardProps {
    title: string;
    status: string;
    toolCallCount: number;
    liveStatus?: string | null;
    transcript: TranscriptStep[];
    isTranscriptLoading?: boolean;
    onCopyLogs?: () => void;
    onCancel?: () => void;
    className?: string;
}
declare function InlineWorkerCard({ title, status, toolCallCount, liveStatus, transcript, isTranscriptLoading, onCopyLogs, onCancel, className, }: InlineWorkerCardProps): react.JSX.Element;

interface InlineBranchCardProps {
    description: string;
    completedAt: string | null;
    conclusion?: string | null;
    className?: string;
}
declare function InlineBranchCard({ description, completedAt, conclusion, className, }: InlineBranchCardProps): react.JSX.Element;

interface ModelSelectorProps {
    /** Available models */
    models: ModelOption[];
    /** Currently selected model ID */
    value: string;
    /** Called when a model is selected */
    onChange: (value: string) => void;
    /** Placeholder when nothing selected */
    placeholder?: string;
    /** Search placeholder */
    searchPlaceholder?: string;
    /** SelectPill variant */
    variant?: SelectPillProps["variant"];
    /** SelectPill size */
    size?: SelectPillProps["size"];
    /** Controlled popover (optional) */
    popover?: ReturnType<typeof usePopover>;
    /** Additional trigger className */
    className?: string;
    /** Disabled state */
    disabled?: boolean;
}
declare const ModelSelector: react.ForwardRefExoticComponent<ModelSelectorProps & react.RefAttributes<HTMLButtonElement>>;

interface ChatComposerProps {
    /** Current draft text */
    draft: string;
    /** Called when draft changes */
    onDraftChange: (value: string) => void;
    /** Called when user hits send (Enter or send button) */
    onSend: () => void;
    /** Placeholder for the textarea */
    placeholder?: string;
    /** Show an optional heading above the composer */
    heading?: ReactNode;
    /** Disable the composer — draft can still change, but send/voice are blocked */
    isSending?: boolean;
    /** Project selector config — omit to hide */
    projectSelector?: {
        value: string;
        options: string[];
        onChange: (project: string) => void;
        popover: ReturnType<typeof usePopover>;
    };
    /** Model selector config — omit to hide */
    modelSelector?: {
        value: string;
        options: ModelOption[];
        onChange: (model: string) => void;
    };
    /** Voice button handler — omit to hide */
    onOpenVoice?: () => void;
    /** Optional content rendered at the far right of the toolbar (before send) */
    toolbarExtra?: ReactNode;
}
/**
 * Generic chat composer with an expanding textarea, optional project pill,
 * model selector, voice button, and an animated send button.
 */
declare function ChatComposer({ draft, onDraftChange, onSend, placeholder, heading, isSending, projectSelector, modelSelector, onOpenVoice, toolbarExtra, }: ChatComposerProps): react.JSX.Element;

interface TaskStatusIconProps {
    status: TaskStatus;
    size?: number;
    className?: string;
}
declare function TaskStatusIcon({ status, size, className }: TaskStatusIconProps): react.JSX.Element;

interface TaskPriorityIconProps {
    priority: TaskPriority;
    size?: number;
    className?: string;
}
declare function TaskPriorityIcon({ priority, size, className }: TaskPriorityIconProps): react.JSX.Element;

interface TaskRowProps {
    task: Task;
    onClick?: (task: Task) => void;
    isActive?: boolean;
    resolveAgentName?: (agentId: string) => string;
    onStatusChange?: (task: Task, status: TaskStatus) => void;
    onDelete?: (task: Task) => void;
    className?: string;
}
declare function TaskRow({ task, onClick, isActive, resolveAgentName, onStatusChange, onDelete, className, }: TaskRowProps): react.JSX.Element;

interface TaskListProps {
    tasks: Task[];
    groups?: TaskStatus[];
    collapsedGroups?: Set<TaskStatus>;
    onToggleGroup?: (status: TaskStatus) => void;
    activeTaskId?: string;
    onTaskClick: (task: Task) => void;
    onStatusChange?: (task: Task, status: TaskStatus) => void;
    onDelete?: (task: Task) => void;
    resolveAgentName?: (agentId: string) => string;
    className?: string;
}
declare function TaskList({ tasks, groups, collapsedGroups, onToggleGroup, activeTaskId, onTaskClick, onStatusChange, onDelete, resolveAgentName, className, }: TaskListProps): react.JSX.Element;

interface TaskDetailProps {
    task: Task;
    resolveAgentName?: (agentId: string) => string;
    onStatusChange?: (task: Task, status: TaskStatus) => void;
    onSubtaskToggle?: (task: Task, index: number, completed: boolean) => void;
    onDelete?: (task: Task) => void;
    onClose?: () => void;
    className?: string;
    /** Rendered between the property row and the subtask list — for
     * host-app sections like an execution plan. */
    beforeSubtasks?: React.ReactNode;
}
declare function TaskDetail({ task, resolveAgentName, onStatusChange, onSubtaskToggle, onDelete, onClose, className, beforeSubtasks, }: TaskDetailProps): react.JSX.Element;

interface TaskCreateFormData {
    title: string;
    description: string;
    priority: TaskPriority;
}
interface TaskCreateFormProps {
    onSubmit: (data: TaskCreateFormData) => void;
    onCancel?: () => void;
    defaultPriority?: TaskPriority;
    isSubmitting?: boolean;
    className?: string;
}
declare function TaskCreateForm({ onSubmit, onCancel, defaultPriority, isSubmitting, className, }: TaskCreateFormProps): react.JSX.Element;

export { type ActionContent, type AgentInfo, ChatComposer, type ChatComposerProps, type CronJobInfo, InlineBranchCard, type InlineBranchCardProps, InlineWorkerCard, type InlineWorkerCardProps, Markdown, type MemoryInfo, MessageBubble, type MessageBubbleProps, type ModelOption, ModelSelector, type ModelSelectorProps, type Subtask, TASK_PRIORITY_LABEL, TASK_STATUS_LABEL, TASK_STATUS_ORDER, type Task, TaskCreateForm, type TaskCreateFormData, type TaskCreateFormProps, TaskDetail, type TaskDetailProps, type TaskInfo, TaskList, type TaskListProps, type TaskPriority, TaskPriorityIcon, type TaskPriorityIconProps, TaskRow, type TaskRowProps, type TaskStatus, TaskStatusIcon, type TaskStatusIconProps, ToolCall, type ToolCallPair, type ToolCallStatus, type TranscriptItem, type TranscriptStep, isErrorResult, pairTranscriptSteps, tryParseJson };
