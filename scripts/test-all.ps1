param(
    [switch]$SkipDesktop
)

. (Join-Path $PSScriptRoot "common.ps1")

$commands = @(
    "cargo fmt -- --check"
    "cargo clippy --all-targets -- -D warnings"
    "cargo test --all"
    "cargo test --doc"
    "cargo check --target wasm32-unknown-unknown"
)

$isWindowsPlatform = [System.Runtime.InteropServices.RuntimeInformation]::IsOSPlatform(
    [System.Runtime.InteropServices.OSPlatform]::Windows
)

if (-not $SkipDesktop) {
    if ($isWindowsPlatform) {
        $commands += "cargo check --no-default-features --features desktop"
    }
    else {
        Write-Host "Skipping desktop feature check on non-Windows platform."
    }
}

$commands += "npm run check:css"

$commandLine = [string]::Join(" && ", $commands)

Invoke-LoggedCommand -LogName "test-all" -CommandLine $commandLine
