# Installs a Windows bundle, proves it runs, then proves it uninstalls.
#
# Usage: windows_smoke.ps1 -Kind nsis|msi -Installer <path> -DaemonSmoke <path to daemon_smoke.py>
#
# Tests the installed files rather than the build output, so a sidecar or DLL
# missing from the bundle fails here instead of on a user's machine.
[CmdletBinding()]
param(
    [Parameter(Mandatory)][ValidateSet('nsis', 'msi')][string]$Kind,
    [Parameter(Mandatory)][string]$Installer,
    [Parameter(Mandatory)][string]$DaemonSmoke
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$workRoot = if ($env:RUNNER_TEMP) { $env:RUNNER_TEMP } else { $env:TEMP }
$installDir = Join-Path $workRoot "wingdrive-smoke-$Kind"
$dataDir = Join-Path $workRoot "wingdrive-smoke-$Kind-data"
$msiLog = Join-Path $workRoot "wingdrive-smoke-$Kind.log"
$installerPath = (Resolve-Path -LiteralPath $Installer).Path

Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
using System.Text;

namespace WingDriveSmoke {
    public static class Windows {
        delegate bool EnumWindowsProc(IntPtr window, IntPtr data);

        [DllImport("user32.dll")]
        static extern bool EnumWindows(EnumWindowsProc callback, IntPtr data);
        [DllImport("user32.dll")]
        static extern bool IsWindowVisible(IntPtr window);
        [DllImport("user32.dll")]
        static extern uint GetWindowThreadProcessId(IntPtr window, out uint processId);
        [DllImport("user32.dll", CharSet = CharSet.Unicode)]
        static extern int GetWindowText(IntPtr window, StringBuilder text, int count);

        // Title of the first visible, titled top-level window the process owns, or null.
        public static string VisibleTitle(uint processId) {
            string found = null;
            EnumWindows((window, data) => {
                uint owner;
                GetWindowThreadProcessId(window, out owner);
                if (owner != processId || !IsWindowVisible(window)) return true;
                var text = new StringBuilder(256);
                GetWindowText(window, text, text.Capacity);
                if (text.Length == 0) return true;
                found = text.ToString();
                return false;
            }, IntPtr.Zero);
            return found;
        }
    }
}
'@

function Assert-ExitCode([int]$Code, [string]$Step) {
    # 3010 means the MSI succeeded and asks for a reboot.
    if ($Code -ne 0 -and $Code -ne 3010) {
        if (Test-Path -LiteralPath $msiLog) { Get-Content -LiteralPath $msiLog -Tail 40 | Write-Host }
        throw "$Step failed with exit code $Code"
    }
}

function Stop-InstalledProcesses {
    Get-Process -ErrorAction SilentlyContinue |
        Where-Object { $_.Path -and $_.Path.StartsWith($installDir, [StringComparison]::OrdinalIgnoreCase) } |
        ForEach-Object {
            $_.Kill()
            $_.WaitForExit(10000) | Out-Null
        }
}

function Uninstall-Bundle {
    if ($Kind -eq 'nsis') {
        $uninstaller = Join-Path $installDir 'uninstall.exe'
        if (Test-Path -LiteralPath $uninstaller) {
            $process = Start-Process -FilePath $uninstaller -ArgumentList '/S' -Wait -PassThru
            Assert-ExitCode $process.ExitCode 'NSIS uninstall'
        }
    } else {
        $process = Start-Process -FilePath 'msiexec.exe' -Wait -PassThru -ArgumentList @(
            '/x', "`"$installerPath`"", '/qn', '/norestart', '/l*v', "`"$msiLog`""
        )
        Assert-ExitCode $process.ExitCode 'MSI uninstall'
    }
}

# The NSIS uninstaller leaves its folder in this key, and the MSI's registry search then
# overrides INSTALLDIR with it. Start from a clean key so each bundle lands where asked.
Remove-Item -Path 'HKCU:\Software\WingDrive contributors' -Recurse -ErrorAction SilentlyContinue

Write-Host "Installing $installerPath into $installDir"
if ($Kind -eq 'nsis') {
    # NSIS reads /D literally, so it stays last and unquoted.
    $process = Start-Process -FilePath $installerPath -ArgumentList @('/S', "/D=$installDir") -Wait -PassThru
    Assert-ExitCode $process.ExitCode 'NSIS install'
} else {
    $process = Start-Process -FilePath 'msiexec.exe' -Wait -PassThru -ArgumentList @(
        '/i', "`"$installerPath`"", '/qn', '/norestart', "INSTALLDIR=`"$installDir`"", '/l*v', "`"$msiLog`""
    )
    Assert-ExitCode $process.ExitCode 'MSI install'
}

try {
    foreach ($expected in 'WingDrive.exe', 'wing-daemon.exe', 'wing.exe', 'avcodec-*.dll', 'libheif.dll') {
        if (-not (Test-Path -Path (Join-Path $installDir $expected))) {
            Get-ChildItem -LiteralPath $installDir | Select-Object -ExpandProperty Name | Write-Host
            throw "$expected is missing from the $Kind install"
        }
    }
    Write-Host 'PASS: installed files are present'

    & python $DaemonSmoke (Join-Path $installDir 'wing-daemon.exe')
    if ($LASTEXITCODE -ne 0) { throw 'The installed daemon failed its smoke test' }

    # Named instance and data directory, so this never touches a real library.
    New-Item -ItemType Directory -Force -Path $dataDir | Out-Null
    $env:WINGDRIVE_DATA_DIR = $dataDir
    $env:WINGDRIVE_INSTANCE = "release-smoke-$Kind"
    $app = Start-Process -FilePath (Join-Path $installDir 'WingDrive.exe') -PassThru
    # The main window starts hidden and the frontend shows it once it has loaded, so a
    # visible "WingDrive" window proves the WebView rendered. MainWindowHandle would
    # also accept the single-instance plugin's hidden message window.
    $deadline = (Get-Date).AddSeconds(90)
    $title = $null
    while ((Get-Date) -lt $deadline) {
        if ($app.HasExited) { throw "WingDrive.exe exited with code $($app.ExitCode)" }
        $title = [WingDriveSmoke.Windows]::VisibleTitle([uint32]$app.Id)
        if ($title -like 'WingDrive*') { break }
        Start-Sleep -Seconds 1
    }
    if ($title -notlike 'WingDrive*') { throw 'WingDrive.exe showed no main window within 90 seconds' }
    Start-Sleep -Seconds 10
    if ($app.HasExited) { throw "WingDrive.exe exited after opening, with code $($app.ExitCode)" }
    if ([WingDriveSmoke.Windows]::VisibleTitle([uint32]$app.Id) -notlike 'WingDrive*') {
        throw 'The WingDrive main window closed within 10 seconds of opening'
    }
    Write-Host "PASS: installed app showed its main window ($title)"
} finally {
    Stop-InstalledProcesses
    Uninstall-Bundle
}

# The NSIS uninstaller copies itself to a temporary folder and returns before it finishes.
$deadline = (Get-Date).AddSeconds(60)
while ((Test-Path -LiteralPath (Join-Path $installDir 'WingDrive.exe')) -and (Get-Date) -lt $deadline) {
    Start-Sleep -Seconds 1
}
if (Test-Path -LiteralPath (Join-Path $installDir 'WingDrive.exe')) {
    throw "WingDrive.exe is still present after the $Kind uninstall"
}
Write-Host "PASS: $Kind bundle uninstalls cleanly"
