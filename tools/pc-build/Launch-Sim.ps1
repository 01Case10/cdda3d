# Launch-Sim.ps1 -- launch the fake engine (sim-server) and the
# renderer (voxel-engine) against it, in a window. Phase 11
# follow-up Lane A's --connect-window path, packaged as one command.
#
# Usage:
#   powershell -ExecutionPolicy Bypass -File tools\pc-build\Launch-Sim.ps1
#       [-Scene street_block] [-Seed 1] [-Record path.toml]
#       [-SkipBuild] [-DebugBuild] [-Headless] [-SocketPath path]
#
# What it does, in order:
#   1. Builds the release binaries (sim-server + voxel-engine).
#      -DebugBuild uses the debug profile; -SkipBuild reuses what is
#      already built.
#   2. Starts sim-server in the background on a unix socket
#      (default scene: street_block; log: target\sim-launch\server.log).
#   3. Waits for the socket to appear (aborts with the server log if
#      the server dies first).
#   4. Runs voxel-engine --connect-window <socket> in the foreground.
#      Play with the game keys: hjkl / yubn / arrows to move, o/c
#      open/close, s smash, g get, d drop, i inventory, & crafting,
#      ; look, . wait, >/< stairs, Esc back. Close the window to end
#      the session.
#   5. On engine exit, stops sim-server and removes the socket files.
#
# -Headless is a smoke mode: instead of a window it runs
# voxel-engine --connect with a few scripted keys (jjojjjQ -- walk
# out of the house and quit), proving the whole sequence end to end
# on a machine with no display.
[CmdletBinding()]
param(
    [string]$Scene = "street_block",
    [string]$Seed = "",
    [string]$Record = "",
    [string]$SocketPath = "",
    [switch]$SkipBuild,
    [switch]$DebugBuild,
    [switch]$Headless
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$LASTEXITCODE = 0

# Script-dir resolution: never trust $PSScriptRoot alone (it can be
# empty under -File); fall back to $PSCommandPath, then the CWD.
$scriptDir = $PSScriptRoot
if ([string]::IsNullOrEmpty($scriptDir)) {
    if (-not [string]::IsNullOrEmpty($PSCommandPath)) {
        $scriptDir = Split-Path $PSCommandPath -Parent
    }
}
if ([string]::IsNullOrEmpty($scriptDir)) { $scriptDir = (Get-Location).Path }
$repoRoot = Split-Path (Split-Path $scriptDir -Parent) -Parent
Set-Location $repoRoot

# Fresh shells can carry a stale PATH (e.g. cargo installed after the
# shell opened): rebuild it from the registry, plus the cargo bin.
$machinePath = [Environment]::GetEnvironmentVariable("Path", "Machine")
$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
$env:Path = "$machinePath;$userPath;$HOME\.cargo\bin"

# The proto crate's build.rs needs flatc exactly 24.3.25 (located via
# $FLATC first, then PATH). If neither provides it, download the
# pinned official Windows binary into target\tools (gitignored) and
# point $FLATC at it -- one less manual prerequisite.
$flatcOk = $false
if (-not [string]::IsNullOrEmpty($env:FLATC)) {
    if (Test-Path $env:FLATC) { $flatcOk = $true }
}
if (-not $flatcOk) {
    $onPath = Get-Command flatc -ErrorAction SilentlyContinue
    if ($onPath) { $flatcOk = $true }
}
if (-not $flatcOk) {
    $flatcDir = Join-Path $repoRoot "target\tools\flatbuffers-24.3.25"
    $flatcExe = Join-Path $flatcDir "flatc.exe"
    if (-not (Test-Path $flatcExe)) {
        Write-Host "== flatc not found; downloading flatbuffers 24.3.25 =="
        New-Item -ItemType Directory -Force $flatcDir | Out-Null
        $zip = Join-Path $flatcDir "Windows.flatc.binary.zip"
        Invoke-WebRequest "https://github.com/google/flatbuffers/releases/download/v24.3.25/Windows.flatc.binary.zip" -OutFile $zip
        Expand-Archive $zip -DestinationPath $flatcDir -Force
        Remove-Item $zip -Force
    }
    if (-not (Test-Path $flatcExe)) { throw "flatc provisioning failed: $flatcExe missing" }
    $env:FLATC = $flatcExe
    Write-Host "   FLATC=$env:FLATC"
}

$profile = "release"
if ($DebugBuild) { $profile = "debug" }
$binDir = Join-Path $repoRoot "target\$profile"
$serverExe = Join-Path $binDir "sim-server.exe"
$engineExe = Join-Path $binDir "voxel-engine.exe"

if ([string]::IsNullOrEmpty($SocketPath)) {
    $SocketPath = Join-Path $env:TEMP "cdda-sim.sock"
}
$logDir = Join-Path $repoRoot "target\sim-launch"
New-Item -ItemType Directory -Force $logDir | Out-Null
$serverLog = Join-Path $logDir "server.log"

if (-not $SkipBuild) {
    Write-Host "== building sim-server + voxel-engine ($profile) =="
    if ($profile -eq "release") {
        cargo build --release -p sim-server -p engine
    } else {
        cargo build -p sim-server -p engine
    }
    if ($LASTEXITCODE -ne 0) { throw "cargo build failed ($LASTEXITCODE)" }
}
if (-not (Test-Path $serverExe)) { throw "missing $serverExe (build first)" }
if (-not (Test-Path $engineExe)) { throw "missing $engineExe (build first)" }

Remove-Item $SocketPath -Force -ErrorAction SilentlyContinue
Remove-Item "$SocketPath.ui-shm" -Force -ErrorAction SilentlyContinue

$serverArgs = @($SocketPath, "--scene", $Scene, "--defs", "tools/sim-defs/generated/defs.json")
if (-not [string]::IsNullOrEmpty($Seed)) { $serverArgs += @("--seed", $Seed) }
if (-not [string]::IsNullOrEmpty($Record)) { $serverArgs += @("--record", $Record) }

Write-Host "== starting sim-server (scene: $Scene, socket: $SocketPath) =="
$server = Start-Process -FilePath $serverExe -ArgumentList $serverArgs `
    -RedirectStandardOutput $serverLog -RedirectStandardError "$serverLog.err" `
    -PassThru -NoNewWindow
try {
    $ready = $false
    for ($i = 0; $i -lt 150; $i++) {
        if (Test-Path $SocketPath) { $ready = $true; break }
        if ($server.HasExited) {
            Get-Content $serverLog -ErrorAction SilentlyContinue | Write-Host
            Get-Content "$serverLog.err" -ErrorAction SilentlyContinue | Write-Host
            throw "sim-server exited early (code $($server.ExitCode)); log above"
        }
        Start-Sleep -Milliseconds 200
    }
    if (-not $ready) { throw "socket $SocketPath never appeared; see $serverLog" }

    if ($Headless) {
        Write-Host "== headless smoke: voxel-engine --connect (keys: jjojjjQ) =="
        "jjojjjQ" | & $engineExe --connect $SocketPath
    } else {
        Write-Host "== launching renderer: voxel-engine --connect-window (close the window to end) =="
        $engineArgs = @("--connect-window", $SocketPath)
        if (Test-Path "assets\ultica_run1\materials.json") {
            $engineArgs += @("--textures", "assets\ultica_run1")
            Write-Host "   live furniture props: ON (model table found in assets\ultica_run1)"
        } else {
            Write-Host "   live furniture props: OFF (no assets\ultica_run1\materials.json -- extract the models pack there to enable)"
        }
        & $engineExe @engineArgs
    }
    $engineExit = $LASTEXITCODE
    Write-Host "== session ended (engine exit $engineExit); server log tail: =="
    Get-Content $serverLog -Tail 5 -ErrorAction SilentlyContinue | Write-Host
} finally {
    if (-not $server.HasExited) { Stop-Process -Id $server.Id -Force -ErrorAction SilentlyContinue }
    Remove-Item $SocketPath -Force -ErrorAction SilentlyContinue
    Remove-Item "$SocketPath.ui-shm" -Force -ErrorAction SilentlyContinue
}
