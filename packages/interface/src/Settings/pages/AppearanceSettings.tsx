import { Check, CircleHalf } from "@phosphor-icons/react";
import clsx from "clsx";
import { useId } from "react";
import { applyTheme, themePreviewClass } from "../../hooks/useTheme";
import { LANGUAGES, applyLanguage, useTranslation } from "../../i18n";
import { SettingsUnavailable, SettingsHeader, SettingsRow, SettingsSection, inputClass, useAppConfig } from "../components";

// Each entry's preview renders inside its own theme class, so swatches always
// match the real colors. Adaptive themes follow the system light/dark setting.
const THEME_GROUPS = [
  { id: "wingdrive", themes: ["system", "light", "dark", "midnight", "noir", "slate", "nord", "mocha"] },
  { id: "t3", themes: ["t3-code", "t3-chat", "grove", "ocean", "ember", "iris"] },
  { id: "community", themes: ["catppuccin", "gruvbox", "solarized", "rose-pine", "dracula", "tokyo-night"] },
];

const ADAPTIVE = new Set(["system", "t3-code", "t3-chat", "grove", "ocean", "ember", "iris", "catppuccin", "gruvbox", "solarized", "rose-pine"]);

export function AppearanceSettings() {
  const { t } = useTranslation();
  const { config, isLoading, isSaving, refetch, save } = useAppConfig();
  const languageId = useId();
  const theme = config?.preferences?.theme || "system";
  const language = config?.preferences?.language || "en";

  const handleThemeChange = async (themeId: string) => {
    if (themeId !== theme && await save({ theme: themeId })) applyTheme(themeId);
  };
  const handleLanguageChange = async (next: string) => {
    if (next !== language && await save({ language: next })) applyLanguage(next);
  };

  if (!config) return (
    <div className="space-y-6">
      <SettingsHeader title={t("sections.appearance")} description={t("appearance.description")} />
      <SettingsUnavailable loading={isLoading} retry={() => void refetch()} />
    </div>
  );

  return (
    <div className="space-y-6">
      <SettingsHeader title={t("sections.appearance")} description={t("appearance.description")} />

      <SettingsSection title={t("appearance.language")}>
        <SettingsRow label={t("appearance.language")} hint={t("appearance.languageHint")} controlId={languageId}>
          <select
            id={languageId}
            value={language}
            disabled={isSaving}
            onChange={(e) => void handleLanguageChange(e.target.value)}
            className={clsx(inputClass, "sm:w-56")}
          >
            {LANGUAGES.map((option) => (
              <option key={option.id} value={option.id}>
                {option.name}
              </option>
            ))}
          </select>
        </SettingsRow>
      </SettingsSection>

      {THEME_GROUPS.map((group) => (
        <section key={group.id} className="space-y-2">
          <h3 className="px-1 text-xs font-semibold uppercase tracking-wide text-ink-faint">
            {group.id === "wingdrive" ? t("appearance.theme") : t(`appearance.themeGroups.${group.id}`)}
          </h3>
          <div role="group" aria-label={t(`appearance.themeGroups.${group.id}`)} className="grid grid-cols-[repeat(auto-fit,minmax(min(100%,9rem),1fr))] gap-2">
            {group.themes.map((id) => (
              <ThemeOption
                key={id}
                id={id}
                label={t(`appearance.themes.${id}`)}
                adaptiveHint={ADAPTIVE.has(id) ? t("appearance.adaptive") : undefined}
                selected={theme === id}
                disabled={isSaving}
              onSelect={() => void handleThemeChange(id)}
              />
            ))}
          </div>
        </section>
      ))}
    </div>
  );
}

function ThemeOption({
  id,
  label,
  adaptiveHint,
  selected,
  onSelect,
  disabled,
}: {
  id: string;
  label: string;
  adaptiveHint?: string;
  selected: boolean;
  onSelect: () => void;
  disabled: boolean;
}) {
  return (
    <button
      type="button"
      aria-pressed={selected}
      disabled={disabled}
      onClick={onSelect}
      title={adaptiveHint ? `${label} · ${adaptiveHint}` : label}
      className={clsx(
        "group rounded-lg border disabled:opacity-60 p-1.5 text-left transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent",
        selected ? "border-accent bg-accent/10" : "border-app-line bg-app-box hover:border-ink-faint"
      )}
    >
      <div className={clsx("flex h-14 overflow-hidden rounded-md border border-app-line", themePreviewClass(id))}>
        <div className="w-1/4 space-y-1 border-r border-sidebar-line bg-sidebar p-1">
          <div className="h-1.5 rounded-sm bg-sidebar-selected" />
          <div className="h-1 w-3/4 rounded-sm bg-sidebar-ink-faint/50" />
          <div className="h-1 w-1/2 rounded-sm bg-sidebar-ink-faint/50" />
        </div>
        <div className="flex flex-1 flex-col gap-1 bg-app p-1.5">
          <div className="h-1 w-3/4 rounded-sm bg-ink-dull/60" />
          <div className="h-1 w-1/2 rounded-sm bg-ink-faint/40" />
          <div className="mt-auto h-2 w-1/3 rounded-sm bg-accent" />
        </div>
      </div>
      <div className="mt-1.5 flex items-center gap-1 px-0.5">
        <span className="min-w-0 flex-1 truncate text-xs font-medium text-ink">{label}</span>
        {adaptiveHint && <CircleHalf className="size-3 shrink-0 text-ink-faint" aria-label={adaptiveHint} />}
        {selected && <Check className="size-3 shrink-0 text-accent" weight="bold" />}
      </div>
    </button>
  );
}
