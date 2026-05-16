<#
.SYNOPSIS
    Install or update TraceTUI on Windows.
.DESCRIPTION
    Copies tracetui.exe to $env:ProgramFiles\TraceTUI\ and adds it to the
    user PATH. Run from the folder where tracetui.exe is located.
.EXAMPLE
    .\installOrUpdate.ps1
#>

$AppDir = Join-Path $env:ProgramFiles "TraceTUI"
$ExeName = "tracetui.exe"
$SourceExe = Join-Path $PSScriptRoot $ExeName

if (-not (Test-Path $SourceExe)) {
    Write-Host "ERROR: $ExeName not found next to this script." -ForegroundColor Red
    Write-Host "Extract the entire zip and run this script from the same folder as $ExeName."
    exit 1
}

# Ensure destination directory exists
if (-not (Test-Path $AppDir)) {
    New-Item -ItemType Directory -Path $AppDir -Force | Out-Null
}

# Copy binary
Copy-Item -Path $SourceExe -Destination (Join-Path $AppDir $ExeName) -Force
Write-Host "Copied $ExeName to $AppDir" -ForegroundColor Green

# Add to PATH (User scope) if not already present
$Path = [Environment]::GetEnvironmentVariable("Path", "User")
if ($Path -notlike "*$AppDir*") {
    $NewPath = "$AppDir;$Path"
    [Environment]::SetEnvironmentVariable("Path", $NewPath, "User")
    Write-Host "Added $AppDir to your user PATH." -ForegroundColor Green
    Write-Host "You may need to restart your terminal for the change to take effect." -ForegroundColor Yellow
} else {
    Write-Host "$AppDir is already in your PATH." -ForegroundColor Cyan
}

Write-Host "`nTraceTUI installed successfully! Run 'tracetui' to start." -ForegroundColor Green
