import { useTranslation } from "../../i18n";
import { Notice, SettingsHeader, SettingsSection, SwitchRow, useAppConfig } from "../components";

// The daemon always writes job logs and takes its log level from RUST_LOG,
// so only the debug switch, which the job logger reads at startup, is shown.
export function AdvancedSettings() {
  const { t } = useTranslation();
  const { config, save, isSaving } = useAppConfig();

  return (
    <div className="space-y-6">
      <SettingsHeader title={t("sections.advanced")} description={t("advanced.description")} />
      <Notice tone="warning">{t("advanced.warning")}</Notice>

      <SettingsSection title={t("advanced.jobLogging")} description={t("advanced.debugLogsBody")}>
        <SwitchRow
          label={t("advanced.debugLogs")}
          hint={t("advanced.debugLogsHint")}
          checked={config?.job_logging?.include_debug ?? false}
          disabled={!config || isSaving}
          onChange={(value) => save({ job_logging_include_debug: value })}
        />
      </SettingsSection>
    </div>
  );
}
