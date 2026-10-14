import { useTranslation } from "../../i18n";
import { LibraryGate, SettingsHeader, SettingsSection, SwitchRow, useLibraryConfig } from "../components";

// Only settings the daemon reads are shown. Thumbnail, AI tagging, sync and
// encryption flags are stored in the library config but nothing consumes
// them, so offering them as switches would promise behavior that does not exist.
export function LibrarySettings() {
  const { t } = useTranslation();
  const { libraryId, config, isLoading, save, isSaving } = useLibraryConfig();

  if (!libraryId || isLoading) {
    return <LibraryGate title={t("sections.library")} libraryId={libraryId} isLoading={isLoading} />;
  }

  return (
    <div className="space-y-6">
      <SettingsHeader title={t("sections.library")} description={t("library.description")} />

      <SettingsSection title={t("library.autoTracking")}>
        <SwitchRow
          label={t("library.systemVolumes")}
          hint={t("library.systemVolumesHint")}
          checked={config?.auto_track_system_volumes ?? true}
          disabled={isSaving}
          onChange={(value) => save({ auto_track_system_volumes: value })}
        />
        <SwitchRow
          label={t("library.externalVolumes")}
          hint={t("library.externalVolumesHint")}
          checked={config?.auto_track_external_volumes ?? false}
          disabled={isSaving}
          onChange={(value) => save({ auto_track_external_volumes: value })}
        />
      </SettingsSection>
    </div>
  );
}
