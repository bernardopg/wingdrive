import type { File, WingPath } from "@wingdrive/ts-client";
import { isVirtualFile } from "@wingdrive/ts-client";
import { toast } from "@wingdrive/primitives";
import { usePlatform } from "../../../contexts/PlatformContext";
import { useOpenWith } from "../../../hooks/useOpenWith";
import { useExplorer } from "../context";

/** Most files one Open launches, so a stray Select All cannot start hundreds of apps. */
const MAX_FILES_TO_OPEN = 20;

export function useOpenFile(navigate?: (path: WingPath) => void) {
	const platform = usePlatform();
	const { navigateToPath } = useExplorer();
	const { openWithDefault } = useOpenWith([]);
	return async (file: File) => {
		const go = navigate ?? navigateToPath;
		if (isVirtualFile(file) || file.kind === "Directory") {
			go(file.wing_path);
			return;
		}
		if (!("Physical" in file.wing_path)) return;
		let path = file.wing_path.Physical.path;
		if (file.kind === "Symlink") {
			if (!platform.resolveSymlink) {
				toast.error("Resolving links requires the desktop app");
				return;
			}
			try {
				const [target, isDirectory] = await platform.resolveSymlink(path);
				path = target;
				if (isDirectory) {
					go({Physical: {...file.wing_path.Physical, path}});
					return;
				}
			} catch (error) {
				toast.error(`Cannot open link: ${error}`);
				return;
			}
		}
		await openWithDefault(path);
	};
}

/**
 * Opens a selection: one item opens like a double-click; several open each
 * file in its default app, and folders are skipped since only one can be
 * shown at a time (the first is entered when nothing else opens).
 */
export function useOpenFiles() {
	const openFile = useOpenFile();
	return async (files: File[]) => {
		const [first] = files;
		if (!first) return;
		if (files.length === 1) {
			await openFile(first);
			return;
		}
		const openable = files.filter((f) => f.kind !== "Directory" && !isVirtualFile(f));
		if (openable.length === 0) {
			await openFile(first);
			return;
		}
		if (openable.length > MAX_FILES_TO_OPEN) {
			toast.error(`Select at most ${MAX_FILES_TO_OPEN} files to open at once`);
			return;
		}
		for (const file of openable) await openFile(file);
	};
}
