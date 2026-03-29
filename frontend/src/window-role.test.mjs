import test from 'node:test';
import assert from 'node:assert/strict';

import {
  WINDOW_ROLE_MAIN,
  WINDOW_ROLE_SETTINGS,
  detectWindowLabel,
  getWindowRole,
} from './window-role.mjs';
import { getTauriBridge } from './tauri-bridge.mjs';

test('getWindowRole returns settings role for settings window label', () => {
  assert.equal(getWindowRole('settings'), WINDOW_ROLE_SETTINGS);
});

test('getWindowRole falls back to main role for unknown labels', () => {
  assert.equal(getWindowRole('main'), WINDOW_ROLE_MAIN);
  assert.equal(getWindowRole('anything-else'), WINDOW_ROLE_MAIN);
});

test('detectWindowLabel prefers current window label when available', () => {
  assert.equal(
    detectWindowLabel({
      __TAURI_INTERNALS__: {
        metadata: {
          currentWindow: { label: 'settings' },
          currentWebview: { label: 'other' },
        },
      },
    }),
    'settings'
  );
});

test('detectWindowLabel prefers the injected opener window role marker', () => {
  assert.equal(
    detectWindowLabel({
      __OPENER_WINDOW_ROLE__: 'settings',
      __TAURI_INTERNALS__: {
        metadata: {
          currentWindow: { label: 'main' },
          currentWebview: { label: 'main' },
        },
      },
    }),
    'settings'
  );
});

test('detectWindowLabel falls back to the window query parameter', () => {
  assert.equal(
    detectWindowLabel({
      location: {
        search: '?window=settings',
      },
    }),
    'settings'
  );
});

test('detectWindowLabel falls back to current webview label when current window metadata is missing', () => {
  assert.equal(
    detectWindowLabel({
      __TAURI_INTERNALS__: {
        metadata: {
          currentWebview: { label: 'settings' },
        },
      },
    }),
    'settings'
  );
});

test('detectWindowLabel defaults to main when tauri metadata is unavailable', () => {
  assert.equal(detectWindowLabel({}), 'main');
});

test('getTauriBridge returns null methods instead of throwing when Tauri global is missing', () => {
  const bridge = getTauriBridge({});

  assert.equal(bridge.invoke, null);
  assert.equal(bridge.listen, null);
});

test('getTauriBridge returns invoke and listen when Tauri global is available', () => {
  const invoke = () => {};
  const listen = () => {};
  const bridge = getTauriBridge({
    __TAURI__: {
      core: { invoke },
      event: { listen },
    },
  });

  assert.equal(bridge.invoke, invoke);
  assert.equal(bridge.listen, listen);
});
