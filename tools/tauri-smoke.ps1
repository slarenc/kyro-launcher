param([string]$ExePath = 'target/debug/kyro-launcher.exe')
$ErrorActionPreference = 'Stop'
$workspacePath = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
Set-Location -LiteralPath $workspacePath
$fixturePath = Join-Path $workspacePath ('target/tauri-ui-smoke/' + [guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $fixturePath -Force | Out-Null
New-Item -ItemType Directory -Path 'output/app-preview' -Force | Out-Null
$previousData = $env:KYRO_DATA_DIR
$previousArguments = $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS
$previousWebviewData = $env:WEBVIEW2_USER_DATA_FOLDER
$process = $null
try {
 $env:KYRO_DATA_DIR = $fixturePath
 $env:WEBVIEW2_USER_DATA_FOLDER = Join-Path $fixturePath 'webview'
 $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = '--remote-debugging-port=9228'
 $process = Start-Process -FilePath (Join-Path $workspacePath $ExePath) -WorkingDirectory $workspacePath -WindowStyle Hidden -PassThru
 node tools/tauri-smoke.cjs
 if ($LASTEXITCODE -ne 0) { throw 'Tauri smoke test failed' }
} finally {
 if ($process -and -not $process.HasExited) { Stop-Process -Id $process.Id }
 $env:KYRO_DATA_DIR = $previousData
 $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = $previousArguments
 $env:WEBVIEW2_USER_DATA_FOLDER = $previousWebviewData
}
