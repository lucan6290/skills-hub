# 06. Windows 发布、安装、升级与 CI

## 1. 目标

将当前 Python/PyInstaller 发布流程替换为 Tauri/Rust 发布流程，同时保留当前用户需要的三种 Windows 交付形态：

- `SkillsHub.exe`
- Portable ZIP
- NSIS 安装包

## 2. 当前代码证据

- `.github/workflows/release.yml`：当前 Windows release 流程。
- `.github/workflows/ci.yml`：当前 frontend/backend CI。
- `backend/build.py`：PyInstaller 入口。
- `scripts/installer.nsi`：NSIS 安装脚本。
- `backend/core/update/checker.py`：更新检查。
- `backend/core/update/updater.py`：更新执行。
- `scripts/version.mjs`：版本同步入口。

当前 release 仍然：

```text
安装 Python → 安装 Node → npm build → PyInstaller → NSIS/ZIP
```

## 3. 目标发布链路

```text
安装 Rust toolchain
        ↓
安装 Node dependencies
        ↓
构建 React
        ↓
构建 Tauri Windows bundle
        ↓
生成 exe / NSIS
        ↓
打包 Portable ZIP
        ↓
上传 release artifacts
```

## 4. 实施步骤

### 4.1 Tauri 构建配置

- 使用 `frontend/src-tauri/tauri.conf.json` 配置应用名、版本、图标和 bundle。
- 版本必须与 `frontend/package.json` 和 `backend/core/version.py` 的现有版本管理策略协调。
- 不新增第二套手工版本源。
- 实际 Tauri 配置字段以安装的 Tauri 版本和构建结果为准。

### 4.2 NSIS

保留当前安装行为：

- 安装目录；
- 开始菜单快捷方式；
- 桌面快捷方式；
- 卸载注册信息；
- 中文/英文安装界面；
- 覆盖安装；
- `installed.flag` 或等价安装模式识别。

需要确认 Tauri 自带 NSIS 与现有 `scripts/installer.nsi` 是否重复：

- 若 Tauri bundle 已满足需求，优先使用 Tauri 配置；
- 若现有 NSIS 仍包含必要的业务行为，则保留并更新输入文件；
- 不同时维护两套互相冲突的安装逻辑。

### 4.3 Portable ZIP

Portable ZIP 至少包含：

```text
SkillsHub.exe
icon.ico（如最终产物仍需要）
portable.flag
 data/
```

`data/` 初始为空时也要保留目录，首次启动时使用 `<exe_dir>/data`。

### 4.4 CI

CI 至少分为：

- frontend lint/build；
- Rust fmt/check/test；
- Windows Tauri build；
- release artifact 检查。

Python CI 在 Rust 完成前继续保留。只有最终删除 Python 后，才删除 Python CI job。

### 4.5 自动更新

迁移当前更新检查和执行逻辑时必须使用 mock 测试：

- setup；
- portable；
- naked/exe；
- 下载失败；
- 文件替换失败；
- 主进程退出等待；
- 重启失败。

不能在 CI 或单元测试中替换正在运行的应用 exe。

## 5. 验收标准

- Windows release workflow 在干净环境可运行。
- 能生成 exe、Portable ZIP 和 NSIS 安装包。
- 安装、启动、卸载和覆盖安装可验证。
- Portable 版本使用便携数据目录。
- 安装版使用既有用户数据目录。
- 更新功能不破坏用户数据库和 Community Repo。
- release workflow 不再安装或调用 PyInstaller，前提是 Rust 功能已完整验收。
