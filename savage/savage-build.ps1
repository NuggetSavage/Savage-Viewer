# Savage Viewer - build the viewer from the code in C:\dev\savage-viewer.
# Keep this file ASCII only: Windows PowerShell 5.1 reads it as ANSI.
#
# Started by Savage-Update-and-Build.cmd, which first gets the latest code.
# What it does, in order:
#   1. Checks that the viewer is not open.
#   2. Builds the viewer (release build, the fast one to use).
#   3. Copies the finished viewer to C:\dev\savage-viewer-app.
# It never deletes anything; it only replaces the previous copy of the viewer.

$ErrorActionPreference = 'Stop'

$Root = 'C:\dev\savage-viewer'
$App = 'C:\dev\savage-viewer-app'
$ExeName = 'aurora-viewer.exe'
$LogFile = 'C:\dev\Savage-Build-log.txt'

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

$ok = $false
$minutes = 0
try { Set-Content -LiteralPath $LogFile -Value "Savage Viewer build log, $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')" -Encoding ASCII } catch { }

try {
    Log 'Savage Viewer - update and build (version 1)' 'White'
    Log "Code folder:   $Root"
    Log "Viewer folder: $App"
    Log "Log file:      $LogFile"

    $machine = [Environment]::GetEnvironmentVariable('Path', 'Machine')
    $user = [Environment]::GetEnvironmentVariable('Path', 'User')
    $env:Path = "$machine;$user;$env:USERPROFILE\.cargo\bin"

    Step 'Checking'
    if (-not (Test-Path -LiteralPath (Join-Path $Root '.git'))) {
        throw "The code is not in $Root yet. Run Savage-Setup.cmd first."
    }
    if (Get-Process -Name 'aurora-viewer' -ErrorAction SilentlyContinue) {
        throw 'The viewer is open. Close it, then run this again.'
    }
    $drive = Get-PSDrive -Name C
    Log "Free space on C: $([math]::Round($drive.Free / 1GB)) GB"

    Push-Location $Root
    try {
        Step 'Code being built'
        Run 'git status --short --branch' | Out-Null
        Run 'git log -1 --oneline' | Out-Null

        Step 'Building (the first time is slow; later builds are much faster)'
        # A leftover release tag in this window must not stamp the build.
        [Environment]::SetEnvironmentVariable('AURORA_VERSION_TAG', $null, 'Process')
        $sw = [Diagnostics.Stopwatch]::StartNew()
        $code = Run 'cargo build --release --locked -p aurora-viewer'
        $minutes = [math]::Round($sw.Elapsed.TotalMinutes)
        if ($code -ne 0) { throw "The build failed after $minutes min (exit code $code)." }
    }
    finally {
        Pop-Location
    }

    Step 'Copying the viewer'
    $built = Join-Path $Root "target\release\$ExeName"
    if (-not (Test-Path -LiteralPath $built)) { throw "The build finished but $built is missing." }
    New-Item -ItemType Directory -Force -Path $App | Out-Null
    Copy-Item -LiteralPath $built -Destination $App -Force
    foreach ($doc in 'LICENSE', 'NOTICE.md') {
        Copy-Item -LiteralPath (Join-Path $Root $doc) -Destination $App -Force
    }
    Run "$App\$ExeName --version" | Out-Null

    $ok = $true
}
catch {
    Log ''
    Log "BUILD STOPPED: $($_.Exception.Message)" 'Red'
}

Log ''
if ($ok) {
    Log "BUILD DONE in about $minutes min." 'Green'
    Log "The viewer is here: $App\$ExeName" 'Green'
    Log 'Tell Claude: "build done".' 'Green'
    exit 0
}
else {
    Log 'The build did not finish. Tell Claude: "build failed".' 'Red'
    Log "Claude can read the log itself: $LogFile" 'Red'
    exit 1
}
