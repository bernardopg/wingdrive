import { useEffect, useId, useState } from "react";
import { usePlatform } from "../../contexts/PlatformContext";
import { useTranslation } from "../../i18n";
import {
  Notice,
  SettingsHeader,
  SettingsRow,
  SettingsSection,
  SwitchRow,
  buttonClass,
  inputClass,
  useAppConfig,
} from "../components";

export function ServicesSettings() {
  const { t } = useTranslation();
  const { config, save, isSaving } = useAppConfig();
  const services = config?.services;
  const pairing = config?.proxy_pairing;
  const wingdrop = config?.wingdrop;
  const platform = usePlatform();
  const folderId = useId();
  const [folder, setFolder] = useState("");
  useEffect(() => setFolder(wingdrop?.destination ?? ""), [wingdrop?.destination]);
  const saveFolder = (value: string) => {
    if (value.trim() !== (wingdrop?.destination ?? "")) void save({ wingdrop_destination: value.trim() });
  };
  const chooseFolder = async () => {
    const picked = await platform.openDirectoryPickerDialog?.({ title: t("services.wingdropFolder") });
    const path = Array.isArray(picked) ? picked[0] : picked;
    if (path) {
      setFolder(path);
      saveFolder(path);
    }
  };

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
          <div className="flex gap-2">
            <input
              id={folderId}
              className={inputClass}
              value={folder}
              placeholder={wingdrop?.effective_destination ?? ""}
              disabled={!config || isSaving || wingdrop?.enabled === false}
              onChange={(e) => setFolder(e.target.value)}
              onBlur={(e) => saveFolder(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter") saveFolder(e.currentTarget.value);
              }}
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
          </div>
        </SettingsRow>
      </SettingsSection>
    </div>
  );
}
