import { toast } from "@wingdrive/primitives";
import { useId, useState } from "react";
import { usePlatform } from "../../contexts/PlatformContext";
import { useTranslation } from "../../i18n";
import {
  Notice,
  SettingsUnavailable, SettingsHeader,
  SettingsRow,
  SettingsSection,
  SwitchRow,
  buttonClass,
  inputClass,
  errorMessage,
  useAppConfig,
} from "../components";

export function ServicesSettings() {
  const { t } = useTranslation();
  const { config, isLoading, refetch, save, isSaving } = useAppConfig();
  const services = config?.services;
  const pairing = config?.proxy_pairing;
  const wingdrop = config?.wingdrop;
  const platform = usePlatform();
  const folderId = useId();
  const [folderDraft, setFolderDraft] = useState<string | null>(null);
  const folder = folderDraft ?? wingdrop?.destination ?? "";
  const saveFolder = async (value: string) => {
    const next = value.trim();
    if (next === (wingdrop?.destination ?? "") || await save({ wingdrop_destination: next })) {
      setFolderDraft(null);
    }
  };
  const chooseFolder = async () => {
    try {
      const picked = await platform.openDirectoryPickerDialog?.({ title: t("services.wingdropFolder") });
      const path = Array.isArray(picked) ? picked[0] : picked;
      if (path) await saveFolder(path);
    } catch (error) {
      toast.error(t("services.wingdropChooseFailed", { error: errorMessage(error) }));
    }
  };
  if (!config) return (
    <div className="space-y-6">
      <SettingsHeader title={t("sections.services")} description={t("services.description")} />
      <SettingsUnavailable loading={isLoading} retry={() => void refetch()} />
    </div>
  );

  return (
    <div className="space-y-6">
      <SettingsHeader title={t("sections.services")} description={t("services.description")} />
      <Notice tone="warning">{t("services.restartWarning")}</Notice>

      <SettingsSection title={t("services.background")}>
        <SwitchRow
          label={t("services.networking")}
          hint={t("services.networkingHint")}
          checked={services?.networking_enabled ?? true}
          disabled={!config || isSaving}
          onChange={(value) => save({ networking_enabled: value })}
        />
        <SwitchRow
          label={t("services.volumeMonitoring")}
          hint={t("services.volumeMonitoringHint")}
          checked={services?.volume_monitoring_enabled ?? true}
          disabled={!config || isSaving}
          onChange={(value) => save({ volume_monitoring_enabled: value })}
        />
        <SwitchRow
          label={t("services.fsWatcher")}
          hint={t("services.fsWatcherHint")}
          checked={services?.fs_watcher_enabled ?? true}
          disabled={!config || isSaving}
          onChange={(value) => save({ fs_watcher_enabled: value })}
        />
        <SwitchRow
          label={t("services.statistics")}
          hint={t("services.statisticsHint")}
          checked={services?.statistics_listener_enabled ?? true}
          disabled={!config || isSaving}
          onChange={(value) => save({ statistics_listener_enabled: value })}
        />
      </SettingsSection>

      <SettingsSection title={t("services.pairing")} description={t("services.pairingDescription")}>
        <SwitchRow
          label={t("services.acceptVouched")}
          hint={t("services.acceptVouchedHint")}
          checked={pairing?.auto_accept_vouched ?? true}
          disabled={!config || isSaving || services?.networking_enabled === false}
          onChange={(value) => save({ proxy_pairing_auto_accept_vouched: value })}
        />
        <SwitchRow
          label={t("services.vouchToAll")}
          hint={t("services.vouchToAllHint")}
          checked={pairing?.auto_vouch_to_all ?? false}
          disabled={!config || isSaving || services?.networking_enabled === false}
          onChange={(value) => save({ proxy_pairing_auto_vouch_to_all: value })}
        />
      </SettingsSection>

      <SettingsSection title={t("services.wingdrop")} description={t("services.wingdropDescription")}>
        <SwitchRow
          label={t("services.wingdropEnabled")}
          hint={t("services.wingdropEnabledHint")}
          checked={wingdrop?.enabled ?? true}
          disabled={!config || isSaving}
          onChange={(value) => save({ wingdrop_enabled: value })}
        />
        <SwitchRow
          label={t("services.wingdropAutoAccept")}
          hint={t("services.wingdropAutoAcceptHint")}
          checked={wingdrop?.auto_accept ?? false}
          disabled={!config || isSaving || wingdrop?.enabled === false}
          onChange={(value) => save({ wingdrop_auto_accept: value })}
        />
        <SettingsRow label={t("services.wingdropFolder")} hint={t("services.wingdropFolderHint")} controlId={folderId} stacked>
          <form className="flex flex-wrap gap-2" onSubmit={(e) => { e.preventDefault(); void saveFolder(folder); }}>
            <input
              id={folderId}
              className={`${inputClass} flex-1 basis-40`}
              value={folder}
              placeholder={wingdrop?.effective_destination ?? ""}
              disabled={!config || isSaving || wingdrop?.enabled === false}
              onChange={(e) => setFolderDraft(e.target.value)}
            />
            {platform.openDirectoryPickerDialog && (
              <button
                type="button"
                className={`${buttonClass} bg-app-button text-ink hover:bg-app-hover shrink-0`}
                disabled={!config || isSaving || wingdrop?.enabled === false}
                onClick={() => void chooseFolder()}
              >
                {t("services.wingdropChoose")}
              </button>
            )}
            <button type="submit" className={`${buttonClass} bg-app-button text-ink hover:bg-app-hover`} disabled={isSaving || wingdrop?.enabled === false || folderDraft === null}>
              {t("saveChanges")}
            </button>
          </form>
        </SettingsRow>
      </SettingsSection>
    </div>
  );
}
