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
//! The exported types mirror the real API client surface so the Spacebot UI
//! typechecks without the private repo; keep them in sync when the UI starts
//! calling new endpoints.

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

export interface Subtask {
	title: string;
	completed: boolean;
}

export interface Task {
	id: string;
	task_number: number;
	title: string;
	description?: string | null;
	status: string;
	priority: string;
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

export interface UpdateTaskRequest {
	title?: string;
	description?: string;
	status?: string;
	priority?: string;
	assignee?: string | null;
	complete_subtask?: number;
}

export interface TimelineItem {
	id: string;
	type: 'message' | 'worker_run';
	role?: 'user' | 'assistant';
	content?: string;
	task?: string;
	status?: string;
	started_at?: string;
	completed_at?: string | null;
	sender_id?: string | null;
	sender_name?: string | null;
}

export interface PortalConversationSummary {
	id: string;
	title: string;
	message_count: number;
	last_message_preview: string | null;
	created_at: string;
	updated_at: string;
}

export interface PortalHistoryMessage {
	id: string;
	role: 'user' | 'assistant';
	content: string;
	created_at: string;
}

export interface PortalConversationResponse {
	conversation: PortalConversationSummary;
	messages: PortalHistoryMessage[];
}

export interface WorkerDetail {
	task: string;
	status: string;
	started_at: string;
	completed_at: string | null;
	result?: string | null;
	transcript: unknown[];
}

export interface InboundMessageEvent {
	type: 'inbound_message';
	agent_id: string;
	channel_id: string;
	text: string;
	sender_id?: string | null;
	sender_name?: string | null;
}

export interface OutboundMessageEvent {
	type: 'outbound_message';
	agent_id: string;
	channel_id: string;
	text: string;
}

export interface OutboundMessageDeltaEvent {
	type: 'outbound_message_delta';
	agent_id: string;
	channel_id: string;
	aggregated_text: string;
}

export interface TypingStateEvent {
	type: 'typing_state';
	agent_id: string;
	channel_id: string;
	is_typing: boolean;
}

const unavailable = (): Promise<never> =>
	Promise.reject(
		new Error(
			'Spacebot is unavailable in this build; no changes were made.'
		)
	);

export const apiClient = {
	ttsProfiles: async (_agentId: string): Promise<TtsProfile[]> => [],
	ttsGenerate: (
		_text: string,
		_opts: {agentId: string; profileId: string}
	): Promise<ArrayBuffer> => unavailable(),
	webChatSendAudio: (
		_agentId: string,
		_sessionId: string,
		_blob: Blob
	): Promise<{ok: boolean; status: number}> => unavailable(),
	listTasks: async (
		_agentId: string,
		_limit?: number
	): Promise<{tasks: Task[]}> => ({tasks: []}),
	updateTask: (
		_taskNumber: number,
		_req: UpdateTaskRequest
	): Promise<{ok: boolean}> => unavailable(),
	deleteTask: (_taskNumber: number): Promise<{ok: boolean}> => unavailable(),
	channelMessages: async (
		_conversationId: string,
		_limit?: number
	): Promise<{items: TimelineItem[]}> => ({items: []}),
	listWorkers: async (_params: {
		agentId: string;
		limit?: number;
	}): Promise<{workers: WorkerListItem[]}> => ({workers: []}),
	workerDetail: async (
		_agentId: string,
		_workerId: string
	): Promise<WorkerDetail> => ({
		task: '',
		status: '',
		started_at: '',
		completed_at: null,
		transcript: []
	}),
	cancelProcess: (_params: {
		channelId: string;
		processType: string;
		processId: string;
	}): Promise<{ok: boolean}> => unavailable(),
	listPortalConversations: async (
		_agentId: string,
		_includeDeleted?: boolean,
		_limit?: number
	): Promise<{conversations: PortalConversationSummary[]}> => ({
		conversations: []
	}),
	portalHistory: async (
		_agentId: string,
		_conversationId: string,
		_limit?: number
	): Promise<{messages: PortalHistoryMessage[]; has_more: boolean}> => ({
		messages: [],
		has_more: false
	}),
	createPortalConversation: (_input: {
		agentId: string;
		title?: string | null;
	}): Promise<PortalConversationResponse> => unavailable(),
	portalSend: (_input: {
		agentId: string;
		sessionId: string;
		senderName: string;
		message: string;
	}): Promise<{ok: boolean}> => unavailable()
};

export function getEventsUrl(): string {
	return '';
}

export function setServerUrl(_url: string): void {}

export default {apiClient, getEventsUrl, setServerUrl};
