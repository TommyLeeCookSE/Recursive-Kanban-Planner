. (Join-Path $PSScriptRoot "common.ps1")

Invoke-LoggedCommand -LogName "test-all" -CommandLine "cargo fmt -- --check && cargo clippy --all-targets -- -D warnings && cargo test --all && cargo test --doc && cargo check --target wasm32-unknown-unknown && cargo check --no-default-features --features desktop && npm run check:css"
