# Update 自动更新 Agent 入口

本文件是 `update/` 的导航入口。Update 模块通过 GitHub Releases API 检查更新，实际下载安装由 `tauri-plugin-updater` 完成。

> 上级入口：[../AGENTS.md](../AGENTS.md)

## 职责

只负责"是否有更新"的信息获取：调用 GitHub Releases API 获取最新版本号、比较版本、提取下载链接和 release notes。实际的下载和安装由原生 `tauri-plugin-updater` 在 `commands/update.rs::do_update` 中执行。

## 文件清单

| 文件 | 职责 |
|------|------|
| `mod.rs` | 全部实现：`CheckUpdateResponse`、`DownloadUrls`、`PerformUpdateResponse`、`check_for_update`、`compare_versions`、`fetch_release_info`、`friendly_network_error` |

## 核心类型与常量

| 常量/类型 | 说明 |
|-----------|------|
| `GITHUB_OWNER` | `"lucan6290"` |
| `GITHUB_REPO` | `"skills-hub"` |
| `RELEASES_PAGE` | GitHub Releases 页面 URL |
| `CHANGELOG_URL` | CHANGELOG.md URL |
| `CheckUpdateResponse` | 检查结果：current/latest version、update_available、release_notes、download_urls、error |
| `DownloadUrls` | 下载链接：setup / portable / exe |
| `PerformUpdateResponse` | 更新执行结果：ok + message |

## 硬规则

1. **本模块只做检查，不做安装**——`check_for_update` 返回信息，`do_update` 由 `commands/update.rs` 调用原生 updater
2. **版本比较** 使用 `compare_versions`（简单 semver 比较：`split('.')` → `Vec<u32>` 比较）
3. **GitHub API 调用** 使用 `reqwest::blocking::Client`，10 秒超时，User-Agent: `SkillsHub-Update-Checker`
4. **网络错误友好提示**：超时 → "网络连接超时"，SSL → "SSL 握手失败"，其他 → "网络错误"
5. **API 限流处理**：HTTP 403 返回 "GitHub API rate limited"
6. **失败不崩溃**：`check_for_update` 出错时返回 `CheckUpdateResponse`（`error` 字段设置），不 panic

## 任务路由

| 任务 | 必读文件 |
|------|---------|
| 修改更新检查逻辑 | 本文件 + `mod.rs` + [../../commands/update.rs](../commands/update.rs) |
| 修改版本比较 | 本文件 + `mod.rs`（`compare_versions`） |
| 修改下载链接提取 | 本文件 + `mod.rs`（`check_for_update` 中 assets 遍历） |
