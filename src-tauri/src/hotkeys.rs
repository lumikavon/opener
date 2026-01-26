// Global hotkey registration helpers

use crate::models::{Hotkey, HotkeyScope};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Runtime};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

#[derive(Clone, Serialize)]
pub struct HotkeyTriggeredPayload {
    pub entry_id: String,
    pub accelerator: String,
}

pub fn register_hotkey<R: Runtime>(app: &AppHandle<R>, hotkey: &Hotkey) -> Result<(), String> {
    if hotkey.scope != HotkeyScope::Global || !hotkey.enabled {
        return Ok(());
    }

    let entry_id = hotkey.entry_id.clone();
    let accelerator = hotkey.accelerator.clone();
    let accelerator_for_register = accelerator.clone();

    app.global_shortcut()
        .on_shortcut(accelerator_for_register.as_str(), move |app_handle, _shortcut, event| {
            if event.state != ShortcutState::Pressed {
                return;
            }

            let payload = HotkeyTriggeredPayload {
                entry_id: entry_id.clone(),
                accelerator: accelerator.clone(),
            };

            if let Err(error) = app_handle.emit("hotkey-triggered", payload) {
                log::error!("Failed to emit hotkey-triggered event: {}", error);
            }
        })
        .map_err(|error| error.to_string())
}

pub fn unregister_hotkey<R: Runtime>(app: &AppHandle<R>, hotkey: &Hotkey) -> Result<(), String> {
    if hotkey.scope != HotkeyScope::Global || !hotkey.enabled {
        return Ok(());
    }

    app.global_shortcut()
        .unregister(hotkey.accelerator.as_str())
        .map_err(|error| error.to_string())
}

pub fn register_all_hotkeys<R: Runtime>(app: &AppHandle<R>, hotkeys: &[Hotkey]) {
    for hotkey in hotkeys {
        if let Err(error) = register_hotkey(app, hotkey) {
            log::warn!(
                "Failed to register global shortcut {} for entry {}: {}",
                hotkey.accelerator,
                hotkey.entry_id,
                error
            );
        }
    }
}
