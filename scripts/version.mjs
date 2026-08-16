#!/usr/bin/env node
/**
 * 版本号管理脚本 — 前后端唯一版本来源。
 *
 * 唯一版本源：
 *   - frontend/package.json（npm 版本，同时被 Vite 构建时注入为 __APP_VERSION__）
 *   - backend/core/version.py（Python 后端读取）
 *
 * 用法：
 *   node scripts/version.mjs set <x.y.z>   同时更新前后端版本号
 *   node scripts/version.mjs check         校验前后端版本一致
 */
import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

const SCRIPT_DIR = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(SCRIPT_DIR, "..");
const FRONTEND_DIR = path.resolve(ROOT, "frontend");
const BACKEND_VERSION_FILE = path.resolve(ROOT, "backend", "core", "version.py");
const PACKAGE_JSON = path.resolve(FRONTEND_DIR, "package.json");

function read(filePath) {
  return fs.readFileSync(filePath, "utf8");
}

function write(filePath, contents) {
  fs.writeFileSync(filePath, contents, "utf8");
}

function getPackageJsonVersion() {
  const pkg = JSON.parse(read(PACKAGE_JSON));
  if (!pkg.version || typeof pkg.version !== "string") {
    throw new Error("package.json missing valid version");
  }
  return pkg.version;
}

function setPackageJsonVersion(newVersion) {
  const original = read(PACKAGE_JSON);
  const re = /("version"\s*:\s*")([^"]*)(")/;
  const m = original.match(re);
  if (!m) throw new Error(`Cannot find "version" in package.json`);
  if (m[2] === newVersion) return { from: m[2], to: newVersion, changed: false };
  write(PACKAGE_JSON, original.replace(re, `$1${newVersion}$3`));
  return { from: m[2], to: newVersion, changed: true };
}

const VERSION_PY_RE = /^(__version__\s*=\s*")([^"]*)(")/m;

function getBackendVersion() {
  const content = read(BACKEND_VERSION_FILE);
  const m = content.match(VERSION_PY_RE);
  if (!m) throw new Error(`Cannot find __version__ in ${BACKEND_VERSION_FILE}`);
  return m[2];
}

function setBackendVersion(newVersion) {
  const original = read(BACKEND_VERSION_FILE);
  const m = original.match(VERSION_PY_RE);
  if (!m) throw new Error(`Cannot find __version__ in ${BACKEND_VERSION_FILE}`);
  if (m[2] === newVersion) return { from: m[2], to: newVersion, changed: false };
  write(BACKEND_VERSION_FILE, original.replace(VERSION_PY_RE, `$1${newVersion}$3`));
  return { from: m[2], to: newVersion, changed: true };
}

function usage() {
  console.log("Usage:");
  console.log("  node scripts/version.mjs set <x.y.z>   set version for frontend & backend");
  console.log("  node scripts/version.mjs check         verify versions are in sync");
}

async function main() {
  const [cmd, arg] = process.argv.slice(2);
  if (!cmd) {
    usage();
    process.exit(1);
  }

  if (cmd === "set") {
    if (!arg) {
      usage();
      process.exit(1);
    }
    if (!/^\d+\.\d+\.\d+$/.test(arg)) {
      console.error(`Invalid version: "${arg}" (expected x.y.z)`);
      process.exit(1);
    }
    const fe = setPackageJsonVersion(arg);
    const be = setBackendVersion(arg);
    if (fe.changed) console.log(`frontend/package.json: ${fe.from} -> ${fe.to}`);
    else console.log(`frontend/package.json: already ${fe.to}`);
    if (be.changed) console.log(`backend/core/version.py: ${be.from} -> ${be.to}`);
    else console.log(`backend/core/version.py: already ${be.to}`);
    console.log(`\nVersion set to ${arg}. Don't forget to commit & tag v${arg}.`);
    return;
  }

  if (cmd === "check") {
    const feVersion = getPackageJsonVersion();
    const beVersion = getBackendVersion();
    if (feVersion !== beVersion) {
      console.error(`Version mismatch! frontend=${feVersion}, backend=${beVersion}`);
      console.error(`Run: node scripts/version.mjs set <version>`);
      process.exit(1);
    }
    console.log(`Version OK (${feVersion}) — frontend & backend in sync`);
    return;
  }

  usage();
  process.exit(1);
}

main().catch((err) => {
  console.error(err?.stack || String(err));
  process.exit(1);
});
