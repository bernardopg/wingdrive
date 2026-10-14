import { useTranslation } from "../../i18n";
import { LibraryGate, Notice, SettingsHeader, SettingsSection, SwitchRow, useLibraryConfig } from "../components";

type IndexerKey = "no_system_files" | "no_git" | "no_dev_dirs" | "no_hidden" | "gitignore" | "only_images";

const DEFAULTS: Record<IndexerKey, boolean> = {
  no_system_files: true,
  no_git: true,
  no_dev_dirs: true,
  no_hidden: false,
  gitignore: true,
  only_images: false,
};

const EXCLUSIONS: [IndexerKey, string][] = [
  ["no_system_files", "skipSystem"],
  ["no_git", "skipGit"],
  ["no_dev_dirs", "skipDev"],
  ["no_hidden", "skipHidden"],
];

const FILTERS: [IndexerKey, string][] = [
  ["gitignore", "gitignore"],
  ["only_images", "onlyImages"],
];

export function IndexerSettings() {
  const { t } = useTranslation();
  const { libraryId, config, isLoading, save, isSaving } = useLibraryConfig();

  if (!libraryId || isLoading) {
    return <LibraryGate title={t("sections.indexer")} libraryId={libraryId} isLoading={isLoading} />;
  }

  const row = ([key, label]: [IndexerKey, string]) => (
    <SwitchRow
      key={key}
      label={t(`indexer.${label}`)}
      hint={t(`indexer.${label}Hint`)}
      checked={config?.indexer?.[key] ?? DEFAULTS[key]}
      disabled={isSaving}
      onChange={(value) => save({ [key]: value })}
    />
  );

  return (
    <div className="space-y-6">
      <SettingsHeader title={t("sections.indexer")} description={t("indexer.description")} />
      <Notice>{t("indexer.appliesNextScan")}</Notice>
      <SettingsSection title={t("indexer.exclusions")}>{EXCLUSIONS.map(row)}</SettingsSection>
      <SettingsSection title={t("indexer.filters")}>{FILTERS.map(row)}</SettingsSection>
    </div>
  );
}
