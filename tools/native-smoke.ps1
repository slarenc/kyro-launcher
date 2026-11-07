param([switch]$SkipBuild, [string]$LauncherName = "KYRO Launcher v0.7.0 Restored.exe")
$ErrorActionPreference = 'Stop'
$workspacePath = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
Set-Location -LiteralPath $workspacePath
if (-not $SkipBuild) { & (Join-Path $workspacePath 'build.ps1') -Mode release -OutputName $LauncherName }
$fixturePath = Join-Path $workspacePath 'target/native-ui-smoke'
$capturePath = Join-Path $fixturePath ('captures-' + (Get-Date -Format 'yyyyMMdd-HHmmss'))
New-Item -ItemType Directory -Path $capturePath -Force | Out-Null
$statePath = Join-Path $fixturePath 'state.json'
if (-not (Test-Path -LiteralPath $statePath)) {
    $sessions = @(0..24 | ForEach-Object { @{ started = [DateTime]::UtcNow.AddDays(-$_ * 2).ToString('o'); seconds = 1600 + $_ * 120; version = '1.21.1'; loader = 'Fabric'; exit_code = 0 } })
    $fixture = @{
        settings = @{ version = '1.21.1'; loader = 'Fabric'; accent = '#a185cf' }
        offline_accounts = @(@{name='NativePreview';uuid='9930180aaec43cf5a97dd629f202bf3c'})
        selected_account = 'offline:9930180aaec43cf5a97dd629f202bf3c'
        sessions = $sessions
        notices = @(@{id='smoke-1';title='Тест нативного интерфейса';body='Изолированные тестовые данные. Пользовательские профили не изменяются.';time=[DateTime]::UtcNow.ToString('o');unread=$true})
    }
    [IO.File]::WriteAllText($statePath, ($fixture | ConvertTo-Json -Depth 10), [Text.UTF8Encoding]::new($false))
    $screenshotsPath = Join-Path $fixturePath 'instances/1.21.1-Fabric/screenshots'
    New-Item -ItemType Directory -Path $screenshotsPath -Force | Out-Null
    Copy-Item -LiteralPath (Join-Path $workspacePath 'src-tauri/icons/icon.png') -Destination (Join-Path $screenshotsPath 'native-smoke.png')
}
$previousData = $env:KYRO_DATA_DIR
$previousSmoke = $env:KYRO_NATIVE_SMOKE
try {
    $env:KYRO_DATA_DIR = $fixturePath
    $env:KYRO_NATIVE_SMOKE = $capturePath
    $exePath = Join-Path $workspacePath ("dist/" + $LauncherName)
    $process = Start-Process -FilePath $exePath -WorkingDirectory $workspacePath -WindowStyle Hidden -PassThru
    if (-not $process.WaitForExit(90000)) { throw "Native smoke did not finish in 90 seconds (PID $($process.Id))" }
    if ($process.ExitCode -ne 0) { throw "Native smoke failed: $($process.ExitCode)" }
    $screens = @(Get-ChildItem -LiteralPath $capturePath -Filter 'screen-*.png')
    if ($screens.Count -ne 20) { throw "Expected 20 screenshots, found $($screens.Count): $capturePath" }
    $report = Get-Content -LiteralPath (Join-Path $capturePath 'report.json') -Raw | ConvertFrom-Json
    if (-not $report.window_restored) { throw 'Native launcher did not return after hide/minimize.' }
    Write-Output "Native smoke passed: $capturePath"
} finally {
    $env:KYRO_DATA_DIR = $previousData
    $env:KYRO_NATIVE_SMOKE = $previousSmoke
}
