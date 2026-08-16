# Changelog

本文件记录 Skills Hub 每个版本的主要变更。格式遵循 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.1.0/)。

## [Unreleased]

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
