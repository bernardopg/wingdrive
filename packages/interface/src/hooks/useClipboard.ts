import {toast} from '@wingdrive/primitives';
import type {WingPath} from '@wingdrive/ts-client';
import {create} from 'zustand';
import {usePlatform} from '../contexts/PlatformContext';

export interface ClipboardState {
	operation: 'copy' | 'cut' | null;
	files: WingPath[];
	sourcePath: WingPath | null;
}

interface ClipboardStore extends ClipboardState {
	exportedToSystem: boolean;
	setClipboard: (
		operation: 'copy' | 'cut',
		files: WingPath[],
		sourcePath: WingPath | null
	) => void;
	clearClipboard: () => void;
	hasClipboard: () => boolean;
}

export const useClipboardStore = create<ClipboardStore>((set, get) => ({
	exportedToSystem: false,
	operation: null,
	files: [],
	sourcePath: null,

	setClipboard: (operation, files, sourcePath) => {
		set({operation, files, sourcePath, exportedToSystem: false});
	},

	clearClipboard: () => {
		set({
			operation: null,
			files: [],
			sourcePath: null,
			exportedToSystem: false
		});
	},

	hasClipboard: () => {
		const state = get();
		return state.operation !== null && state.files.length > 0;
	}
}));

/**
 * Hook to access clipboard state and operations
 */
export function useClipboard() {
	const store = useClipboardStore();
	const platform = usePlatform();
	const write = (
		operation: 'copy' | 'cut',
		files: WingPath[],
		sourcePath: WingPath | null,
		local: boolean
	) => {
		store.setClipboard(operation, files, sourcePath);
		if (
			local &&
			platform.writeFileClipboard &&
			files.every((file) => 'Physical' in file)
		) {
			void platform
				.writeFileClipboard(
					files.flatMap((file) =>
						'Physical' in file ? [file.Physical.path] : []
					),
					operation === 'cut'
				)
				.then(() => {
					if (useClipboardStore.getState().files === files)
						useClipboardStore.setState({exportedToSystem: true});
				})
				.catch((error) =>
					toast.error(
						`Could not copy files to the system clipboard: ${error}`
					)
				);
		}
	};

	return {
		operation: store.operation,
		files: store.files,
		sourcePath: store.sourcePath,
		setClipboard: store.setClipboard,
		clearClipboard: store.clearClipboard,
		hasClipboard: store.hasClipboard,
		canPaste: () => store.hasClipboard() || !!platform.readFileClipboard,
		readFiles: async (): Promise<ClipboardState> => {
			const internal = useClipboardStore.getState();
			if (
				!platform.readFileClipboard ||
				(internal.files.length > 0 && !internal.exportedToSystem)
			)
				return internal;
			const [paths, cut] = await platform.readFileClipboard();
			if (!paths.length) return internal;
			return {
				operation: cut ? 'cut' : 'copy',
				files: paths.map((path): WingPath => ({
					Physical: {device_slug: 'local', path}
				})),
				sourcePath: null
			};
		},
		finishPaste: async (contents: ClipboardState) => {
			if (contents.operation !== 'cut') return;
			if (platform.readFileClipboard && platform.writeFileClipboard) {
				const [paths, cut] = await platform.readFileClipboard();
				const copiedPaths = contents.files.flatMap((file) =>
					'Physical' in file ? [file.Physical.path] : []
				);
				if (
					cut &&
					JSON.stringify(paths) === JSON.stringify(copiedPaths)
				) {
					await platform.writeFileClipboard([], false);
				}
			}
			if (
				JSON.stringify(useClipboardStore.getState().files) ===
				JSON.stringify(contents.files)
			)
				store.clearClipboard();
		},

		// Helper to copy files
		copyFiles: (
			files: WingPath[],
			sourcePath: WingPath | null = null,
			local = true
		) => {
			write('copy', files, sourcePath, local);
		},

		// Helper to cut files
		cutFiles: (
			files: WingPath[],
			sourcePath: WingPath | null = null,
			local = true
		) => {
			write('cut', files, sourcePath, local);
		}
	};
}
