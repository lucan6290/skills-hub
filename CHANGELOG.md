# Changelog

本文件记录 Skills Hub 每个版本的主要变更。格式遵循 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.1.0/)。

## [Unreleased]

## [0.7.1] - 2026-08-16

### Added
- Skills Hub 品牌 Logo
- 设置通用页面优化：卡片范式布局、存储路径复制/打开操作、自定义仓库路径校验、恢复默认按钮
- 更新面板增强：自动检查更新开关、changelog 快捷链接、404 不视为错误
- 版本检查 GitHub API 403 限流时自动回退到 Releases 页面 HTML 解析
- 数据库设置界面优化：碎片状态色标识、快捷整理按钮、路径操作、表排序、错误兜底
- 设置页存储路径增加 label 标签区分不同路径类型

### Changed
- 前端架构重构：迁移到 features/ 目录结构 + 路径别名
- 代码分割：React.lazy + Suspense 懒加载视图与弹窗，优化首屏加载
- 提取 Service 层和 handler hooks（useTagActions / useSkillActions），解耦业务逻辑
- 更新页整合进关于页，移除独立更新标签
- 拆分各页面样式为独立 CSS 模块（buttons、markdown、database、settings、skill-card、modal、tools-page、tags-page 等）
- settings 分段按钮组改为全宽 flex 布局
- ErrorBoundary 全局接入，捕获渲染错误

### Fixed
- 设置页仓库路径标签区分问题、DB 表统计在 dbstat 不可用时的回退估算
- 技能详情页高度未撑满问题
- 数据库概览页按钮显示、表头文案、卡片布局问题
- 返回按钮与标题同行显示，设置页按钮统一为灰色轮廓胶囊样式
- 通用设置页面主内容区水平居中，消除右侧空白

## [0.7.0] - 2026-08-16

### Added
- Skills Hub 首次公开发布
- 跨平台桌面应用（React 19 + Python FastAPI + PyInstaller 单文件打包）
- 管理 AI Agent Skills 并同步到 44 款 AI 编程工具
- 中英双语国际化（i18next）
- SQLite 数据持久化（Setup 版存 `%APPDATA%`，Portable 版存 `data/`）
- 三种发布产物：NSIS 安装包（Setup）、Portable ZIP、裸 exe
- 应用内自动更新：统一更新界面，支持在线检查 GitHub Releases 新版本、一键下载更新
- 后端 `core/update` 模块：版本检查 + 更新执行（外部 bat 脚本等待进程退出后安装）
- Release notes 展示、GitHub Releases 下载页面链接
- GitHub Actions CI/CD 自动构建与发布（推送 tag 触发，自动生成三种产物并创建 Draft Release）

### Technical
- 前后端版本号统一管理（`scripts/version.mjs` 一键同步）
- 安装模式自动检测（dev / setup / portable / naked），通过 `installed.flag` 和 `portable.flag` 标记文件识别
- CI 自动从 CHANGELOG.md 提取 release notes 作为 GitHub Release 说明
