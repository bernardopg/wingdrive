//! # Spacebot API Client Stub
//!
//! Stand-in for `@spacebot/api-client` when the private spacebot repo is not
//! checked out next to this one. The Spacebot UI is optional, so the desktop
//! build must still link without it. Previously the build externalized the
//! bare specifier, which left an unresolvable `import ... from
//! "@spacebot/api-client"` in the bundle and killed the whole app at load
//! time with a blank window.
//!
//! Only runtime values need stubbing. Every other import from this module in
//! `packages/interface` is `import type`, which is erased at compile time.

export interface TtsProfile {
	id: string;
	name: string;
	voice_id: string;
	language: string;
}

export interface WorkerListItem {
	id: string;
	task: string;
	status: string;
	tool_calls: number;
	live_status: string | null;
	worker_type: string;
	channel_id: string | null;
	channel_name: string | null;
	started_at: string;
	completed_at: string | null;
	has_transcript: boolean;
	opencode_port: number | null;
	opencode_session_id: string | null;
	directory: string | null;
	interactive: boolean;
	project_id: string | null;
	project_name: string | null;
}

export interface Task {
	id: string;
	task_number: number;
	title: string;
	description: string;
	status: string;
	priority: string;
	assignee: string | null;
	created_at: string;
	updated_at: string;
	completed_at: string | null;
}

export interface UpdateTaskRequest {
	title?: string;
	description?: string;
	status?: string;
	priority?: string;
	assignee?: string | null;
}

export interface TimelineItem {
	[key: string]: unknown;
}

export interface PortalConversationSummary {
	[key: string]: unknown;
}

export interface PortalHistoryMessage {
	[key: string]: unknown;
}

export interface PortalConversationResponse {
	conversation: PortalConversationSummary;
	messages: PortalHistoryMessage[];
}

export interface InboundMessageEvent {
	type: 'inbound_message';
	message: string;
}

export interface OutboundMessageEvent {
	type: 'outbound_message';
	message: string;
}

export interface OutboundMessageDeltaEvent {
	type: 'outbound_message_delta';
	delta: string;
}

export interface TypingStateEvent {
	type: 'typing_state';
	is_typing: boolean;
}

export const apiClient = {
	ttsProfiles: async (): Promise<TtsProfile[]> => [],
	webChatSendAudio: async () => ({ok: false}),
	listTasks: async () => ({tasks: [] as Task[]}),
	updateTask: async () => ({ok: true}),
	deleteTask: async () => ({ok: true}),
	channelMessages: async () => ({items: [] as TimelineItem[]}),
	listWorkers: async () => ({workers: [] as WorkerListItem[]}),
	workerDetail: async (): Promise<unknown> => ({}),
	cancelProcess: async () => ({ok: true}),
};

export function getEventsUrl(): string {
	return '';
}

export function setServerUrl(_url: string): void {}

export default {apiClient, getEventsUrl, setServerUrl};
