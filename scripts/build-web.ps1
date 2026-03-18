. (Join-Path $PSScriptRoot "common.ps1")

Invoke-LoggedCommand -LogName "build-web" -CommandLine "cargo build --no-default-features --features web"
