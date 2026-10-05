import {toast} from '@wingdrive/primitives';
import type {FileOperationOutcome} from './fileOperationOutcome';

/** Surfaces any outcome other than a clean success; quiet on success. */
export function reportFileOperation(outcome: FileOperationOutcome) {
	if (outcome.status === 'success') return;
	const detail = outcome.details[0];
	const text = detail ? `${outcome.message}. ${detail}` : outcome.message;
	if (outcome.status === 'running' || outcome.status === 'cancelled') {
		toast.info(text);
	} else {
		toast.error(text);
	}
}
