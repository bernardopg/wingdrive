/// <reference types="vite/client" />
/// <reference path="../../../packages/interface/src/env.d.ts" />

interface WingDriveGlobal {
	showContextMenu?: (
		items: import("@wingdrive/interface").ContextMenuItem[],
		position: { x: number; y: number }
	) => Promise<void>;
	registerKeybind?: (
		id: string,
		accelerator: string,
		handler: () => void | Promise<void>
	) => Promise<void>;
	unregisterKeybind?: (id: string) => Promise<void>;
}

interface Window {
	__WINGDRIVE__: WingDriveGlobal;
}
