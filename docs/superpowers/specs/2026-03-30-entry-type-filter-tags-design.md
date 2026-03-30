# Entry Type Filter Tags Design

## Goal

Add single-select type filter tags to the Settings -> Entries tab.

The tags render below the "Search entries" input and work together with the existing text search:

- text query
- selected type tag

The final list is filtered by `query AND type`.

## UX

- Show one `All` tag plus one tag for each entry type that exists in the current entries list.
- Only one tag can be active at a time.
- The active type defaults to `all`.
- If the currently selected type disappears after data changes, reset the filter back to `all`.
- Tag labels reuse the existing entry type translation labels.

## Scope

- Session-only UI state in the settings window.
- No persistence in settings/database.
- No changes to main-window search behavior.

## Implementation Notes

- Add a small pure frontend helper module for:
  - available type extraction
  - type filter normalization
  - combined query/type filtering
- Render the filter tags in the settings entries toolbar under the search input.
- Keep selection, bulk delete, and empty-state logic working against the already filtered entry list.

## Verification

- Helper tests cover type extraction order, combined filtering, and invalid filter fallback.
- Existing frontend tests still pass.
- Existing Tauri/Rust tests still pass.
