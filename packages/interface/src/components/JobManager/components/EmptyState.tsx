import { CheckCircle } from "@phosphor-icons/react";
import { CARD_HEIGHT } from "../types";
import { useTranslation } from "../../../i18n";

export function EmptyState() {
  const { t } = useTranslation("dialogs");
  return (
    <div
      className="flex items-center justify-center px-4"
      style={{ height: CARD_HEIGHT }}
    >
      <div className="flex items-center gap-2 text-ink-faint">
        <CheckCircle size={16} weight="duotone" />
        <span className="text-sm">{t("jobs.noActiveJobs")}</span>
      </div>
    </div>
  );
}
