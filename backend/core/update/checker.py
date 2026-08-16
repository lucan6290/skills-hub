"""版本检查模块 — 通过 GitHub API 检查最新发布版本"""
import json
import re
import socket
import ssl
import urllib.error
import urllib.request

GITHUB_OWNER = "lucan6290"
GITHUB_REPO = "skills-hub"
RELEASES_PAGE = f"https://github.com/{GITHUB_OWNER}/{GITHUB_REPO}/releases"
CHANGELOG_URL = f"https://github.com/{GITHUB_OWNER}/{GITHUB_REPO}/blob/main/CHANGELOG.md"


def _friendly_network_error(error: Exception) -> str:
    reason = error.reason if isinstance(error, urllib.error.URLError) else error
    text = str(reason).lower()
    if isinstance(reason, (TimeoutError, socket.timeout)) or "timed out" in text or "timeout" in text:
        return "网络连接超时，请检查网络或稍后重试"
    if isinstance(reason, ssl.SSLError) or "ssl" in text or "certificate" in text:
        return "网络 SSL 握手失败，请检查网络代理、证书或稍后重试"
    return f"网络错误：{reason}"


def _empty_download_urls() -> dict:
    return {"setup": "", "portable": "", "exe": ""}


def _release_download_urls(tag_name: str) -> dict:
    base_url = f"https://github.com/{GITHUB_OWNER}/{GITHUB_REPO}/releases/download/{tag_name}"
    return {
        "setup": f"{base_url}/SkillsHub-Setup-{tag_name}.exe",
        "portable": f"{base_url}/SkillsHub-Portable-{tag_name}.zip",
        "exe": f"{base_url}/SkillsHub.exe",
    }


def _check_for_update_from_releases_page(current_version: str) -> dict:
    request = urllib.request.Request(
        f"{RELEASES_PAGE}/latest", headers={"User-Agent": "SkillsHub-Update-Checker"}
    )
    with urllib.request.urlopen(request, timeout=10) as response:
        latest_url = response.geturl()

    match = re.search(r"/releases/tag/(?P<tag>v?\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?)", latest_url)
    if not match:
        raise ValueError("无法从 GitHub Releases 页面解析最新版本")

    tag_name = match.group("tag")
    latest_version = tag_name.lstrip("v")
    update_available = _compare_versions(current_version, latest_version)
    return {
        "current_version": current_version,
        "latest_version": latest_version,
        "update_available": update_available,
        "release_url": latest_url,
        "release_notes": "",
        "download_urls": _release_download_urls(tag_name),
        "changelog_url": CHANGELOG_URL,
    }


def _compare_versions(current: str, latest: str) -> bool:
    """比较版本号，返回 True 表示 latest 比 current 新"""
    current_tuple = tuple(map(int, current.split(".")))
    latest_tuple = tuple(map(int, latest.split(".")))
    return latest_tuple > current_tuple


def check_for_update(current_version: str) -> dict:
    """检查 GitHub Releases 是否有新版本

    调用 GitHub API 获取最新 release 信息，解析版本号并比较。
    返回包含版本信息、更新状态和下载链接的字典。
    """
    url = f"https://api.github.com/repos/{GITHUB_OWNER}/{GITHUB_REPO}/releases/latest"
    try:
        request = urllib.request.Request(
            url, headers={"User-Agent": "SkillsHub-Update-Checker"}
        )
        with urllib.request.urlopen(request, timeout=10) as response:
            data = json.loads(response.read().decode("utf-8"))

        # 提取版本号（去掉 'v' 前缀）
        tag_name = data.get("tag_name", "")
        latest_version = tag_name.lstrip("v")

        # 提取 release 页面链接和 release notes
        html_url = data.get("html_url", "")
        body = data.get("body", "")

        # 从 assets 中找到三种产物的下载 URL
        setup_url = ""
        portable_url = ""
        exe_url = ""
        for asset in data.get("assets", []):
            name = asset.get("name", "")
            browser_download_url = asset.get("browser_download_url", "")
            if "Setup" in name:
                setup_url = browser_download_url
            elif "Portable" in name:
                portable_url = browser_download_url
            elif name == "SkillsHub.exe":
                exe_url = browser_download_url

        update_available = _compare_versions(current_version, latest_version)

        return {
            "current_version": current_version,
            "latest_version": latest_version,
            "update_available": update_available,
            "release_url": html_url,
            "release_notes": body,
            "download_urls": {
                "setup": setup_url,
                "portable": portable_url,
                "exe": exe_url,
            },
            "changelog_url": CHANGELOG_URL,
        }
    except urllib.error.HTTPError as e:
        # 404 = 仓库尚未发布任何 release：无可用更新，属正常状态，不视为错误
        if e.code == 404:
            return {
                "current_version": current_version,
                "latest_version": current_version,
                "update_available": False,
                "release_url": RELEASES_PAGE,
                "release_notes": "",
                "download_urls": _empty_download_urls(),
                "changelog_url": CHANGELOG_URL,
            }
        if e.code == 403:
            try:
                return _check_for_update_from_releases_page(current_version)
            except Exception:
                error_msg = "GitHub API 访问受限，请稍后重试或打开下载页面查看最新版本"
        else:
            error_msg = f"GitHub API 错误 ({e.code})"
        return {
            "current_version": current_version,
            "latest_version": current_version,
            "update_available": False,
            "release_url": RELEASES_PAGE,
            "release_notes": "",
            "download_urls": _empty_download_urls(),
            "changelog_url": CHANGELOG_URL,
            "error": error_msg,
        }
    except Exception as e:
        return {
            "current_version": current_version,
            "latest_version": current_version,
            "update_available": False,
            "release_url": RELEASES_PAGE,
            "release_notes": "",
            "download_urls": _empty_download_urls(),
            "changelog_url": CHANGELOG_URL,
            "error": _friendly_network_error(e),
        }
