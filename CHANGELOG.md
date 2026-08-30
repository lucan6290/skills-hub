# Changelog

本文件记录 Skills Hub 每个版本的主要变更。格式遵循 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.1.0/)。

## [Unreleased]

## [0.2.3] - 2026-08-30

### Fixed
- 修复应用内"一键更新"失败：配置 Tauri 自动更新签名并上传 latest.json 清单
- 修复 panic 崩溃捕获，改用 force_capture 确保 Backtrace 信息完整

### Technical
- 企业级日志规范改进，关键错误处理分支补充详细日志上下文，新增专用错误日志和 panic 崩溃捕获
- 补充签名密钥初始化流程到发布工作流文档
- 更新 Cargo.lock 和 Tauri 生成的 schema 文件

## [0.2.2] - 2026-08-30

### Fixed
- 添加更新检查结果内存缓存，成功结果缓存30分钟、错误结果缓存5分钟，避免频繁点击"检查更新"触发 GitHub API 限流

## [0.2.1] - 2026-08-30

### Changed
- 引入 Glassmorphism 毛玻璃设计系统，优化整体视觉风格

### Fixed
- 对齐 Tauri command 参数命名规范，清理无用依赖
- 同步更新 package-lock.json 依赖版本

### Technical
- 修复版本管理脚本 version.mjs 同步 package-lock.json 版本号
- 添加 Agent 发布工作流文档 docs/release-workflow.md

## [0.2.0] - 2026-08-26

### Changed
- 后端架构从 Python FastAPI 完全迁移到 Rust + Tauri，前端从 HTTP fetch 迁移到 Tauri invoke
- 数据目录统一到 `~/.skills-hub`，社区仓库目录重命名为 `community-skills`
- NSIS 安装程序默认安装到 D 盘，自定义模板使安装界面默认显示 `D:\skills-hub` 路径
- 所有 Tauri command 参数统一 `rename_all snake_case`，`get_managed_skills` 返回完整 DTO
- 设计令牌系统完善，硬编码 CSS transition 时长与 skill-card.css 值替换为主题变量

### Added
- 9 项 Tauri 原生桌面功能：系统托盘、文件关联、Deep Link、开机启动、通知、全局快捷键、应用更新、多窗口、权限管理
- 网络代理设置，支持检查更新和下载使用代理
- tauri-plugin-log 文件日志集成
- 提示词文件管理模块，工具页面嵌入提示词文件管理区块
- 数据库维护功能增强：WAL 检查点、重建索引、优化、清除使用记录、导出/导入备份
- 应用设置扩展：关闭行为、托盘图标、日志级别、启动自动刷新
- 技能套件子技能分别同步到 Agent 工具
- 启动时自动迁移旧 `community_path` 到新目录结构
- 刷新时重新扫描仓库路径更新数据库
- NSIS 安装模板添加详细日志输出（安装/升级/降级场景覆盖）

### Fixed
- NSIS 安装路径检测改用 `GetDriveTypeW` 精确判断驱动器类型
- 修复托盘常驻、全局快捷键和 updater 的运行时问题
- 隐藏 subprocess 调用产生的控制台窗口，修复启动/同步时任务栏闪现 cmd 图标问题
- 优化更新检查逻辑，修复事件循环阻塞与自动检查时序问题
- 修复非全屏模式下搜索框超出窗口的布局溢出问题
- 修复数据库维护 Bug：导入 WAL 一致性、导出 checkpoint、重置 FK 顺序、本地时间戳
- 修复 Header 导航栏添加提示词 Tab 后布局挤压问题
- 添加 updater 签名公钥，修复插件初始化失败
- 修复 Tauri webview 拖拽拦截导致的 HTML5 拖拽排序失效
- 套件同步检测改用直接路径 + API 探测，修复自制路径兼容

### Technical
- 实现 Rust SQLite 数据库层迁移，文件系统操作和同步引擎从 Python 迁移到 Rust
- 实现全部 56 个 Tauri command 并注册到 `invoke_handler`
- 移除 Python 后端遗留代码，统一为 Tauri 架构

## [0.1.1] - 2026-08-16

### Fixed
- NSIS 安装程序组件选择页面中文乱码（添加 `Unicode true` 指令）
- 安装程序和 exe 使用错误的蓝色原子图标，替换为 Skills Hub 品牌 S 形 logo
- 移除 pywebview `create_window` 不支持的 `icon` 参数，修复桌面窗口启动报错

### Changed
- 统一使用 `favicon.svg` 作为唯一品牌图标源，删除冗余的 `logo.png` 和 Vite 默认 `vite.svg`

## [0.1.0] - 2026-08-16

### Added
- Skills Hub 首次发布
- 跨平台桌面应用（React 19 + Python FastAPI + PyInstaller 单文件打包）
- 管理 AI Agent Skills 并同步到 44 款 AI 编程工具
- 中英双语国际化（i18next）
- SQLite 数据持久化（Setup 版存 `%APPDATA%`，Portable 版存 `data/`）
- 三种发布产物：NSIS 安装包（Setup）、Portable ZIP、裸 exe
- Skills 管理：技能 CRUD、标签分类、文件编辑、双源技能（社区 + 本地共享目录）
- 工具管理：支持 44 款 AI 编程工具的技能同步/卸载
- 数据库概览：表统计、碎片整理、存储路径管理
- 导入流程：支持从本地和社区仓库批量导入技能
- 应用内自动更新：统一更新界面，支持在线检查 GitHub Releases 新版本、一键下载更新
- 版本检查 GitHub API 403 限流时自动回退到 Releases 页面 HTML 解析
- Skills Hub 品牌 Logo 与应用图标
- GitHub Actions CI/CD 自动构建与发布（推送 tag 触发，自动生成三种产物并创建 Draft Release）

### Technical
- 前端架构：features/ 目录结构 + 路径别名，React.lazy + Suspense 懒加载
- 前后端版本号统一管理（`scripts/version.mjs` 一键同步）
- 安装模式自动检测（dev / setup / portable / naked），通过 `installed.flag` 和 `portable.flag` 标记文件识别
- Service 层与 handler hooks 解耦业务逻辑
- ErrorBoundary 全局错误捕获
- CI 自动从 CHANGELOG.md 提取 release notes 作为 GitHub Release 说明
