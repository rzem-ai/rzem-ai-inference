@echo off
REM Build standalone executable for Windows
REM Requires: Python 3.10+, Node.js

echo RZEM AI Inference - Windows Build
echo ===================================
echo.

REM Check Python version
python --version > nul 2>&1
if errorlevel 1 (
    echo Error: Python not found in PATH
    exit /b 1
)

echo [OK] Python found

REM Create virtual environment if needed
if not exist "venv" (
    echo Creating virtual environment...
    python -m venv venv
)

REM Activate virtual environment
call venv\Scripts\activate.bat

REM Install build dependencies
echo Installing build dependencies...
pip install --upgrade pip
pip install pyinstaller
pip install -r src-python\requirements.txt

echo [OK] Dependencies installed

REM Build frontend if needed
if not exist "dist" (
    echo Building frontend...
    call npm install
    call npm run build
    echo [OK] Frontend built
) else (
    echo [OK] Frontend already built
)

REM Clean previous build
echo Cleaning previous build...
if exist "build" rmdir /s /q build
if exist "dist-standalone" rmdir /s /q dist-standalone
mkdir dist-standalone

REM Build with PyInstaller
echo Building standalone executable...
pyinstaller build.spec --clean --noconfirm --distpath dist-standalone

if errorlevel 1 (
    echo [ERROR] Build failed
    exit /b 1
)

echo.
echo [SUCCESS] Build complete!
echo.
echo Executable location:
echo   dist-standalone\RZEM-AI-Inference.exe
echo.
echo To create release archive:
echo   cd dist-standalone
echo   tar -a -c -f RZEM-AI-Inference-Windows.zip RZEM-AI-Inference.exe
echo.
echo To generate checksums:
echo   certutil -hashfile dist-standalone\RZEM-AI-Inference.exe SHA256

pause
