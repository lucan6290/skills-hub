# Skills Hub NSIS 安装日志验证脚本
# 用法: .\scripts\test-installer-log.ps1
#
# 模拟三种场景并检查 %TEMP%\skills-hub-install.log 的输出:
#   1. 首次安装 — 无注册表记录，验证路径判断、WebView2 检测等日志
#   2. 升级安装 — 模拟已安装旧版本，验证版本比较、升级判定、路径恢复
#   3. 降级安装 — 模拟已安装高版本，验证降级判定、静默中止日志
#
# 前置条件: 需要先构建 release 安装包
#   cd frontend && npm run tauri build

$ErrorActionPreference = "Stop"

# ---------- 动态定位安装包 ----------
$BundleDir = Join-Path $PSScriptRoot "..\frontend\src-tauri\target\release\bundle\nsis"
$Installer = Get-ChildItem -Path $BundleDir -Filter "*_x64-setup.exe" -ErrorAction SilentlyContinue |
             Sort-Object LastWriteTime -Descending | Select-Object -First 1

if (-not $Installer) {
    Write-Host "[错误] 未找到安装包，请先执行: cd frontend && npm run tauri build" -ForegroundColor Red
    exit 1
}

# 从文件名提取版本号 (例: Skills Hub_0.1.1_x64-setup.exe → 0.1.1)
if ($Installer.Name -match '_(\d+\.\d+\.\d+)_') {
    $CurrentVersion = $Matches[1]
} else {
    Write-Host "[错误] 无法从文件名解析版本号: $($Installer.Name)" -ForegroundColor Red
    exit 1
}

$LogFile   = "$env:TEMP\skills-hub-install.log"
$TestDir   = "$env:TEMP\skills-hub-test"
$Results   = @()
$StartTime = Get-Date

Write-Host "安装包: $($Installer.FullName)" -ForegroundColor Gray
Write-Host "版本号: $CurrentVersion" -ForegroundColor Gray
Write-Host "日志文件: $LogFile" -ForegroundColor Gray

# ---------- 辅助函数 ----------

function Write-Header($text) {
    Write-Host "`n========================================" -ForegroundColor Cyan
    Write-Host " $text" -ForegroundColor Cyan
    Write-Host "========================================" -ForegroundColor Cyan
}

function Clear-Log {
    if (Test-Path $LogFile) { Remove-Item $LogFile -Force }
}

function Show-Log {
    if (Test-Path $LogFile) {
        Write-Host "`n--- 日志内容 ---" -ForegroundColor Yellow
        Get-Content $LogFile | ForEach-Object { Write-Host "  $_" }
        Write-Host "--- 日志结束 ---`n" -ForegroundColor Yellow
    } else {
        Write-Host "  [警告] 日志文件不存在: $LogFile" -ForegroundColor Red
    }
}

function Check-LogContains($pattern, $description) {
    if (-not (Test-Path $LogFile)) {
        Write-Host "  [FAIL] $description — 日志文件不存在" -ForegroundColor Red
        return $false
    }
    $content = Get-Content $LogFile -Raw
    if ($content -match $pattern) {
        Write-Host "  [PASS] $description" -ForegroundColor Green
        return $true
    } else {
        Write-Host "  [FAIL] $description — 未匹配: $pattern" -ForegroundColor Red
        return $false
    }
}

function Check-LogNotContains($pattern, $description) {
    if (-not (Test-Path $LogFile)) {
        Write-Host "  [FAIL] $description — 日志文件不存在" -ForegroundColor Red
        return $false
    }
    $content = Get-Content $LogFile -Raw
    if ($content -notmatch $pattern) {
        Write-Host "  [PASS] $description" -ForegroundColor Green
        return $true
    } else {
        Write-Host "  [FAIL] $description — 不应出现但匹配到: $pattern" -ForegroundColor Red
        return $false
    }
}

function Check-LogFileExists($description) {
    if (Test-Path $LogFile) {
        $size = (Get-Item $LogFile).Length
        Write-Host "  [PASS] $description (大小: ${size} bytes)" -ForegroundColor Green
        return $true
    } else {
        Write-Host "  [FAIL] $description — 文件不存在" -ForegroundColor Red
        return $false
    }
}

function Set-MockRegistry($version, $installDir) {
    # 设置模拟的注册表信息，用于升级/降级场景
    $regPath = "HKLM:\SOFTWARE\com.lucan\skillshub"
    if (-not (Test-Path $regPath)) { New-Item $regPath -Force | Out-Null }
    Set-ItemProperty $regPath -Name "(default)" -Value $installDir

    $uninstRegPath = "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\com.lucan.skillshub"
    if (-not (Test-Path $uninstRegPath)) { New-Item $uninstRegPath -Force | Out-Null }
    Set-ItemProperty $uninstRegPath -Name "DisplayName"      -Value "Skills Hub"
    Set-ItemProperty $uninstRegPath -Name "DisplayVersion"    -Value $version
    Set-ItemProperty $uninstRegPath -Name "Publisher"         -Value "lucan"
    Set-ItemProperty $uninstRegPath -Name "UninstallString"   -Value "`"$installDir\uninstall.exe`""
    Set-ItemProperty $uninstRegPath -Name "(default)"         -Value $installDir
}

function Clear-MockRegistry {
    Remove-Item "HKLM:\SOFTWARE\com.lucan" -Recurse -Force -ErrorAction SilentlyContinue
    Remove-Item "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\com.lucan.skillshub" -Recurse -Force -ErrorAction SilentlyContinue
}

function Cleanup-Environment {
    # 尝试卸载已安装的实例
    $uninstKey = "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\com.lucan.skillshub"
    if (Test-Path $uninstKey) {
        $uninstStr = (Get-ItemProperty $uninstKey -Name "UninstallString" -ErrorAction SilentlyContinue).UninstallString
        if ($uninstStr) {
            $exePath = $uninstStr.Trim('"')
            if (Test-Path $exePath) {
                Write-Host "  卸载已有安装..." -ForegroundColor Gray
                $proc = Start-Process -FilePath $exePath -ArgumentList "/S", "_?=$TestDir" -Wait -PassThru -NoNewWindow
                Start-Sleep -Seconds 2
            }
        }
    }
    # 清理测试目录
    if (Test-Path $TestDir) { Remove-Item $TestDir -Recurse -Force -ErrorAction SilentlyContinue }
    # 清理注册表
    Clear-MockRegistry
}

# ============================================================
# 场景 1: 首次安装 (静默模式，无注册表记录)
# ============================================================
Write-Header "场景 1: 首次安装"
Cleanup-Environment
Clear-Log

Write-Host "  执行静默安装到 $TestDir ..." -ForegroundColor Gray
$proc = Start-Process -FilePath $Installer.FullName -ArgumentList "/S", "/NS", "/D=$TestDir" -Wait -PassThru -NoNewWindow
Write-Host "  安装退出码: $($proc.ExitCode)" -ForegroundColor Gray
Start-Sleep -Seconds 2

Show-Log

$r1 = @(
    # 基础: 日志文件存在且非空
    (Check-LogFileExists "日志文件已创建")
    # .onInit 阶段
    (Check-LogContains "\.onInit 路径判断"       ".onInit 路径判断日志")
    (Check-LogContains "INSTDIR 初始值"          "INSTDIR 初始值记录")
    (Check-LogContains "GetDriveTypeW"           "GetDriveTypeW 调用记录")
    (Check-LogContains "判定:.*默认路径设为"     "路径判定结果日志")
    # RestorePreviousInstallLocation 阶段
    (Check-LogContains "RestorePreviousInstallLocation" "RestorePreviousInstallLocation 调用")
    (Check-LogContains "注册表读取路径"          "注册表读取路径日志")
    (Check-LogContains "注册表为空.*保持当前"    "首次安装: 注册表为空确认")
    # WebView2 检测阶段
    (Check-LogContains "WebView2 检测"           "WebView2 检测日志")
    (Check-LogContains "INSTALLWEBVIEW2MODE"     "WebView2 安装模式记录")
    # 负面断言: 不应出现升级/降级相关日志
    (Check-LogNotContains "PageReinstall 版本比较" "不应出现版本比较日志")
    (Check-LogNotContains "EarlyChecks 降级中止"   "不应出现降级中止日志")
)
$Results += @{ Scenario = "首次安装"; Passed = ($r1 | Where-Object { $_ }).Count; Total = $r1.Count }

# ============================================================
# 场景 2: 升级安装 (模拟已安装旧版本 → 当前版本)
# ============================================================
Write-Header "场景 2: 升级安装 (v0.1.0 → v$CurrentVersion)"
Clear-Log

# 计算一个比当前版本低的模拟旧版本
$versionParts = $CurrentVersion.Split('.')
$oldMinor = [int]$versionParts[1]
if ($oldMinor -gt 0) {
    $OldVersion = "$($versionParts[0]).$($oldMinor - 1).$($versionParts[2])"
} else {
    $OldVersion = "0.0.1"
}

Set-MockRegistry -version $OldVersion -installDir $TestDir

Write-Host "  模拟已安装 v$OldVersion, 升级到 v$CurrentVersion ..." -ForegroundColor Gray
$proc = Start-Process -FilePath $Installer.FullName -ArgumentList "/S", "/NS", "/UPDATE", "/D=$TestDir" -Wait -PassThru -NoNewWindow
Write-Host "  安装退出码: $($proc.ExitCode)" -ForegroundColor Gray
Start-Sleep -Seconds 2

Show-Log

# 转义版本号中的点号用于正则匹配
$escapedVersion = $CurrentVersion.Replace('.', '\.')
$escapedOldVersion = $OldVersion.Replace('.', '\.')

$r2 = @(
    # 基础: 日志文件存在
    (Check-LogFileExists "日志文件已创建")
    # .onInit 阶段 (升级也会走 .onInit)
    (Check-LogContains "\.onInit 路径判断"       ".onInit 路径判断日志")
    (Check-LogContains "GetDriveTypeW"           "GetDriveTypeW 调用记录")
    # RestorePreviousInstallLocation: 应恢复上次路径
    (Check-LogContains "RestorePreviousInstallLocation" "RestorePreviousInstallLocation 调用")
    (Check-LogContains "恢复上次路径"            "恢复上次安装路径")
    # PageReinstall 版本比较
    (Check-LogContains "PageReinstall 版本比较"  "版本比较日志")
    (Check-LogContains "当前版本.*$escapedVersion" "当前版本号记录 ($CurrentVersion)")
    (Check-LogContains "已安装版本.*$escapedOldVersion" "已安装版本记录 ($OldVersion)")
    (Check-LogContains "判定.*升级"              "升级判定日志")
    # WebView2 检测
    (Check-LogContains "WebView2 检测"           "WebView2 检测日志")
    # 负面断言: 不应出现降级中止
    (Check-LogNotContains "EarlyChecks 降级中止"  "不应出现降级中止日志")
)
$Results += @{ Scenario = "升级安装"; Passed = ($r2 | Where-Object { $_ }).Count; Total = $r2.Count }

# ============================================================
# 场景 3: 降级安装 (模拟已安装高版本，静默模式下应被中止)
# ============================================================
Write-Header "场景 3: 降级安装 (v99.99.99 → v$CurrentVersion, 静默中止)"
Clear-Log

# 修改注册表为远高于当前的版本
Set-MockRegistry -version "99.99.99" -installDir $TestDir

Write-Host "  模拟已安装 v99.99.99, 执行降级安装(静默)..." -ForegroundColor Gray
$proc = Start-Process -FilePath $Installer.FullName -ArgumentList "/S", "/NS", "/D=$TestDir" -Wait -PassThru -NoNewWindow
Write-Host "  安装退出码: $($proc.ExitCode)" -ForegroundColor Gray
Start-Sleep -Seconds 2

Show-Log

$r3 = @(
    # 基础: 日志文件存在
    (Check-LogFileExists "日志文件已创建")
    # .onInit 阶段
    (Check-LogContains "\.onInit 路径判断"       ".onInit 路径判断日志")
    # RestorePreviousInstallLocation
    (Check-LogContains "RestorePreviousInstallLocation" "RestorePreviousInstallLocation 调用")
    (Check-LogContains "恢复上次路径"            "恢复上次安装路径")
    # PageReinstall 版本比较
    (Check-LogContains "PageReinstall 版本比较"  "版本比较日志")
    (Check-LogContains "当前版本.*$escapedVersion" "当前版本号记录 ($CurrentVersion)")
    (Check-LogContains "已安装版本.*99\.99\.99"  "已安装版本记录 (99.99.99)")
    (Check-LogContains "判定.*降级"              "降级判定日志")
    (Check-LogContains "ALLOWDOWNGRADES"         "ALLOWDOWNGRADES 配置记录")
    # EarlyChecks 降级中止
    (Check-LogContains "EarlyChecks 降级中止"    "降级中止日志")
    (Check-LogContains "静默安装模式下检测到降级" "静默降级中止原因记录")
)
$Results += @{ Scenario = "降级安装"; Passed = ($r3 | Where-Object { $_ }).Count; Total = $r3.Count }

# ============================================================
# 汇总结果
# ============================================================
Write-Header "测试结果汇总"

$allPassed = $true
foreach ($r in $Results) {
    $passed = $r.Passed -eq $r.Total
    if (-not $passed) { $allPassed = $false }
    $color = if ($passed) { "Green" } else { "Red" }
    Write-Host "  $($r.Scenario): $($r.Passed)/$($r.Total) 通过" -ForegroundColor $color
}

$totalPassed = ($Results | ForEach-Object { $_.Passed } | Measure-Object -Sum).Sum
$totalTests  = ($Results | ForEach-Object { $_.Total }  | Measure-Object -Sum).Sum
$elapsed     = (Get-Date) - $StartTime

Write-Host ""
if ($allPassed) {
    Write-Host "  ✓ 全部通过: $totalPassed/$totalTests" -ForegroundColor Green
} else {
    Write-Host "  ✗ 存在失败: $totalPassed/$totalTests" -ForegroundColor Red
}
Write-Host "  耗时: $([math]::Round($elapsed.TotalSeconds, 1))s" -ForegroundColor Gray

# 清理
Cleanup-Environment
Write-Host "`n测试完成。日志文件: $LogFile" -ForegroundColor Cyan

# 以退出码反映测试结果
if (-not $allPassed) { exit 1 }
