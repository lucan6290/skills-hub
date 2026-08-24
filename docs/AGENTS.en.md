# Skills Hub - Project Conventions

> [中文版本](../AGENTS.md) | English

This file is the project's navigation entry point (the Agent's first map). Only global rules that apply to every task are kept here; detailed conventions are loaded level by level via **Task Routing** below.

> For a human-oriented project introduction and development guide, see [README.en.md](README.en.md).

## Overview

Skills Hub is a cross-platform desktop application (React 19 + Rust Tauri) for managing AI Agent Skills and syncing them to 44 AI coding tools. Core philosophy: *"Install once, sync everywhere."*

## Tech Stack

- **Frontend**: React 19 + TypeScript 5.9 (strict mode) + Vite 7 + Tailwind CSS 4
- **Backend**: Rust + Tauri 2 + SQLite (rusqlite)
- **i18n**: i18next (English / Chinese bilingual)
- **Notifications**: sonner (Toast)
- **Icons**: lucide-react

## Architecture Overview

### Directory Structure (high-level)

```
skills-hub/
├── frontend/          # React frontend → see frontend/AGENTS.md
│   └── src-tauri/     # Rust Tauri backend
├── docs/              # Cross-cutting docs (database schema, naming conventions, English translations)
├── scripts/           # Build & version scripts
├── .github/workflows/ # CI/CD (CI checks + Release builds)
├── AGENTS.md          # This file (project navigation entry, Chinese)
├── AGENTS.en.md       # This file (English version, in docs/)
├── CLAUDE.md          # Claude Code entry (points to AGENTS.md)
└── README.md          # Human-oriented project intro (Chinese) / README.en.md (English)
```

### Frontend ↔ Backend Communication

- Frontend calls the Rust backend via Tauri `invoke`
- API adapter: `frontend/src/lib/api.ts` (`invokeCommand` wrapper)
- Call pattern: `invokeCommand('command_name', { param })` → Rust `#[tauri::command]`

### Error Handling

- Rust backend uses the `AppError` enum for structured error codes
- Responses use a unified format: `{ ok, code, message, detail }`
- Frontend catches errors via try-catch and displays them with sonner toasts

## Global Naming Conventions (cross-cutting, mandatory)

**Mandatory**: [naming-conventions.md](naming-conventions.md). Core principles:

- **Cross-end communication fields use `snake_case`**: Frontend DTO types, Tauri command parameters, JSON payloads, and Rust struct fields all use `snake_case`
- **Frontend-internal state uses `camelCase`**: Component Props fields, useState variables, and function names use `camelCase`
- **Forbidden**: `toSnakeCase()`/`toCamelCase()` transforms, `#[serde(rename = "camelCaseName")]`, or any mismatch between frontend and backend field names

## Version Management

- A single version source is managed via `scripts/version.mjs`, syncing frontend and backend with one command
- Frontend version: `frontend/package.json` (injected as `__APP_VERSION__` at Vite build time)
- Backend version: `frontend/src-tauri/Cargo.toml` (`version` field)
- Release process: see [README.en.md#release](README.en.md#release)

## Task Routing

When receiving a task, first determine its type, then read the corresponding module entry file; the module entry will guide you to finer-grained topic documents.

| Scope | Required Entry |
|-------|---------------|
| Frontend code (components/styles/API calls/DTOs) | [../frontend/AGENTS.md](../frontend/AGENTS.md) |
| Rust backend code (commands/repositories/services) | [../frontend/src-tauri/AGENTS.md](../frontend/src-tauri/AGENTS.md) |
| Database table structure (field details) | [database-schema.md](database-schema.md) |
| Documentation only | Edit the corresponding document directly |

## Development Workflow

1. Start dev: `cd frontend && npm run tauri dev`
2. Frontend: http://localhost:5173 (Vite HMR), Backend: managed by Tauri automatically
3. Frontend pre-commit check: `cd frontend && npm run check`
4. Rust tests: `cd frontend/src-tauri && cargo test`

## Git Workflow Rules

1. **Auto-commit locally**: Whenever files change (add, modify, delete), immediately run `git add` + `git commit` locally to ensure every change is recorded.
2. **No remote push**: All commits stay in the local repository. Never execute `git push` unless the user explicitly requests it.
3. **Commit convention**: Commit message format is `type: brief description` (in Chinese), with each commit covering exactly one feature or one fix.
4. **Default remote & branch**: The default remote is `origin` (`https://github.com/lucan6290/skills-hub.git`), and the default branch is `main`.
