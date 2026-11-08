param([switch]$SkipBuild, [string]$LauncherName = 'KYRO Launcher.exe')
$ErrorActionPreference = 'Stop'
$workspacePath = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
if (-not $SkipBuild) { & (Join-Path $workspacePath 'build.ps1') -Mode release -OutputName $LauncherName }
& (Join-Path $PSScriptRoot 'tauri-smoke.ps1') -ExePath (Join-Path 'dist' $LauncherName)
