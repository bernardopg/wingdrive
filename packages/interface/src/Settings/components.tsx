import { useQueryClient } from "@tanstack/react-query";
import { toast } from "@wingdrive/primitives";
import { Info, Warning } from "@phosphor-icons/react";
import clsx from "clsx";
import { useId, type ReactNode } from "react";
import {
  useCoreMutation,
  useCoreQuery,
  useLibraryMutation,
  useLibraryQuery,
  useWingDriveClient,
} from "../contexts/WingDriveContext";
import { i18n } from "../i18n";

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
}: {
  label: ReactNode;
  hint?: ReactNode;
  children: ReactNode;
  controlId?: string;
  stacked?: boolean;
}) {
  return (
    <div
      className={clsx(
        "flex gap-3 px-4 py-3",
        stacked ? "flex-col" : "items-center justify-between sm:gap-6"
      )}
    >
      <div className="min-w-0">
        <label htmlFor={controlId} className="block text-sm font-medium text-ink">
          {label}
        </label>
        {hint && <p className="mt-0.5 text-xs text-ink-dull">{hint}</p>}
      </div>
      <div className={clsx("shrink-0", stacked && "w-full")}>{children}</div>
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
    <SettingsRow label={label} hint={hint} controlId={id}>
      <Switch id={id} checked={checked} onChange={onChange} disabled={disabled} />
    </SettingsRow>
  );
}

export function Switch({
  id,
  checked,
  onChange,
  disabled,
}: {
  id?: string;
  checked: boolean;
  onChange: (checked: boolean) => void;
  disabled?: boolean;
}) {
  return (
    <button
      id={id}
      type="button"
      role="switch"
      aria-checked={checked}
      disabled={disabled}
      onClick={() => onChange(!checked)}
      className={clsx(
        "relative inline-flex h-5 w-9 items-center rounded-full transition-colors",
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
  "w-full rounded-md border border-app-line bg-app-input px-3 py-2 text-sm text-ink placeholder:text-ink-faint focus:outline-none focus:ring-2 focus:ring-accent";

export const buttonClass =
  "rounded-md px-4 py-2 text-sm font-medium transition-colors disabled:cursor-not-allowed disabled:opacity-50";

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
      <div>{children}</div>
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
  const query = useCoreQuery({ type: "config.app.get", input: null as any });
  const mutation = useCoreMutation("config.app.update");

  const save = async (patch: Record<string, unknown>) => {
    try {
      const result = await mutation.mutateAsync(patch as any);
      await queryClient.refetchQueries({ queryKey: ["config.app.get"] });
      if (result?.requires_restart) toast.info(i18n.t("settings:restartNeeded"));
    } catch (error) {
      toast.error(i18n.t("settings:saveFailed", { error: errorMessage(error) }));
    }
  };

  return { config: query.data, isLoading: query.isLoading, save, isSaving: mutation.isPending };
}

/** Library config for the current library, saved per change like {@link useAppConfig}. */
export function useLibraryConfig() {
  const client = useWingDriveClient();
  const libraryId = client.getCurrentLibraryId();
  const query = useLibraryQuery({ type: "config.library.get", input: null as any }, { enabled: !!libraryId });
  const mutation = useLibraryMutation("config.library.update");

  const save = async (patch: Record<string, unknown>) => {
    try {
      await mutation.mutateAsync(patch as any);
      await query.refetch();
    } catch (error) {
      toast.error(i18n.t("settings:saveFailed", { error: errorMessage(error) }));
    }
  };

  return { libraryId, config: query.data, isLoading: query.isLoading, save, isSaving: mutation.isPending };
}

/** Placeholder body for library pages when no library is open or it is loading. */
export function LibraryGate({ title, libraryId, isLoading }: { title: string; libraryId: string | null; isLoading: boolean }) {
  return (
    <div>
      <SettingsHeader title={title} description={i18n.t(libraryId ? "settings:loading" : "settings:noLibrary")} />
      {libraryId && isLoading && <div className="h-32 animate-pulse rounded-lg border border-app-line bg-app-box" />}
    </div>
  );
}
