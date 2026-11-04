# Start the Windows desktop app against disposable WingDrive state.
[CmdletBinding()]
param(
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$TauriArgs
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$repoRoot = Split-Path -Path $PSScriptRoot -Parent
$stateRoot = if ($env:WINGDRIVE_DEV_ROOT) { $env:WINGDRIVE_DEV_ROOT } else { Join-Path $env:LOCALAPPDATA 'wingdrive-dev' }
$instance = if ($env:WINGDRIVE_DEV_INSTANCE) { $env:WINGDRIVE_DEV_INSTANCE } else { 'desktop-dev' }

if ($instance -notmatch '^[A-Za-z0-9_-]{1,64}$') {
    Write-Error 'WINGDRIVE_DEV_INSTANCE must contain only letters, numbers, hyphens, and underscores.'
    exit 2
}

$env:WINGDRIVE_DATA_DIR = Join-Path $stateRoot 'data'
$env:WINGDRIVE_INSTANCE = $instance
if (-not $env:RUST_LOG) { $env:RUST_LOG = 'wing_core=info,WingDrive=info' }

New-Item -ItemType Directory -Force -Path $env:WINGDRIVE_DATA_DIR | Out-Null
Write-Host "WingDrive development state: $(Join-Path $env:WINGDRIVE_DATA_DIR "instances\$instance")"
Write-Host "Daemon instance: $instance"
Write-Host 'Production data directories are not used by this command.'

Set-Location (Join-Path $repoRoot 'apps\tauri')
& bun run tauri:dev @TauriArgs
exit $LASTEXITCODE
