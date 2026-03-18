Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function New-BuildLogPath {
    param(
        [Parameter(Mandatory = $true)]
        [string]$LogName
    )

    $dateFolder = Get-Date -Format "yyyy-MM-dd"
    $timestamp = Get-Date -Format "yyyy-MM-dd_HH-mm-ss"
    $logDir = Join-Path $PSScriptRoot "..\\logs\\build\\$dateFolder"
    $resolvedLogDir = [System.IO.Path]::GetFullPath($logDir)
    New-Item -ItemType Directory -Path $resolvedLogDir -Force | Out-Null

    return (Join-Path $resolvedLogDir "$timestamp-$LogName.log")
}

function Write-LogHeader {
    param(
        [Parameter(Mandatory = $true)]
        [string]$LogFile,
        [Parameter(Mandatory = $true)]
        [string]$CommandLine
    )

    $cwd = (Get-Location).Path
    $timestamp = (Get-Date).ToString("o")
    $rustcVersion = (& rustc -Vv) 2>&1 | Out-String
    $cargoVersion = (& cargo -Vv) 2>&1 | Out-String
    $dxCommand = Get-Command dx -ErrorAction SilentlyContinue
    if ($null -ne $dxCommand) {
        $dxVersion = (& dx --version) 2>&1 | Out-String
    }
    else {
        $dxVersion = "dx not installed`n"
    }

    @(
        "Timestamp: $timestamp"
        "Working Directory: $cwd"
        "Command: $CommandLine"
        "RUST_BACKTRACE: $env:RUST_BACKTRACE"
        ""
        "[rustc -Vv]"
        $rustcVersion.TrimEnd()
        ""
        "[cargo -Vv]"
        $cargoVersion.TrimEnd()
        ""
        "[dx --version]"
        $dxVersion.TrimEnd()
        ""
        "[command output]"
    ) | Set-Content -Path $LogFile
}

function Invoke-LoggedCommand {
    param(
        [Parameter(Mandatory = $true)]
        [string]$LogName,
        [Parameter(Mandatory = $true)]
        [string]$CommandLine
    )

    $env:RUST_BACKTRACE = "1"
    $logFile = New-BuildLogPath -LogName $LogName
    Write-LogHeader -LogFile $logFile -CommandLine $CommandLine

    Write-Host "Writing build log to $logFile"
    & cmd.exe /c "$CommandLine 2>&1" | Tee-Object -FilePath $logFile -Append
    $exitCode = $LASTEXITCODE
    if ($null -eq $exitCode) {
        $exitCode = 0
    }

    exit $exitCode
}
