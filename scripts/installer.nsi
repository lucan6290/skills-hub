; Skills Hub NSIS 安装脚本
; 使用方式: makensis /DVERSION=x.y.z scripts\installer.nsi

; 启用 Unicode 模式，让 NSIS 以 UTF-8 读取脚本并生成 Unicode 安装程序，避免中文乱码
Unicode true

!define APP_NAME "Skills Hub"
!define APP_EXE "SkillsHub.exe"

!ifndef VERSION
  !define VERSION "0.0.0"
!endif

!define PUBLISHER "Skills Hub Contributors"

Name "${APP_NAME} ${VERSION}"
OutFile "..\SkillsHub-Setup-v${VERSION}.exe"
InstallDir "$PROGRAMFILES\${APP_NAME}"
InstallDirRegKey HKLM "Software\${APP_NAME}" "InstallDir"
RequestExecutionLevel admin

; ── MUI 头文件 ────────────────────────────────────────
!include "MUI2.nsh"

; ── 界面设置 ──────────────────────────────────────────
; 启用 XP 风格界面（现代外观）
XPStyle on

; 安装包名称/品牌
!define MUI_ABORTWARNING

; 安装/卸载图标
!define MUI_ICON "..\backend\icon.ico"
!define MUI_UNICON "..\backend\icon.ico"

; ── 页面定义 ──────────────────────────────────────────
; 欢迎页
!insertmacro MUI_PAGE_WELCOME
; 许可协议页
!insertmacro MUI_PAGE_LICENSE "..\LICENSE"
; 安装目录选择页
!insertmacro MUI_PAGE_DIRECTORY
; 组件选择页（开始菜单/桌面快捷方式等）
!insertmacro MUI_PAGE_COMPONENTS
; 安装进度页
!insertmacro MUI_PAGE_INSTFILES
; 完成页
!define MUI_FINISHPAGE_RUN "$INSTDIR\${APP_EXE}"
!insertmacro MUI_PAGE_FINISH

; 卸载向导页面
!insertmacro MUI_UNPAGE_WELCOME
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_UNPAGE_FINISH

; ── 语言 ──────────────────────────────────────────────
!insertmacro MUI_LANGUAGE "SimpChinese"
!insertmacro MUI_LANGUAGE "English"

; ── 安装段 ────────────────────────────────────────────
Section "核心文件 (必须)"
  SectionIn RO  ; 必选，用户不可取消

  SetOutPath "$INSTDIR"
  File "..\dist\${APP_EXE}"
  File "..\backend\icon.ico"

  ; 写入卸载信息
  WriteUninstaller "$INSTDIR\uninstall.exe"

  ; 写入安装标记文件（用于应用检测安装模式）
  FileOpen $0 "$INSTDIR\installed.flag" w
  FileWrite $0 "${VERSION}"
  FileClose $0

  ; 写入注册表卸载信息
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APP_NAME}" "DisplayName" "${APP_NAME}"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APP_NAME}" "DisplayVersion" "${VERSION}"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APP_NAME}" "Publisher" "${PUBLISHER}"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APP_NAME}" "InstallLocation" "$INSTDIR"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APP_NAME}" "DisplayIcon" "$INSTDIR\${APP_EXE},0"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APP_NAME}" "UninstallString" '"$INSTDIR\uninstall.exe"'
  WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APP_NAME}" "NoModify" 1
  WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APP_NAME}" "NoRepair" 1

  ; 记录安装目录，便于后续更新/覆盖安装
  WriteRegStr HKLM "Software\${APP_NAME}" "InstallDir" "$INSTDIR"
  WriteRegStr HKLM "Software\${APP_NAME}" "Version" "${VERSION}"
SectionEnd

Section "开始菜单快捷方式" SEC_STARTMENU
  CreateDirectory "$SMPROGRAMS\${APP_NAME}"
  CreateShortcut "$SMPROGRAMS\${APP_NAME}\${APP_NAME}.lnk" "$INSTDIR\${APP_EXE}" "" "$INSTDIR\icon.ico"
SectionEnd

Section "桌面快捷方式" SEC_DESKTOP
  CreateShortcut "$DESKTOP\${APP_NAME}.lnk" "$INSTDIR\${APP_EXE}" "" "$INSTDIR\icon.ico"
SectionEnd

; ── 组件描述 ──────────────────────────────────────────
!insertmacro MUI_FUNCTION_DESCRIPTION_BEGIN
  !insertmacro MUI_DESCRIPTION_TEXT ${SEC_STARTMENU} "在开始菜单创建 ${APP_NAME} 快捷方式"
  !insertmacro MUI_DESCRIPTION_TEXT ${SEC_DESKTOP} "在桌面创建 ${APP_NAME} 快捷方式"
!insertmacro MUI_FUNCTION_DESCRIPTION_END

; ── 卸载段 ────────────────────────────────────────────
Section "Uninstall"
  Delete "$INSTDIR\${APP_EXE}"
  Delete "$INSTDIR\icon.ico"
  Delete "$INSTDIR\uninstall.exe"
  Delete "$INSTDIR\installed.flag"
  Delete "$SMPROGRAMS\${APP_NAME}\${APP_NAME}.lnk"
  Delete "$DESKTOP\${APP_NAME}.lnk"
  RMDir "$SMPROGRAMS\${APP_NAME}"

  ; 清理注册表
  DeleteRegKey HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APP_NAME}"
  DeleteRegKey HKLM "Software\${APP_NAME}"

  RMDir "$INSTDIR"
SectionEnd
