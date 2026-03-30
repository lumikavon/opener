export const ALL_ENTRY_TYPE_FILTER = 'all';

const ENTRY_TYPE_ORDER = [
  'app',
  'url',
  'file',
  'dir',
  'cmd',
  'wsl',
  'ssh',
  'script',
  'shortcut',
  'ahk',
  'hotkey_app',
];

export function getAvailableEntryTypes(entries) {
  const presentTypes = new Set(
    entries
      .map((entry) => entry?.type)
      .filter(Boolean)
  );

  const knownTypes = ENTRY_TYPE_ORDER.filter((type) => presentTypes.has(type));
  const unknownTypes = Array.from(presentTypes)
    .filter((type) => !ENTRY_TYPE_ORDER.includes(type))
    .sort();

  return knownTypes.concat(unknownTypes);
}

export function normalizeEntryTypeFilter(typeFilter, entries) {
  if (!typeFilter || typeFilter === ALL_ENTRY_TYPE_FILTER) {
    return ALL_ENTRY_TYPE_FILTER;
  }

  return getAvailableEntryTypes(entries).includes(typeFilter)
    ? typeFilter
    : ALL_ENTRY_TYPE_FILTER;
}

function buildEntrySearchHaystack(entry) {
  return [
    entry.name,
    entry.target,
    entry.description,
    entry.tags,
    entry.type,
    entry.hotkey_filter,
    entry.hotkey_position,
  ]
    .filter(Boolean)
    .join(' ')
    .toLowerCase();
}

export function getFilteredEntriesByQueryAndType(
  entries,
  {
    query = '',
    typeFilter = ALL_ENTRY_TYPE_FILTER,
  } = {}
) {
  const normalizedQuery = query.trim().toLowerCase();
  const normalizedTypeFilter = normalizeEntryTypeFilter(typeFilter, entries);

  return entries.filter((entry) => {
    if (normalizedTypeFilter !== ALL_ENTRY_TYPE_FILTER && entry.type !== normalizedTypeFilter) {
      return false;
    }

    if (!normalizedQuery) {
      return true;
    }

    return buildEntrySearchHaystack(entry).includes(normalizedQuery);
  });
}
