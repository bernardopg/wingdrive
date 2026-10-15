import { useTranslation } from "../../../i18n";

export function EmptyView() {
  const { t } = useTranslation("explorer");
  return (
    <div className="flex items-center justify-center h-full">
      <div className="text-center">
        <div className="text-ink-dull text-sm">
          {t("emptyState.selectLocation")}
        </div>
      </div>
    </div>
  );
}
