# Skills Hub

> [中文版本](../README.md) | English

A cross-platform desktop application (React 19 + Rust Tauri) for managing AI Agent Skills and syncing them to multiple AI coding tools' global or project-level skills directories (symlink/junction preferred, copy fallback), achieving "Install once, sync everywhere".

## Features

- **Tags Page**: Create, rename, and delete custom tags on a dedicated page, with quick navigation to the corresponding skill list.
- **Tag Filtering**: Add multiple tags to skills and filter by tag in My Skills, including viewing `Untagged` skills.
- **Global / Project-level Sync**: Skills can be synced to a global directory (effective across all projects) or scoped to specific project directories.
- **Scope Control**: Switch skills between global and project scope, manage project directories, and filter My Skills by scope.
- **Skill Detail View**: Click a skill name to view full file contents with Markdown rendering and code syntax highlighting (40+ languages).
- **Unified View**: See total managed skills count, scope badges, and sync status across all tools.
- **Onboarding Migration**: Scan existing skills from installed tools, import them into the Community Repo, and sync them.
- **Import Sources**: Local folders (multi-skill directory selection, `.claude/skills/` directories supported).
- **New Tool Detection**: Detect newly installed tools and prompt to sync managed skills.

## Tech Stack

- **Frontend**: React 19 + TypeScript 5.9 (strict mode) + Vite 7 + Tailwind CSS 4
- **Backend**: Rust + Tauri 2 + SQLite (rusqlite)
- **i18n**: i18next (English / Chinese bilingual)

## Development

### Requirements

- Node.js 18+ (20+ recommended)
- Rust 1.77+ (stable)
- [Tauri Prerequisites](https://v2.tauri.app/start/prerequisites/)

### Development Mode

```bash
cd frontend
npm install
npm run tauri dev              # Vite HMR + Tauri desktop window
```

### Building for Production

```bash
cd frontend
npm run tauri build            # Outputs NSIS installer to src-tauri/target/release/bundle/
```

### Quality Checks

```bash
cd frontend
npm run lint            # ESLint
npm run build           # tsc + vite build
npm run check           # lint + build
```

### Rust Tests

```bash
cd frontend/src-tauri
cargo test
```

### Version Management

Project version numbers are managed uniformly across frontend and backend via a single script:

```bash
# Run from project root
node scripts/version.mjs check              # Verify frontend and backend versions are in sync
node scripts/version.mjs set <x.y.z>        # Set a new version (updates both frontend/package.json and frontend/src-tauri/Cargo.toml)
```

Version sources:
- Frontend: `version` in `frontend/package.json` (injected into frontend code at build time by Vite)
- Backend: `version` in `frontend/src-tauri/Cargo.toml`
- The two files are kept in sync by `scripts/version.mjs`; do not modify either one manually.

### Release

**Important: Update CHANGELOG.md before tagging, otherwise CI cannot automatically extract release notes.**

```bash
# 1. Update CHANGELOG.md with a new version entry under "## [Unreleased]"
#    Format: ## [0.x.x] - YYYY-MM-DD
#    Categories: Added / Changed / Fixed / Technical

# 2. Update version (updates both frontend/package.json and frontend/src-tauri/Cargo.toml)
node scripts/version.mjs set 0.x.x

# 3. Verify frontend and backend versions are in sync
node scripts/version.mjs check

# 4. Commit and tag
git add -A
git commit -m "chore: bump version to v0.x.x"
git tag -a v0.x.x -m "Release v0.x.x"

# 5. Push code and tag (pushing the tag triggers GitHub Actions to build the Release)
git push origin main
git push origin v0.x.x
```

If CI fails and you need to push the same tag again after fixing it:

```bash
git checkout main
git pull origin main

# Confirm the current commit includes the CI fix; do not rerun the old failed workflow directly
git push origin main
git tag -d v0.x.x
git push origin :refs/tags/v0.x.x
git tag -a v0.x.x -m "Release v0.x.x"
git push origin v0.x.x
```

After pushing the tag, GitHub Actions will automatically build the exe, ZIP, and NSIS installer on a Windows runner, extract the corresponding version's content from CHANGELOG.md as release notes, and create a draft Release. You can edit release notes on the GitHub Releases page and publish manually.

## Project Structure

```
skills-hub/
├── frontend/               # React 19 + Vite frontend
│   ├── src/
│   │   ├── lib/                    # api.ts, errors.ts, pickFolder.ts, utils.ts
│   │   ├── hooks/                  # Custom hooks (useApi, useSkills, useScopeState, etc.)
│   │   ├── features/               # Feature modules (skills, settings, tools, tags, import-flow)
│   │   └── i18n/                   # English/Chinese translations
│   ├── src-tauri/          # Rust Tauri backend
│   │   ├── src/
│   │   │   ├── commands/           # Tauri command entry points
│   │   │   ├── repositories/       # Database access layer
│   │   │   ├── services/           # Business logic
│   │   │   └── update/             # Update check and execution
│   │   └── Cargo.toml
│   └── package.json
├── docs/                   # Documentation
├── scripts/                # Build & version scripts
└── README.md
```

For full architecture and coding conventions, see [`AGENTS.en.md`](AGENTS.en.md) (English) or [`../AGENTS.md`](../AGENTS.md) (中文).

## Supported AI Coding Tools

Project-level skills directories are relative to the chosen project root. Tools marked `N/A` have no confirmed project-level skills directory and only support global sync.

| tool key | Display Name | Global Skills Dir (relative to `~`) | Project Skills Dir (relative to project) | Detection (relative to `~`) |
| --- | --- | --- | --- | --- |
| `cursor` | Cursor | `.cursor/skills` | `.agents/skills` | `.cursor` |
| `claude_code` | Claude Code | `.claude/skills` | `.claude/skills` | `.claude` |
| `codex` | Codex | `.codex/skills` | `.agents/skills` | `.codex` |
| `opencode` | OpenCode | `.config/opencode/skills` | `.agents/skills` | `.config/opencode` |
| `antigravity` | Antigravity | `.gemini/antigravity/skills` | `.agents/skills` | `.gemini/antigravity` |
| `amp` | Amp | `.config/agents/skills` | `.agents/skills` | `.config/agents` |
| `kimi_cli` | Kimi Code CLI | `.config/agents/skills` | `.agents/skills` | `.config/agents` |
| `augment` | Augment | `.augment/skills` | `.augment/skills` | `.augment` |
| `openclaw` | OpenClaw | `.openclaw/skills` | `skills` | `.openclaw` |
| `copaw` | Copaw | `.copaw/skill_pool` | `.copaw/skills` | `.copaw` |
| `cline` | Cline | `.agents/skills` | `.agents/skills` | `.agents` |
| `codebuddy` | CodeBuddy | `.codebuddy/skills` | `.codebuddy/skills` | `.codebuddy` |
| `command_code` | Command Code | `.commandcode/skills` | `.commandcode/skills` | `.commandcode` |
| `continue` | Continue | `.continue/skills` | `.continue/skills` | `.continue` |
| `crush` | Crush | `.config/crush/skills` | `.crush/skills` | `.config/crush` |
| `junie` | Junie | `.junie/skills` | `.junie/skills` | `.junie` |
| `iflow_cli` | iFlow CLI | `.iflow/skills` | `.iflow/skills` | `.iflow` |
| `kiro_cli` | Kiro CLI | `.kiro/skills` | `.kiro/skills` | `.kiro` |
| `kode` | Kode | `.kode/skills` | `.kode/skills` | `.kode` |
| `mcpjam` | MCPJam | `.mcpjam/skills` | `.mcpjam/skills` | `.mcpjam` |
| `mistral_vibe` | Mistral Vibe | `.vibe/skills` | `.vibe/skills` | `.vibe` |
| `mux` | Mux | `.mux/skills` | `.mux/skills` | `.mux` |
| `openclaude` | OpenClaude IDE | `.openclaude/skills` | `.openclaude/skills` | `.openclaude` |
| `openhands` | OpenHands | `.openhands/skills` | `.openhands/skills` | `.openhands` |
| `pi` | Pi | `.pi/agent/skills` | `.pi/skills` | `.pi` |
| `qoder` | Qoder | `.qoder/skills` | `.qoder/skills` | `.qoder` |
| `qoderwork` | QoderWork | `.qoderwork/skills` | `.qoderwork/skills` | `.qoderwork` |
| `qwen_code` | Qwen Code | `.qwen/skills` | `.qwen/skills` | `.qwen` |
| `trae` | Trae | `.trae/skills` | `.trae/skills` | `.trae` |
| `trae_cn` | Trae CN | `.trae-cn/skills` | `.trae/skills` | `.trae-cn` |
| `zencoder` | Zencoder | `.zencoder/skills` | `.zencoder/skills` | `.zencoder` |
| `neovate` | Neovate | `.neovate/skills` | `.neovate/skills` | `.neovate` |
| `pochi` | Pochi | `.pochi/skills` | `.pochi/skills` | `.pochi` |
| `adal` | AdaL | `.adal/skills` | `.adal/skills` | `.adal` |
| `kilo_code` | Kilo Code | `.kilocode/skills` | `.kilocode/skills` | `.kilocode` |
| `roo_code` | Roo Code | `.roo/skills` | `.roo/skills` | `.roo` |
| `goose` | Goose | `.config/goose/skills` | `.goose/skills` | `.config/goose` |
| `gemini_cli` | Gemini CLI | `.gemini/skills` | `.agents/skills` | `.gemini` |
| `github_copilot` | GitHub Copilot | `.copilot/skills` | `.agents/skills` | `.copilot` |
| `clawdbot` | Clawdbot | `.clawdbot/skills` | `.clawdbot/skills` | `.clawdbot` |
| `droid` | Droid | `.factory/skills` | `.factory/skills` | `.factory` |
| `windsurf` | Windsurf | `.codeium/windsurf/skills` | `.windsurf/skills` | `.codeium/windsurf` |
| `moltbot` | MoltBot | `.moltbot/skills` | `.moltbot/skills` | `.moltbot` |
| `hermes_agent` | Hermes Agent | `.hermes/skills` | N/A | `.hermes` |

## Contributing & Security

- Code of Conduct: [`CODE_OF_CONDUCT.en.md`](CODE_OF_CONDUCT.en.md) (English) / [`../CODE_OF_CONDUCT.md`](../CODE_OF_CONDUCT.md) (中文)
- Security Policy: [`SECURITY.en.md`](SECURITY.en.md) (English) / [`../SECURITY.md`](../SECURITY.md) (中文)

## FAQ

- **Where are skills stored?** The Community Repo defaults to `~/.skillshub` (configurable in Settings).
- **What are tags for?** Tags help you find and organize skills; they don't change where skills are synced or which tools can use them.
- **What is project-level sync?** A skill is still stored once in the Community Repo, but its sync target is a specific project directory — e.g. `<project>/.agents/skills`, `<project>/.claude/skills`, or other tool-specific project skills paths.
- **Why does syncing to Cursor always use copy?** Cursor currently does not support symlink/junction-based skills directories, so Skills Hub forces directory copy when syncing to Cursor.
- **Why does sync sometimes fall back to copy?** Skills Hub prefers symlink/junction, but on some systems (especially Windows) symlinks may be restricted, in which case it falls back to directory copy.

## License

MIT License — see [`LICENSE`](../LICENSE).
