# Entry Editor Webview Design

Date: 2026-03-30
Status: Draft for user review

## Summary

Replace the in-page `entry-modal` editor with a dedicated Tauri webview window that behaves like a child modal of the settings window.

The new flow is:

`main -> settings -> entry-editor`

When `entry-editor` is open:

- `main` remains disabled
- `settings` becomes disabled
- `entry-editor` receives focus

When `entry-editor` closes:

- `settings` is re-enabled and focused
- `main` stays disabled until `settings` closes

## Goals

- Reuse the existing add/edit entry form and save logic
- Make both `Add Entry` and `Edit Entry` use the same independent webview window
- Match the current settings-window interaction model
- Refresh the settings entry list after a successful save

## Non-Goals

- Redesign the entry form layout
- Change entry persistence or execution behavior
- Move template editing into a separate window
- Refactor unrelated settings or main-window UI

## Approved Decisions

- Use a separate webview window, not an in-page modal
- Reuse the existing form structure and behavior as much as possible
- Apply the same disable/focus behavior as the settings window
- Treat the entry editor as a second-level child of settings

## Architecture

### Window Roles

Add a new window role and label:

- label: `entry-editor`
- role: `entry-editor`

The app will continue to use `index.html` for all windows, with frontend behavior switching by window role:

- `main`
- `settings`
- `entry-editor`

### Startup Window Creation

Create the `entry-editor` window during app startup, just like `settings`, and keep it hidden until needed.

Reason:

- avoids runtime window-creation failures
- keeps role initialization and IPC wiring predictable
- matches the now-stable settings window pattern

### Parent / Child Behavior

Opening `entry-editor` from `settings` will:

1. show and focus `entry-editor`
2. disable `settings`
3. keep `main` disabled

Closing `entry-editor` will:

1. hide `entry-editor`
2. re-enable `settings`
3. focus `settings`

Closing `settings` while `entry-editor` is open should first close or hide `entry-editor`, then restore `main`.

## Data Flow

### Open Context

Backend window management stores the current editor session context:

- mode: `create` or `edit`
- `entry_id`: optional
- opener: `settings`

Frontend entry-editor initialization reads this context and then:

- resets the form for `create`
- fetches the target entry and hotkey state for `edit`

### Save Flow

The editor window owns form submission and validation.

On successful save:

1. emit `entry-editor-saved` to `settings`
2. close `entry-editor`

`settings` listens for `entry-editor-saved` and refreshes the entries list.

`main` does not listen to this event directly. It still refreshes when `settings` closes, preserving the existing main-window refresh model.

## Frontend Changes

### Main / Settings Windows

Replace current in-page modal entry points:

- `showAddEntryModal`
- `showEditEntryModal`

with commands that open the `entry-editor` window.

The old `entry-modal` overlay markup and close/open logic will be removed once the standalone window path is complete.

### Entry Editor Window

Reuse the current entry form fields, translations, and submit/test logic, but render them as standalone page content instead of a modal overlay.

Expected behavior:

- title bar matches the existing frameless window style
- close button hides the editor window
- escape closes the editor window
- save keeps existing create/update semantics
- test uses the same execution preview flow

## Backend Changes

Add window-management support for:

- create/open/close `entry-editor`
- editor session context storage
- restore focus to `settings` on close
- propagate `entry-editor-saved` to `settings`

Add commands for:

- open entry editor in `create` mode
- open entry editor in `edit` mode
- fetch current editor session context

## Testing

### Rust

- config test for `entry-editor` window declaration
- startup test confirming hidden pre-creation
- lifecycle tests for open/focus/disable/restore ordering
- close-chain test for `entry-editor -> settings -> main`

### Frontend Node Tests

- window-role detection for `entry-editor`
- context-loading behavior for `create` vs `edit`
- event-driven refresh contract for settings

## Risks

- `app.js` is already large, so entry-form logic should be extracted instead of copied
- nested window disable/focus behavior can regress if close ordering is inconsistent
- partial migration must not leave stale `entry-modal` handlers active

## Implementation Notes

- prefer extracting reusable entry-editor helpers before wiring the new window role
- keep save/test logic shared to avoid behavior drift
- remove the old in-page entry modal only after the standalone path is verified
