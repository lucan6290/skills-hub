"""共享常量：忽略文件名、扫描基目录、frontmatter 已知字段。"""
from __future__ import annotations

IGNORE_NAMES = frozenset({".git", ".DS_Store", "Thumbs.db", ".gitignore"})
SKILL_SCAN_BASES = ("skills", ".claude/skills")
KNOWN_FRONTMATTER_KEYS = frozenset({"name", "description", "version", "author", "license", "category", "homepage"})
