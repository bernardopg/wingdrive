import { useCallback, useEffect, useRef, useState } from "react";
import { useQueryClient } from "@tanstack/react-query";
import {
	WIRE_METHODS,
	type ConnectStatus,
	type GoogleService,
	type IntegrationAccountInfo,
} from "@wingdrive/ts-client";
import { usePlatform } from "../contexts/PlatformContext";
import { useLibraryMutation, useWingDriveClient } from "../contexts/WingDriveContext";

/** How often the sign-in status is checked while the browser is open. */
const POLL_MS = 1500;

export type GoogleConnectState =
	| { phase: "idle" }
	| { phase: "waiting"; authUrl: string; sessionId: string }
	| { phase: "done"; account: IntegrationAccountInfo }
	| { phase: "error"; message: string };

/** Query keys that change when an account connects: accounts and the Drive location. */
export const INTEGRATION_QUERY_KEYS = [
	["integrations.accounts.list"],
	["locations.list"],
	["volumes.list"],
] as const;

/**
 * Runs the Google sign-in: the daemon returns a consent URL, the browser
 * opens it, and the hook polls the daemon until Google redirects back.
 * Unmounting stops polling and cancels the daemon's session so its loopback
 * listener closes.
 */
export function useGoogleConnect(onConnected?: (account: IntegrationAccountInfo) => void) {
	const client = useWingDriveClient();
	const platform = usePlatform();
	const queryClient = useQueryClient();
	const connect = useLibraryMutation("integrations.google.connect");
	const cancelSession = useLibraryMutation("integrations.google.connect_cancel");
	const [state, setState] = useState<GoogleConnectState>({ phase: "idle" });
	const active = useRef<string | null>(null);
	const onConnectedRef = useRef(onConnected);
	onConnectedRef.current = onConnected;

	useEffect(
		() => () => {
			const sessionId = active.current;
			active.current = null;
			if (sessionId) cancelSession.mutate({ session_id: sessionId });
		},
		// eslint-disable-next-line react-hooks/exhaustive-deps
		[]
	);

	const start = useCallback(
		async (services: GoogleService[], loginHint?: string) => {
			let sessionId: string;
			try {
				const out = await connect.mutateAsync({ services, login_hint: loginHint ?? null });
				sessionId = out.session_id;
				active.current = sessionId;
				setState({ phase: "waiting", authUrl: out.auth_url, sessionId });
				platform.openLink(out.auth_url);
			} catch (error) {
				setState({ phase: "error", message: error instanceof Error ? error.message : String(error) });
				return;
			}
			while (active.current === sessionId) {
				await new Promise((resolve) => setTimeout(resolve, POLL_MS));
				if (active.current !== sessionId) return;
				let status: ConnectStatus;
				try {
					status = await client.execute<{ session_id: string }, ConnectStatus>(
						WIRE_METHODS.libraryQueries["integrations.google.connect_status"],
						{ session_id: sessionId }
					);
				} catch {
					// A dropped request is retried on the next tick.
					continue;
				}
				if (status.status === "pending") continue;
				active.current = null;
				if (status.status === "completed") {
					setState({ phase: "done", account: status.account });
					for (const queryKey of INTEGRATION_QUERY_KEYS) {
						void queryClient.invalidateQueries({ queryKey: [...queryKey] });
					}
					onConnectedRef.current?.(status.account);
				} else if (status.status === "failed") {
					setState({ phase: "error", message: status.message });
				} else {
					setState({ phase: "idle" });
				}
			}
		},
		[client, connect, platform, queryClient]
	);

	const cancel = useCallback(async () => {
		const sessionId = active.current;
		active.current = null;
		setState({ phase: "idle" });
		if (sessionId) await cancelSession.mutateAsync({ session_id: sessionId }).catch(() => {});
	}, [cancelSession]);

	const reset = useCallback(() => setState({ phase: "idle" }), []);

	return { state, start, cancel, reset, isStarting: connect.isPending };
}
