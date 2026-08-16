"""测试版本检查模块"""
import json
import urllib.error
from unittest.mock import patch, MagicMock
from core.update.checker import check_for_update, _compare_versions


class TestCompareVersions:
    """测试版本比较逻辑"""

    def test_same_version(self):
        assert _compare_versions("0.8.0", "0.8.0") is False

    def test_higher_version(self):
        assert _compare_versions("0.8.0", "0.9.0") is True

    def test_lower_version(self):
        assert _compare_versions("0.9.0", "0.8.0") is False

    def test_major_version_increase(self):
        assert _compare_versions("0.8.0", "1.0.0") is True

    def test_patch_version(self):
        assert _compare_versions("0.8.0", "0.8.1") is True


class TestCheckForUpdate:
    """测试 check_for_update 函数"""

    @patch("core.update.checker.urllib.request.urlopen")
    def test_update_available(self, mock_urlopen):
        """有更新时正确解析"""
        mock_response = MagicMock()
        mock_response.read.return_value = json.dumps({
            "tag_name": "v0.9.0",
            "html_url": "https://github.com/lucan6290/skills-hub/releases/v0.9.0",
            "body": "Bug fixes and improvements",
            "assets": [
                {"name": "SkillsHub-Setup-v0.9.0.exe", "browser_download_url": "https://example.com/setup.exe"},
                {"name": "SkillsHub-Portable-v0.9.0.zip", "browser_download_url": "https://example.com/portable.zip"},
                {"name": "SkillsHub.exe", "browser_download_url": "https://example.com/SkillsHub.exe"},
            ],
        }).encode()
        mock_response.__enter__ = MagicMock(return_value=mock_response)
        mock_response.__exit__ = MagicMock(return_value=False)
        mock_urlopen.return_value = mock_response

        result = check_for_update("0.8.0")

        assert result["current_version"] == "0.8.0"
        assert result["latest_version"] == "0.9.0"
        assert result["update_available"] is True
        assert result["release_url"] == "https://github.com/lucan6290/skills-hub/releases/v0.9.0"
        assert result["release_notes"] == "Bug fixes and improvements"
        assert result["download_urls"]["setup"] == "https://example.com/setup.exe"
        assert result["download_urls"]["portable"] == "https://example.com/portable.zip"
        assert result["download_urls"]["exe"] == "https://example.com/SkillsHub.exe"

    @patch("core.update.checker.urllib.request.urlopen")
    def test_no_update(self, mock_urlopen):
        """已是最新版本"""
        mock_response = MagicMock()
        mock_response.read.return_value = json.dumps({
            "tag_name": "v0.8.0",
            "html_url": "https://github.com/lucan6290/skills-hub/releases/v0.8.0",
            "body": "",
            "assets": [],
        }).encode()
        mock_response.__enter__ = MagicMock(return_value=mock_response)
        mock_response.__exit__ = MagicMock(return_value=False)
        mock_urlopen.return_value = mock_response

        result = check_for_update("0.8.0")

        assert result["update_available"] is False
        assert result["latest_version"] == "0.8.0"

    @patch("core.update.checker.urllib.request.urlopen")
    def test_network_error(self, mock_urlopen):
        """网络错误时降级返回"""
        mock_urlopen.side_effect = Exception("Connection failed")

        result = check_for_update("0.8.0")

        assert result["update_available"] is False
        assert result["current_version"] == "0.8.0"
        assert result["latest_version"] == "0.8.0"
        assert "error" in result

    @patch("core.update.checker.urllib.request.urlopen")
    def test_no_releases_404_is_not_error(self, mock_urlopen):
        """404（仓库尚未发布 release）应视为无更新，不返回 error"""
        mock_urlopen.side_effect = urllib.error.HTTPError(
            "https://api.github.com/repos/lucan6290/skills-hub/releases/latest",
            404,
            "Not Found",
            {},
            None,
        )

        result = check_for_update("0.8.0")

        assert result["update_available"] is False
        assert result["current_version"] == "0.8.0"
        assert result["latest_version"] == "0.8.0"
        assert "error" not in result
        assert result["changelog_url"].endswith("CHANGELOG.md")

    @patch("core.update.checker.urllib.request.urlopen")
    def test_ssl_handshake_timeout_returns_friendly_error(self, mock_urlopen):
        mock_urlopen.side_effect = urllib.error.URLError(
            TimeoutError("_ssl.c:983: The handshake operation timed out")
        )

        result = check_for_update("0.8.0")

        assert result["update_available"] is False
        assert result["current_version"] == "0.8.0"
        assert result["latest_version"] == "0.8.0"
        assert result["error"] == "网络连接超时，请检查网络或稍后重试"

    @patch("core.update.checker.urllib.request.urlopen")
    def test_api_403_falls_back_to_latest_release_page(self, mock_urlopen):
        mock_response = MagicMock()
        mock_response.geturl.return_value = "https://github.com/lucan6290/skills-hub/releases/tag/v0.8.1"
        mock_response.__enter__ = MagicMock(return_value=mock_response)
        mock_response.__exit__ = MagicMock(return_value=False)
        mock_urlopen.side_effect = [
            urllib.error.HTTPError(
                "https://api.github.com/repos/lucan6290/skills-hub/releases/latest",
                403,
                "Forbidden",
                {},
                None,
            ),
            mock_response,
        ]

        result = check_for_update("0.8.0")

        assert result["current_version"] == "0.8.0"
        assert result["latest_version"] == "0.8.1"
        assert result["update_available"] is True
        assert result["release_url"] == "https://github.com/lucan6290/skills-hub/releases/tag/v0.8.1"
        assert result["download_urls"]["setup"] == "https://github.com/lucan6290/skills-hub/releases/download/v0.8.1/SkillsHub-Setup-v0.8.1.exe"
        assert "error" not in result
