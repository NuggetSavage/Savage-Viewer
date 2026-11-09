@echo off
rem Savage Viewer - get the latest code and build it. Double-click this file.
title Savage Viewer - Update and Build
echo Getting the latest Savage Viewer code...
git -C "C:\dev\savage-viewer" pull --ff-only
if errorlevel 1 echo Could not get the latest code. Building the code that is already here.
powershell -NoProfile -ExecutionPolicy Bypass -File "C:\dev\savage-viewer\savage\savage-build.ps1"
echo.
pause
