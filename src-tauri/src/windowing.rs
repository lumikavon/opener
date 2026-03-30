use tauri::{
    AppHandle, Emitter, Manager, Runtime, WebviewWindow, WebviewWindowBuilder, WindowEvent,
};

pub const MAIN_WINDOW_LABEL: &str = "main";
pub const SETTINGS_WINDOW_LABEL: &str = "settings";
pub const SETTINGS_WINDOW_CLOSED_EVENT: &str = "settings-window-closed";
const SETTINGS_WINDOW_ROLE_INIT_SCRIPT: &str = "window.__OPENER_WINDOW_ROLE__ = 'settings';";

pub fn is_settings_window(label: &str) -> bool {
    label == SETTINGS_WINDOW_LABEL
}

fn emit_settings_window_closed<R: Runtime>(app: &AppHandle<R>) {
    let _ = app.emit_to(MAIN_WINDOW_LABEL, SETTINGS_WINDOW_CLOSED_EVENT, ());
}

fn reenable_main_window<R: Runtime>(app: &AppHandle<R>) {
    if let Some(main_window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
        let _ = main_window.set_enabled(true);
        if main_window.is_visible().unwrap_or(false) {
            let _ = main_window.set_focus();
        }
    }
}

pub fn show_main_window<R: Runtime>(app: &AppHandle<R>) {
    if let Some(settings_window) = app.get_webview_window(SETTINGS_WINDOW_LABEL) {
        if settings_window.is_visible().unwrap_or(false) {
            if let Some(main_window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
                let _ = main_window.show();
                let _ = main_window.set_enabled(false);
            }
            let _ = settings_window.set_focus();
            return;
        }
    }

    if let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
        let _ = window.set_enabled(true);
        let _ = window.show();
        let _ = window.set_focus();
    }
}

pub fn close_settings_window<R: Runtime>(app: &AppHandle<R>) {
    if let Some(settings_window) = app.get_webview_window(SETTINGS_WINDOW_LABEL) {
        let _ = settings_window.hide();
    }

    reenable_main_window(app);
    emit_settings_window_closed(app);
}

pub fn hide_main_window<R: Runtime>(app: &AppHandle<R>) {
    if let Some(settings_window) = app.get_webview_window(SETTINGS_WINDOW_LABEL) {
        if settings_window.is_visible().unwrap_or(false) {
            close_settings_window(app);
        }
    }

    if let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
        let _ = window.hide();
    }
}

#[cfg(windows)]
pub fn remove_system_menu<R: Runtime>(window: &WebviewWindow<R>) {
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    use std::ptr;
    use winapi::shared::windef::HWND;
    use winapi::um::winuser::{
        GetWindowLongW, SetWindowLongW, SetWindowPos, GWL_STYLE, SWP_FRAMECHANGED, SWP_NOMOVE,
        SWP_NOOWNERZORDER, SWP_NOSIZE, SWP_NOZORDER, WS_SYSMENU,
    };

    let hwnd = match window.window_handle().ok().map(|handle| handle.as_raw()) {
        Some(RawWindowHandle::Win32(handle)) => handle.hwnd.get() as HWND,
        _ => return,
    };

    unsafe {
        let style = GetWindowLongW(hwnd, GWL_STYLE);
        if style & (WS_SYSMENU as i32) != 0 {
            SetWindowLongW(hwnd, GWL_STYLE, style & !(WS_SYSMENU as i32));
            SetWindowPos(
                hwnd,
                ptr::null_mut(),
                0,
                0,
                0,
                0,
                SWP_FRAMECHANGED | SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOOWNERZORDER,
            );
        }
    }
}

fn attach_settings_window_close_handler<R: Runtime>(app: &AppHandle<R>, window: &WebviewWindow<R>) {
    let app_handle = app.clone();
    let window_clone = window.clone();

    window.on_window_event(move |event| {
        if let WindowEvent::CloseRequested { api, .. } = event {
            let _ = window_clone.hide();
            reenable_main_window(&app_handle);
            emit_settings_window_closed(&app_handle);
            api.prevent_close();
        }
    });
}

pub fn prepare_settings_window<R: Runtime>(app: &AppHandle<R>) {
    if let Some(settings_window) = app.get_webview_window(SETTINGS_WINDOW_LABEL) {
        #[cfg(windows)]
        remove_system_menu(&settings_window);

        attach_settings_window_close_handler(app, &settings_window);
        let _ = settings_window.hide();
    }
}

pub fn create_main_window<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<WebviewWindow<R>> {
    let window_config = app
        .config()
        .app
        .windows
        .iter()
        .find(|window| window.label == MAIN_WINDOW_LABEL)
        .unwrap_or_else(|| panic!("missing {MAIN_WINDOW_LABEL} window config"));

    WebviewWindowBuilder::from_config(app, window_config)?.build()
}

pub fn create_settings_window<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<WebviewWindow<R>> {
    if let Some(settings_window) = app.get_webview_window(SETTINGS_WINDOW_LABEL) {
        return Ok(settings_window);
    }

    let window_config = app
        .config()
        .app
        .windows
        .iter()
        .find(|window| window.label == SETTINGS_WINDOW_LABEL)
        .unwrap_or_else(|| panic!("missing {SETTINGS_WINDOW_LABEL} window config"));

    let builder = WebviewWindowBuilder::from_config(app, window_config)?
        .initialization_script(SETTINGS_WINDOW_ROLE_INIT_SCRIPT);

    let builder = if let Some(main_window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
        builder.parent(&main_window)?
    } else {
        builder
    };

    builder.build()
}

pub fn open_settings_window<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    let Some(main_window) = app.get_webview_window(MAIN_WINDOW_LABEL) else {
        return Ok(());
    };

    if let Some(settings_window) = app.get_webview_window(SETTINGS_WINDOW_LABEL) {
        if let Err(error) = settings_window.show() {
            return Err(error);
        }
        if let Err(error) = settings_window.set_focus() {
            return Err(error);
        }
        if let Err(error) = main_window.set_enabled(false) {
            log::warn!("Failed to disable main window after showing settings: {}", error);
        }
        return Ok(());
    }

    let settings_window = create_settings_window(app)?;

    #[cfg(windows)]
    remove_system_menu(&settings_window);

    attach_settings_window_close_handler(app, &settings_window);
    if let Err(error) = settings_window.show() {
        return Err(error);
    }
    if let Err(error) = settings_window.set_focus() {
        let _ = settings_window.hide();
        return Err(error);
    }
    if let Err(error) = main_window.set_enabled(false) {
        log::warn!(
            "Failed to disable main window after creating settings window: {}",
            error
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{is_settings_window, SETTINGS_WINDOW_LABEL};
    use serde_json::Value;

    #[test]
    fn test_is_settings_window() {
        assert!(is_settings_window(SETTINGS_WINDOW_LABEL));
        assert!(!is_settings_window("main"));
    }

    #[test]
    fn test_startup_windows_are_created_manually_after_setup() {
        let config: Value =
            serde_json::from_str(include_str!("../tauri.conf.json")).expect("valid tauri config");
        let windows = config["app"]["windows"]
            .as_array()
            .expect("tauri app windows to be an array");

        for label in ["main", SETTINGS_WINDOW_LABEL] {
            let window = windows
                .iter()
                .find(|window| window["label"].as_str() == Some(label))
                .unwrap_or_else(|| panic!("missing {label} window config"));

            assert_eq!(
                window.get("create").and_then(Value::as_bool),
                Some(false),
                "{label} window must be created after setup so managed state exists before frontend invokes commands",
            );
        }
    }

    #[test]
    fn test_open_settings_window_shows_new_window_before_focus() {
        let source = include_str!("windowing.rs");
        let function_start = source
            .find("pub fn open_settings_window")
            .expect("windowing.rs must define open_settings_window");
        let test_module_start = source
            .find("#[cfg(test)]")
            .expect("windowing.rs must define its test module");
        let function_source = &source[function_start..test_module_start];
        let show_index = function_source
            .find("settings_window.show()")
            .expect("open_settings_window must show the settings window explicitly");
        let focus_index = function_source
            .find("settings_window.set_focus()")
            .expect("open_settings_window must focus the settings window");

        assert!(
            show_index < focus_index,
            "open_settings_window must show the settings window before focusing it",
        );
    }

    #[test]
    fn test_open_settings_window_injects_settings_role_marker() {
        let source = include_str!("windowing.rs");
        let function_start = source
            .find("pub fn create_settings_window")
            .expect("windowing.rs must define create_settings_window");
        let test_module_start = source
            .find("#[cfg(test)]")
            .expect("windowing.rs must define its test module");
        let function_source = &source[function_start..test_module_start];
        let builder_index = function_source
            .find("WebviewWindowBuilder::from_config(app, window_config)?")
            .expect("open_settings_window must build the settings window from config");
        let injection_index = function_source
            .find(".initialization_script(SETTINGS_WINDOW_ROLE_INIT_SCRIPT)")
            .expect("settings window creation must inject a role marker for frontend layout detection");

        assert!(
            builder_index < injection_index,
            "settings window creation must inject the role marker immediately after loading the window config",
        );
    }

    #[test]
    fn test_open_settings_window_disables_main_only_after_settings_focus() {
        let source = include_str!("windowing.rs");
        let function_start = source
            .find("pub fn open_settings_window")
            .expect("windowing.rs must define open_settings_window");
        let test_module_start = source
            .find("#[cfg(test)]")
            .expect("windowing.rs must define its test module");
        let function_source = &source[function_start..test_module_start];
        let first_focus_index = function_source
            .find("settings_window.set_focus()")
            .expect("open_settings_window must focus the settings window");
        let first_disable_index = function_source
            .find("main_window.set_enabled(false)")
            .expect("open_settings_window must disable the main window once settings is visible");

        assert!(
            first_focus_index < first_disable_index,
            "main window should stay enabled until an existing settings window has been shown and focused",
        );
    }

    #[test]
    fn test_app_setup_creates_settings_window_before_preparing_it() {
        let source = include_str!("main.rs");
        let create_index = source
            .find("windowing::create_settings_window(&app_handle)?;")
            .expect("main setup must create the settings window during startup");
        let prepare_index = source
            .find("windowing::prepare_settings_window(&app_handle);")
            .expect("main setup must prepare the settings window after it exists");

        assert!(
            create_index < prepare_index,
            "main setup must create the settings window before preparing it",
        );
    }
}
