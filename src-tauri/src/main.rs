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
use commands::AppState;

fn main() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_os::init())
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir().expect("Failed to get app data directory");
            std::fs::create_dir_all(&app_data_dir).expect("Failed to create app data directory");

            let db_path = app_data_dir.join("opener.db");
            let db = database::Database::new(&db_path).expect("Failed to initialize database");

            let hotkeys = db.get_all_hotkeys().unwrap_or_else(|error| {
                log::error!("Failed to load hotkeys: {}", error);
                Vec::new()
            });

            app.manage(AppState { db: std::sync::Mutex::new(db) });

            hotkeys::register_all_hotkeys(&app.handle(), &hotkeys);

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
