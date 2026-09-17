@echo off
setlocal EnableDelayedExpansion

REM Install script for warp-tui (Windows, cmd.exe)
REM
REM Downloads the latest release binary for the current architecture from
REM GitHub releases and installs it into %INSTALL_DIR% (defaults to
REM "%LOCALAPPDATA%\warp-tui\bin").
REM
REM Requires curl.exe (bundled with Windows 10 1803+ and Windows 11).
REM
REM Usage:
REM   curl -fsSL -o install.bat https://raw.githubusercontent.com/mertssmnoglu/warp-tui/main/scripts/install.bat && install.bat
REM
REM To customize, set environment variables before running:
REM   set INSTALL_DIR=C:\tools\warp-tui
REM   set VERSION=v1.2.3

set "REPO=mertssmnoglu/warp-tui"
set "BIN_NAME=warp-tui"

if not defined INSTALL_DIR set "INSTALL_DIR=%LOCALAPPDATA%\warp-tui\bin"
if not defined VERSION set "VERSION=latest"

where curl >nul 2>nul
if errorlevel 1 (
    echo error: curl is required to install %BIN_NAME% >&2
    exit /b 1
)

REM --- Detect architecture ---
set "ARCH=x86_64"
if /I "%PROCESSOR_ARCHITECTURE%"=="ARM64" set "ARCH=aarch64"
if /I "%PROCESSOR_ARCHITEW6432%"=="ARM64" set "ARCH=aarch64"

REM --- Resolve version tag (via the /releases/latest redirect target) ---
set "TAG=%VERSION%"
if /I "%TAG%"=="latest" (
    set "EFFECTIVE_URL="
    for /f "usebackq delims=" %%u in (`curl -s -o NUL -w "%%{url_effective}" -L "https://github.com/%REPO%/releases/latest"`) do set "EFFECTIVE_URL=%%u"
    if not defined EFFECTIVE_URL (
        echo error: failed to resolve the latest release tag >&2
        exit /b 1
    )
    set "TAG=!EFFECTIVE_URL:*/tag/=!"
    if "!TAG!"=="!EFFECTIVE_URL!" (
        echo error: failed to resolve the latest release tag >&2
        exit /b 1
    )
)

set "VERSION_NUM=%TAG%"
if "%VERSION_NUM:~0,1%"=="v" set "VERSION_NUM=%VERSION_NUM:~1%"

set "TARGET=%ARCH%-pc-windows-msvc"
set "ASSET=%BIN_NAME%-%VERSION_NUM%-%TARGET%.exe"
set "URL=https://github.com/%REPO%/releases/download/%TAG%/%ASSET%"

set "TMP_DIR=%TEMP%\warp-tui-install-%RANDOM%"
mkdir "%TMP_DIR%" >nul 2>nul
set "TMP_FILE=%TMP_DIR%\%ASSET%"

echo ==^> Downloading %ASSET% (%TAG%)
curl -fsSL "%URL%" -o "%TMP_FILE%"
if errorlevel 1 (
    echo error: failed to download %URL% >&2
    rmdir /s /q "%TMP_DIR%" >nul 2>nul
    exit /b 1
)

if not exist "%INSTALL_DIR%" mkdir "%INSTALL_DIR%" >nul 2>nul
copy /y "%TMP_FILE%" "%INSTALL_DIR%\%BIN_NAME%.exe" >nul

rmdir /s /q "%TMP_DIR%" >nul 2>nul

echo ==^> Installed %BIN_NAME% to %INSTALL_DIR%\%BIN_NAME%.exe

set "USER_PATH="
for /f "usebackq tokens=2,*" %%A in (`reg query "HKCU\Environment" /v Path 2^>nul`) do set "USER_PATH=%%B"

echo !USER_PATH! | findstr /I /C:"%INSTALL_DIR%" >nul
if errorlevel 1 (
    if defined USER_PATH (
        setx PATH "!USER_PATH!;%INSTALL_DIR%" >nul
    ) else (
        setx PATH "%INSTALL_DIR%" >nul
    )
    set "PATH=%PATH%;%INSTALL_DIR%"
    echo ==^> Added %INSTALL_DIR% to your user PATH. Restart your shell for it to take effect in new windows.
)

endlocal
