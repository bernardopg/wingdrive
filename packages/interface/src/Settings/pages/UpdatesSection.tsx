import { useEffect, useState } from "react";
import clsx from "clsx";
import {
  usePlatform,
  type AvailableUpdate,
  type DesktopSettings,
  type UpdaterInfo,
} from "../../contexts/PlatformContext";
import { useTranslation } from "../../i18n";
import { SettingsRow, SettingsSection, SwitchRow, buttonClass, errorMessage } from "../components";

type CheckState =
  | { kind: "idle" }
  | { kind: "checking" }
  | { kind: "upToDate" }
  | { kind: "available"; update: AvailableUpdate }
  | { kind: "installing"; percent: number | null }
  | { kind: "failed"; error: string };

/**
 * Updates in Settings > General. Copies installed by a package manager (the AUR
 * packages, the `.deb`) say so and offer nothing to download, because an update
 * written behind pacman or apt would leave their database wrong.
 */
export function UpdatesSection({
  desktopSettings,
  onSettingChange,
}: {
  desktopSettings: DesktopSettings | null;
  onSettingChange: (value: boolean) => void;
}) {
  const { t } = useTranslation();
  const platform = usePlatform();
  const [info, setInfo] = useState<UpdaterInfo | null>(null);
  const [state, setState] = useState<CheckState>({ kind: "idle" });

  useEffect(() => {
    platform.getUpdaterInfo?.().then(setInfo).catch(() => setInfo(null));
  }, [platform]);

  if (!info) return null;

  const check = async () => {
    if (!platform.checkForUpdate) return;
    setState({ kind: "checking" });
    try {
      const update = await platform.checkForUpdate();
      setState(update ? { kind: "available", update } : { kind: "upToDate" });
    } catch (error) {
      setState({ kind: "failed", error: errorMessage(error) });
    }
  };

  const install = async () => {
    if (!platform.installUpdate) return;
    setState({ kind: "installing", percent: null });
    try {
      await platform.installUpdate(({ downloaded, total }) =>
        setState({
          kind: "installing",
          percent: total ? Math.min(100, Math.round((downloaded / total) * 100)) : null,
        }),
      );
    } catch (error) {
      setState({ kind: "failed", error: errorMessage(error) });
    }
  };

  const status = (() => {
    switch (state.kind) {
      case "checking":
        return t("general.updates.checking");
      case "upToDate":
        return t("general.updates.upToDate");
      case "available":
        return t("general.updates.available", { version: state.update.version });
      case "installing":
        return state.percent === null
          ? t("general.updates.installing")
          : t("general.updates.downloading", { percent: state.percent });
      case "failed":
        return t("general.updates.failed", { error: state.error });
      default:
        return null;
    }
  })();

  const busy = state.kind === "checking" || state.kind === "installing";

  return (
    <SettingsSection title={t("general.updates.title")}>
      <SettingsRow
        label={t("general.updates.version", { version: info.current_version })}
        hint={
          info.channel === "self_update"
            ? (status ?? t("general.updates.selfHint"))
            : info.channel === "package_manager"
              ? t("general.updates.packageManagerHint")
              : t("general.updates.unsupportedHint")
        }
      >
        {info.channel === "self_update" &&
          (state.kind === "available" ? (
            <button
              type="button"
              onClick={install}
              className={clsx(buttonClass, "bg-accent text-white hover:bg-accent-deep")}
            >
              {t("general.updates.install")}
            </button>
          ) : (
            <button type="button" onClick={check} disabled={busy} className={buttonClass}>
              {t("general.updates.check")}
            </button>
          ))}
      </SettingsRow>
      {info.channel === "self_update" && desktopSettings && (
        <SwitchRow
          label={t("general.updates.automatic")}
          hint={t("general.updates.automaticHint")}
          checked={desktopSettings.check_for_updates}
          onChange={onSettingChange}
        />
      )}
    </SettingsSection>
  );
}
