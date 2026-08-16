@echo off
setlocal enabledelayedexpansion

set "SCRIPT_DIR=%~dp0"
set "ROOT=%SCRIPT_DIR%.."

echo ============================================
echo Skills Hub Build Script
echo ============================================

REM 获取版本号
set VERSION=0.0.0
if not "%1"=="" set VERSION=%1
echo Version: %VERSION%

REM 1. 安装前端依赖（如有需要）
echo.
echo [1/4] Installing frontend dependencies...
cd /d "%ROOT%\frontend"
if not exist "node_modules" (
    call npm install
    if !ERRORLEVEL! neq 0 (
        echo ERROR: npm install failed
        exit /b 1
    )
) else (
    echo node_modules already exists, skipping npm install.
)

REM 2. 构建前端
echo.
echo [2/4] Building frontend...
call npm run build
if !ERRORLEVEL! neq 0 (
    echo ERROR: npm build failed
    exit /b 1
)

REM 3. 复制静态文件到 backend/static/
echo.
echo [3/4] Copying static files...
if exist "%ROOT%\backend\static" rmdir /s /q "%ROOT%\backend\static"
xcopy /e /y "%ROOT%\frontend\dist\*" "%ROOT%\backend\static\"
echo Done.

REM 4. PyInstaller 打包
echo.
echo [4/4] Building SkillsHub.exe...
cd /d "%ROOT%\backend"
python build.py
if !ERRORLEVEL! neq 0 (
    echo ERROR: PyInstaller build failed
    exit /b 1
)

echo.
echo ============================================
echo Build complete!
echo   dist\SkillsHub.exe
echo ============================================
