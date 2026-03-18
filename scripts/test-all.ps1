. (Join-Path $PSScriptRoot "common.ps1")

Invoke-LoggedCommand -LogName "test-all" -CommandLine "cargo test --all"
