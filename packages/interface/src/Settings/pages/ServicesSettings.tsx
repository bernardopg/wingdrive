import { useTranslation } from "../../i18n";
import { Notice, SettingsHeader, SettingsSection, SwitchRow, useAppConfig } from "../components";

export function ServicesSettings() {
  const { t } = useTranslation();
  const { config, save, isSaving } = useAppConfig();
  const services = config?.services;
  const pairing = config?.proxy_pairing;

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
    </div>
  );
}
