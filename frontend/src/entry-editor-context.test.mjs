import test from 'node:test';
import assert from 'node:assert/strict';

import {
  ENTRY_EDITOR_MODE_CREATE,
  ENTRY_EDITOR_MODE_EDIT,
  ENTRY_EDITOR_OPENED_EVENT,
  ENTRY_EDITOR_SAVED_EVENT,
  normalizeEntryEditorContext,
} from './entry-editor-context.mjs';

test('normalizeEntryEditorContext returns create context with blank entry id', () => {
  const context = normalizeEntryEditorContext({
    mode: ENTRY_EDITOR_MODE_CREATE,
    entry_id: null,
    opener: 'settings',
  });

  assert.deepEqual(context, {
    mode: ENTRY_EDITOR_MODE_CREATE,
    entryId: null,
    opener: 'settings',
  });
});

test('normalizeEntryEditorContext requires entry id for edit mode', () => {
  assert.throws(
    () => normalizeEntryEditorContext({
      mode: ENTRY_EDITOR_MODE_EDIT,
      entry_id: null,
      opener: 'settings',
    }),
    /entry id/i
  );
});

test('entry editor saved event stays stable for settings refresh', () => {
  assert.equal(ENTRY_EDITOR_SAVED_EVENT, 'entry-editor-saved');
});

test('entry editor opened event stays stable for session refresh', () => {
  assert.equal(ENTRY_EDITOR_OPENED_EVENT, 'entry-editor-opened');
});
