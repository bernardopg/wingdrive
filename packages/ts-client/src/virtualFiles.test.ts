import {expect, test} from 'bun:test';
import {
	getVirtualMetadata,
	isVirtualFile,
	mapDeviceToFile,
	mapLocationToFile,
	mapVolumeToFile
} from './virtualFiles';

test('mapLocationToFile creates virtual directory file', () => {
	const file = mapLocationToFile({id: 'loc1', name: 'Docs', wing_path: {Physical: {device_slug: 'd', path: '/docs'}}}, 'icon');
	expect(file.id).toBe('virtual:location:loc1');
	expect(file.name).toBe('Docs');
	expect(file.kind).toBe('Directory');
	expect(isVirtualFile(file)).toBe(true);
	expect(getVirtualMetadata(file)?.type).toBe('location');
});

test('mapVolumeToFile uses mount point and capacity', () => {
	const file = mapVolumeToFile({fingerprint: 'fp1', name: 'Disk', display_name: 'My Disk', mount_point: '/mnt', total_capacity: 1024n as any}, 'dev', 1);
	expect(file.id).toBe('virtual:volume:fp1');
	expect(file.name).toBe('My Disk');
	expect(file.size).toBe(1024);
	expect(getVirtualMetadata(file)?.type).toBe('volume');
});

test('mapVolumeToFile handles missing mount/capacity', () => {
	const file = mapVolumeToFile({fingerprint: 'fp2', name: 'Vol'}, 'dev');
	expect(file.wing_path).toEqual({Physical: {device_slug: 'dev', path: '/'}});
	expect(file.size).toBe(0);
});

test('mapDeviceToFile and virtual helpers handle null', () => {
	const file = mapDeviceToFile({id: 'dev1', name: 'Phone', slug: 'phone'});
	expect(file.id).toBe('virtual:device:dev1');
	expect(getVirtualMetadata(file)?.type).toBe('device');
	expect(isVirtualFile(null)).toBe(false);
	expect(isVirtualFile(undefined)).toBe(false);
	expect(getVirtualMetadata(null)).toBeNull();
	expect(getVirtualMetadata({} as any)).toBeNull();
});
