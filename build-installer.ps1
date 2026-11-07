param([switch]$SkipBuild, [string]$IsccPath, [string]$TestRoot)
$ErrorActionPreference = 'Stop'
Set-Location -LiteralPath $PSScriptRoot
if (-not $SkipBuild) {
    & (Join-Path $PSScriptRoot 'build.ps1') -Mode release -OutputName 'KyroLauncher.exe'
}
$sourceExe = Join-Path $PSScriptRoot 'dist/KyroLauncher.exe'
if (-not (Test-Path -LiteralPath $sourceExe)) { throw 'Build dist/KyroLauncher.exe first.' }
if (-not $IsccPath) {
    $compilerCommand = Get-Command ISCC.exe -ErrorAction SilentlyContinue
    $compilerCandidates = @(
        (Join-Path $PSScriptRoot '.tools/InnoSetup7/ISCC.exe'),
        (Join-Path $env:LOCALAPPDATA 'Programs/Inno Setup 7/ISCC.exe'),
        (Join-Path $env:ProgramFiles 'Inno Setup 7/ISCC.exe'),
        (Join-Path ${env:ProgramFiles(x86)} 'Inno Setup 6/ISCC.exe')
    )
    if ($compilerCommand) { $compilerCandidates = @($compilerCommand.Source) + $compilerCandidates }
    $IsccPath = $compilerCandidates | Where-Object { Test-Path -LiteralPath $_ } | Select-Object -First 1
}
if (-not $IsccPath -or -not (Test-Path -LiteralPath $IsccPath)) {
    throw 'Install Inno Setup from https://jrsoftware.org/isdl.php or pass -IsccPath.'
}
$packageToml = Get-Content -LiteralPath (Join-Path $PSScriptRoot 'src-tauri/Cargo.toml') -Raw
$versionMatch = [regex]::Match($packageToml, '(?m)^version\s*=\s*"(\d+\.\d+\.\d+)"')
if (-not $versionMatch.Success) { throw 'Cannot read launcher version.' }
$compilerArgs = @('/Qp', ('/DAppVersion=' + $versionMatch.Groups[1].Value), ('/DSourceExe=' + $sourceExe))
if ($TestRoot) {
    $resolvedTestRoot = [IO.Path]::GetFullPath($TestRoot)
    $targetBoundary = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot 'target')) + [IO.Path]::DirectorySeparatorChar
    if (-not $resolvedTestRoot.StartsWith($targetBoundary, [StringComparison]::OrdinalIgnoreCase)) {
        throw 'TestRoot must be inside this workspace target directory.'
    }
    $compilerArgs += '/DTestRoot=' + $resolvedTestRoot
}
$compilerArgs += Join-Path $PSScriptRoot 'installer/KyroLauncher.iss'
& $IsccPath @compilerArgs
if ($LASTEXITCODE -ne 0) { throw "Installer compiler exited with code $LASTEXITCODE" }
$installerName = if ($TestRoot) { 'KyroLauncherSetup-Test.exe' } else { 'KyroLauncherSetup.exe' }
Write-Output ('Built: ' + (Join-Path $PSScriptRoot ('dist/' + $installerName)))
