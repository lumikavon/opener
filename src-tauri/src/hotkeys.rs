// Global hotkey registration helpers

use crate::models::{Hotkey, HotkeyScope};
use crate::windowing;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, Runtime};
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
        .on_shortcut(
            accelerator_for_register.as_str(),
            move |app_handle, _shortcut, event| {
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
            },
        )
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

fn toggle_main_window<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window(windowing::MAIN_WINDOW_LABEL) {
        let is_visible = window.is_visible().unwrap_or(true);
        if is_visible {
            windowing::hide_main_window(app);
        } else {
            windowing::show_main_window(app);
        }
    }
}

pub fn register_app_hotkey<R: Runtime>(
    app: &AppHandle<R>,
    accelerator: &str,
) -> Result<(), String> {
    let trimmed = accelerator.trim();
    if trimmed.is_empty() {
        return Ok(());
    }

    let accelerator_for_register = trimmed.to_string();
    app.global_shortcut()
        .on_shortcut(
            accelerator_for_register.as_str(),
            move |app_handle, _shortcut, event| {
                if event.state != ShortcutState::Pressed {
                    return;
                }

                toggle_main_window(app_handle);
            },
        )
        .map_err(|error| error.to_string())
}

pub fn unregister_app_hotkey<R: Runtime>(
    app: &AppHandle<R>,
    accelerator: &str,
) -> Result<(), String> {
    let trimmed = accelerator.trim();
    if trimmed.is_empty() {
        return Ok(());
    }

    app.global_shortcut()
        .unregister(trimmed)
        .map_err(|error| error.to_string())
}
