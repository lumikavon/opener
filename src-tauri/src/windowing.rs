use tauri::{
    AppHandle, Emitter, Manager, Runtime, WebviewUrl, WebviewWindow, WebviewWindowBuilder,
    WindowEvent,
};

pub const MAIN_WINDOW_LABEL: &str = "main";
pub const SETTINGS_WINDOW_LABEL: &str = "settings";
pub const SETTINGS_WINDOW_CLOSED_EVENT: &str = "settings-window-closed";

const SETTINGS_WINDOW_TITLE: &str = "Settings";
const SETTINGS_WINDOW_WIDTH: f64 = 960.0;
const SETTINGS_WINDOW_HEIGHT: f64 = 680.0;
const SETTINGS_WINDOW_MIN_WIDTH: f64 = 720.0;
const SETTINGS_WINDOW_MIN_HEIGHT: f64 = 520.0;
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

pub fn open_settings_window<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    let Some(main_window) = app.get_webview_window(MAIN_WINDOW_LABEL) else {
        return Ok(());
    };

    main_window.set_enabled(false)?;

    if let Some(settings_window) = app.get_webview_window(SETTINGS_WINDOW_LABEL) {
        if let Err(error) = settings_window.show() {
            let _ = main_window.set_enabled(true);
            return Err(error);
        }
        if let Err(error) = settings_window.set_focus() {
            let _ = main_window.set_enabled(true);
            return Err(error);
        }
        return Ok(());
    }

    let builder = WebviewWindowBuilder::new(
        app,
        SETTINGS_WINDOW_LABEL,
        WebviewUrl::App("index.html".into()),
    )
    .initialization_script(SETTINGS_WINDOW_ROLE_INIT_SCRIPT)
    .title(SETTINGS_WINDOW_TITLE)
    .inner_size(SETTINGS_WINDOW_WIDTH, SETTINGS_WINDOW_HEIGHT)
    .min_inner_size(SETTINGS_WINDOW_MIN_WIDTH, SETTINGS_WINDOW_MIN_HEIGHT)
    .resizable(true)
    .maximizable(false)
    .minimizable(false)
    .decorations(false)
    .center();

    let builder = match builder.parent(&main_window) {
        Ok(builder) => builder,
        Err(error) => {
            let _ = main_window.set_enabled(true);
            return Err(error);
        }
    };

    let settings_window = match builder.build() {
        Ok(window) => window,
        Err(error) => {
            let _ = main_window.set_enabled(true);
            return Err(error);
        }
    };

    #[cfg(windows)]
    remove_system_menu(&settings_window);

    attach_settings_window_close_handler(app, &settings_window);
    if let Err(error) = settings_window.set_focus() {
        let _ = settings_window.hide();
        let _ = main_window.set_enabled(true);
        return Err(error);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{is_settings_window, SETTINGS_WINDOW_LABEL};

    #[test]
    fn test_is_settings_window() {
        assert!(is_settings_window(SETTINGS_WINDOW_LABEL));
        assert!(!is_settings_window("main"));
    }
}
