export function getTauriBridge(globalObject = window) {
  const tauri = globalObject?.__TAURI__;

  return {
    invoke: tauri?.core?.invoke || null,
    listen: tauri?.event?.listen || null,
  };
}
