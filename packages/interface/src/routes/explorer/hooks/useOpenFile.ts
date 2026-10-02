import type { File, WingPath } from "@wingdrive/ts-client";
import { isVirtualFile } from "@wingdrive/ts-client";
import { toast } from "@wingdrive/primitives";
import { usePlatform } from "../../../contexts/PlatformContext";
import { useOpenWith } from "../../../hooks/useOpenWith";
import { useExplorer } from "../context";

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
