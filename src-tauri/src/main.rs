// Opener - Desktop Launcher Application
// Main entry point

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod database;
mod executor;
mod hotkeys;
mod models;
mod security;

use tauri::Manager;
use tauri::menu::MenuBuilder;
use tauri::tray::{MouseButton, TrayIconBuilder, TrayIconEvent};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt};
use commands::AppState;
use models::Settings;

const MAIN_WINDOW_LABEL: &str = "main";
const TRAY_SHOW_ID: &str = "tray_show";
const TRAY_HIDE_ID: &str = "tray_hide";
const TRAY_QUIT_ID: &str = "tray_quit";

struct TrayLabels {
    show: &'static str,
    hide: &'static str,
    quit: &'static str,
}

fn tray_labels(language: &str) -> TrayLabels {
    let normalized = language.to_lowercase();
    if normalized.starts_with("zh") {
        TrayLabels {
            show: "显示",
            hide: "隐藏",
            quit: "退出",
        }
    } else {
        TrayLabels {
            show: "Show",
            hide: "Hide",
            quit: "Quit",
        }
    }
}

fn show_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn hide_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
        let _ = window.hide();
    }
}

fn setup_tray(app: &tauri::AppHandle, labels: TrayLabels) -> tauri::Result<()> {
    let menu = MenuBuilder::new(app)
        .text(TRAY_SHOW_ID, labels.show)
        .text(TRAY_HIDE_ID, labels.hide)
        .separator()
        .text(TRAY_QUIT_ID, labels.quit)
        .build()?;

    let mut tray_builder = TrayIconBuilder::new()
        .menu(&menu)
        .tooltip("Opener")
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().0.as_str() {
            TRAY_SHOW_ID => show_main_window(app),
            TRAY_HIDE_ID => hide_main_window(app),
            TRAY_QUIT_ID => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click { button: MouseButton::Left, .. } = event {
                show_main_window(tray.app_handle());
            }
        });

    if let Some(icon) = app.default_window_icon().cloned() {
        tray_builder = tray_builder.icon(icon);
    }

    tray_builder.build(app)?;
    Ok(())
}

fn main() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, None))
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir().expect("Failed to get app data directory");
            std::fs::create_dir_all(&app_data_dir).expect("Failed to create app data directory");

            let db_path = app_data_dir.join("opener.db");
            let db = database::Database::new(&db_path).expect("Failed to initialize database");

            let settings = db.get_settings().unwrap_or_else(|error| {
                log::error!("Failed to load settings: {}", error);
                Settings::default()
            });
            let auto_launch = settings.auto_launch;
            let tray_label_set = tray_labels(&settings.language);

            let hotkeys = db.get_all_hotkeys().unwrap_or_else(|error| {
                log::error!("Failed to load hotkeys: {}", error);
                Vec::new()
            });

            app.manage(AppState { db: std::sync::Mutex::new(db) });

            hotkeys::register_all_hotkeys(&app.handle(), &hotkeys);

            let app_handle = app.handle();
            setup_tray(&app_handle, tray_label_set)?;
            if let Some(window) = app_handle.get_webview_window(MAIN_WINDOW_LABEL) {
                let window_clone = window.clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        let _ = window_clone.hide();
                        api.prevent_close();
                    }
                });
            }

            if let Err(error) = hotkeys::register_app_hotkey(&app_handle, &settings.app_hotkey) {
                log::warn!("Failed to register app hotkey {}: {}", settings.app_hotkey, error);
            }

            let autolaunch = app.autolaunch();
            let auto_launch_result = if auto_launch {
                autolaunch.enable()
            } else {
                autolaunch.disable()
            };
            if let Err(error) = auto_launch_result {
                log::warn!("Failed to update auto launch setting: {}", error);
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Entry commands
            commands::search_entries,
            commands::get_all_entries,
            commands::get_entry,
            commands::create_entry,
            commands::import_entries_from_yaml,
            commands::update_entry,
            commands::delete_entry,
            commands::toggle_entry,
            commands::execute_entry,
            commands::record_entry_usage,
            // Hotkey commands
            commands::get_all_hotkeys,
            commands::create_hotkey,
            commands::update_hotkey,
            commands::delete_hotkey,
            commands::check_hotkey_conflict,
            // Settings commands
            commands::get_settings,
            commands::update_settings,
            // Script template commands
            commands::get_all_templates,
            commands::get_template,
            commands::create_template,
            commands::update_template,
            commands::delete_template,
            commands::render_template,
            // Import/Export commands
            commands::export_data,
            commands::import_data,
            // Utility commands
            commands::minimize_window,
            commands::close_window,
            commands::open_file_dialog,
            commands::open_directory_dialog,
            commands::save_file_dialog,
            commands::store_secure_credential,
            commands::get_secure_credential,
            commands::delete_secure_credential,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
