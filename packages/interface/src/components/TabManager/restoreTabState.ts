/** A restored tab has no live shell, so opening it must not start one. */
export function restoreTabState<T extends {terminalVisible?: boolean}>(state: T): T {
	return {...state, terminalVisible: false};
}
