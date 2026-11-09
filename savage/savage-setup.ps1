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
$Log = Join-Path $PSScriptRoot 'Savage-Setup-log.txt'

function Say([string]$Text, [string]$Color = 'Gray') {
    Write-Host $Text -ForegroundColor $Color
}

function Step([string]$Text) {
    Write-Host ''
    Write-Host "== $Text ==" -ForegroundColor Cyan
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
    Say "Installing $What. Windows may ask you to approve: click Yes." 'Yellow'
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
try { Start-Transcript -Path $Log -Force | Out-Null } catch { }

try {
    Say 'Savage Viewer - first-time setup' 'White'
    Say "Code folder: $Root"
    Say "Log file:    $Log"

    Step 'Checking this PC'
    $drive = Get-PSDrive -Name C
    $freeGb = [math]::Round($drive.Free / 1GB)
    Say "Free space on C: $freeGb GB"
    if (-not (Test-Tool 'winget')) {
        throw 'winget (the Windows package manager) was not found. Install "App Installer" from the Microsoft Store, then run this again.'
    }
    Update-SessionPath

    Step 'Build tools (1 of 3): Git'
    if (Test-Tool 'git') { Say 'Git: already installed.' 'Green' }
    else { Install-Tool 'Git.Git' 'Git' }
    if (-not (Test-Tool 'git')) {
        $env:Path += ";$env:ProgramFiles\Git\cmd"
    }
    if (-not (Test-Tool 'git')) { throw 'Git was installed but is not found yet. Close this window and run the setup again.' }

    Step 'Build tools (2 of 3): Visual Studio C++ tools'
    if (Get-MsvcName) { Say 'Visual Studio C++ tools: already installed.' 'Green' }
    else {
        Say 'This one is several GB and can take a while.' 'Yellow'
        Install-Tool 'Microsoft.VisualStudio.BuildTools' 'the Visual Studio C++ tools' @(
            '--override', '--passive --wait --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended')
    }

    Step 'Build tools (3 of 3): Rust'
    if (Test-Tool 'rustup') { Say 'Rust (rustup): already installed.' 'Green' }
    else { Install-Tool 'Rustlang.Rustup' 'Rust (rustup)' }
    if (-not (Test-Tool 'rustup')) { throw 'Rust was installed but is not found yet. Close this window and run the setup again.' }

    Step 'Savage Viewer code'
    if (Test-Path -LiteralPath (Join-Path $Root '.git')) {
        Say "The code is already in $Root. Left as it is." 'Green'
    }
    elseif ((Test-Path -LiteralPath $Root) -and (Get-ChildItem -LiteralPath $Root -Force | Select-Object -First 1)) {
        throw "$Root already exists and is not empty, so nothing was downloaded into it. Tell Claude what is in that folder."
    }
    else {
        New-Item -ItemType Directory -Force -Path (Split-Path $Root -Parent) | Out-Null
        git clone --branch $Branch $RepoUrl $Root
        if ($LASTEXITCODE -ne 0) { throw 'Downloading the code (git clone) failed.' }
    }

    Step 'Rust toolchain and libraries'
    Push-Location $Root
    try {
        # rust-toolchain.toml in the code folder picks the Rust version.
        rustup toolchain install
        if ($LASTEXITCODE -ne 0) { rustup show }
        if (-not (Test-Tool 'cargo')) { throw 'cargo (the Rust build tool) is not found. Close this window and run the setup again.' }
        cargo fetch --locked
        if ($LASTEXITCODE -ne 0) { throw 'Downloading the libraries (cargo fetch) failed.' }
    }
    finally {
        Pop-Location
    }

    $ok = $true
}
catch {
    Write-Host ''
    Write-Host "SETUP STOPPED: $($_.Exception.Message)" -ForegroundColor Red
}
finally {
    try { Stop-Transcript | Out-Null } catch { }
}

Write-Host ''
if ($ok) {
    Write-Host 'SETUP DONE. Nothing was built yet.' -ForegroundColor Green
    Write-Host 'Tell Claude: "setup done".' -ForegroundColor Green
    exit 0
}
else {
    Write-Host 'Setup did not finish. Send Claude this file:' -ForegroundColor Red
    Write-Host "  $Log" -ForegroundColor Red
    exit 1
}
