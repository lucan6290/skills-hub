#!/usr/bin/env node
/**
 * 从 CHANGELOG.md 提取指定版本的变更内容。
 *
 * 用法：
 *   node scripts/extract-changelog.mjs v0.8.1
 *
 * 解析 CHANGELOG.md 中 "## [x.y.z]" 对应的段落，输出到 stdout。
 * 找不到时退出码非 0，调用方回退到默认内容。
 */
import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

const SCRIPT_DIR = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(SCRIPT_DIR, "..");
const CHANGELOG = path.resolve(ROOT, "CHANGELOG.md");

function extractVersion(changelog, version) {
  // version 形如 "v0.8.1" 或 "0.8.1"，提取数字部分
  const verNum = version.replace(/^v/, "");
  // 匹配 ## [0.8.1] 或 ## [0.8.1] - 2026-08-15
  const headerRe = new RegExp(
    `^##\\s+\\[${verNum.replace(/\./g, "\\.")}\\]`,
    "m"
  );
  const lines = changelog.split("\n");
  let inSection = false;
  const result = [];

  for (const line of lines) {
    if (!inSection) {
      if (headerRe.test(line)) {
        inSection = true;
      }
      continue;
    }
    // 遇到下一个 ## 版本标题或 ## [Unreleased] 时停止
    if (/^##\s+/.test(line)) {
      break;
    }
    result.push(line);
  }

  const body = result.join("\n").trim();
  return body ? `## ${version}\n\n${body}` : null;
}

function main() {
  const version = process.argv[2];
  if (!version) {
    console.error("Usage: node extract-changelog.mjs <vX.Y.Z>");
    process.exit(1);
  }

  if (!fs.existsSync(CHANGELOG)) {
    console.error(`CHANGELOG.md not found at ${CHANGELOG}`);
    process.exit(1);
  }

  const content = fs.readFileSync(CHANGELOG, "utf8");
  const section = extractVersion(content, version);

  if (!section) {
    console.error(`No changelog section found for ${version}`);
    process.exit(1);
  }

  process.stdout.write(section);
}

main();
