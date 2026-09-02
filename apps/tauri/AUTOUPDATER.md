# WingDrive desktop updates

Automatic updates are disabled until WingDrive owns a signing key and the
release workflow publishes signed updater artifacts. Normal unsigned desktop
bundles are still produced by `.github/workflows/release.yml` and attached to
this repository's GitHub release.

To enable updates later:

1. Create and securely store a WingDrive Tauri signing key.
2. Add the public key and this repository's `latest.json` endpoint to
   `src-tauri/tauri.conf.json`.
3. Set `bundle.createUpdaterArtifacts` to `true`.
4. Provide the private key to the release workflow through GitHub Actions
   secrets.
5. Upload `latest.json`, signatures, and platform archives in the release job.
6. Verify a signed update from an older installed WingDrive build before
   enabling the updater in production.

Do not reuse the original project's signing key or update endpoint.
