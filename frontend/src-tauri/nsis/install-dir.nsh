; NSIS 自定义钩子：安装目录 D 盘优先，无 D 盘回退 C 盘
; 在 NSIS_HOOK_PREINSTALL 中重写安装路径为 D:\skills-hub 或 C:\skills-hub

!macro NSIS_HOOK_PREINSTALL
  ; 检查 D 盘是否存在
  IfFileExists "D:\*.*" use_d 0
    ; D 盘不存在，使用 C 盘
    StrCpy $INSTDIR "C:\skills-hub"
    MessageBox MB_OK|MB_ICONINFORMATION "未检测到 D 盘，安装路径：C:\skills-hub"
    Goto done
  use_d:
    StrCpy $INSTDIR "D:\skills-hub"
  done:
!macroend
