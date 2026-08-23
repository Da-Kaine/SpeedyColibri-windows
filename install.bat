@echo off
:: SpeedyColibri Single-Click Installer for Windows
title Installing SpeedyColibri (coli)...
echo ==================================================
echo  Starting SpeedyColibri Installation...
echo ==================================================
echo.

:: Launch PowerShell script to perform the download and PATH setup
powershell.exe -NoProfile -ExecutionPolicy Bypass -Command "& { if (Test-Path '%~dp0scripts\install.ps1') { & '%~dp0scripts\install.ps1' } else { iwr -useb 'https://raw.githubusercontent.com/GriffinPilz/SpeedyColibri/main/scripts/install.ps1' | iex } }"

if %ERRORLEVEL% NEQ 0 (
    echo.
    echo ==================================================
    echo  Installation failed! Error code: %ERRORLEVEL%
    echo ==================================================
    pause
    exit /b %ERRORLEVEL%
)

echo.
echo Press any key to exit this installer window...
pause >nul
