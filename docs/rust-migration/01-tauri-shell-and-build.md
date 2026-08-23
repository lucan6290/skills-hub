# 01. Tauri 桌面壳、工程结构与 Rust 构建

## 1. 目标

在 `frontend/src-tauri/` 建立最小可运行 Tauri 2 桌面工程，使 React 页面可以在原生窗口中运行，并为后续 Rust service、repository 和 command 提供稳定入口。

本工作包不迁移业务逻辑，不实现 SQLite、文件同步和工具适配器。

## 2. 当前代码证据

当前桌面启动链路：

```text
backend/desktop.py
  → uvicorn.run(app)
  → webview.create_window(url="http://127.0.0.1:18921")
  → pywebview 窗口
```

当前构建链路：

```text
frontend/npm run build
  → frontend/dist
  → backend/build.py 复制到 backend/static
  → PyInstaller 打包 backend/desktop.py
```

当前窗口参数来自 `backend/desktop.py`：

- 标题：`Skills Hub`
- 默认尺寸：`1200 x 800`
- 最小尺寸：`900 x 600`

这些是迁移时的行为基线，不得无理由改变。

## 3. 前置检查

必须先确认：

```powershell
rustc --version
cargo --version
node --version
npm --version
```

如果 `rustc` 或 `cargo` 不存在，停止本工作包并报告环境缺失。不得通过猜测安装路径继续实施。

同时确认 Windows 构建工具、WebView2、NSIS 是否可用；实际检查命令和版本写入本文件的实施记录中。

## 4. 允许修改范围

```text
frontend/src-tauri/**
frontend/package.json
frontend/vite.config.ts
scripts/installer.nsi
.github/workflows/release.yml
```

以下文件由主 Agent 或其他工作包负责，禁止直接修改：

```text
frontend/src/lib/**
frontend/src/services/**
frontend/src-tauri/src/db/**
frontend/src-tauri/src/repositories/**
frontend/src-tauri/src/filesystem/**
frontend/src-tauri/src/services/**
```

## 5. 实施步骤

### 5.1 创建 Tauri 工程

建立：

```text
frontend/src-tauri/
├── Cargo.toml
├── Cargo.lock
├── tauri.conf.json
├── capabilities/
└── src/
    ├── main.rs
    ├── lib.rs
    ├── error.rs
    ├── state.rs
    ├── contracts.rs
    └── commands/
```

crate 版本必须在实施时根据实际工具链和 Tauri 兼容要求确定，并锁定到 `Cargo.lock`。文档不得预先虚构版本号。

### 5.2 配置前端资源

开发模式：

```text
Tauri → Vite dev server → React
```

生产模式：

```text
Tauri → frontend/dist 静态资源
```

生产模式不得依赖 FastAPI 静态文件托管。

### 5.3 建立共享入口

由本工作包独占：

- `main.rs`
- `lib.rs`
- `error.rs`
- `state.rs`
- `contracts.rs`
- `commands/**`

`commands/**` 只负责：

1. 反序列化参数；
2. 调用 service；
3. 处理 `AppError`；
4. 返回 DTO；
5. 发出任务事件。

不得在 command 中直接写 SQL 或执行复杂文件操作。

### 5.4 实现最小健康检查

先实现 `health_check`，只验证：

- Tauri command 注册成功；
- 前端可以 invoke；
- 返回当前应用版本；
- Rust 入口可以正常启动。

版本来源必须与项目版本管理规则协调，不能在多个文件中硬编码不同版本。

### 5.5 窗口和单实例

迁移当前桌面行为：

- 窗口标题、默认尺寸、最小尺寸保持一致；
- 应用只能运行一个实例；
- 第二次启动时激活已有窗口并退出；
- 不再通过检查端口实现单实例；
- 不执行强制杀死任意占用端口的进程。

### 5.6 权限配置

只授予实际需要的能力：

- 应用数据目录读写；
- 用户选择目录后的受控访问；
- 打开文件夹；
- 必要的 Windows 平台能力。

禁止使用“全盘读写”或“任意 shell 执行”作为快速实现方式。

### 5.7 构建验证

```powershell
cd frontend
npm run check

cd src-tauri
cargo fmt --check
cargo check
cargo test
```

至少执行一次 Windows 生产构建，记录生成文件、启动结果和退出结果。

## 6. 验收标准

- 能启动 Tauri 窗口。
- 页面可显示现有 React 应用。
- `health_check` invoke 成功。
- 启动过程中没有 Python 进程。
- 没有监听 `18921`。
- 生产资源来自 `frontend/dist`。
- 单实例行为可验证。
- 未覆盖实施前已有的 `frontend/package-lock.json` 修改。
