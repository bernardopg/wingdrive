import { useQuery } from "@tanstack/react-query";
import { usePlatform, type OpenResult } from "../contexts/PlatformContext";
import { toast } from "@wingdrive/primitives";
import { i18n, useTranslation } from "../i18n";

export function useOpenWith(paths: string[]) {
	const { t } = useTranslation("common");
	const platform = usePlatform();

	const { data: apps, isLoading, refetch } = useQuery({
		queryKey: ["openWith", ...paths],
		queryFn: async () => {
			if (!platform.getAppsForPaths) {
				return [];
			}
			return platform.getAppsForPaths(paths);
		},
		enabled: paths.length > 0 && !!platform.getAppsForPaths,
	});

	const openWithDefault = async (path: string) => {
		if (!platform.openPathDefault) {
			toast.error(t("openWith.unsupported"));
			return;
		}

		try {
			const result = await platform.openPathDefault(path);
			handleOpenResult(result);
		} catch (e) {
			toast.error(t("openWith.failedFile", { error: String(e) }));
		}
	};

	const openWithApp = async (path: string, appId: string) => {
		if (!platform.openPathWithApp) {
			toast.error(t("openWith.unsupported"));
			return;
		}

		try {
			const result = await platform.openPathWithApp(path, appId);
			handleOpenResult(result);
		} catch (e) {
			toast.error(t("openWith.failedFile", { error: String(e) }));
		}
	};

	const openMultipleWithApp = async (paths: string[], appId: string) => {
		if (!platform.openPathsWithApp) {
			toast.error(t("openWith.unsupported"));
			return;
		}

		try {
			const results = await platform.openPathsWithApp(paths, appId);
			results.forEach(handleOpenResult);
		} catch (e) {
			toast.error(t("openWith.failedFiles", { error: String(e) }));
		}
	};

	const setDefaultApp = async (path: string, appId: string, appName: string) => {
		if (!platform.setDefaultAppForPath) return;
		try {
			await platform.setDefaultAppForPath(path, appId);
			await refetch();
			toast.success(t("openWith.defaultSet", { appName }));
		} catch (e) {
			toast.error(t("openWith.defaultFailed", { error: String(e) }));
		}
	};

	return {
		apps: apps ?? [],
		canSetDefault: !!platform.setDefaultAppForPath,
		setDefaultApp,
		isLoading,
		openWithDefault,
		openWithApp,
		openMultipleWithApp,
	};
}

function handleOpenResult(result: OpenResult) {
	switch (result.status) {
		case "success":
			// Silent success
			break;
		case "file_not_found":
			toast.error(i18n.t("common:openWith.fileNotFound", { path: result.path }));
			break;
		case "app_not_found":
			toast.error(i18n.t("common:openWith.appNotFound", { appId: result.app_id }));
			break;
		case "permission_denied":
			toast.error(i18n.t("common:openWith.permissionDenied", { path: result.path }));
			break;
		case "platform_error":
			toast.error(i18n.t("common:openWith.error", { error: result.message }));
			break;
	}
}
