# Repository Guidelines

## Project Structure & Module Organization
- `frontend/` houses the UI: `frontend/index.html`, `frontend/src/app.js`, and Tailwind source in `frontend/src/input.css`. Static assets live in `frontend/assets/`.
- `src-tauri/` is the Rust backend and Tauri config: `src-tauri/src/` (commands, database, models, security), `src-tauri/tauri.conf.json`, and `src-tauri/icons/`.
- Generated/build outputs live in `frontend/dist/`, `node_modules/`, and `src-tauri/target/`; avoid manual edits.

## Build, Test, and Development Commands
- `npm run tauri:dev` runs the full Tauri app with the frontend.
- `npm run tauri:build` produces production bundles for the desktop app.
- `npm run dev` runs the Vite frontend only (useful for UI iteration).
- `npm run tailwind:watch` regenerates CSS while editing `frontend/src/input.css`.
- `npm run tailwind:build` creates a one-off Tailwind build for packaging.
- `npm test` runs Rust tests (`cargo test --manifest-path src-tauri/Cargo.toml`).

## Coding Style & Naming Conventions
- Frontend JS uses 2-space indentation, semicolons, and single quotes; keep function/variable names in camelCase.
- HTML IDs and button targets are kebab-case (e.g., `btn-settings`, `entry-modal`).
- Rust follows standard conventions: `snake_case` for functions/variables, `PascalCase` for types/enums; format with `cargo fmt` when needed.
- Tailwind utility classes are used heavily; keep class lists grouped by layout, spacing, color, then state.

## Testing Guidelines
- Unit tests live next to Rust modules in `src-tauri/src/*.rs` under `#[cfg(test)]`.
- Name tests `test_*` and focus on database, execution preview, and security logic.
- No coverage target is documented; add tests when touching command execution or persistence.

## Commit & Pull Request Guidelines
- Git history is not available in this checkout; use concise, imperative commit subjects (e.g., `Add hotkey conflict check`).
- PRs should include a short summary, test commands run, and the platform tested.
- UI changes should include a screenshot of the Tauri window.

## Security & Offline Constraints
- The app can execute Cmd/WSL/SSH/Script/AHK entries; avoid logging secrets and keep confirmation prompts intact.
- Keep dependencies offline-friendly; avoid CDN references and prefer bundled assets.
