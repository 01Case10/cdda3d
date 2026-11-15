# CDDA Voxel Frontend -- one-command PC build for gates 0.1 / 0.2 and the
# Phase 3 fork tests. See README.md in this folder for what it does, how
# long it takes, and what it deliberately does NOT do (hooks H1-H7).
#
# Usage (from this folder, in any PowerShell):
#   .\Build-CddaVoxel.ps1                full run: baseline + fork
#   .\Build-CddaVoxel.ps1 -Stage Baseline
#   .\Build-CddaVoxel.ps1 -Stage Fork    re-run fork stage only
#   .\Build-CddaVoxel.ps1 -DryRun        print the plan, touch nothing
#   .\Build-CddaVoxel.ps1 -SkipFullSuite quick: builds + 0.1 + [voxel] only
#
# Everything heavy runs inside MSYS2 UCRT64 (upstream's supported Windows
# route for the curses build); this wrapper only finds MSYS2, maps the
# parameters to environment variables, and launches
# build-cdda-voxel.sh there.

param(
    [ValidateSet("All", "Baseline", "Fork")]
    [string]$Stage = "All",
    [int]$Jobs = 0,
    [string]$WorkDir = "",
    [string]$Msys2Root = "",
    [switch]$SkipDeps,
    [switch]$Clean,
    [switch]$SkipFullSuite,
    [switch]$DryRun
)

$LASTEXITCODE = 0
$ErrorActionPreference = "Stop"

# Resolve this script's directory without trusting $PSScriptRoot alone
# (it has been observed empty under -File launches).
$scriptDir = $PSScriptRoot
if ([string]::IsNullOrEmpty($scriptDir)) {
    if (-not [string]::IsNullOrEmpty($PSCommandPath)) {
        $scriptDir = Split-Path $PSCommandPath -Parent
    }
}
if ([string]::IsNullOrEmpty($scriptDir)) {
    $scriptDir = (Get-Location).Path
}
$repoRoot = Split-Path (Split-Path $scriptDir -Parent) -Parent

if ([string]::IsNullOrEmpty($WorkDir)) {
    $WorkDir = Join-Path ([Environment]::GetFolderPath("MyDocuments")) "cdda-voxel-build"
}

# Find MSYS2.
$candidates = @()
if (-not [string]::IsNullOrEmpty($Msys2Root)) { $candidates += $Msys2Root }
if (-not [string]::IsNullOrEmpty($env:MSYS2_ROOT)) { $candidates += $env:MSYS2_ROOT }
$candidates += "C:\msys64"
$candidates += "D:\msys64"
$msysRoot = $null
foreach ($cand in $candidates) {
    if (Test-Path (Join-Path $cand "usr\bin\bash.exe")) {
        $msysRoot = $cand
        break
    }
}
if ($null -eq $msysRoot) {
    Write-Host "MSYS2 not found (looked in: $($candidates -join ', '))."
    Write-Host "Install it from https://www.msys2.org, then re-run. Nothing else to set up:"
    Write-Host "this script pacman-installs the UCRT64 toolchain itself on the first run."
    exit 2
}
$bashExe = Join-Path $msysRoot "usr\bin\bash.exe"

function ConvertTo-MsysPath([string]$winPath) {
    $p = $winPath -replace "\\", "/"
    if ($p -match "^([A-Za-z]):(.*)$") {
        $drive = $Matches[1].ToLower()
        $rest = $Matches[2]
        return "/" + $drive + $rest
    }
    return $p
}

$shWin = Join-Path $scriptDir "build-cdda-voxel.sh"
$shMsys = ConvertTo-MsysPath $shWin

# Hand the parameters to the shell script as environment variables.
$env:MSYSTEM = "UCRT64"
$env:CHERE_INVOKING = "1"
$env:MSYS2_PATH_TYPE = "inherit"
$env:CDDAVOXEL_REPO_WIN = $repoRoot
$env:CDDA_WORK_WIN = $WorkDir
$env:CDDA_STAGE = $Stage.ToLower()
$env:CDDA_SKIP_DEPS = $(if ($SkipDeps) { "1" } else { "0" })
$env:CDDA_CLEAN = $(if ($Clean) { "1" } else { "0" })
$env:CDDA_RUN_FULL = $(if ($SkipFullSuite) { "0" } else { "1" })
$env:CDDA_DRY_RUN = $(if ($DryRun) { "1" } else { "0" })
if ($Jobs -gt 0) {
    $env:CDDA_JOBS = [string]$Jobs
} else {
    Remove-Item Env:CDDA_JOBS -ErrorAction SilentlyContinue
}

Write-Host "CDDA Voxel PC build"
Write-Host "  repo    : $repoRoot"
Write-Host "  work dir: $WorkDir"
Write-Host "  MSYS2   : $msysRoot (UCRT64)"
Write-Host "  stage   : $Stage"
Write-Host ""

& $bashExe -lc "bash '$shMsys'"
$rc = $LASTEXITCODE

Write-Host ""
switch ($rc) {
    0 { Write-Host "DONE -- all gates that ran passed. Results: $WorkDir\results" }
    2 { Write-Host "STOPPED -- environment problem (see the message above). Nothing was half-built on purpose; fix and re-run." }
    3 { Write-Host "BUILD FAILED -- see the log tail above and $WorkDir\logs. For a first fork compile, docs\phase3-fork.md section 4 lists the expected fix candidates." }
    4 { Write-Host "TEST GATE FAILED -- the build worked; see $WorkDir\results (voxel-failed.txt / comparison.txt) for exactly which cases." }
    default { Write-Host "Exited with code $rc -- see $WorkDir\logs" }
}
exit $rc
