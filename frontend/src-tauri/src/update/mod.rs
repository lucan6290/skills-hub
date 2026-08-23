//! Update checker and executor — mirrors `backend/core/update/checker.py` and
//! `backend/core/update/updater.py`.
//!
//! Checks GitHub Releases API for new versions and handles download + install.

use serde::{Deserialize, Serialize};

pub const GITHUB_OWNER: &str = "lucan6290";
pub const GITHUB_REPO: &str = "skills-hub";
pub const RELEASES_PAGE: &str = "https://github.com/lucan6290/skills-hub/releases";
pub const CHANGELOG_URL: &str = "https://github.com/lucan6290/skills-hub/blob/main/CHANGELOG.md";

/// Response from the update check.
#[derive(Debug, Clone, Serialize)]
pub struct CheckUpdateResponse {
    pub current_version: String,
    pub latest_version: String,
    pub update_available: bool,
    pub install_mode: String,
    pub release_url: String,
    pub release_notes: String,
    pub download_urls: DownloadUrls,
    pub changelog_url: String,
    pub error: Option<String>,
}

/// Download URLs for different install modes.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DownloadUrls {
    pub setup: String,
    pub portable: String,
    pub exe: String,
}

/// Response from performing an update.
#[derive(Debug, Clone, Serialize)]
pub struct PerformUpdateResponse {
    pub ok: bool,
    pub message: String,
}

/// Compare two semver version strings. Returns true if latest > current.
fn compare_versions(current: &str, latest: &str) -> bool {
    let parse =
        |v: &str| -> Vec<u32> { v.split('.').filter_map(|p| p.parse::<u32>().ok()).collect() };
    let c = parse(current);
    let l = parse(latest);
    l > c
}

/// Build download URLs for a given tag name.
#[allow(dead_code)]
fn release_download_urls(tag_name: &str) -> DownloadUrls {
    let base = format!(
        "https://github.com/{}/{}/releases/download/{}",
        GITHUB_OWNER, GITHUB_REPO, tag_name
    );
    DownloadUrls {
        setup: format!("{}/SkillsHub-Setup-{}.exe", base, tag_name),
        portable: format!("{}/SkillsHub-Portable-{}.zip", base, tag_name),
        exe: format!("{}/SkillsHub.exe", base),
    }
}

fn empty_download_urls() -> DownloadUrls {
    DownloadUrls::default()
}

/// GitHub API response structure (partial).
#[derive(Debug, Deserialize)]
struct GithubRelease {
    tag_name: Option<String>,
    html_url: Option<String>,
    body: Option<String>,
    assets: Option<Vec<GithubAsset>>,
}

#[derive(Debug, Deserialize)]
struct GithubAsset {
    name: Option<String>,
    browser_download_url: Option<String>,
}

/// Check for updates via GitHub Releases API.
pub fn check_for_update(current_version: &str, install_mode: &str) -> CheckUpdateResponse {
    let url = format!(
        "https://api.github.com/repos/{}/{}/releases/latest",
        GITHUB_OWNER, GITHUB_REPO
    );

    match fetch_release_info(&url) {
        Ok(release) => {
            let tag_name = release.tag_name.unwrap_or_default();
            let latest_version = tag_name.trim_start_matches('v').to_string();
            let html_url = release
                .html_url
                .unwrap_or_else(|| RELEASES_PAGE.to_string());
            let body = release.body.unwrap_or_default();

            // Extract download URLs from assets
            let mut urls = empty_download_urls();
            if let Some(assets) = &release.assets {
                for asset in assets {
                    let name = asset.name.as_deref().unwrap_or("");
                    let download_url = asset.browser_download_url.as_deref().unwrap_or("");
                    if name.contains("Setup") {
                        urls.setup = download_url.to_string();
                    } else if name.contains("Portable") {
                        urls.portable = download_url.to_string();
                    } else if name == "SkillsHub.exe" {
                        urls.exe = download_url.to_string();
                    }
                }
            }

            let update_available = compare_versions(current_version, &latest_version);

            CheckUpdateResponse {
                current_version: current_version.to_string(),
                latest_version,
                update_available,
                install_mode: install_mode.to_string(),
                release_url: html_url,
                release_notes: body,
                download_urls: urls,
                changelog_url: CHANGELOG_URL.to_string(),
                error: None,
            }
        }
        Err(err) => CheckUpdateResponse {
            current_version: current_version.to_string(),
            latest_version: current_version.to_string(),
            update_available: false,
            install_mode: install_mode.to_string(),
            release_url: RELEASES_PAGE.to_string(),
            release_notes: String::new(),
            download_urls: empty_download_urls(),
            changelog_url: CHANGELOG_URL.to_string(),
            error: Some(err),
        },
    }
}

/// Fetch release info from GitHub API using reqwest blocking client.
fn fetch_release_info(url: &str) -> Result<GithubRelease, String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .user_agent("SkillsHub-Update-Checker")
        .build()
        .map_err(|e| format!("failed to create HTTP client: {}", e))?;

    let response = client
        .get(url)
        .send()
        .map_err(|e| friendly_network_error(&e))?;

    let status = response.status();
    if status == reqwest::StatusCode::NOT_FOUND {
        return Err("no releases found".to_string());
    }
    if status == reqwest::StatusCode::FORBIDDEN {
        return Err("GitHub API rate limited, try again later".to_string());
    }
    if !status.is_success() {
        return Err(format!("GitHub API error ({})", status));
    }

    response
        .json::<GithubRelease>()
        .map_err(|e| format!("failed to parse response: {}", e))
}

fn friendly_network_error(e: &reqwest::Error) -> String {
    let text = e.to_string().to_lowercase();
    if text.contains("timed out") || text.contains("timeout") {
        "网络连接超时，请检查网络或稍后重试".to_string()
    } else if text.contains("ssl") || text.contains("certificate") {
        "网络 SSL 握手失败，请检查网络代理、证书或稍后重试".to_string()
    } else {
        format!("网络错误：{}", e)
    }
}

/// Perform the update: download and prepare the updater script.
/// In test/mock mode, this does not actually replace the running executable.
pub fn perform_update(install_mode: &str, download_urls: &DownloadUrls) -> PerformUpdateResponse {
    if install_mode == "dev" {
        return PerformUpdateResponse {
            ok: false,
            message: "开发模式不支持自动更新".to_string(),
        };
    }

    let url = match install_mode {
        "setup" => &download_urls.setup,
        "portable" => &download_urls.portable,
        "naked" => &download_urls.exe,
        _ => {
            return PerformUpdateResponse {
                ok: false,
                message: format!("未知的安装模式: {}", install_mode),
            };
        }
    };

    if url.is_empty() {
        return PerformUpdateResponse {
            ok: false,
            message: "下载失败: 未找到对应产物下载链接".to_string(),
        };
    }

    // Download to temp directory
    let update_dir = std::env::temp_dir().join("skillshub_update");
    if let Err(e) = std::fs::create_dir_all(&update_dir) {
        return PerformUpdateResponse {
            ok: false,
            message: format!("创建临时目录失败: {}", e),
        };
    }

    let filename = url.rsplit('/').next().unwrap_or("update_file");
    let dest_path = update_dir.join(filename);

    // Download the file
    match download_file(url, &dest_path) {
        Ok(_) => {}
        Err(e) => {
            return PerformUpdateResponse {
                ok: false,
                message: format!("下载失败: {}", e),
            };
        }
    }

    // Generate updater script
    let pid = std::process::id().to_string();
    let app_exe_path = std::env::current_exe()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();

    let bat_content = generate_updater_bat(
        &pid,
        &dest_path.to_string_lossy(),
        install_mode,
        &app_exe_path,
    );
    let bat_path = update_dir.join("updater.bat");

    if let Err(e) = std::fs::write(&bat_path, bat_content.as_bytes()) {
        return PerformUpdateResponse {
            ok: false,
            message: format!("写入更新脚本失败: {}", e),
        };
    }

    // Launch updater script (detached)
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const DETACHED_PROCESS: u32 = 0x00000008;
        const CREATE_NO_WINDOW: u32 = 0x08000000;

        match std::process::Command::new("cmd.exe")
            .args(["/c", &bat_path.to_string_lossy()])
            .creation_flags(DETACHED_PROCESS | CREATE_NO_WINDOW)
            .spawn()
        {
            Ok(_) => PerformUpdateResponse {
                ok: true,
                message: "更新已准备就绪，应用即将重启...".to_string(),
            },
            Err(e) => PerformUpdateResponse {
                ok: false,
                message: format!("启动更新脚本失败: {}", e),
            },
        }
    }

    #[cfg(not(windows))]
    {
        PerformUpdateResponse {
            ok: false,
            message: "自动更新仅支持 Windows".to_string(),
        }
    }
}

fn download_file(url: &str, dest: &std::path::Path) -> Result<(), String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(300))
        .build()
        .map_err(|e| format!("HTTP client error: {}", e))?;

    let response = client
        .get(url)
        .send()
        .map_err(|e| format!("download failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("download HTTP error: {}", response.status()));
    }

    let bytes = response
        .bytes()
        .map_err(|e| format!("failed to read response: {}", e))?;

    std::fs::write(dest, &bytes).map_err(|e| format!("failed to write file: {}", e))
}

fn generate_updater_bat(
    _pid: &str,
    _file_path: &str,
    _install_mode: &str,
    _app_exe_path: &str,
) -> String {
    format!(
        r#"@echo off
:: SkillsHub Updater Script
:: Args: %1=current PID, %2=downloaded file, %3=install mode, %4=app exe path

echo Waiting for application to exit...

:wait_loop
tasklist /FI "PID eq %1" 2>nul | find "%1" >nul
if not errorlevel 1 (
    timeout /t 1 /nobreak >nul
    goto wait_loop
)

echo Application exited, starting update...

if "%3"=="setup" (
    echo Running NSIS silent install...
    "%2" /S
) else if "%3"=="portable" (
    echo Extracting portable ZIP...
    powershell -Command "Expand-Archive -Path '%2' -DestinationPath '%~dp4' -Force"
) else if "%3"=="naked" (
    echo Replacing exe...
    copy /Y "%2" "%4"
)

echo Restarting application...
start "" "%4"

:: Self-delete
(goto) 2>nul & del "%~f0"
"#,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compare_versions_newer() {
        assert!(compare_versions("0.1.0", "0.2.0"));
        assert!(compare_versions("0.1.0", "1.0.0"));
        assert!(compare_versions("1.0.0", "1.0.1"));
    }

    #[test]
    fn test_compare_versions_same() {
        assert!(!compare_versions("1.0.0", "1.0.0"));
    }

    #[test]
    fn test_compare_versions_older() {
        assert!(!compare_versions("1.0.0", "0.9.0"));
        assert!(!compare_versions("2.0.0", "1.9.9"));
    }

    #[test]
    fn test_release_download_urls() {
        let urls = release_download_urls("v1.2.3");
        assert!(urls.setup.contains("Setup"));
        assert!(urls.setup.contains("1.2.3"));
        assert!(urls.portable.contains("Portable"));
        assert!(urls.exe.contains("SkillsHub.exe"));
    }

    #[test]
    fn test_perform_update_dev_mode() {
        let urls = empty_download_urls();
        let result = perform_update("dev", &urls);
        assert!(!result.ok);
        assert!(result.message.contains("开发模式"));
    }

    #[test]
    fn test_perform_update_unknown_mode() {
        let urls = empty_download_urls();
        let result = perform_update("unknown", &urls);
        assert!(!result.ok);
    }

    #[test]
    fn test_perform_update_empty_url() {
        let urls = empty_download_urls();
        let result = perform_update("setup", &urls);
        assert!(!result.ok);
        assert!(result.message.contains("下载链接"));
    }

    #[test]
    fn test_generate_updater_bat_contains_modes() {
        let bat = generate_updater_bat("12345", "/tmp/file.exe", "setup", "/app/exe");
        assert!(bat.contains("setup"));
        assert!(bat.contains("portable"));
        assert!(bat.contains("naked"));
        assert!(bat.contains("Waiting for application"));
    }

    #[test]
    fn test_check_update_returns_response_on_error() {
        // This will fail because we can't reach GitHub in tests,
        // but it should return a valid response with error field set
        let response = check_for_update("0.1.0", "dev");
        assert_eq!(response.current_version, "0.1.0");
        assert!(!response.update_available);
        // Error may or may not be set depending on network
    }
}
