export function getEntryFormRequirements({
  type,
  targetValue = '',
  scriptContentValue = '',
}) {
  const normalizedType = String(type || '').toLowerCase();
  const isSsh = normalizedType === 'ssh';
  const isScript = normalizedType === 'script' || normalizedType === 'ahk';
  const isHotkeyApp = normalizedType === 'hotkey_app';

  const targetRequired = !isSsh && !isScript;
  const scriptContentRequired = false;

  return {
    targetDisabled: isSsh,
    targetRequired,
    workdirRequired: isHotkeyApp,
    hotkeyRequired: isHotkeyApp,
    sshHostDisabled: !isSsh,
    sshHostRequired: isSsh,
    sshUserDisabled: !isSsh,
    sshPortDisabled: !isSsh,
    scriptContentDisabled: !isScript,
    scriptContentRequired,
    hotkeyFilterDisabled: !isHotkeyApp,
    hotkeyFilterRequired: isHotkeyApp,
    hotkeyPositionDisabled: !isHotkeyApp,
    hotkeyPositionRequired: isHotkeyApp,
    hotkeyDetectHiddenDisabled: !isHotkeyApp,
    hasTargetValue: targetValue.trim().length > 0,
    hasScriptContentValue: scriptContentValue.trim().length > 0,
  };
}
