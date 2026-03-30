export const WINDOW_ROLE_MAIN = 'main';
export const WINDOW_ROLE_SETTINGS = 'settings';
export const WINDOW_ROLE_ENTRY_EDITOR = 'entry-editor';
export const SETTINGS_WINDOW_LABEL = 'settings';
export const ENTRY_EDITOR_WINDOW_LABEL = 'entry-editor';
export const SETTINGS_WINDOW_CLOSED_EVENT = 'settings-window-closed';
export const ENTRY_EDITOR_OPENED_EVENT = 'entry-editor-opened';
export const ENTRY_EDITOR_SAVED_EVENT = 'entry-editor-saved';

export function detectWindowLabel(globalObject = window) {
  if (typeof globalObject?.__OPENER_WINDOW_ROLE__ === 'string') {
    return globalObject.__OPENER_WINDOW_ROLE__;
  }

  const search = globalObject?.location?.search;
  if (typeof search === 'string' && search.length > 1) {
    const windowRole = new URLSearchParams(search).get('window');
    if (windowRole) {
      return windowRole;
    }
  }

  const metadata = globalObject?.__TAURI_INTERNALS__?.metadata;
  return metadata?.currentWindow?.label
    || metadata?.currentWebview?.label
    || WINDOW_ROLE_MAIN;
}

export function getWindowRole(windowLabel) {
  if (windowLabel === SETTINGS_WINDOW_LABEL) {
    return WINDOW_ROLE_SETTINGS;
  }

  if (windowLabel === ENTRY_EDITOR_WINDOW_LABEL) {
    return WINDOW_ROLE_ENTRY_EDITOR;
  }

  return WINDOW_ROLE_MAIN;
}
