# Set default value of 0 for external command exit code
$LASTEXITCODE = 0
# Enables strict mode, which causes PowerShell to treat uninitialized variables, undefined functions, and other common errors as terminating errors.
$ErrorActionPreference = if ($env:CI) { 'Stop' } else { 'Inquire' }
Set-StrictMode -Version Latest

function Reset-Path {
    $env:Path = [System.Environment]::ExpandEnvironmentVariables(
        [System.Environment]::GetEnvironmentVariable('Path', 'Machine') +
        [IO.Path]::PathSeparator +
        [System.Environment]::GetEnvironmentVariable('Path', 'User')
    )
}

# Verify if environment is Windows 64-bit and if the user is an administrator
if ((-not [string]::IsNullOrEmpty($env:PROCESSOR_ARCHITEW6432)) -or (
        "$env:PROCESSOR_ARCHITECTURE" -eq 'ARM64'
    ) -or (
        -not [System.Environment]::Is64BitOperatingSystem
        # Powershell >= 6 is cross-platform, check if running on Windows
    ) -or (($PSVersionTable.PSVersion.Major -ge 6) -and (-not $IsWindows))
) {
    $ErrorActionPreference = 'Continue'
    Write-Host # There is no oficial ffmpeg binaries for Windows 32 or ARM
    if (Test-Path "$($env:WINDIR)\SysNative\WindowsPowerShell\v1.0\powershell.exe" -PathType Leaf) {
        throw 'You are using PowerShell (32-bit), please re-run in PowerShell (64-bit)'
    } else {
        throw 'This script is only supported on Windows 64-bit'
    }
    Exit 1
} elseif (
    -not ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
) {
    # Start a new PowerShell process with administrator privileges and set the working directory to the directory where the script is located
    $proc = Start-Process -PassThru -Wait -FilePath 'PowerShell.exe' -Verb RunAs -ArgumentList "-NoProfile -ExecutionPolicy Bypass -File `"$($MyInvocation.MyCommand.Definition)`"" -WorkingDirectory "$PSScriptRoot"
    # Reset path so the user doesn't have to restart the shell to use the tools installed by this script
    Reset-Path
    Exit $proc.ExitCode
}

function Exit-WithError($err, $help = $null) {
    if ($null -ne $help) {
        Write-Host
        Write-Host $help -ForegroundColor DarkRed
    }
    throw $err
    Exit 1
}

function Add-DirectoryToPath($directory) {
    Reset-Path
    if ($env:Path.Split([IO.Path]::PathSeparator) -notcontains $directory) {
        [System.Environment]::SetEnvironmentVariable(
            'Path',
            [System.Environment]::GetEnvironmentVariable('Path', 'User') + [IO.Path]::PathSeparator + $directory,
            'User'
        )

        if ($env:CI) {
            # If running in CI, we need to use GITHUB_PATH instead of the normal PATH env variables
            Add-Content $env:GITHUB_PATH "$directory`n"
        }
    }
    Reset-Path
}

# Reset PATH to ensure the script doesn't have stale Path entries
Reset-Path

# Get project dir (get grandparent dir from script location: <PROJECT_ROOT>\scripts\setup.ps1)
$projectRoot = Split-Path -Path $PSScriptRoot -Parent
$packageJson = Get-Content -Raw -Path "$projectRoot\package.json" | ConvertFrom-Json

# Valid winget exit status
$wingetValidExit = 0, -1978335189, -1978335153, -1978335135

# Currently LLVM >= 16 is not supported due to incompatibilities with ffmpeg-sys-next
# See https://github.com/bernardopg/wingdrive/issues/677
$llvmVersion = [Version]'15.0.7'
$libclangDir = "$env:SystemDrive\libclang"

Write-Host 'WingDrive Development Environment Setup' -ForegroundColor Magenta
Write-Host @"

To set up your machine for WingDrive development, this script will do the following:
1) Enable long paths and Developer Mode, and exclude target\ from Defender scans
2) Install Windows C++ build tools
3) Install Edge Webview 2
4) Install Rust and Cargo
5) Install Rust tools
6) Install Nasm, CMake and 7-Zip
7) Install Strawberry perl (used by to build the openssl-sys crate)
8) Install Node.js (when missing) and Bun
9) Install libclang $llvmVersion (used by bindgen for the ffmpeg-sys-next crate)
10) Add Git's sh to the PATH (just runs its recipes with it)
"@

# Install System dependencies (GitHub Actions already has all of those installed)
if (-not $env:CI) {
    if (-not (Get-Command winget -ea 0)) {
        Exit-WithError 'winget not available' @'
Follow the instructions here to install winget:
https://learn.microsoft.com/windows/package-manager/winget/
'@
    }

    # Check system winget version is greater or equal to v1.4.10052
    $wingetVersion = [Version]((winget --version) -replace '.*?(\d+)\.(\d+)\.(\d+).*', '$1.$2.$3')
    $requiredVersion = [Version]'1.4.10052'
    if ($wingetVersion.CompareTo($requiredVersion) -lt 0) {
        $errorMessage = "You need to update your winget to version $requiredVersion or higher."
        Exit-WithError $errorMessage
    }

    # Check connectivity to GitHub
    $ProgressPreference = 'SilentlyContinue'
    if (-not ((Test-NetConnection -ComputerName 'github.com' -Port 443 -WarningAction SilentlyContinue).TcpTestSucceeded)) {
        Exit-WithError "Can't connect to github, check your internet connection and run this script again"
    }
    $ProgressPreference = 'Continue'

    Write-Host
    Read-Host 'Press Enter to continue'

    Write-Host
    Write-Host 'Configuring Windows for development...' -ForegroundColor Yellow
    # Build scripts write below target\ at depths that pass the 260 character limit.
    Set-ItemProperty 'HKLM:\SYSTEM\CurrentControlSet\Control\FileSystem' -Name LongPathsEnabled -Value 1 -Type DWord
    # Without Developer Mode, creating a symlink needs an elevated process.
    $appModelUnlock = 'HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\AppModelUnlock'
    if (-not (Test-Path $appModelUnlock)) {
        New-Item -Path $appModelUnlock -Force | Out-Null
    }
    Set-ItemProperty $appModelUnlock -Name AllowDevelopmentWithoutDevLicense -Value 1 -Type DWord
    # Real-time scans of build output slow down compilation and linking.
    New-Item -ItemType Directory -Force -Path "$projectRoot\target" | Out-Null
    try {
        Add-MpPreference -ExclusionPath "$projectRoot\target" -ErrorAction Stop
    } catch {
        Write-Host "Could not add a Defender exclusion for target\: $_" -ForegroundColor DarkYellow
    }

    # TODO: Force update Visual Studio build tools
    Write-Host
    Write-Host 'Installing Visual Studio Build Tools...' -ForegroundColor Yellow
    Write-Host 'This will take some time as it involves downloading several gigabytes of data....' -ForegroundColor Cyan
    winget install -e --accept-source-agreements --force --disable-interactivity --id Microsoft.VisualStudio.2022.BuildTools `
        --override 'updateall --quiet --wait'
    # Force install because BuildTools is itself a package manager, so let it decide if something needs to be installed or not
    winget install -e --accept-source-agreements --force --disable-interactivity --id Microsoft.VisualStudio.2022.BuildTools `
        --override '--wait --quiet --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended'
    if (-not ($wingetValidExit -contains $LASTEXITCODE)) {
        Exit-WithError 'Failed to install Visual Studio Build Tools'
    } else {
        $LASTEXITCODE = 0
    }

    Write-Host
    Write-Host 'Installing Edge Webview 2...' -ForegroundColor Yellow
    # This is normally already available, but on some early Windows 10 versions it isn't
    winget install -e --accept-source-agreements --disable-interactivity --id Microsoft.EdgeWebView2Runtime
    if (-not ($wingetValidExit -contains $LASTEXITCODE)) {
        Exit-WithError 'Failed to install Edge Webview 2'
    } else {
        $LASTEXITCODE = 0
    }

    Write-Host
    Write-Host 'Installing Rust and Cargo...' -ForegroundColor Yellow
    winget install -e --accept-source-agreements --disable-interactivity --id Rustlang.Rustup
    if (-not ($wingetValidExit -contains $LASTEXITCODE)) {
        Exit-WithError 'Failed to install Rust and Cargo'
    } else {
        $LASTEXITCODE = 0
    }

    Write-Host
    Write-Host 'Installing Nasm...' -ForegroundColor Yellow
    winget install -e --accept-source-agreements --disable-interactivity --id NASM.NASM
    if (-not ($wingetValidExit -contains $LASTEXITCODE)) {
        Exit-WithError 'Failed to install Nasm'
    } else {
        $LASTEXITCODE = 0
    }
    # The Nasm installer does not touch the PATH.
    Add-DirectoryToPath "$env:ProgramFiles\NASM"

    Write-Host
    Write-Host 'Installing CMake...' -ForegroundColor Yellow
    winget install -e --accept-source-agreements --disable-interactivity --scope machine --id Kitware.CMake
    if (-not ($wingetValidExit -contains $LASTEXITCODE)) {
        Exit-WithError 'Failed to install CMake'
    } else {
        $LASTEXITCODE = 0
    }

    Write-Host
    Write-Host 'Installing 7-Zip...' -ForegroundColor Yellow
    winget install -e --accept-source-agreements --disable-interactivity --id 7zip.7zip
    if (-not ($wingetValidExit -contains $LASTEXITCODE)) {
        Exit-WithError 'Failed to install 7-Zip'
    } else {
        $LASTEXITCODE = 0
    }

    Write-Host
    Write-Host 'Installing Strawberry perl...' -ForegroundColor Yellow
    winget install -e --accept-source-agreements --disable-interactivity --id StrawberryPerl.StrawberryPerl
    if (-not ($wingetValidExit -contains $LASTEXITCODE)) {
        Exit-WithError 'Failed to install Strawberry perl'
    } else {
        $LASTEXITCODE = 0
    }
    # Strawberry's c\bin ships MinGW gcc, cmake and nasm, and the machine PATH wins over the user PATH.
    # Only perl is needed, and those tools would shadow the MSVC toolchain and the versions above.
    $machinePath = [System.Environment]::GetEnvironmentVariable('Path', 'Machine').Split([IO.Path]::PathSeparator) |
        Where-Object { $_ -and ($_ -ne "$env:SystemDrive\Strawberry\c\bin") }
    [System.Environment]::SetEnvironmentVariable('Path', ($machinePath -join [IO.Path]::PathSeparator), 'Machine')
    Reset-Path
    # just runs recipes with Git's sh, which finds MSYS perl first. It lacks the modules OpenSSL's Configure needs.
    [System.Environment]::SetEnvironmentVariable('OPENSSL_SRC_PERL', "$env:SystemDrive\Strawberry\perl\bin\perl.exe", 'User')

    Write-Host
    Write-Host 'Installing NodeJS...' -ForegroundColor Yellow
    # Check if Node.JS is already installed and if it's compatible with the project
    $currentNode = Get-Command node -ea 0
    $currentNodeVersion = if (-not $currentNode) { $null } elseif ($currentNode.Version) { $currentNode.Version } elseif ((node --version) -match '(?sm)(\d+(\.\d+)*)') { [Version]$matches[1] } else { $null }
    $enginesNodeVersion = if ($packageJson.engines.node -match '(?sm)(\d+(\.\d+)*)') { [Version]$matches[1] } else { $null }
    if ($currentNodeVersion -and $enginesNodeVersion -and $currentNodeVersion.CompareTo($enginesNodeVersion) -lt 0) {
        Exit-WithError "Current Node.JS version: $currentNodeVersion (required: $enginesNodeVersion)" `
            'Uninstall the current version of Node.JS and run this script again'
    }
    # Leave a compatible Node.JS alone, so the script never replaces the version you chose.
    if (-not $currentNode) {
        winget install -e --accept-source-agreements --disable-interactivity --id OpenJS.NodeJS
        if (-not ($wingetValidExit -contains $LASTEXITCODE)) {
            Exit-WithError 'Failed to install NodeJS'
        } else {
            $LASTEXITCODE = 0
        }
        # Add NodeJS to the PATH
        Add-DirectoryToPath "$env:SystemDrive\Program Files\nodejs"
    }

    Write-Host
    Write-Host 'Checking for libclang...' -ForegroundColor Yellow
    # Only libclang.dll is needed, as in .github/actions/setup-system. A full LLVM on the PATH exposes
    # clang-cl.exe, which makes aws-lc-sys mis-detect __builtin_bswap and fail to link with MSVC.
    if (-not (Test-Path "$libclangDir\libclang.dll" -PathType Leaf)) {
        $llvmInstaller = Join-Path $env:TEMP "LLVM-$llvmVersion-win64.exe"
        curl.exe -fsSL -C - --retry 5 --retry-all-errors -o "$llvmInstaller" "https://github.com/llvm/llvm-project/releases/download/llvmorg-$llvmVersion/LLVM-$llvmVersion-win64.exe"
        if ($LASTEXITCODE -ne 0) {
            Exit-WithError 'Failed to download LLVM'
        }
        New-Item -ItemType Directory -Force -Path $libclangDir | Out-Null
        & "$env:ProgramFiles\7-Zip\7z.exe" e "$llvmInstaller" "-o$libclangDir" 'bin\libclang.dll' -r -y | Out-Null
        if ($LASTEXITCODE -ne 0) {
            Exit-WithError 'Failed to extract libclang.dll'
        }
        Remove-Item -LiteralPath $llvmInstaller -Force
    }
    [System.Environment]::SetEnvironmentVariable('LIBCLANG_PATH', $libclangDir, 'User')
    $env:LIBCLANG_PATH = $libclangDir

    # just runs its recipes with sh by default.
    Add-DirectoryToPath "$env:ProgramFiles\Git\bin"

    # Reset Path to ensure that executable installed above are available to rest of the script
    Reset-Path

    Write-Host
    Write-Host 'Installing the pinned Rust toolchain...' -ForegroundColor Yellow
    # rustup reads the version from rust-toolchain.toml.
    Set-Location $projectRoot
    rustup toolchain install
    if ($LASTEXITCODE -ne 0) {
        Exit-WithError 'Failed to install the Rust toolchain'
    }

    Write-Host
    Write-Host 'Installing just...' -ForegroundColor Yellow
    winget install -e --accept-source-agreements --disable-interactivity --id Casey.Just
    if (-not ($wingetValidExit -contains $LASTEXITCODE)) {
        Exit-WithError 'Failed to install just'
    } else {
        $LASTEXITCODE = 0
    }

    Write-Host
    Write-Host 'Installing Rust tools...' -ForegroundColor Yellow
    cargo install cargo-watch
    if ($LASTEXITCODE -ne 0) {
        Exit-WithError 'Failed to install Rust tools'
    }

    Write-Host
    Write-Host 'Installing Bun...' -ForegroundColor Yellow
    # Check if Bun is already installed and if it's compatible with the project
    $currentBunVersion = if (-not (Get-Command bun -ea 0)) { $null } elseif ((bun --version) -match '(?sm)(\d+(\.\d+)*)') { [Version]$matches[1] } else { $null }
    $enginesBunVersion = if ($packageJson.engines.bun -match '(?sm)(\d+(\.\d+)*)') { [Version]$matches[1] } else { $null }

    if (-not $currentBunVersion) {
        # Install the version pinned by packageManager in package.json
        $pinnedBunVersion = $packageJson.packageManager -replace '^bun@', ''
        winget install -e --accept-source-agreements --disable-interactivity --id Oven-sh.Bun --version "$pinnedBunVersion"
        if (-not ($wingetValidExit -contains $LASTEXITCODE)) {
            Exit-WithError 'Failed to install Bun'
        } else {
            $LASTEXITCODE = 0
        }
        Reset-Path
    } elseif ($currentBunVersion -and $enginesBunVersion -and $currentBunVersion.CompareTo($enginesBunVersion) -lt 0) {
        Exit-WithError "Current Bun version: $currentBunVersion (required: $enginesBunVersion)" `
            'Uninstall the current version of Bun and run this script again'
    }
}

if ($LASTEXITCODE -ne 0) {
    Exit-WithError "Something went wrong, exit code: $LASTEXITCODE"
}

# Run xtask setup to download native dependencies and configure cargo
if (-not $env:CI) {
    Write-Host
    Write-Host 'Running cargo xtask setup to download native dependencies...' -ForegroundColor Yellow
    Set-Location $projectRoot
    cargo xtask setup
    if ($LASTEXITCODE -ne 0) {
        Exit-WithError 'Failed to run cargo xtask setup'
    }
}

if (-not $env:CI) {
    Write-Host
    Write-Host 'Your machine has been setup for WingDrive development!' -ForegroundColor Green
    Write-Host
    Read-Host 'Press Enter to continue'
}
