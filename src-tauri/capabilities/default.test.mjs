import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';

async function readDefaultCapability() {
  const content = await readFile(new URL('./default.json', import.meta.url), 'utf8');
  return JSON.parse(content);
}

test('default capability grants the settings window access to Tauri IPC', async () => {
  const capability = await readDefaultCapability();

  assert.ok(Array.isArray(capability.windows), 'capability.windows must be an array');
  assert.ok(
    capability.windows.includes('settings'),
    'the settings window must be listed in src-tauri/capabilities/default.json',
  );
  assert.ok(
    capability.windows.includes('entry-editor'),
    'the entry-editor window must be listed in src-tauri/capabilities/default.json',
  );
});
