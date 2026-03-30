import test from 'node:test';
import assert from 'node:assert/strict';

import {
  ALL_ENTRY_TYPE_FILTER,
  getAvailableEntryTypes,
  getFilteredEntriesByQueryAndType,
  normalizeEntryTypeFilter,
} from './entry-list-filters.mjs';

test('getAvailableEntryTypes deduplicates present types and keeps known display order', () => {
  const entries = [
    { id: '1', type: 'ssh' },
    { id: '2', type: 'app' },
    { id: '3', type: 'cmd' },
    { id: '4', type: 'ssh' },
    { id: '5', type: 'url' },
  ];

  assert.deepEqual(getAvailableEntryTypes(entries), ['app', 'url', 'cmd', 'ssh']);
});

test('getFilteredEntriesByQueryAndType applies text query and type filter together', () => {
  const entries = [
    { id: '1', name: 'Docs', target: 'https://example.com', description: '', tags: '', type: 'url', hotkey_filter: '', hotkey_position: '' },
    { id: '2', name: 'Docs Sync', target: 'pwsh ./sync.ps1', description: 'sync docs', tags: 'docs', type: 'cmd', hotkey_filter: '', hotkey_position: '' },
    { id: '3', name: 'Portal', target: 'https://intranet.example.com', description: '', tags: 'internal', type: 'url', hotkey_filter: '', hotkey_position: '' },
  ];

  const filtered = getFilteredEntriesByQueryAndType(entries, {
    query: 'docs',
    typeFilter: 'url',
  });

  assert.deepEqual(filtered.map((entry) => entry.id), ['1']);
});

test('normalizeEntryTypeFilter falls back to all when the selected type is unavailable', () => {
  const entries = [
    { id: '1', type: 'app' },
    { id: '2', type: 'url' },
  ];

  assert.equal(normalizeEntryTypeFilter('ssh', entries), ALL_ENTRY_TYPE_FILTER);
});
