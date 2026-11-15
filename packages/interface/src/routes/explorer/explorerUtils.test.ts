import {describe, expect, it} from 'bun:test';

const {deriveTitleFromPath} = await import('../../components/TabManager/deriveTitle');
const {folderOfRoute, wingPathFromParam} = await import('./wingPathParam');
const {formatBytes, formatRelativeTime, getDeviceIcon, wdPathToUri} = await import('./utils');

const physical = {Physical: {device_slug: 'local', path: '/home/me/100%'}};

describe('wingPathFromParam', () => {
	it('reads paths once or twice encoded and rejects anything else', () => {
		const raw = JSON.stringify(physical);
		expect(wingPathFromParam(raw)).toEqual(physical);
		expect(wingPathFromParam(encodeURIComponent(raw))).toEqual(physical);
		expect(wingPathFromParam('%E0%A4%A')).toBeNull();
		expect(wingPathFromParam('"just a string"')).toBeNull();
		expect(wingPathFromParam('not json')).toBeNull();
	});

	it('finds the folder of an explorer route only', () => {
		const route = `/explorer?path=${encodeURIComponent(JSON.stringify(physical))}`;
		expect(folderOfRoute(route)).toEqual(physical);
		const cloud = {Cloud: {service: 'S3', identifier: 'b', path: 'x'}};
		expect(folderOfRoute(`/explorer?path=${encodeURIComponent(JSON.stringify(cloud))}`)).toEqual(
			cloud
		);
		expect(
			folderOfRoute(`/explorer?path=${encodeURIComponent(JSON.stringify({Content: {}}))}`)
		).toBeNull();
		expect(folderOfRoute('/explorer?view=device')).toBeNull();
		expect(folderOfRoute('/recents')).toBeNull();
	});
});

describe('deriveTitleFromPath', () => {
	it('names fixed routes and leaves tag and source titles to their views', () => {
		for (const route of ['/', '/favorites', '/recents', '/trash', '/wingbot', '/redundancy/at-risk']) {
			expect(deriveTitleFromPath(route, '')).toBeString();
		}
		expect(deriveTitleFromPath('/wingbot/chat', '')).toBe(deriveTitleFromPath('/wingbot', ''));
		expect(deriveTitleFromPath('/tag/abc', '')).toBeNull();
		expect(deriveTitleFromPath('/sources/s1', '')).toBeNull();
		expect(deriveTitleFromPath('/file-kinds/Unknown%20Kind', '')).toBe('Unknown Kind');
	});

	it('uses the folder name for explorer paths', () => {
		const search = `?path=${encodeURIComponent(JSON.stringify(physical))}`;
		expect(deriveTitleFromPath('/explorer', search)).toBe('100%');
		const rootSearch = `?path=${encodeURIComponent(
			JSON.stringify({Physical: {device_slug: 'local', path: '/'}})
		)}`;
		const explorerTitle = deriveTitleFromPath('/explorer', '?view=other');
		expect(deriveTitleFromPath('/explorer', rootSearch)).toBe(explorerTitle);
		expect(deriveTitleFromPath('/explorer', '?view=device')).not.toBe(explorerTitle);
		expect(deriveTitleFromPath('/somewhere-else', '')).toBeString();
	});
});

describe('explorer formatting helpers', () => {
	it('formats byte counts', () => {
		expect(formatBytes(null)).toBe('0 B');
		expect(formatBytes(0)).toBe('0 B');
		expect(formatBytes(512)).toBe('512 B');
		expect(formatBytes(1536)).toBe('2 KB');
		expect(formatBytes(BigInt(5 * 1024 ** 3))).toBe('5 GB');
	});

	it('formats relative times', () => {
		const ago = (ms: number) => new Date(Date.now() - ms);
		expect(formatRelativeTime(ago(1000))).toBeString();
		expect(formatRelativeTime(ago(5 * 60_000))).toBeString();
		expect(formatRelativeTime(ago(3 * 3_600_000))).toBeString();
		expect(formatRelativeTime(ago(2 * 86_400_000).toISOString())).toBeString();
		const old = ago(30 * 86_400_000);
		expect(formatRelativeTime(old)).toBe(old.toLocaleDateString());
	});

	it('picks a device icon by system and model', () => {
		const mobile = getDeviceIcon('Android');
		expect(getDeviceIcon('iOS')).toBe(mobile);
		const pc = getDeviceIcon('Windows 11');
		const server = getDeviceIcon('Linux server');
		expect(getDeviceIcon('Linux', 'HP Server')).toBe(server);
		const laptop = getDeviceIcon('macOS');
		expect(new Set([mobile, pc, server, laptop]).size).toBe(4);
	});

	it('turns paths into uris', () => {
		expect(wdPathToUri(physical as never)).toBe('local://local/home/me/100%');
		expect(
			wdPathToUri({Cloud: {service: 'S3', identifier: 'bucket', path: 'a/b'}} as never)
		).toBe('s3://bucket/a/b');
		expect(wdPathToUri({Content: {content_id: 'c1'}} as never)).toBe('content://c1');
		expect(wdPathToUri({Sidecar: {}} as never)).toBe('');
	});
});
