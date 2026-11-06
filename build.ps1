param([ValidateSet('check','test','debug','release')][string]$Mode = 'release', [switch]$NetworkTests, [string]$OutputName = 'KYRO Launcher.exe')
$ErrorActionPreference = 'Stop'
if ($OutputName -notmatch '^[A-Za-z0-9 ._-]+\.exe$') { throw 'OutputName must be an EXE filename' }
Set-Location -LiteralPath $PSScriptRoot
$cargoExe = Join-Path $env:USERPROFILE '.cargo/bin/cargo.exe'
if (-not (Test-Path -LiteralPath $cargoExe)) { throw 'Install Rust: https://rustup.rs' }
$vswhereExe = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio/Installer/vswhere.exe'
if (Test-Path -LiteralPath $vswhereExe) {
    $vsInstall = & $vswhereExe -latest -all -products '*' -property installationPath
    if ($vsInstall) {
        $vcvarsFile = Join-Path $vsInstall 'VC/Auxiliary/Build/vcvars64.bat'
        if (Test-Path -LiteralPath $vcvarsFile) {
            $vsEnvironment = & cmd.exe /d /c "call `"$vcvarsFile`" >nul && set"
            foreach ($entry in $vsEnvironment) {
                if ($entry -match '^([^=]+)=(.*)$') { [Environment]::SetEnvironmentVariable($matches[1], $matches[2], 'Process') }
            }
        }
    }
}
switch ($Mode) {
    'check' { & $cargoExe check --locked }
    'test' {
        if ($NetworkTests) { & $cargoExe test --release --locked -- --include-ignored }
        else { & $cargoExe test --release --locked }
    }
    'debug' { & $cargoExe build --locked }
    'release' { & $cargoExe build --release --locked }
}
if ($LASTEXITCODE -ne 0) { throw "Cargo exited with code $LASTEXITCODE" }
if ($Mode -in @('debug', 'release')) {
    $profileDir = if ($Mode -eq 'debug') { 'debug' } else { 'release' }
    $destination = Join-Path $PSScriptRoot 'dist'
    New-Item -ItemType Directory -Path $destination -Force | Out-Null
    Copy-Item -LiteralPath (Join-Path $PSScriptRoot "target/$profileDir/kyro-launcher.exe") -Destination (Join-Path $destination $OutputName) -Force
    Write-Output "Built: $destination/$OutputName"
}
