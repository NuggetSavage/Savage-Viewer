@echo off
rem Savage Viewer - first-time setup. Double-click this file.
rem It runs savage-setup.ps1, which must sit in the same folder.
title Savage Viewer - Setup
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0savage-setup.ps1"
echo.
pause
