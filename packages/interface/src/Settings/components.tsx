import { useQueryClient } from "@tanstack/react-query";
import type { UpdateAppConfigInput, UpdateLibraryConfigInput } from "@wingdrive/ts-client";
import { toast } from "@wingdrive/primitives";
import { Info, Warning } from "@phosphor-icons/react";
import clsx from "clsx";
import { useId, useState, type ReactNode } from "react";
import {
  useCoreMutation,
  useCoreQuery,
  useLibraryMutation,
  useLibraryQuery,
  useWingDriveClient,
} from "../contexts/WingDriveContext";
import { i18n, useTranslation } from "../i18n";

export function SettingsHeader({ title, description }: { title: string; description: string }) {
  return (
    <header className="mb-6">
      <h2 className="text-lg font-semibold text-ink">{title}</h2>
      <p className="mt-1 text-sm text-ink-dull">{description}</p>
    </header>
  );
}

export function SettingsSection({
  title,
  description,
  children,
}: {
  title?: string;
  description?: string;
  children: ReactNode;
}) {
  return (
    <section className="space-y-2">
      {title && (
        <div className="px-1">
          <h3 className="text-xs font-semibold uppercase tracking-wide text-ink-faint">{title}</h3>
          {description && <p className="mt-0.5 text-xs text-ink-dull">{description}</p>}
        </div>
      )}
      <div className="divide-y divide-app-line rounded-lg border border-app-line bg-app-box">{children}</div>
    </section>
  );
}

/** One setting: label and hint on the left, control on the right; stacks on narrow windows. */
export function SettingsRow({
  label,
  hint,
  children,
  controlId,
  stacked,
  fixedControl,
}: {
  label: ReactNode;
  hint?: ReactNode;
  children: ReactNode;
  controlId?: string;
  stacked?: boolean;
  fixedControl?: boolean;
}) {
  return (
    <div
      className={clsx(
        "flex gap-3 px-3 py-3 sm:px-4",
        stacked ? "flex-col" : clsx("items-center justify-between", !fixedControl && "flex-wrap")
      )}
    >
      <div className={clsx("min-w-0 break-words", !stacked && "flex-1", !stacked && !fixedControl && "basis-40")}>
        <label htmlFor={controlId} className="block text-sm font-medium text-ink">
          {label}
        </label>
        {hint && <p className="mt-0.5 text-xs leading-relaxed text-ink-dull [overflow-wrap:anywhere]">{hint}</p>}
      </div>
      <div className={clsx("max-w-full", stacked && "w-full")}>{children}</div>
    </div>
  );
}

/** A toggle row whose label is wired to an accessible switch. */
export function SwitchRow({
  label,
  hint,
  checked,
  onChange,
  disabled,
}: {
  label: string;
  hint?: string;
  checked: boolean;
  onChange: (checked: boolean) => void;
  disabled?: boolean;
}) {
  const id = useId();
  return (
    <SettingsRow label={label} hint={hint} controlId={id} fixedControl>
      <Switch id={id} checked={checked} onChange={onChange} disabled={disabled} />
    </SettingsRow>
  );
}

export function Switch({
  id,
  label,
  checked,
  onChange,
  disabled,
}: {
  id?: string;
  label?: string;
  checked: boolean;
  onChange: (checked: boolean) => void;
  disabled?: boolean;
}) {
  return (
    <button
      id={id}
      type="button"
      role="switch"
      aria-label={label}
      aria-checked={checked}
      disabled={disabled}
      onClick={() => onChange(!checked)}
      className={clsx(
        "relative inline-flex h-5 w-9 min-w-9 shrink-0 items-center align-middle rounded-full transition-colors",
        "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent focus-visible:ring-offset-2 focus-visible:ring-offset-app-box",
        "disabled:cursor-not-allowed disabled:opacity-50",
        checked ? "bg-accent" : "bg-app-line"
      )}
    >
      <span
        className={clsx(
          "inline-block size-4 rounded-full bg-white shadow-sm transition-transform",
          checked ? "translate-x-[18px]" : "translate-x-0.5"
        )}
      />
    </button>
  );
}

export const inputClass =
  "w-full min-w-0 rounded-md border border-app-line bg-app-input px-3 py-2 text-sm text-ink placeholder:text-ink-faint focus:outline-none focus:ring-2 focus:ring-accent";

export const buttonClass =
  "rounded-md px-4 py-2 text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent disabled:cursor-not-allowed disabled:opacity-50";

export function Notice({ tone = "info", children }: { tone?: "info" | "warning"; children: ReactNode }) {
  const Icon = tone === "warning" ? Warning : Info;
  return (
    <div
      className={clsx(
        "flex items-start gap-2 rounded-lg border px-3 py-2.5 text-sm",
        tone === "warning"
          ? "border-status-warning/25 bg-status-warning/10 text-status-warning"
          : "border-accent/25 bg-accent/10 text-ink"
      )}
    >
      <Icon className="mt-0.5 size-4 shrink-0" weight="fill" />
      <div className="min-w-0 [overflow-wrap:anywhere]">{children}</div>
    </div>
  );
}

export function errorMessage(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}

/**
 * App config plus a saver that applies each change immediately. Toggles save
 * on change instead of behind a Save button, so the switch always shows the
 * daemon's real state.
 */
export function useAppConfig() {
  const queryClient = useQueryClient();
  const query = useCoreQuery({ type: "config.app.get", input: null });
  const mutation = useCoreMutation("config.app.update");

  const [refreshing, setRefreshing] = useState(false);
  const save = async (patch: UpdateAppConfigInput) => {
    setRefreshing(true);
    try {
      const result = await mutation.mutateAsync(patch);
      if (!result.success) throw new Error(result.message);
      await queryClient.refetchQueries({ queryKey: ["config.app.get"] });
      if (result.requires_restart) toast.info(i18n.t("settings:restartNeeded"));
      return true;
    } catch (error) {
      toast.error(i18n.t("settings:saveFailed", { error: errorMessage(error) }));
      return false;
    } finally {
      setRefreshing(false);
    }
  };

  return { ...query, config: query.data, save, isSaving: mutation.isPending || refreshing };
}

/** Library config for the current library, saved per change like {@link useAppConfig}. */
export function useLibraryConfig() {
  const client = useWingDriveClient();
  const libraryId = client.getCurrentLibraryId();
  const query = useLibraryQuery({ type: "config.library.get", input: null }, { enabled: !!libraryId });
  const mutation = useLibraryMutation("config.library.update");

  const [refreshing, setRefreshing] = useState(false);
  const save = async (patch: UpdateLibraryConfigInput) => {
    setRefreshing(true);
    try {
      const result = await mutation.mutateAsync(patch);
      if (!result.success) throw new Error(result.message);
      await query.refetch();
    } catch (error) {
      toast.error(i18n.t("settings:saveFailed", { error: errorMessage(error) }));
    } finally {
      setRefreshing(false);
    }
  };

  return { ...query, libraryId, config: query.data, save, isSaving: mutation.isPending || refreshing };
}

export function LibraryGate({ title }: { title: string }) {
  const { t } = useTranslation("settings");
  return <SettingsHeader title={title} description={t("noLibrary")} />;
}

export function SettingsUnavailable({
  loading,
  retry,
}: {
  loading: boolean;
  retry: () => void;
}) {
  const { t } = useTranslation("settings");
  if (loading) return <p role="status" className="text-sm text-ink-dull">{t("loading")}</p>;
  return (
    <Notice tone="warning">
      <p role="alert">{t("loadFailed")}</p>
      <button type="button" className="mt-2 rounded text-accent underline focus-visible:ring-2 focus-visible:ring-accent" onClick={retry}>
        {t("retry")}
      </button>
    </Notice>
  );
}
