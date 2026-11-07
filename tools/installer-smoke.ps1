param([switch]$SkipCompile)
$ErrorActionPreference = 'Stop'
$workspacePath = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$testRoot = Join-Path $workspacePath 'target/installer-smoke'
$appPath = Join-Path $testRoot 'App'
$installerPath = Join-Path $workspacePath 'dist/KyroLauncherSetup-Test.exe'
Set-Location -LiteralPath $workspacePath
if (-not $SkipCompile) {
    & (Join-Path $workspacePath 'build-installer.ps1') -SkipBuild -TestRoot $testRoot
}
New-Item -ItemType Directory -Path $testRoot -Force | Out-Null
$expectedHash = (Get-FileHash -LiteralPath 'dist/KyroLauncher.exe' -Algorithm SHA256).Hash
$shortcutShell = New-Object -ComObject WScript.Shell
function Install-TestLauncher([string]$LogName) {
    $installProcess = Start-Process -FilePath $installerPath -ArgumentList @(
        '/VERYSILENT', '/SUPPRESSMSGBOXES', '/NORESTART', '/SP-',
        ('/DIR="' + $appPath + '"'), ('/LOG="' + (Join-Path $testRoot $LogName) + '"')
    ) -WindowStyle Hidden -Wait -PassThru
    if ($installProcess.ExitCode -ne 0) { throw "Installer failed: $($installProcess.ExitCode)" }
    $installedExe = Join-Path $appPath 'KyroLauncher.exe'
    if ((Get-FileHash -LiteralPath $installedExe -Algorithm SHA256).Hash -ne $expectedHash) { throw 'Installed EXE differs from the release EXE.' }
    foreach ($folder in @('Desktop', 'StartMenu')) {
        $linkPath = Join-Path $testRoot ($folder + '/KyroLauncher.lnk')
        if (-not (Test-Path -LiteralPath $linkPath)) { throw "Missing shortcut: $linkPath" }
        $link = $shortcutShell.CreateShortcut($linkPath)
        if ($link.TargetPath -ne $installedExe -or $link.WorkingDirectory -ne $appPath) { throw 'Shortcut points to the wrong file or working directory.' }
    }
}
Install-TestLauncher 'install.log'
$preservedPath = Join-Path $appPath 'data/state.json'
New-Item -ItemType Directory -Path (Split-Path -Parent $preservedPath) -Force | Out-Null
$preservedText = '{"installer_smoke":"user-created data must survive"}'
[IO.File]::WriteAllText($preservedPath, $preservedText)
Install-TestLauncher 'reinstall.log'
if ([IO.File]::ReadAllText($preservedPath) -ne $preservedText) { throw 'Reinstall changed user data.' }
$uninstaller = Join-Path $appPath 'unins000.exe'
if (-not ([IO.Path]::GetFullPath($uninstaller)).StartsWith($testRoot + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)) { throw 'Unsafe test uninstaller path.' }
$uninstallProcess = Start-Process -FilePath $uninstaller -ArgumentList @('/VERYSILENT','/SUPPRESSMSGBOXES','/NORESTART',('/LOG="'+(Join-Path $testRoot 'uninstall.log')+'"')) -WindowStyle Hidden -Wait -PassThru
if ($uninstallProcess.ExitCode -ne 0) { throw "Uninstaller failed: $($uninstallProcess.ExitCode)" }
if (Test-Path -LiteralPath (Join-Path $appPath 'KyroLauncher.exe')) { throw 'Uninstaller left its executable.' }
foreach ($folder in @('Desktop', 'StartMenu')) {
    if (Test-Path -LiteralPath (Join-Path $testRoot ($folder + '/KyroLauncher.lnk'))) { throw 'Uninstaller left its shortcut.' }
}
if ([IO.File]::ReadAllText($preservedPath) -ne $preservedText) { throw 'Uninstaller changed user data.' }
Write-Output "Installer smoke passed: install, both shortcuts, reinstall, uninstall and preserved user data ($testRoot)"
