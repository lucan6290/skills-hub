//! Update checker — version comparison + release info via the GitHub Releases API.
//!
//! The actual download/install is performed by the native `tauri-plugin-updater`
//! (see `commands::update::do_update`). This module only provides the "is there an
//! update?" information used to display version + release notes in the UI.
//!
//! # Release setup (required for `do_update` to actually install)
//!
//! The native updater needs a signed `latest.json` manifest and a signing key:
//! 1. Generate a keypair: `npm run tauri signer generate -w ~/.tauri/skills-hub.key`
//!    (keep `TAURI_PRIVATE_KEY` / `TAURI_KEY_PASSWORD` as CI secrets; never commit them).
//! 2. Add to `tauri.conf.json`:
//!    ```json
//!    "plugins": { "updater": {
//!      "active": true,
//!      "endpoints": ["https://github.com/lucan6290/skills-hub/releases/latest/download/latest.json"],
//!      "pubkey": "<the public key from step 1>"
//!    }}
//!    ```
//! 3. In CI, build with the signing env vars set so Tauri emits a signed
//!    `latest.json` + signed installers, and upload `latest.json` as a release asset.

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
    fn test_check_update_returns_response_on_error() {
        // This will fail because we can't reach GitHub in tests,
        // but it should return a valid response with error field set
        let response = check_for_update("0.1.0", "dev");
        assert_eq!(response.current_version, "0.1.0");
        assert!(!response.update_available);
        // Error may or may not be set depending on network
    }
}
