. (Join-Path $PSScriptRoot "common.ps1")

Invoke-LoggedCommand -LogName "build-desktop" -CommandLine "cargo build --no-default-features --features desktop"
