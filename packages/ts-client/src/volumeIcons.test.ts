import {expect, test} from 'bun:test';
import {getVolumeIcon} from './volumeIcons';

test('getVolumeIcon maps cloud mount points', () => {
	for (const scheme of ['s3','gdrive','dropbox','onedrive','gcs','azblob','b2','wasabi','spaces','cloud']) {
		expect(typeof getVolumeIcon({mount_point: `${scheme}://bucket`, volume_type: 'External'})).toBe('string');
	}
});

test('getVolumeIcon uses HDD for external/removable and drive for others', () => {
	expect(typeof getVolumeIcon({mount_point: '/mnt/usb', volume_type: 'External'})).toBe('string');
	expect(typeof getVolumeIcon({mount_point: '/mnt/sd', volume_type: 'Removable'})).toBe('string');
	expect(typeof getVolumeIcon({mount_point: '/mnt/disk', volume_type: 'Internal'})).toBe('string');
	expect(typeof getVolumeIcon({mount_point: null})).toBe('string');
	expect(typeof getVolumeIcon({mount_point: 'file:///mnt', volume_type: 'External'})).toBe('string');
});
