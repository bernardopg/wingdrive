import { useForm } from "react-hook-form";
import { useLibraryQuery, useLibraryMutation, useWingDriveClient } from "../../contexts/WingDriveContext";

// Only settings the daemon reads are shown. Thumbnail, AI tagging, sync and
// encryption flags are stored in the library config but nothing consumes
// them, so offering them as switches would promise behavior that does not exist.
interface LibrarySettingsForm {
  auto_track_system_volumes: boolean;
  auto_track_external_volumes: boolean;
}

export function LibrarySettings() {
  const client = useWingDriveClient();
  const libraryId = client.getCurrentLibraryId();
  const { data: config, refetch, isLoading } = useLibraryQuery(
    { type: "config.library.get", input: null as any },
    { enabled: !!libraryId }
  );
  const updateConfig = useLibraryMutation("config.library.update");

  const form = useForm<LibrarySettingsForm>({
    values: {
      auto_track_system_volumes: config?.auto_track_system_volumes ?? true,
      auto_track_external_volumes: config?.auto_track_external_volumes ?? false,
    },
  });

  const onSubmit = form.handleSubmit(async (data) => {
    await updateConfig.mutateAsync(data);
    refetch();
  });

  if (!libraryId) {
    return (
      <div className="space-y-6">
        <div>
          <h2 className="text-lg font-semibold text-ink mb-2">Library</h2>
          <p className="text-sm text-ink-dull">
            No library selected. Please select a library first.
          </p>
        </div>
      </div>
    );
  }

  if (isLoading) {
    return (
      <div className="space-y-6">
        <div>
          <h2 className="text-lg font-semibold text-ink mb-2">Library</h2>
          <p className="text-sm text-ink-dull">Loading...</p>
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-lg font-semibold text-ink mb-2">Library</h2>
        <p className="text-sm text-ink-dull">
          Configure settings for the current library.
        </p>
      </div>

      <form onSubmit={onSubmit} className="space-y-4">
        {/* Auto-Tracking Section */}
        <div className="p-4 bg-app-box rounded-lg border border-app-line space-y-4">
          <h3 className="text-sm font-medium text-ink">Auto-Tracking</h3>

          <label className="flex items-center justify-between">
            <div>
              <span className="text-sm text-ink">System Volumes</span>
              <p className="text-xs text-ink-dull">Track internal drives when the library opens</p>
            </div>
            <input
              type="checkbox"
              {...form.register("auto_track_system_volumes")}
              className="h-4 w-4 rounded border-app-line text-accent focus:ring-accent"
            />
          </label>

          <label className="flex items-center justify-between">
            <div>
              <span className="text-sm text-ink">External Volumes</span>
              <p className="text-xs text-ink-dull">Automatically track external drives when connected</p>
            </div>
            <input
              type="checkbox"
              {...form.register("auto_track_external_volumes")}
              className="h-4 w-4 rounded border-app-line text-accent focus:ring-accent"
            />
          </label>
        </div>

        {form.formState.isDirty && (
          <button
            type="submit"
            disabled={updateConfig.isPending}
            className="px-4 py-2 bg-accent hover:bg-accent-deep text-white rounded-md text-sm font-medium transition-colors disabled:opacity-50"
          >
            {updateConfig.isPending ? "Saving..." : "Save Changes"}
          </button>
        )}
      </form>
    </div>
  );
}
