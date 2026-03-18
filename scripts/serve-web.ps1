. (Join-Path $PSScriptRoot "common.ps1")

Invoke-LoggedCommand -LogName "serve-web" -CommandLine "dx serve --platform web"
