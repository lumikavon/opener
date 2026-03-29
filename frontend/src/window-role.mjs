export const WINDOW_ROLE_MAIN = 'main';
export const WINDOW_ROLE_SETTINGS = 'settings';
export const SETTINGS_WINDOW_LABEL = 'settings';
export const SETTINGS_WINDOW_CLOSED_EVENT = 'settings-window-closed';

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
  return windowLabel === SETTINGS_WINDOW_LABEL ? WINDOW_ROLE_SETTINGS : WINDOW_ROLE_MAIN;
}
