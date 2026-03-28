# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**Opener** is an offline-capable desktop launcher built with Tauri v2 (Rust backend) and a plain HTML/JS frontend (no framework, Vite-bundled, TailwindCSS). It supports 11 entry types (App, URL, File, Dir, Cmd, WSL, SSH, Script, Shortcut, AHK, HotkeyApp) with fuzzy search, global hotkeys, script templates, and credential storage via the system keychain.

## Commands

```bash
# Full Tauri dev app (hot reload)
npm run tauri:dev

# Production build (Windows MSI, macOS DMG, Linux AppImage/Deb)
npm run tauri:build

# Frontend only (Vite dev server, no Rust)
npm run dev

# TailwindCSS
npm run tailwind:watch   # watch mode during frontend edits
npm run tailwind:build   # one-off build

# Rust tests
npm test
# or directly:
cargo test --manifest-path src-tauri/Cargo.toml

# Run a single Rust test
cargo test --manifest-path src-tauri/Cargo.toml -- test_name

# Format Rust code
cargo fmt --manifest-path src-tauri/Cargo.toml
```

## Architecture

### Three-layer structure

**Frontend** (`frontend/`)
- `index.html` + `src/app.js` (~3000 lines): single-page app, all UI logic in one file
- `src/input.css` → compiled to `dist/output.css` (never edit `dist/` manually)
- `assets/`: SVG icons only; no external CDN references (fully offline)
- Vite builds to `frontend/dist/`

**Rust Backend** (`src-tauri/src/`)
- `main.rs`: Tauri app setup, tray icon, window management
- `models.rs`: all shared data types (`Entry`, `Hotkey`, `Settings`, `ScriptTemplate`, etc.)
- `database.rs`: SQLite persistence layer — all CRUD, import/export logic
- `commands.rs`: Tauri IPC commands exposed to the frontend via `invoke()`
- `executor.rs`: entry execution dispatcher — handles all 11 entry types including Win32 API calls for windowless Cmd execution
- `hotkeys.rs`: global and app-scoped hotkey registration/conflict detection
- `security.rs`: system keychain integration for SSH/credential storage

**Data layer**: SQLite3 (bundled via `rusqlite`), tables: `entries`, `hotkeys`, `settings`, `script_templates`. Indexes on `name`, `type`, `tags`, `enabled` for fuzzy search performance.

### Frontend ↔ Backend communication

All IPC is through Tauri's `invoke()` API. Commands are defined in `commands.rs` with `#[tauri::command]` and registered in `main.rs`. The frontend calls them as `await invoke('command_name', { args })`.

### Key design constraints

- **Offline only**: all assets must be bundled locally; do not add CDN references
- **No frontend framework**: plain JS with camelCase functions/variables, 2-space indent, semicolons, single quotes
- **HTML IDs**: kebab-case (e.g., `btn-settings`, `entry-modal`)
- **Rust**: `snake_case` functions/variables, `PascalCase` types/enums
- **Secrets**: never log credentials; keep confirmation prompts intact for Cmd/WSL/SSH/Script/AHK execution

## Testing

Unit tests live in `src-tauri/src/*.rs` under `#[cfg(test)]`, named `test_*`. Focus on database operations, execution preview, and security logic. Add tests when modifying `commands.rs`, `database.rs`, or `executor.rs`.

## Commit & PR Guidelines

- Concise imperative subjects: `Add hotkey conflict check`
- PRs: short summary + test commands run + platform tested
- UI changes: include a screenshot of the Tauri window
