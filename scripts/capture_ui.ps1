<#
.SYNOPSIS
    Headless screenshot harness for Realmseed.

.DESCRIPTION
    Thin wrapper around the shared macroquad-toolkit capture script. Builds the
    debug exe and drives it through the env-var capture hook
    (REALMSEED_CAPTURE_*) provided by macroquad_toolkit::capture in
    src/main.rs. Game::begin_capture_scene seeds the title screen, gameplay
    (fresh campaign), or the pause menu depending on the scene name.

.EXAMPLE
    ./scripts/capture_ui.ps1
    ./scripts/capture_ui.ps1 -Frames 60 -SkipBuild
#>
param(
    [string[]]$Scenes = @("title", "gameplay", "pause"),
    [int]$Frames = 150,
    [string]$OutputDir = "docs\verification",
    [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"
$gameDir = Split-Path -Parent $PSScriptRoot
$shared = Join-Path (Split-Path -Parent $gameDir) "macroquad-toolkit\scripts\capture_ui.ps1"

& $shared -GameDir $gameDir -Scenes $Scenes -Frames $Frames -OutputDir $OutputDir -SkipBuild:$SkipBuild
