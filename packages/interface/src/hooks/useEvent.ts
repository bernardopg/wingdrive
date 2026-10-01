import { useEffect } from 'react';
import { useWingDriveClient } from '../contexts/WingDriveContext';

/**
 * Subscribe to core events
 * @param eventType - The event type to listen for (e.g., "JobProgress", "FileCreated")
 * @param handler - Callback when event is received
 */
export function useEvent(eventType: string, handler: (event: any) => void) {
	const client = useWingDriveClient();

	useEffect(() => {
		if (!client) return;

		const handleEvent = (event: any) => {
			// Fast path: check event type match before doing anything else
			// Events come as { EventName: { ...data } } not { type: "EventName", ...data }
			if (!eventType || eventType in event) {
				handler(event);
			}
		};

		// Listen to all events from the client
		client.on('wingdrive-event', handleEvent);

		return () => {
			client.off('wingdrive-event', handleEvent);
		};
	}, [eventType, client]);
}

/**
 * Subscribe to all core events
 */
export function useAllEvents(handler: (event: any) => void) {
	return useEvent('', handler);
}