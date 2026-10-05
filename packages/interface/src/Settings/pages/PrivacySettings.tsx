// WingDrive ships no telemetry client, so this page states that instead of
// offering a switch that would change nothing.
export function PrivacySettings() {
  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-lg font-semibold text-ink mb-2">Privacy</h2>
        <p className="text-sm text-ink-dull">
          What WingDrive shares about you.
        </p>
      </div>

      <div className="p-4 bg-app-box rounded-lg border border-app-line space-y-2">
        <h3 className="text-sm font-medium text-ink">Telemetry</h3>
        <p className="text-xs text-ink-dull">
          WingDrive does not collect usage data. Your files, file names and
          library contents stay on your devices unless you share them.
        </p>
        <a
          href="https://github.com/bernardopg/wingdrive/blob/main/SECURITY.md"
          target="_blank"
          rel="noopener noreferrer"
          className="text-xs text-accent hover:underline inline-block"
        >
          Read the security policy
        </a>
      </div>
    </div>
  );
}
