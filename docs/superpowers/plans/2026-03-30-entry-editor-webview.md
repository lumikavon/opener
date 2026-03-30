# Entry Editor Webview Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the in-page entry add/edit modal with a dedicated child webview window that opens from settings, disables settings while active, and refreshes the settings entry list after save.

**Architecture:** Reuse `index.html` and add a new `entry-editor` window role. Move window orchestration and editor session state into Rust window management, then reuse the existing entry form logic inside the new standalone role instead of maintaining a separate modal path.

**Tech Stack:** Tauri v2, Rust, plain ES modules, HTML, Tailwind CSS, Node built-in test runner, Cargo tests

---

### File Map

**Create:**
- `docs/superpowers/plans/2026-03-30-entry-editor-webview.md`
- `frontend/src/entry-editor-context.test.mjs`

**Modify:**
- `src-tauri/tauri.conf.json`
- `src-tauri/windows-config.test.mjs`
- `src-tauri/src/windowing.rs`
- `src-tauri/src/commands.rs`
- `src-tauri/src/main.rs`
- `frontend/src/window-role.mjs`
- `frontend/src/window-role.test.mjs`
- `frontend/index.html`
- `frontend/src/app.js`

### Task 1: Add Entry Editor Window Shell

**Files:**
- Modify: `src-tauri/tauri.conf.json`
- Modify: `src-tauri/windows-config.test.mjs`
- Modify: `src-tauri/src/windowing.rs`
- Modify: `src-tauri/src/main.rs`

- [ ] **Step 1: Write the failing window config tests**

Add assertions for an `entry-editor` window entry in `src-tauri/windows-config.test.mjs` and matching Rust source-order assertions in `src-tauri/src/windowing.rs`.

- [ ] **Step 2: Run the failing tests**

Run: `node --test src-tauri/windows-config.test.mjs`
Expected: FAIL because `entry-editor` is not declared yet

Run: `cargo test --manifest-path src-tauri/Cargo.toml entry_editor`
Expected: FAIL because entry-editor window lifecycle helpers do not exist yet

- [ ] **Step 3: Add the Tauri window config**

Declare a hidden `entry-editor` window in `src-tauri/tauri.conf.json` with:
- `label: "entry-editor"`
- `url: "index.html?window=entry-editor"`
- hidden by default
- frameless style aligned with `settings`
- compact editor dimensions

- [ ] **Step 4: Add Rust window creation and lifecycle helpers**

In `src-tauri/src/windowing.rs`:
- add window label and event constants for `entry-editor`
- add role initialization script support for `entry-editor`
- create and prepare the editor window during startup
- implement show/hide/focus/restore logic so opening the editor disables settings and closing it re-enables settings
- ensure closing settings also hides any open entry-editor before restoring main

- [ ] **Step 5: Wire startup creation**

In `src-tauri/src/main.rs`, create the hidden `entry-editor` window during setup before prepare hooks run.

- [ ] **Step 6: Re-run focused tests**

Run: `node --test src-tauri/windows-config.test.mjs`
Expected: PASS

Run: `cargo test --manifest-path src-tauri/Cargo.toml entry_editor`
Expected: PASS

### Task 2: Add Editor Session Commands

**Files:**
- Modify: `src-tauri/src/windowing.rs`
- Modify: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/main.rs`

- [ ] **Step 1: Write failing command/lifecycle tests**

Add Rust tests in `src-tauri/src/windowing.rs` that assert:
- entry-editor open path emits/uses the new event names
- close order restores settings before main
- command source includes entry-editor open/context support

- [ ] **Step 2: Run the failing tests**

Run: `cargo test --manifest-path src-tauri/Cargo.toml entry_editor`
Expected: FAIL because editor context commands do not exist yet

- [ ] **Step 3: Implement editor session state**

Add a small in-memory session context for:
- mode: `create` or `edit`
- `entry_id`
- opener window label

Store it in a Tauri-managed state type.

- [ ] **Step 4: Implement commands**

In `src-tauri/src/commands.rs` and handler registration:
- add `open_entry_editor_create`
- add `open_entry_editor_edit`
- add `get_entry_editor_context`

Each open command should set the current session and then open/focus the editor window.

- [ ] **Step 5: Re-run focused Rust tests**

Run: `cargo test --manifest-path src-tauri/Cargo.toml entry_editor`
Expected: PASS

### Task 3: Teach Frontend About the New Window Role

**Files:**
- Modify: `frontend/src/window-role.mjs`
- Modify: `frontend/src/window-role.test.mjs`
- Modify: `frontend/index.html`

- [ ] **Step 1: Write the failing role tests**

Add Node tests for:
- `getWindowRole('entry-editor')`
- `detectWindowLabel` from `?window=entry-editor`

- [ ] **Step 2: Run the failing tests**

Run: `node --test frontend/src/window-role.test.mjs`
Expected: FAIL because `entry-editor` is not mapped yet

- [ ] **Step 3: Add the window role**

Update `frontend/src/window-role.mjs` to support:
- `WINDOW_ROLE_ENTRY_EDITOR`
- `ENTRY_EDITOR_WINDOW_LABEL`

- [ ] **Step 4: Add standalone editor layout container**

In `frontend/index.html`, add an `entry-editor-window-root` section that renders the existing entry form as a page, not overlay content. Reuse the existing form fields and action buttons.

- [ ] **Step 5: Keep old modal markup out of the active path**

Remove or fully detach the old `entry-modal` overlay markup so there is only one active add/edit form implementation.

- [ ] **Step 6: Re-run role tests**

Run: `node --test frontend/src/window-role.test.mjs`
Expected: PASS

### Task 4: Move Entry Form Logic to the Standalone Window

**Files:**
- Modify: `frontend/src/app.js`
- Create: `frontend/src/entry-editor-context.test.mjs`

- [ ] **Step 1: Write the failing frontend tests**

Add Node tests for helper logic that interprets editor context:
- create mode yields blank form state
- edit mode requires `entry_id`
- settings refresh event name stays stable

- [ ] **Step 2: Run the failing tests**

Run: `node --test frontend/src/entry-editor-context.test.mjs`
Expected: FAIL because the helper/context logic does not exist yet

- [ ] **Step 3: Extract entry-editor context helpers**

In `frontend/src/app.js`, add small helpers for:
- detecting entry-editor role
- loading the current editor session context
- applying create/edit form state

- [ ] **Step 4: Replace modal open handlers**

Update current settings-window entry actions so:
- add entry calls `open_entry_editor_create`
- edit entry calls `open_entry_editor_edit`
- duplicate entry creates first, then opens the standalone editor window for the new entry

- [ ] **Step 5: Rebind form submission to the standalone role**

Make the entry-editor window own:
- close button behavior
- escape-to-close behavior
- save/test/browse handlers
- hotkey recording for editor inputs

Ensure settings window no longer binds `entry-form` modal controls.

- [ ] **Step 6: Emit refresh event on successful save**

After create or update succeeds in the editor window:
- emit `entry-editor-saved` to settings
- close the editor window

In settings:
- listen for `entry-editor-saved`
- reload entries and rerender the entries list

- [ ] **Step 7: Re-run focused frontend tests**

Run: `node --test frontend/src/window-role.test.mjs frontend/src/entry-editor-context.test.mjs`
Expected: PASS

### Task 5: Full Verification

**Files:**
- Verify existing modified files only

- [ ] **Step 1: Run frontend verification**

Run: `node --test frontend/src/window-role.test.mjs frontend/src/entry-form-requirements.test.mjs frontend/src/entry-editor-context.test.mjs src-tauri/windows-config.test.mjs src-tauri/capabilities/default.test.mjs`
Expected: all tests PASS

- [ ] **Step 2: Run Rust verification**

Run: `cargo test --manifest-path src-tauri/Cargo.toml`
Expected: all tests PASS except the existing ignored security test

- [ ] **Step 3: Run production build verification**

Run: `npm run tailwind:build`
Expected: exit 0

- [ ] **Step 4: Manual smoke checklist**

Verify in the running app:
- open settings from main
- open add entry from settings
- settings becomes disabled while editor is open
- close editor returns focus to settings
- save entry refreshes settings list
- close settings returns focus and refresh to main

- [ ] **Step 5: Commit**

```bash
git add src-tauri/tauri.conf.json src-tauri/windows-config.test.mjs src-tauri/src/windowing.rs src-tauri/src/commands.rs src-tauri/src/main.rs frontend/src/window-role.mjs frontend/src/window-role.test.mjs frontend/index.html frontend/src/app.js frontend/src/entry-editor-context.test.mjs docs/superpowers/specs/2026-03-30-entry-editor-webview-design.md docs/superpowers/plans/2026-03-30-entry-editor-webview.md
git commit -m "feat: move entry editor into child webview"
```
