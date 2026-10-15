import { ShieldCheck } from "@phosphor-icons/react";
import { usePlatform } from "../../contexts/PlatformContext";
import { useTranslation } from "../../i18n";
import { SettingsHeader, SettingsSection } from "../components";

// WingDrive ships no telemetry client, so this page states that instead of
// offering a switch that would change nothing.
export function PrivacySettings() {
  const { t } = useTranslation();
  const platform = usePlatform();
  return (
    <div className="space-y-6">
      <SettingsHeader title={t("sections.privacy")} description={t("privacy.description")} />
      <SettingsSection title={t("privacy.telemetry")}>
        <div className="flex gap-3 px-4 py-3">
          <ShieldCheck className="mt-0.5 size-5 shrink-0 text-status-success" weight="fill" />
          <div className="space-y-2">
            <p className="text-sm text-ink">{t("privacy.telemetryBody")}</p>
            <button
              type="button"
              onClick={() => platform.openLink("https://github.com/bernardopg/wingdrive/blob/main/SECURITY.md")}
              className="text-xs text-accent hover:underline"
            >
              {t("privacy.securityPolicy")}
            </button>
          </div>
        </div>
      </SettingsSection>
    </div>
  );
}
