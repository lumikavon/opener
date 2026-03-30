import test from 'node:test';
import assert from 'node:assert/strict';

import { getEntryFormRequirements } from './entry-form-requirements.mjs';

test('script entries keep both script path and script content optional in the editor', () => {
  const requirements = getEntryFormRequirements({
    type: 'script',
    targetValue: '',
    scriptContentValue: '',
  });

  assert.equal(requirements.targetDisabled, false);
  assert.equal(requirements.targetRequired, false);
  assert.equal(requirements.scriptContentDisabled, false);
  assert.equal(requirements.scriptContentRequired, false);
});

test('ahk entries keep both script path and script content optional in the editor', () => {
  const requirements = getEntryFormRequirements({
    type: 'ahk',
    targetValue: '',
    scriptContentValue: '',
  });

  assert.equal(requirements.targetDisabled, false);
  assert.equal(requirements.targetRequired, false);
  assert.equal(requirements.scriptContentDisabled, false);
  assert.equal(requirements.scriptContentRequired, false);
});

test('non-script command entries still require a target', () => {
  const requirements = getEntryFormRequirements({
    type: 'cmd',
    targetValue: '',
    scriptContentValue: '',
  });

  assert.equal(requirements.targetDisabled, false);
  assert.equal(requirements.targetRequired, true);
  assert.equal(requirements.scriptContentDisabled, true);
  assert.equal(requirements.scriptContentRequired, false);
});
