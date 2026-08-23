# SpeedyColibri Windows Installer Script
# Downloads and installs coli.exe to %LocalAppData%\SpeedyColibri\bin and updates User PATH

$ErrorActionPreference = "Stop"

$repo = "GriffinPilz/SpeedyColibri"
$installDir = "$env:LocalAppData\SpeedyColibri\bin"
$assetName = "coli-x86_64-pc-windows-msvc.zip"

Write-Host "==================================================" -ForegroundColor Cyan
Write-Host " Installing SpeedyColibri (coli.exe) for Windows" -ForegroundColor Cyan
Write-Host "==================================================" -ForegroundColor Cyan

# 1. Fetch latest release tag
Write-Host "[1/4] Checking latest release from GitHub ($repo)..." -ForegroundColor Yellow
try {
    $latestRelease = Invoke-RestMethod -Uri "https://api.github.com/repos/$repo/releases/latest"
    $tag = $latestRelease.tag_name
    $downloadUrl = "https://github.com/$repo/releases/download/$tag/$assetName"
} catch {
    Write-Host "Could not query GitHub API; falling back to latest release URL..." -ForegroundColor Gray
    $downloadUrl = "https://github.com/$repo/releases/latest/download/$assetName"
}

# 2. Download release zip
$tempZip = Join-Path $env:TEMP "coli_release.zip"
Write-Host "[2/4] Downloading SpeedyColibri ($assetName)..." -ForegroundColor Yellow
Invoke-WebRequest -Uri $downloadUrl -OutFile $tempZip

# 3. Extract executable
Write-Host "[3/4] Installing executable to $installDir..." -ForegroundColor Yellow
if (-not (Test-Path $installDir)) {
    New-Item -ItemType Directory -Force -Path $installDir | Out-Null
}

$tempExtract = Join-Path $env:TEMP "coli_extract"
if (Test-Path $tempExtract) { Remove-Item -Recurse -Force $tempExtract }
Expand-Archive -Path $tempZip -DestinationPath $tempExtract -Force

$exeSource = Get-ChildItem -Path $tempExtract -Filter "coli.exe" -Recurse | Select-Object -First 1
if (-not $exeSource) {
    Write-Error "Could not find coli.exe in the downloaded package."
    exit 1
}

Copy-Item -Path $exeSource.FullName -Destination (Join-Path $installDir "coli.exe") -Force
Remove-Item -Force $tempZip
Remove-Item -Recurse -Force $tempExtract

# 4. Add to User PATH if not already present
Write-Host "[4/4] Configuring PATH environment variable..." -ForegroundColor Yellow
$userPath = [Environment]::GetEnvironmentVariable("Path", [EnvironmentVariableTarget]::User)
if ($userPath -split ";" -notcontains $installDir) {
    $newUserPath = "$userPath;$installDir"
    [Environment]::SetEnvironmentVariable("Path", $newUserPath, [EnvironmentVariableTarget]::User)
    $env:Path = "$env:Path;$installDir"
    Write-Host "Added $installDir to User PATH." -ForegroundColor Green
} else {
    Write-Host "$installDir is already in User PATH." -ForegroundColor Green
}

Write-Host "`n==================================================" -ForegroundColor Green
Write-Host " SpeedyColibri installation complete!" -ForegroundColor Green
Write-Host " Run 'coli --help' or 'coli serve <model>' to start." -ForegroundColor Green
Write-Host "==================================================" -ForegroundColor Green
