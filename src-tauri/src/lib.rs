// Re-export modules for library use
pub mod commands;
pub mod database;
pub mod executor;
pub mod hotkeys;
pub mod models;
pub mod security;

use std::sync::Mutex;

pub struct AppState {
    pub db: Mutex<database::Database>,
}
