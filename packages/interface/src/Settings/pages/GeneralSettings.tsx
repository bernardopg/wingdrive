import { useEffect, useState } from "react";
import { useForm } from "react-hook-form";
import { toast } from "@wingdrive/primitives";
import { useCoreQuery, useCoreMutation } from "../../contexts/WingDriveContext";
import { usePlatform, type DesktopSettings } from "../../contexts/PlatformContext";

interface DeviceSettingsForm {
  name: string;
  slug: string;
}

export function GeneralSettings() {
  const statusQuery = useCoreQuery({ type: "core.status", input: null as any });
  const configQuery = useCoreQuery({ type: "config.app.get", input: null as any });
  const updateDevice = useCoreMutation("device.update");
  const resetData = useCoreMutation("core.reset");
  const platform = usePlatform();

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
      toast.error(`Failed to save: ${error instanceof Error ? error.message : String(error)}`);
    }
  };

  const { data: status } = statusQuery;
  const { data: config } = configQuery;

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
        name: data.name,
        slug: data.slug,
      });
      await statusQuery.refetch();
      deviceForm.reset(data);
      toast.success("Device settings saved");
    } catch (error) {
      toast.error(
        `Failed to save device settings: ${
          error instanceof Error ? error.message : String(error)
        }`
      );
    }
  });

  // platform.confirm, not window.confirm: WebView2 returns true from
  // window.confirm without showing anything, which would wipe data unasked.
  const handleResetData = () => {
    platform.confirm(
      "Reset All Data\n\nThis will permanently delete all libraries, settings, and cached data. The app will need to be restarted. Are you sure?",
      (confirmed) => {
        if (!confirmed) return;
        resetData.mutate(
          { confirm: true },
          {
            onSuccess: (result) => {
              toast.success(
                result.message || "Data has been reset. Please restart the application."
              );
            },
            onError: (error) => {
              toast.error(error.message || "Failed to reset data");
            },
          }
        );
      }
    );
  };

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-lg font-semibold text-ink mb-2">General</h2>
        <p className="text-sm text-ink-dull">
          Configure general application settings.
        </p>
      </div>

      <div className="space-y-4">
        {/* Device Configuration */}
        <form onSubmit={onDeviceSubmit} className="p-4 bg-app-box rounded-lg border border-app-line space-y-4">
          <h3 className="text-sm font-medium text-ink">Device</h3>

          <label className="block">
            <span className="text-sm font-medium text-ink mb-1 block">Device Name</span>
            <p className="text-xs text-ink-dull mb-2">
              User-friendly name for this device
            </p>
            <input
              type="text"
              {...deviceForm.register("name")}
              className="w-full px-3 py-2 bg-app border border-app-line rounded-md text-ink text-sm focus:outline-none focus:ring-2 focus:ring-accent"
              placeholder="My Computer"
            />
          </label>

          <label className="block">
            <span className="text-sm font-medium text-ink mb-1 block">Device Slug</span>
            <p className="text-xs text-ink-dull mb-2">
              Unique identifier for this device (alphanumeric and hyphens only)
            </p>
            <input
              type="text"
              {...deviceForm.register("slug")}
              className="w-full px-3 py-2 bg-app border border-app-line rounded-md text-ink text-sm focus:outline-none focus:ring-2 focus:ring-accent font-mono"
              placeholder="my-computer"
            />
          </label>

          <button
            type="submit"
            disabled={!deviceForm.formState.isDirty || updateDevice.isPending}
            className="px-4 py-2 bg-accent hover:bg-accent-deep text-white rounded-md text-sm font-medium transition-colors disabled:opacity-50"
          >
            {updateDevice.isPending ? "Saving..." : "Save Changes"}
          </button>
        </form>

        {desktopSettings && (
          <div className="p-4 bg-app-box rounded-lg border border-app-line space-y-4">
            <h3 className="text-sm font-medium text-ink">Background</h3>
            <label className="flex items-start justify-between gap-4">
              <span>
                <span className="text-sm font-medium text-ink block">Keep running in the background</span>
                <span className="text-xs text-ink-dull">
                  Closing the window keeps WingDrive in the tray so folders open instantly
                </span>
              </span>
              <input
                type="checkbox"
                className="mt-1 accent-accent"
                checked={desktopSettings.keep_in_background}
                onChange={(e) => updateDesktopSetting("keep_in_background", e.target.checked)}
              />
            </label>
            <label className="flex items-start justify-between gap-4">
              <span>
                <span className="text-sm font-medium text-ink block">Start at login</span>
                <span className="text-xs text-ink-dull">
                  Start WingDrive hidden in the tray when you log in
                </span>
              </span>
              <input
                type="checkbox"
                className="mt-1 accent-accent"
                checked={desktopSettings.start_at_login}
                onChange={(e) => updateDesktopSetting("start_at_login", e.target.checked)}
              />
            </label>
            <label className="block">
              <span className="text-sm font-medium text-ink block">Terminal</span>
              <span className="text-xs text-ink-dull block mb-2">
                Command for Open Terminal Here. Leave empty to use xdg-terminal-exec, $TERMINAL or an installed terminal.
              </span>
              <input
                type="text"
                defaultValue={desktopSettings.terminal_command ?? ""}
                onBlur={(e) => {
                  const value = e.target.value.trim();
                  if (value !== (desktopSettings.terminal_command ?? "")) {
                    void updateDesktopSetting("terminal_command", value || null);
                  }
                }}
                className="w-full px-3 py-2 bg-app border border-app-line rounded-md text-ink text-sm focus:outline-none focus:ring-2 focus:ring-accent font-mono"
                placeholder="kitty --single-instance"
              />
            </label>
          </div>
        )}

        {/* Version Info */}
        <div className="p-4 bg-app-box rounded-lg border border-app-line space-y-3">
          <h3 className="text-sm font-medium text-ink">Version Information</h3>
          <div className="flex justify-between items-center">
            <span className="text-sm text-ink">Version</span>
            <span className="text-sm text-ink-dull font-mono">
              {status?.version || "Loading..."}
            </span>
          </div>
          <div className="flex justify-between items-center">
            <span className="text-sm text-ink">Built</span>
            <span className="text-sm text-ink-dull font-mono">
              {status?.built_at || "Loading..."}
            </span>
          </div>
        </div>

        <div className="p-4 bg-app-box rounded-lg border border-app-line">
          <h3 className="text-sm font-medium text-ink mb-1">Data Directory</h3>
					<p className="text-xs text-ink-dull mb-2">Where WingDrive stores its data</p>
          <code className="block text-xs text-ink-dull bg-app rounded px-2 py-1 overflow-x-auto">
            {config?.data_dir || status?.system?.data_directory || "Loading..."}
          </code>
        </div>

        <div className="p-4 bg-app-box rounded-lg border border-app-line">
          <div className="flex items-center justify-between">
            <div>
              <h3 className="text-sm font-medium text-ink mb-1">Reset All Data</h3>
              <p className="text-xs text-ink-dull">
                Permanently delete all libraries and settings
              </p>
            </div>
            <button
              type="button"
              onClick={handleResetData}
              disabled={resetData.isPending}
              className="px-4 py-2 bg-red-600 hover:bg-red-700 disabled:opacity-50 text-white rounded-lg text-sm font-medium transition-colors"
            >
              {resetData.isPending ? "Resetting..." : "Reset"}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
