import {ChatCircleDots, Checks} from '@phosphor-icons/react';
import {WingDriveLogo} from '@wingdrive/assets/images';
import {
	apiClient,
	getEventsUrl,
	setServerUrl,
	type InboundMessageEvent,
	type OutboundMessageDeltaEvent,
	type OutboundMessageEvent,
	type PortalConversationResponse,
	type PortalConversationSummary,
	type TypingStateEvent
} from '@spacebot/api-client';
import {usePopover} from '@wingdrive/primitives';
import {useMutation, useQuery, useQueryClient} from '@tanstack/react-query';
import {
	createContext,
	useCallback,
	useContext,
	useEffect,
	useMemo,
	useState,
	type ReactNode
} from 'react';
import {useNavigate, useParams} from 'react-router-dom';
import {usePlatform} from '../contexts/PlatformContext';
import {useWingbotEventSource} from './useWingbotEventSource';

export const primaryItems = [
	{icon: ChatCircleDots, label: 'Chat', path: '/wingbot/chat'},
	{icon: Checks, label: 'Tasks', path: '/wingbot/tasks'}
];

export const projects = [
	{name: 'WingDrive', detail: 'Main workspace', ball: WingDriveLogo},
	{name: 'Wingbot Runtime', detail: 'Remote control plane', ball: WingDriveLogo},
	{name: 'Hosted Platform', detail: 'Deploy and observe', ball: WingDriveLogo}
];

export const agents = [
	{id: 'main', name: 'Star', detail: 'WingDrive COO'},
	{id: 'operations', name: 'Operations', detail: 'Scheduling and triage'},
	{id: 'builder', name: 'Builder', detail: 'Code and tooling'}
];

export const projectOptions = [
	'WingDrive v3',
	'Wingbot Runtime',
	'Hosted Platform'
];
export const models = [
	{
		id: 'claude-3.7-sonnet',
		name: 'Claude 3.7 Sonnet',
		provider: 'Anthropic',
		context_window: 200000
	},
	{id: 'gpt-5', name: 'GPT-5', provider: 'OpenAI', context_window: 128000},
	{
		id: 'qwen-2.5-72b',
		name: 'Qwen 2.5 72B',
		provider: 'Qwen',
		context_window: 32000
	}
];

export interface WingbotContextType {
	// Navigation state
	search: string;
	setSearch: (value: string) => void;
	selectedAgent: string;
	setSelectedAgent: (value: string) => void;
	activeTab: string;

	// Agent data
	currentAgent: (typeof agents)[number];
	agentSelector: ReturnType<typeof usePopover>;

	// Composer state
	selectedProject: string;
	setSelectedProject: (value: string) => void;
	selectedModel: string;
	setSelectedModel: (value: string) => void;
	projectOptions: string[];
	models: typeof models;
	composerProjectSelector: ReturnType<typeof usePopover>;

	// Conversation state
	draft: string;
	setDraft: (value: string) => void;
	isTyping: boolean;
	streamingAssistantText: string;
	conversations: PortalConversationSummary[];
	conversationsLoading: boolean;
	conversationsError: Error | null;

	// Actions
	handleSendMessage: () => Promise<void>;
	isSending: boolean;
	createConversation: (
		title?: string | null
	) => Promise<PortalConversationResponse>;
	getConversationById: (id: string) => PortalConversationSummary | undefined;
	getConversationMessages: (id: string) => PortalHistoryItem[];
	openVoiceOverlay: () => void;

	// Navigation
	navigateToChat: () => void;
	navigateToConversation: (conversationId: string) => void;
}

interface PortalHistoryItem {
	role: 'user' | 'assistant';
	content: string;
	timestamp: string;
}

const WingbotContext = createContext<WingbotContextType | null>(null);

export function useWingbot() {
	const context = useContext(WingbotContext);
	if (!context) {
		throw new Error('useWingbot must be used within WingbotProvider');
	}
	return context;
}

interface WingbotProviderProps {
	children: ReactNode;
}

export const isMacOS =
	typeof navigator !== 'undefined' &&
	(navigator.platform.toLowerCase().includes('mac') ||
		navigator.userAgent.includes('Mac'));

export function WingbotProvider({children}: WingbotProviderProps) {
	if (import.meta.env.VITE_SPACEBOT_AVAILABLE !== 'true') {
		return (
			<div className="bg-app text-ink flex h-full items-center justify-center p-8">
				<div className="border-app-line bg-app-box max-w-md rounded-2xl border p-6 text-center">
					<h1 className="text-lg font-semibold">
						Wingbot unavailable
					</h1>
					<p className="text-ink-dull mt-2 text-sm">
						This build does not include the Wingbot runtime. No
						messages or tasks were changed.
					</p>
				</div>
			</div>
		);
	}

	return <AvailableWingbotProvider>{children}</AvailableWingbotProvider>;
}

function AvailableWingbotProvider({children}: WingbotProviderProps) {
	const platform = usePlatform();
	const queryClient = useQueryClient();
	const navigate = useNavigate();
	const params = useParams();

	useEffect(() => {
		setServerUrl('http://127.0.0.1:19898');
	}, []);

	useEffect(() => {
		if (platform.applyMacOSStyling) {
			platform.applyMacOSStyling().catch((error) => {
				console.warn('Failed to apply macOS styling:', error);
			});
		}
	}, [platform]);

	// Navigation state
	const [search, setSearch] = useState('');
	const [selectedAgent, setSelectedAgent] = useState('main');
	const [activeTab, setActiveTab] = useState('Chat');

	// Composer state
	const [selectedProject, setSelectedProject] = useState(
		projectOptions[0] ?? ''
	);
	const [selectedModel, setSelectedModel] = useState(models[0]?.id ?? '');

	// Conversation state
	const [draft, setDraft] = useState('');
	const [isTyping, setIsTyping] = useState(false);
	const [streamingAssistantText, setStreamingAssistantText] = useState('');
	const [conversationMessages, setConversationMessages] = useState<
		Map<string, PortalHistoryItem[]>
	>(new Map());

	const agentSelector = usePopover();
	const composerProjectSelector = usePopover();

	const currentAgent = useMemo(
		() => agents.find((agent) => agent.id === selectedAgent) ?? agents[0],
		[selectedAgent]
	);

	// Reset state when agent changes
	useEffect(() => {
		setIsTyping(false);
		setStreamingAssistantText('');
	}, [selectedAgent]);

	// Conversations query
	const conversationsQuery = useQuery({
		queryKey: ['wingbot', 'conversations', selectedAgent],
		queryFn: () =>
			apiClient.listPortalConversations(selectedAgent, false, 100),
		refetchInterval: 4000
	});

	const conversations = conversationsQuery.data?.conversations ?? [];

	// Conversation messages query - fetch when viewing a conversation
	// With splat route "conversation/*", the ID is in params["*"]
	const conversationId = params['*']
		? decodeURIComponent(params['*'])
		: undefined;
	const historyQuery = useQuery({
		queryKey: ['wingbot', 'portal-history', selectedAgent, conversationId],
		queryFn: () =>
			apiClient.portalHistory(selectedAgent, conversationId!, 200),
		enabled: Boolean(conversationId),
		refetchInterval: false
	});

	// Update conversation messages cache
	useEffect(() => {
		if (historyQuery.data && conversationId) {
			setConversationMessages((prev) => {
				const next = new Map(prev);
				next.set(
					conversationId,
					historyQuery.data as unknown as PortalHistoryItem[]
				);
				return next;
			});
		}
	}, [historyQuery.data, conversationId]);

	// Create conversation mutation
	const createConversationMutation = useMutation({
		mutationFn: (title?: string | null) =>
			apiClient.createPortalConversation({
				agentId: selectedAgent,
				title
			}),
		onSuccess: async (response: PortalConversationResponse) => {
			navigateToConversation(response.conversation.id);
			await queryClient.invalidateQueries({
				queryKey: ['wingbot', 'conversations', selectedAgent]
			});
		}
	});

	// Send message mutation
	const sendMessageMutation = useMutation({
		mutationFn: async (message: string) => {
			let targetConversationId = conversationId;
			if (!targetConversationId) {
				const response =
					await createConversationMutation.mutateAsync(null);
				targetConversationId = response.conversation.id;
			}

			await apiClient.portalSend({
				agentId: selectedAgent,
				sessionId: targetConversationId!,
				senderName: currentAgent?.name ?? 'user',
				message
			});

			return targetConversationId;
		},
		onSuccess: async (targetConversationId) => {
			setDraft('');
			if (targetConversationId) {
				navigateToConversation(targetConversationId);
			}
			await Promise.all([
				queryClient.invalidateQueries({
					queryKey: ['wingbot', 'conversations', selectedAgent]
				}),
				queryClient.invalidateQueries({
					queryKey: [
						'wingbot',
						'portal-history',
						selectedAgent,
						targetConversationId
					]
				})
			]);
		}
	});

	// SSE event source
	useWingbotEventSource(getEventsUrl(), {
		enabled: activeTab === 'Chat',
		onReconnect: () => {
			void queryClient.invalidateQueries({
				queryKey: ['wingbot', 'conversations', selectedAgent]
			});
			if (conversationId) {
				void Promise.all([
					queryClient.invalidateQueries({
						queryKey: [
							'wingbot',
							'portal-history',
							selectedAgent,
							conversationId
						]
					}),
					queryClient.invalidateQueries({
						queryKey: [
							'wingbot',
							'channel-timeline',
							conversationId
						]
					})
				]);
			}
		},
		handlers: {
			typing_state: (payload) => {
				const event = payload as TypingStateEvent;
				if (
					event.agent_id !== selectedAgent ||
					event.channel_id !== conversationId
				) {
					return;
				}
				// Typing can stop before the final message arrives; keep the streamed
				// text on screen until outbound_message replaces it.
				setIsTyping(event.is_typing);
			},
			outbound_message_delta: (payload) => {
				const event = payload as OutboundMessageDeltaEvent;
				if (
					event.agent_id !== selectedAgent ||
					event.channel_id !== conversationId
				) {
					return;
				}
				setIsTyping(true);
				setStreamingAssistantText(event.aggregated_text);
			},
			outbound_message: (payload) => {
				const event = payload as OutboundMessageEvent;
				if (
					event.agent_id !== selectedAgent ||
					event.channel_id !== conversationId
				) {
					return;
				}
				setIsTyping(false);
				setStreamingAssistantText('');
				const timelineKey = ['wingbot', 'channel-timeline', conversationId];
				// Without a loaded timeline there is nothing to append to; refetch so
				// the reply is not lost along with the cleared stream.
				if (!queryClient.getQueryData(timelineKey)) {
					void queryClient.invalidateQueries({queryKey: timelineKey});
				}
				// Push the assistant message directly into the timeline cache.
				queryClient.setQueryData(
					['wingbot', 'channel-timeline', conversationId],
					(
						old: {items: unknown[]; has_more: boolean} | undefined
					) => {
						if (!old) return old;
						return {
							...old,
							items: [
								...old.items,
								{
									type: 'message' as const,
									id: `sse-${Date.now()}`,
									role: 'assistant',
									content: event.text,
									sender_id: event.agent_id ?? null,
									sender_name: event.agent_id ?? null,
									created_at: new Date().toISOString()
								}
							]
						};
					}
				);
				void queryClient.invalidateQueries({
					queryKey: ['wingbot', 'conversations', selectedAgent]
				});
			},
			inbound_message: (payload) => {
				const event = payload as InboundMessageEvent;
				if (
					event.agent_id !== selectedAgent ||
					event.channel_id !== conversationId
				) {
					return;
				}
				// Push the user message directly into the timeline cache
				// so it appears instantly (like the portal SSE-driven approach).
				queryClient.setQueryData(
					['wingbot', 'channel-timeline', conversationId],
					(
						old: {items: unknown[]; has_more: boolean} | undefined
					) => {
						if (!old) return old;
						return {
							...old,
							items: [
								...old.items,
								{
									type: 'message' as const,
									id: `sse-${Date.now()}`,
									role: 'user',
									content: event.text,
									sender_id: event.sender_id ?? null,
									sender_name: event.sender_name ?? null,
									created_at: new Date().toISOString()
								}
							]
						};
					}
				);
				void queryClient.invalidateQueries({
					queryKey: ['wingbot', 'conversations', selectedAgent]
				});
			}
		}
	});

	// Actions
	const handleSendMessage = useCallback(async () => {
		const message = draft.trim();
		if (!message || sendMessageMutation.isPending) return;
		await sendMessageMutation.mutateAsync(message);
	}, [draft, sendMessageMutation]);

	const createConversation = useCallback(
		async (title?: string | null) => {
			return createConversationMutation.mutateAsync(title);
		},
		[createConversationMutation]
	);

	const getConversationById = useCallback(
		(id: string) => conversations.find((c) => c.id === id),
		[conversations]
	);

	const getConversationMessages = useCallback(
		(id: string) => conversationMessages.get(id) ?? [],
		[conversationMessages]
	);

	const openVoiceOverlay = useCallback(() => {
		if (!platform.showWindow) return;
		platform.showWindow({type: 'VoiceOverlay'}).catch((error) => {
			console.warn('Failed to open voice overlay:', error);
		});
	}, [platform]);

	const navigateToChat = useCallback(() => {
		setActiveTab('Chat');
		setIsTyping(false);
		setStreamingAssistantText('');
		navigate('/wingbot/chat');
	}, [navigate]);

	const navigateToConversation = useCallback(
		(conversationId: string) => {
			setActiveTab('Chat');
			setIsTyping(false);
			setStreamingAssistantText('');
			navigate(`/wingbot/chat/conversation/${conversationId}`);
		},
		[navigate]
	);

	const value = useMemo(
		() => ({
			search,
			setSearch,
			selectedAgent,
			setSelectedAgent,
			activeTab,
			currentAgent,
			agentSelector,
			selectedProject,
			setSelectedProject,
			selectedModel,
			setSelectedModel,
			projectOptions,
			models,
			composerProjectSelector,
			draft,
			setDraft,
			isTyping,
			streamingAssistantText,
			conversations,
			conversationsLoading: conversationsQuery.isLoading,
			conversationsError: conversationsQuery.error ?? null,
			handleSendMessage,
			isSending:
				sendMessageMutation.isPending ||
				createConversationMutation.isPending,
			createConversation,
			getConversationById,
			getConversationMessages,
			openVoiceOverlay,
			navigateToChat,
			navigateToConversation
		}),
		[
			search,
			selectedAgent,
			activeTab,
			currentAgent,
			agentSelector,
			selectedProject,
			selectedModel,
			projectOptions,
			models,
			composerProjectSelector,
			draft,
			isTyping,
			streamingAssistantText,
			conversations,
			conversationsQuery.isLoading,
			conversationsQuery.error,
			handleSendMessage,
			sendMessageMutation.isPending,
			createConversationMutation.isPending,
			createConversation,
			getConversationById,
			getConversationMessages,
			openVoiceOverlay,
			navigateToChat,
			navigateToConversation
		]
	);

	return (
		<WingbotContext.Provider value={value}>
			{children}
		</WingbotContext.Provider>
	);
}
