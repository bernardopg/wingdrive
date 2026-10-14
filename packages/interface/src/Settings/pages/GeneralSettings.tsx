import { useEffect, useId, useState } from "react";
import { useForm } from "react-hook-form";
import { Copy, FolderOpen } from "@phosphor-icons/react";
import { toast } from "@wingdrive/primitives";
import clsx from "clsx";
import { useCoreQuery, useCoreMutation } from "../../contexts/WingDriveContext";
import { usePlatform, type DesktopSettings } from "../../contexts/PlatformContext";
import { useTranslation } from "../../i18n";
import {
  SettingsHeader,
  SettingsRow,
  SettingsSection,
  SwitchRow,
  buttonClass,
  errorMessage,
  inputClass,
} from "../components";

interface DeviceSettingsForm {
  name: string;
  slug: string;
}

// Mirrors device.update: letters, numbers and hyphens, at most 50 characters
function formatBuildDate(value: string, language: string) {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  return date.toLocaleString(language === "pt" ? "pt-BR" : "en-US", { dateStyle: "medium", timeStyle: "short" });
}

const SLUG_PATTERN = /^[\p{L}\p{N}-]{1,50}$/u;

export function GeneralSettings() {
  const { t, i18n } = useTranslation();
  const statusQuery = useCoreQuery({ type: "core.status", input: null as any });
  const configQuery = useCoreQuery({ type: "config.app.get", input: null as any });
  const updateDevice = useCoreMutation("device.update");
  const resetData = useCoreMutation("core.reset");
  const platform = usePlatform();
  const nameId = useId();
  const slugId = useId();
  const terminalId = useId();

  const [desktopSettings, setDesktopSettings] = useState<DesktopSettings | null>(null);
  useEffect(() => {
    platform.getDesktopSettings?.().then(setDesktopSettings).catch(() => setDesktopSettings(null));
  }, [platform]);

  const updateDesktopSetting = async <K extends keyof DesktopSettings>(
    key: K,
    value: DesktopSettings[K],
  ) => {
    if (!desktopSettings || !platform.setDesktopSettings) return;
    try {
      setDesktopSettings(await platform.setDesktopSettings({ ...desktopSettings, [key]: value }));
    } catch (error) {
      toast.error(t("general.saveFailed", { error: errorMessage(error) }));
    }
  };

  const { data: status } = statusQuery;
  const { data: config } = configQuery;
  const dataDir = config?.data_dir || status?.system?.data_directory;

  const deviceForm = useForm<DeviceSettingsForm>({
    defaultValues: {
      name: status?.device_info?.name || "",
      slug: status?.device_info?.slug || "",
    },
  });

  // Sync server values into the form, but never while the user has unsaved
  // edits: react-hook-form's `values` option resets on every reference change,
  // so any background refetch of core.status wiped in-progress typing and
  // cleared isDirty, making the Save button disappear mid-edit.
  useEffect(() => {
    if (!status?.device_info || deviceForm.formState.isDirty) return;
    deviceForm.reset({
      name: status.device_info.name || "",
      slug: status.device_info.slug || "",
    });
  }, [status, deviceForm]);

  const onDeviceSubmit = deviceForm.handleSubmit(async (data) => {
    try {
      await updateDevice.mutateAsync({
        name: data.name.trim(),
        slug: data.slug,
      });
      await statusQuery.refetch();
      deviceForm.reset(data);
      toast.success(t("general.device.saved"));
    } catch (error) {
      toast.error(t("general.device.saveFailed", { error: errorMessage(error) }));
    }
  });

  const copyDataDir = async () => {
    if (!dataDir) return;
    try {
      await navigator.clipboard.writeText(dataDir);
      toast.success(t("general.dataDir.copied"));
    } catch (error) {
      toast.error(errorMessage(error));
    }
  };

  // platform.confirm, not window.confirm: WebView2 returns true from
  // window.confirm without showing anything, which would wipe data unasked.
  const handleResetData = () => {
    platform.confirm(
      t("general.reset.confirm"),
      (confirmed) => {
        if (!confirmed) return;
        resetData.mutate(
          { confirm: true },
          {
            onSuccess: (result) => {
              toast.success(
                result.message || t("general.reset.done")
              );
            },
            onError: (error) => {
              toast.error(error.message || t("general.reset.failed"));
            },
          }
        );
      }
    );
  };

  const { errors, isDirty } = deviceForm.formState;

  return (
    <div className="space-y-6">
      <SettingsHeader title={t("sections.general")} description={t("general.description")} />

      <form onSubmit={onDeviceSubmit}>
        <SettingsSection title={t("general.device.title")}>
          <SettingsRow label={t("general.device.name")} hint={t("general.device.nameHint")} controlId={nameId} stacked>
            <input
              id={nameId}
              type="text"
              {...deviceForm.register("name", { required: true, validate: (v) => v.trim().length > 0 })}
              className={inputClass}
              placeholder={t("general.device.namePlaceholder")}
            />
          </SettingsRow>
          <SettingsRow label={t("general.device.slug")} hint={t("general.device.slugHint")} controlId={slugId} stacked>
            <input
              id={slugId}
              type="text"
              aria-invalid={!!errors.slug}
              {...deviceForm.register("slug", { required: true, pattern: SLUG_PATTERN })}
              className={clsx(inputClass, "font-mono", errors.slug && "ring-2 ring-status-error")}
              placeholder="my-computer"
            />
            {errors.slug && <p className="mt-1 text-xs text-status-error">{t("general.device.slugInvalid")}</p>}
          </SettingsRow>
          {isDirty && (
            <div className="flex justify-end gap-2 px-4 py-3">
              <button
                type="button"
                onClick={() => deviceForm.reset()}
                className={clsx(buttonClass, "text-ink-dull hover:bg-app-hover hover:text-ink")}
              >
                {t("cancel")}
              </button>
              <button
                type="submit"
                disabled={updateDevice.isPending}
                className={clsx(buttonClass, "bg-accent text-white hover:bg-accent-deep")}
              >
                {updateDevice.isPending ? t("saving") : t("saveChanges")}
              </button>
            </div>
          )}
        </SettingsSection>
      </form>

      {desktopSettings && (
        <SettingsSection title={t("general.background.title")}>
          <SwitchRow
            label={t("general.background.keepRunning")}
            hint={t("general.background.keepRunningHint")}
            checked={desktopSettings.keep_in_background}
            onChange={(value) => updateDesktopSetting("keep_in_background", value)}
          />
          <SwitchRow
            label={t("general.background.startAtLogin")}
            hint={t("general.background.startAtLoginHint")}
            checked={desktopSettings.start_at_login}
            onChange={(value) => updateDesktopSetting("start_at_login", value)}
          />
          <SettingsRow
            label={t("general.background.terminal")}
            hint={t("general.background.terminalHint")}
            controlId={terminalId}
            stacked
          >
            <input
              id={terminalId}
              type="text"
              defaultValue={desktopSettings.terminal_command ?? ""}
              onBlur={(e) => {
                const value = e.target.value.trim();
                if (value !== (desktopSettings.terminal_command ?? "")) {
                  void updateDesktopSetting("terminal_command", value || null);
                }
              }}
              className={clsx(inputClass, "font-mono")}
              placeholder="kitty --single-instance"
            />
          </SettingsRow>
        </SettingsSection>
      )}

      <SettingsSection title={t("general.version.title")}>
        <SettingsRow label={t("general.version.version")}>
          <span className="font-mono text-sm text-ink-dull">{status?.version || t("loading")}</span>
        </SettingsRow>
        <SettingsRow label={t("general.version.built")}>
          <span className="font-mono text-sm text-ink-dull">{status?.built_at ? formatBuildDate(status.built_at, i18n.language) : t("loading")}</span>
        </SettingsRow>
        <SettingsRow label={t("general.dataDir.title")} hint={t("general.dataDir.hint")} stacked>
          <div className="flex items-center gap-2">
            <code className="min-w-0 flex-1 truncate rounded-md bg-app px-2 py-1.5 text-xs text-ink-dull" title={dataDir}>
              {dataDir || t("loading")}
            </code>
            <button
              type="button"
              onClick={copyDataDir}
              disabled={!dataDir}
              title={t("general.dataDir.copy")}
              aria-label={t("general.dataDir.copy")}
              className="rounded-md p-1.5 text-ink-dull hover:bg-app-hover hover:text-ink disabled:opacity-50"
            >
              <Copy className="size-4" />
            </button>
            {platform.openPathDefault && (
              <button
                type="button"
                onClick={() => dataDir && platform.openPathDefault?.(dataDir).catch((e) => toast.error(errorMessage(e)))}
                disabled={!dataDir}
                title={t("general.dataDir.open")}
                aria-label={t("general.dataDir.open")}
                className="rounded-md p-1.5 text-ink-dull hover:bg-app-hover hover:text-ink disabled:opacity-50"
              >
                <FolderOpen className="size-4" />
              </button>
            )}
          </div>
        </SettingsRow>
      </SettingsSection>

      <section className="space-y-2">
        <h3 className="px-1 text-xs font-semibold uppercase tracking-wide text-status-error">
          {t("general.reset.zone")}
        </h3>
        <div className="flex flex-col gap-3 rounded-lg border border-status-error/30 bg-status-error/5 px-4 py-3 sm:flex-row sm:items-center sm:justify-between">
          <div>
            <p className="text-sm font-medium text-ink">{t("general.reset.title")}</p>
            <p className="mt-0.5 text-xs text-ink-dull">{t("general.reset.hint")}</p>
          </div>
          <button
            type="button"
            onClick={handleResetData}
            disabled={resetData.isPending}
            className={clsx(buttonClass, "shrink-0 bg-status-error text-white hover:bg-status-error/80")}
          >
            {resetData.isPending ? t("general.reset.pending") : t("general.reset.button")}
          </button>
        </div>
      </section>
    </div>
  );
}
