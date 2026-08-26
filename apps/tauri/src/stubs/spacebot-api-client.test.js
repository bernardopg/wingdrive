import {describe, expect, test} from 'bun:test';
import {apiClient} from './spacebot-api-client.ts';

describe('Spacebot fallback client', () => {
	test('rejects mutations instead of reporting false success', async () => {
		const mutations = [
			apiClient.ttsGenerate('hello', {
				agentId: 'main',
				profileId: 'test'
			}),
			apiClient.webChatSendAudio('main', 'test', new Blob()),
			apiClient.updateTask(1, {status: 'completed'}),
			apiClient.deleteTask(1),
			apiClient.cancelProcess({
				channelId: 'test',
				processType: 'worker',
				processId: 'test'
			}),
			apiClient.createPortalConversation({agentId: 'main'}),
			apiClient.portalSend({
				agentId: 'main',
				sessionId: 'test',
				senderName: 'test',
				message: 'hello'
			})
		];

		const results = await Promise.allSettled(mutations);
		expect(results).toHaveLength(7);
		for (const result of results) {
			expect(result.status).toBe('rejected');
			expect(String(result.reason)).toContain('Spacebot is unavailable');
		}
	});
});
