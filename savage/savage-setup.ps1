# Savage Viewer - first-time setup.
# Keep this file ASCII only: Windows PowerShell 5.1 reads it as ANSI.
#
# What it does, in order:
#   1. Installs what is missing to build the viewer: Git, the Visual Studio
#      C++ build tools, Rust. Uses winget (the Windows package manager).
#      Anything already installed is left alone.
#   2. Downloads the Savage Viewer code into C:\dev\savage-viewer.
#   3. Downloads the Rust toolchain and the libraries the code needs.
# It never deletes anything. Running it twice is safe.

$ErrorActionPreference = 'Stop'

$Root = 'C:\dev\savage-viewer'
$RepoUrl = 'https://github.com/NuggetSavage/savage-viewer.git'
$Branch = 'savage'
$LogFile = Join-Path $PSScriptRoot 'Savage-Setup-log.txt'

# Everything shown on screen is also written to the log file.
function Log([string]$Text, [string]$Color = 'Gray') {
    Write-Host $Text -ForegroundColor $Color
    try { Add-Content -LiteralPath $LogFile -Value $Text -Encoding ASCII } catch { }
}

function Step([string]$Text) {
    Log ''
    Log "== $Text ==" 'Cyan'
}

# Runs one command line through cmd, with its normal and error output merged,
# shows and logs every line, and returns the exit code. Going through cmd
# keeps PowerShell from treating a tool's progress messages as errors.
function Run([string]$CommandLine) {
    Log "> $CommandLine" 'DarkGray'
    cmd /d /c "$CommandLine 2>&1" | ForEach-Object { Log ([string]$_) }
    $code = $LASTEXITCODE
    Log "  (exit code $code)" 'DarkGray'
    return $code
}

# Tools installed by winget are on PATH for new windows only: reload it here.
function Update-SessionPath {
    $machine = [Environment]::GetEnvironmentVariable('Path', 'Machine')
    $user = [Environment]::GetEnvironmentVariable('Path', 'User')
    $env:Path = "$machine;$user;$env:USERPROFILE\.cargo\bin"
}

function Test-Tool([string]$Name) {
    return [bool](Get-Command $Name -ErrorAction SilentlyContinue)
}

function Install-Tool([string]$Id, [string]$What, [string[]]$Extra = @()) {
    Log "Installing $What. Windows may ask you to approve: click Yes." 'Yellow'
    winget install --id $Id --exact --source winget --accept-package-agreements --accept-source-agreements @Extra
    if ($LASTEXITCODE -ne 0) { throw "Installing $What failed (winget code $LASTEXITCODE)." }
    Update-SessionPath
}

function Get-MsvcName {
    $vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\Installer\vswhere.exe'
    if (Test-Path -LiteralPath $vswhere) {
        & $vswhere -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property displayName | Select-Object -First 1
    }
}

$ok = $false
try { Set-Content -LiteralPath $LogFile -Value "Savage Viewer setup log, $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')" -Encoding ASCII } catch { }

try {
    Log 'Savage Viewer - first-time setup (version 2)' 'White'
    Log "Code folder: $Root"
    Log "Log file:    $LogFile"

    Step 'Checking this PC'
    $drive = Get-PSDrive -Name C
    $freeGb = [math]::Round($drive.Free / 1GB)
    Log "Free space on C: $freeGb GB"
    if (-not (Test-Tool 'winget')) {
        throw 'winget (the Windows package manager) was not found. Install "App Installer" from the Microsoft Store, then run this again.'
    }
    Update-SessionPath

    Step 'Build tools (1 of 3): Git'
    if (Test-Tool 'git') { Log 'Git: already installed.' 'Green' }
    else { Install-Tool 'Git.Git' 'Git' }
    if (-not (Test-Tool 'git')) {
        $env:Path += ";$env:ProgramFiles\Git\cmd"
    }
    if (-not (Test-Tool 'git')) { throw 'Git was installed but is not found yet. Close this window and run the setup again.' }

    Step 'Build tools (2 of 3): Visual Studio C++ tools'
    if (Get-MsvcName) { Log 'Visual Studio C++ tools: already installed.' 'Green' }
    else {
        Log 'This one is several GB and can take a while.' 'Yellow'
        Install-Tool 'Microsoft.VisualStudio.BuildTools' 'the Visual Studio C++ tools' @(
            '--override', '--passive --wait --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended')
    }

    Step 'Build tools (3 of 3): Rust'
    if (Test-Tool 'rustup') { Log 'Rust (rustup): already installed.' 'Green' }
    else { Install-Tool 'Rustlang.Rustup' 'Rust (rustup)' }
    if (-not (Test-Tool 'rustup')) { throw 'Rust was installed but is not found yet. Close this window and run the setup again.' }

    Step 'Savage Viewer code'
    if (Test-Path -LiteralPath (Join-Path $Root '.git')) {
        Log "The code is already in $Root. Left as it is." 'Green'
    }
    elseif ((Test-Path -LiteralPath $Root) -and (Get-ChildItem -LiteralPath $Root -Force | Select-Object -First 1)) {
        throw "$Root already exists and is not empty, so nothing was downloaded into it. Tell Claude what is in that folder."
    }
    else {
        New-Item -ItemType Directory -Force -Path (Split-Path $Root -Parent) | Out-Null
        $code = Run "git clone --branch $Branch $RepoUrl $Root"
        if ($code -ne 0) { throw "Downloading the code (git clone) failed with exit code $code." }
    }

    Step 'Rust toolchain and libraries'
    Push-Location $Root
    try {
        $code = Run 'rustup --version'
        if ($code -ne 0) { throw "rustup does not start (exit code $code)." }
        # rust-toolchain.toml in the code folder picks the Rust version.
        # --no-self-update: rustup must not replace its own files while this
        # script is using them (that is what stopped the first version).
        $code = Run 'rustup toolchain install --no-self-update'
        if ($code -ne 0) { Run 'rustup show' | Out-Null }
        $code = Run 'cargo --version'
        if ($code -ne 0) { throw "cargo (the Rust build tool) does not start (exit code $code)." }
        $code = Run 'cargo fetch --locked'
        if ($code -ne 0) { throw "Downloading the libraries (cargo fetch) failed with exit code $code." }
    }
    finally {
        Pop-Location
    }

    $ok = $true
}
catch {
    Log ''
    Log "SETUP STOPPED: $($_.Exception.Message)" 'Red'
}

Log ''
if ($ok) {
    Log 'SETUP DONE. Nothing was built yet.' 'Green'
    Log 'Tell Claude: "setup done".' 'Green'
    exit 0
}
else {
    Log 'Setup did not finish. Tell Claude: "setup failed".' 'Red'
    Log "Claude can read the log itself: $LogFile" 'Red'
    exit 1
}
