import {apiClient, type WorkerListItem} from '@spacebot/api-client';
import {
	InlineWorkerCard as InlineWorkerCardUI,
	type TranscriptStep
} from '@wingdrive/ai';
import {useMutation, useQuery, useQueryClient} from '@tanstack/react-query';
import {useTranslation} from '../i18n';

export function InlineWorkerCard({
	agentId,
	worker
}: {
	agentId: string;
	worker: WorkerListItem;
}) {
	const {t} = useTranslation('common');
	const queryClient = useQueryClient();
	const detailQuery = useQuery({
		queryKey: ['wingbot', 'worker-detail', agentId, worker.id],
		queryFn: () => apiClient.workerDetail(agentId, worker.id),
		refetchInterval: worker.status === 'running' ? 1500 : false
	});

	const copyLogs = async () => {
		const detail = detailQuery.data;
		if (!detail) return;

		const transcriptText = (detail.transcript ?? [])
			.map((step) => JSON.stringify(step, null, 2))
			.join('\n\n');

		const payload = [
			t('wingbot.workerLog.worker', {task: detail.task}),
			t('wingbot.workerLog.status', {status: detail.status}),
			t('wingbot.workerLog.started', {date: detail.started_at}),
			detail.completed_at ? t('wingbot.workerLog.completed', {date: detail.completed_at}) : null,
			detail.result ? t('wingbot.workerLog.result', {result: detail.result}) : null,
			transcriptText ? t('wingbot.workerLog.transcript', {transcript: transcriptText}) : null
		]
			.filter(Boolean)
			.join('\n\n');

		await navigator.clipboard.writeText(payload);
	};

	const cancelMutation = useMutation({
		mutationFn: () =>
			apiClient.cancelProcess({
				channelId: worker.channel_id ?? '',
				processType: 'worker',
				processId: worker.id
			}),
		onSuccess: async () => {
			await Promise.all([
				queryClient.invalidateQueries({
					queryKey: [
						'wingbot',
						'conversation-workers',
						agentId,
						worker.channel_id
					]
				}),
				queryClient.invalidateQueries({
					queryKey: ['wingbot', 'worker-detail', agentId, worker.id]
				}),
				queryClient.invalidateQueries({
					queryKey: ['wingbot', 'channel-timeline', worker.channel_id]
				})
			]);
		}
	});

	const isRunning = worker.status === 'running';
	const canCancel = isRunning && !!worker.channel_id && !cancelMutation.isPending;

	return (
		<InlineWorkerCardUI
			title={worker.task}
			status={worker.status}
			toolCallCount={worker.tool_calls}
			liveStatus={worker.live_status}
			transcript={(detailQuery.data?.transcript ?? []) as TranscriptStep[]}
			isTranscriptLoading={detailQuery.isLoading}
			onCopyLogs={detailQuery.data ? () => void copyLogs() : undefined}
			onCancel={canCancel ? () => void cancelMutation.mutateAsync() : undefined}
		/>
	);
}
