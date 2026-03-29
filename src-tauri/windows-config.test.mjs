import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';

async function readTauriConfig() {
  const content = await readFile(new URL('./tauri.conf.json', import.meta.url), 'utf8');
  return JSON.parse(content);
}

test('tauri windows config pre-registers the settings window', async () => {
  const config = await readTauriConfig();
  const windows = config?.app?.windows;

  assert.ok(Array.isArray(windows), 'app.windows must be an array');

  const settingsWindow = windows.find((window) => window.label === 'settings');
  assert.ok(settingsWindow, 'settings window must be declared in tauri.conf.json');
  assert.equal(settingsWindow.url, 'index.html?window=settings');
  assert.equal(settingsWindow.visible, false);
});
