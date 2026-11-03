# Refuse to eject system, root and /home mounts

Branch: `bernardopg:fix/eject-system-mount-guard` (commit `8057bce4a`), base `spacedriveapp/spacedrive:main` at `4d41a379f`.

## Problem

`VolumeEjectAction` (`core/src/ops/volumes/eject/action.rs`) only checks that the volume is mounted, then runs `umount <mount_point>` on Linux or `diskutil eject <mount_point>` on macOS. Nothing stops a client from asking it to eject the system volume, a filesystem root (`/`) or `/home`. Usually the OS refuses because the mount is busy, but a lazy or privileged daemon can take the running system down with it, and the UI has no reason to offer these volumes as ejectable anyway.

## Fix

A `protected_eject_reason(mount_point, mount_type)` check runs before the platform call. It refuses:

- volumes with `MountType::System`,
- filesystem roots (an empty path or one with no parent),
- `/home`.

Refused requests return `VolumeEjectOutput { success: false, message: Some(reason) }`, the same shape as the existing "Volume is not mounted" case, so clients need no change. External, network and user mounts behave as before.

## Test

Unit tests in `action.rs`:

- `protects_system_mounts_from_ejection`: `/` and `/home` with `MountType::User`, and `/mnt/data` with `MountType::System`, are refused.
- `allows_external_and_network_mounts`: `/run/media/user/usb` (External) and `/mnt/share` (Network) are allowed.

## How verified

- `cargo test -p sd-core --lib ops::volumes::eject`: 2 passed.
- `cargo fmt` clean; `cargo clippy -p sd-core --lib` shows no new warnings.
