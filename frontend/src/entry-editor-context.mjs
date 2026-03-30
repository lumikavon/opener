import {
  ENTRY_EDITOR_OPENED_EVENT,
  ENTRY_EDITOR_SAVED_EVENT,
} from './window-role.mjs';

export const ENTRY_EDITOR_MODE_CREATE = 'create';
export const ENTRY_EDITOR_MODE_EDIT = 'edit';

export function normalizeEntryEditorContext(rawContext) {
  const mode = rawContext?.mode || ENTRY_EDITOR_MODE_CREATE;
  const entryId = rawContext?.entry_id || null;
  const opener = rawContext?.opener || 'settings';

  if (mode === ENTRY_EDITOR_MODE_EDIT && !entryId) {
    throw new Error('Entry editor edit mode requires an entry id.');
  }

  return {
    mode,
    entryId,
    opener,
  };
}

export { ENTRY_EDITOR_OPENED_EVENT, ENTRY_EDITOR_SAVED_EVENT };
