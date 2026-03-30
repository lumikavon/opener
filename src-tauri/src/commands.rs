// Tauri commands module - exposes functionality to the frontend

use crate::database::{DatabaseError, ImportResult};
use crate::hotkeys;
use crate::models::*;
use crate::windowing;
use crate::{executor, security};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
#[cfg(target_os = "windows")]
use std::path::Path;
#[cfg(target_os = "windows")]
use std::process::Command;
use std::sync::Mutex;
use tauri::{AppHandle, State, Window};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_dialog::DialogExt;

pub struct AppState {
    pub db: Mutex<crate::database::Database>,
    pub entry_editor_session: Mutex<Option<EntryEditorSession>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntryEditorMode {
    Create,
    Edit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryEditorSession {
    pub mode: EntryEditorMode,
    pub entry_id: Option<String>,
    pub opener: String,
}

#[derive(Debug, Serialize)]
pub struct CommandError {
    pub message: String,
    pub code: String,
}

impl From<DatabaseError> for CommandError {
    fn from(err: DatabaseError) -> Self {
        CommandError {
            message: err.to_string(),
            code: match err {
                DatabaseError::NotFound(_) => "NOT_FOUND".to_string(),
                DatabaseError::InvalidData(_) => "INVALID_DATA".to_string(),
                _ => "DATABASE_ERROR".to_string(),
            },
        }
    }
}

impl From<executor::ExecutorError> for CommandError {
    fn from(err: executor::ExecutorError) -> Self {
        CommandError {
            message: err.to_string(),
            code: "EXECUTOR_ERROR".to_string(),
        }
    }
}

impl From<security::SecurityError> for CommandError {
    fn from(err: security::SecurityError) -> Self {
        CommandError {
            message: err.to_string(),
            code: "SECURITY_ERROR".to_string(),
        }
    }
}

impl From<std::io::Error> for CommandError {
    fn from(err: std::io::Error) -> Self {
        CommandError {
            message: err.to_string(),
            code: "IO_ERROR".to_string(),
        }
    }
}

impl From<serde_json::Error> for CommandError {
    fn from(err: serde_json::Error) -> Self {
        CommandError {
            message: err.to_string(),
            code: "JSON_ERROR".to_string(),
        }
    }
}

impl From<serde_yaml::Error> for CommandError {
    fn from(err: serde_yaml::Error) -> Self {
        CommandError {
            message: err.to_string(),
            code: "YAML_ERROR".to_string(),
        }
    }
}

impl From<tauri_plugin_autostart::Error> for CommandError {
    fn from(err: tauri_plugin_autostart::Error) -> Self {
        CommandError {
            message: err.to_string(),
            code: "AUTOSTART_ERROR".to_string(),
        }
    }
}

pub type CommandResult<T> = Result<T, CommandError>;

fn lock_error() -> CommandError {
    CommandError {
        message: "Failed to acquire database lock".to_string(),
        code: "LOCK_ERROR".to_string(),
    }
}

// ==================== Entry Commands ====================

#[tauri::command]
pub fn search_entries(state: State<AppState>, query: String) -> CommandResult<Vec<SearchResult>> {
    let db = state.db.lock().map_err(|_| lock_error())?;

    let settings = db.get_settings()?;
    let results = db.search_entries(&query, &settings)?;
    Ok(results)
}

#[tauri::command]
pub fn get_all_entries(state: State<AppState>) -> CommandResult<Vec<Entry>> {
    let db = state.db.lock().map_err(|_| lock_error())?;
    Ok(db.get_all_entries()?)
}

#[tauri::command]
pub fn get_entry(state: State<AppState>, id: String) -> CommandResult<Entry> {
    let db = state.db.lock().map_err(|_| lock_error())?;
    Ok(db.get_entry(&id)?)
}

#[tauri::command]
pub fn create_entry(state: State<AppState>, input: CreateEntryInput) -> CommandResult<Entry> {
    let db = state.db.lock().map_err(|_| lock_error())?;
    Ok(db.create_entry(&input)?)
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum YamlEntriesInput {
    List(Vec<CreateEntryInput>),
    Wrapped { entries: Vec<CreateEntryInput> },
}

#[derive(Debug, Serialize)]
pub struct YamlImportResult {
    pub created: i32,
}

#[tauri::command]
pub fn import_entries_from_yaml(
    state: State<AppState>,
    yaml: String,
) -> CommandResult<YamlImportResult> {
    let db = state.db.lock().map_err(|_| lock_error())?;
    let parsed: YamlEntriesInput = serde_yaml::from_str(&yaml)?;

    let entries = match parsed {
        YamlEntriesInput::List(list) => list,
        YamlEntriesInput::Wrapped { entries } => entries,
    };

    let mut created = 0;
    for input in entries {
        db.create_entry(&input)?;
        created += 1;
    }

    Ok(YamlImportResult { created })
}

#[tauri::command]
pub fn update_entry(
    state: State<AppState>,
    id: String,
    input: UpdateEntryInput,
) -> CommandResult<Entry> {
    let db = state.db.lock().map_err(|_| lock_error())?;
    Ok(db.update_entry(&id, &input)?)
}

#[tauri::command]
pub fn delete_entry(state: State<AppState>, id: String) -> CommandResult<()> {
    let db = state.db.lock().map_err(|_| lock_error())?;
    db.delete_entry(&id)?;
    Ok(())
}

#[tauri::command]
pub fn toggle_entry(state: State<AppState>, id: String, enabled: bool) -> CommandResult<Entry> {
    let db = state.db.lock().map_err(|_| lock_error())?;
    Ok(db.toggle_entry(&id, enabled)?)
}

fn entry_from_input(input: CreateEntryInput) -> Entry {
    let mut entry = Entry::new(input.name, input.entry_type, input.target);
    entry.args = input.args;
    entry.workdir = input.workdir;
    entry.icon_path = input.icon_path;
    entry.tags = input.tags;
    entry.description = input.description;
    entry.enabled = input.enabled.unwrap_or(true);
    entry.confirm_before_run = input.confirm_before_run;
    entry.show_terminal = input.show_terminal;
    entry.wsl_distro = input.wsl_distro;
    entry.ssh_host = input.ssh_host;
    entry.ssh_user = input.ssh_user;
    entry.ssh_port = input.ssh_port;
    entry.ssh_key_id = input.ssh_key_id;
    entry.env_vars = input.env_vars;
    entry.hotkey_filter = input.hotkey_filter;
    entry.hotkey_position = input.hotkey_position;
    entry.hotkey_detect_hidden = input.hotkey_detect_hidden;
    entry.script_content = input.script_content;
    entry.script_type = input.script_type;
    entry
}

#[tauri::command]
pub async fn execute_entry(
    state: State<'_, AppState>,
    id: String,
    ahk_path: Option<String>,
) -> CommandResult<String> {
    let entry = {
        let db = state.db.lock().map_err(|_| lock_error())?;
        db.get_entry(&id)?
    };

    let resolved = executor::resolve_entry_env(&entry);
    let preview = executor::get_execution_preview(&resolved);
    let execution_result = tauri::async_runtime::spawn_blocking(move || {
        executor::execute_entry(&resolved, ahk_path.as_deref())
    })
    .await
    .map_err(|error| CommandError {
        message: format!("Execution task failed: {}", error),
        code: "EXECUTOR_ERROR".to_string(),
    })?;
    execution_result?;

    // Record usage (re-acquire lock)
    let db = state.db.lock().map_err(|_| lock_error())?;
    let _ = db.record_entry_usage(&id);

    Ok(preview)
}

#[tauri::command]
pub async fn execute_entry_input(
    input: CreateEntryInput,
    ahk_path: Option<String>,
) -> CommandResult<String> {
    let entry = entry_from_input(input);
    let resolved = executor::resolve_entry_env(&entry);
    let preview = executor::get_execution_preview(&resolved);
    let execution_result = tauri::async_runtime::spawn_blocking(move || {
        executor::execute_entry(&resolved, ahk_path.as_deref())
    })
    .await
    .map_err(|error| CommandError {
        message: format!("Execution task failed: {}", error),
        code: "EXECUTOR_ERROR".to_string(),
    })?;
    execution_result?;
    Ok(preview)
}

#[tauri::command]
pub fn record_entry_usage(state: State<AppState>, id: String) -> CommandResult<()> {
    let db = state.db.lock().map_err(|_| lock_error())?;
    db.record_entry_usage(&id)?;
    Ok(())
}

// ==================== Hotkey Commands ====================

#[tauri::command]
pub fn get_all_hotkeys(state: State<AppState>) -> CommandResult<Vec<Hotkey>> {
    let db = state.db.lock().map_err(|_| lock_error())?;
    Ok(db.get_all_hotkeys()?)
}

#[tauri::command]
pub fn create_hotkey(
    app: AppHandle,
    state: State<AppState>,
    entry_id: String,
    accelerator: String,
    _scope: String,
) -> CommandResult<Hotkey> {
    let db = state.db.lock().map_err(|_| lock_error())?;

    let scope_enum = HotkeyScope::Global;

    // Check for conflicts
    if let Some(conflict) = db.check_hotkey_conflict(&accelerator, &scope_enum, None)? {
        return Err(CommandError {
            message: format!("Hotkey conflict with entry: {}", conflict.entry_id),
            code: "HOTKEY_CONFLICT".to_string(),
        });
    }

    let hotkey = db.create_hotkey(&entry_id, &accelerator, scope_enum)?;
    drop(db);

    if let Err(error) = hotkeys::register_hotkey(&app, &hotkey) {
        let db = state.db.lock().map_err(|_| lock_error())?;
        let _ = db.delete_hotkey(&hotkey.id);
        return Err(CommandError {
            message: error,
            code: "HOTKEY_REGISTER_FAILED".to_string(),
        });
    }

    Ok(hotkey)
}

#[tauri::command]
pub fn update_hotkey(
    app: AppHandle,
    state: State<AppState>,
    id: String,
    accelerator: Option<String>,
    scope: Option<String>,
    enabled: Option<bool>,
) -> CommandResult<Hotkey> {
    let db = state.db.lock().map_err(|_| lock_error())?;

    let scope_enum = scope.as_ref().and_then(|s| HotkeyScope::from_str(s));

    // Check for conflicts if accelerator or scope changed
    if accelerator.is_some() || scope_enum.is_some() {
        let current = db.get_hotkey(&id)?;
        let new_accelerator = accelerator.as_ref().unwrap_or(&current.accelerator);
        let new_scope = scope_enum.clone().unwrap_or(current.scope);

        if let Some(conflict) = db.check_hotkey_conflict(new_accelerator, &new_scope, Some(&id))? {
            return Err(CommandError {
                message: format!("Hotkey conflict with entry: {}", conflict.entry_id),
                code: "HOTKEY_CONFLICT".to_string(),
            });
        }
    }

    let current = db.get_hotkey(&id)?;
    let updated = db.update_hotkey(&id, accelerator.as_deref(), scope_enum.clone(), enabled)?;
    drop(db);

    let accelerator_changed = accelerator
        .as_ref()
        .map(|value| value != &current.accelerator)
        .unwrap_or(false);
    let scope_changed = scope_enum
        .as_ref()
        .map(|value| *value != current.scope)
        .unwrap_or(false);
    let enabled_changed = enabled
        .map(|value| value != current.enabled)
        .unwrap_or(false);

    if accelerator_changed || scope_changed || enabled_changed {
        if let Err(error) = hotkeys::unregister_hotkey(&app, &current) {
            log::warn!(
                "Failed to unregister hotkey {}: {}",
                current.accelerator,
                error
            );
        }

        if let Err(error) = hotkeys::register_hotkey(&app, &updated) {
            return Err(CommandError {
                message: error,
                code: "HOTKEY_REGISTER_FAILED".to_string(),
            });
        }
    }

    Ok(updated)
}

#[tauri::command]
pub fn delete_hotkey(app: AppHandle, state: State<AppState>, id: String) -> CommandResult<()> {
    let db = state.db.lock().map_err(|_| lock_error())?;
    let hotkey = db.get_hotkey(&id)?;
    db.delete_hotkey(&id)?;
    drop(db);

    if let Err(error) = hotkeys::unregister_hotkey(&app, &hotkey) {
        log::warn!(
            "Failed to unregister hotkey {}: {}",
            hotkey.accelerator,
            error
        );
    }
    Ok(())
}

#[derive(Serialize)]
pub struct ConflictCheckResult {
    pub has_conflict: bool,
    pub conflicting_hotkey: Option<Hotkey>,
}

#[tauri::command]
pub fn check_hotkey_conflict(
    state: State<AppState>,
    accelerator: String,
    scope: String,
    exclude_id: Option<String>,
) -> CommandResult<ConflictCheckResult> {
    let db = state.db.lock().map_err(|_| lock_error())?;

    let scope_enum = HotkeyScope::from_str(&scope).unwrap_or(HotkeyScope::App);
    let conflict = db.check_hotkey_conflict(&accelerator, &scope_enum, exclude_id.as_deref())?;

    Ok(ConflictCheckResult {
        has_conflict: conflict.is_some(),
        conflicting_hotkey: conflict,
    })
}

// ==================== Settings Commands ====================

#[tauri::command]
pub fn get_settings(state: State<AppState>) -> CommandResult<Settings> {
    let db = state.db.lock().map_err(|_| lock_error())?;
    Ok(db.get_settings()?)
}

#[tauri::command]
pub fn update_settings(
    app: AppHandle,
    state: State<AppState>,
    settings: Settings,
) -> CommandResult<Settings> {
    let mut settings = settings;
    settings.app_hotkey = settings.app_hotkey.trim().to_string();

    let previous = {
        let db = state.db.lock().map_err(|_| lock_error())?;
        db.get_settings()?
    };

    if previous.auto_launch != settings.auto_launch {
        if settings.auto_launch {
            app.autolaunch().enable()?;
        } else {
            app.autolaunch().disable()?;
        }
    }

    let previous_app_hotkey = previous.app_hotkey.trim().to_string();
    let next_app_hotkey = settings.app_hotkey.trim().to_string();
    if previous_app_hotkey != next_app_hotkey {
        if !previous_app_hotkey.is_empty() {
            let _ = hotkeys::unregister_app_hotkey(&app, &previous_app_hotkey);
        }

        if !next_app_hotkey.is_empty() {
            if let Err(error) = hotkeys::register_app_hotkey(&app, &next_app_hotkey) {
                if !previous_app_hotkey.is_empty() {
                    let _ = hotkeys::register_app_hotkey(&app, &previous_app_hotkey);
                }
                return Err(CommandError {
                    message: error,
                    code: "HOTKEY_REGISTER_FAILED".to_string(),
                });
            }
        }
    }

    let db = state.db.lock().map_err(|_| lock_error())?;
    db.save_settings(&settings)?;
    Ok(db.get_settings()?)
}

#[tauri::command]
pub fn open_settings_window(app: AppHandle) -> CommandResult<()> {
    windowing::open_settings_window(&app).map_err(|error| CommandError {
        message: error.to_string(),
        code: "WINDOW_ERROR".to_string(),
    })?;
    Ok(())
}

#[tauri::command]
pub fn open_entry_editor_create(
    app: AppHandle,
    state: State<AppState>,
    window: Window,
) -> CommandResult<()> {
    let mut session = state.entry_editor_session.lock().map_err(|_| lock_error())?;
    *session = Some(EntryEditorSession {
        mode: EntryEditorMode::Create,
        entry_id: None,
        opener: window.label().to_string(),
    });
    drop(session);

    windowing::open_entry_editor_window(&app).map_err(|error| CommandError {
        message: error.to_string(),
        code: "WINDOW_ERROR".to_string(),
    })?;
    Ok(())
}

#[tauri::command]
pub fn open_entry_editor_edit(
    app: AppHandle,
    state: State<AppState>,
    window: Window,
    id: String,
) -> CommandResult<()> {
    let mut session = state.entry_editor_session.lock().map_err(|_| lock_error())?;
    *session = Some(EntryEditorSession {
        mode: EntryEditorMode::Edit,
        entry_id: Some(id),
        opener: window.label().to_string(),
    });
    drop(session);

    windowing::open_entry_editor_window(&app).map_err(|error| CommandError {
        message: error.to_string(),
        code: "WINDOW_ERROR".to_string(),
    })?;
    Ok(())
}

#[tauri::command]
pub fn get_entry_editor_context(state: State<AppState>) -> CommandResult<Option<EntryEditorSession>> {
    let session = state.entry_editor_session.lock().map_err(|_| lock_error())?;
    Ok(session.clone())
}

#[tauri::command]
pub fn notify_entry_editor_saved(app: AppHandle) -> CommandResult<()> {
    windowing::emit_entry_editor_saved(&app);
    Ok(())
}

// ==================== Template Commands ====================

#[tauri::command]
pub fn get_all_templates(state: State<AppState>) -> CommandResult<Vec<ScriptTemplate>> {
    let db = state.db.lock().map_err(|_| lock_error())?;
    Ok(db.get_all_templates()?)
}

#[tauri::command]
pub fn get_template(state: State<AppState>, id: String) -> CommandResult<ScriptTemplate> {
    let db = state.db.lock().map_err(|_| lock_error())?;
    Ok(db.get_template(&id)?)
}

#[derive(Deserialize)]
pub struct CreateTemplateInput {
    pub name: String,
    pub description: Option<String>,
    pub language: String,
    pub template_content: String,
    pub variables: Vec<VariableDefinition>,
}

#[tauri::command]
pub fn create_template(
    state: State<AppState>,
    input: CreateTemplateInput,
) -> CommandResult<ScriptTemplate> {
    let db = state.db.lock().map_err(|_| lock_error())?;

    let mut template = ScriptTemplate::new(input.name, input.language, input.template_content);
    template.description = input.description;
    template.variables = input.variables;

    Ok(db.create_template(&template)?)
}

#[derive(Deserialize)]
pub struct UpdateTemplateInput {
    pub name: Option<String>,
    pub description: Option<String>,
    pub language: Option<String>,
    pub template_content: Option<String>,
    pub variables: Option<Vec<VariableDefinition>>,
}

#[tauri::command]
pub fn update_template(
    state: State<AppState>,
    id: String,
    input: UpdateTemplateInput,
) -> CommandResult<ScriptTemplate> {
    let db = state.db.lock().map_err(|_| lock_error())?;

    Ok(db.update_template(
        &id,
        input.name.as_deref(),
        input.description.as_deref(),
        input.language.as_deref(),
        input.template_content.as_deref(),
        input.variables.as_ref(),
    )?)
}

#[tauri::command]
pub fn delete_template(state: State<AppState>, id: String) -> CommandResult<()> {
    let db = state.db.lock().map_err(|_| lock_error())?;
    db.delete_template(&id)?;
    Ok(())
}

#[tauri::command]
pub fn render_template(
    state: State<AppState>,
    id: String,
    variables: HashMap<String, String>,
) -> CommandResult<String> {
    let db = state.db.lock().map_err(|_| lock_error())?;

    let template = db.get_template(&id)?;
    let rendered = executor::render_template(&template.template_content, &variables);
    Ok(rendered)
}

// ==================== Import/Export Commands ====================

#[tauri::command]
pub fn export_data(state: State<AppState>) -> CommandResult<String> {
    let db = state.db.lock().map_err(|_| lock_error())?;

    let export_data = db.export_all()?;
    let json = serde_json::to_string_pretty(&export_data)?;
    Ok(json)
}

#[derive(Deserialize)]
pub struct ImportDataInput {
    pub json_data: String,
    pub strategy: String,
}

#[tauri::command]
pub fn import_data(state: State<AppState>, input: ImportDataInput) -> CommandResult<ImportResult> {
    let db = state.db.lock().map_err(|_| lock_error())?;

    let data: ExportData = serde_json::from_str(&input.json_data).map_err(|e| CommandError {
        message: format!("Invalid JSON format: {}", e),
        code: "INVALID_FORMAT".to_string(),
    })?;

    // Check schema version
    if data.schema_version != ExportData::CURRENT_SCHEMA_VERSION {
        log::warn!(
            "Import schema version mismatch: expected {}, got {}",
            ExportData::CURRENT_SCHEMA_VERSION,
            data.schema_version
        );
    }

    let strategy = match input.strategy.as_str() {
        "overwrite_all" => ImportStrategy::OverwriteAll,
        "add_only" => ImportStrategy::AddOnly,
        "merge_by_name" => ImportStrategy::MergeByName,
        _ => ImportStrategy::AddOnly,
    };

    Ok(db.import_data(&data, strategy)?)
}

// ==================== Window Commands ====================

#[tauri::command]
pub fn minimize_window(window: Window) -> CommandResult<()> {
    window.hide().map_err(|e| CommandError {
        message: e.to_string(),
        code: "WINDOW_ERROR".to_string(),
    })?;
    Ok(())
}

#[tauri::command]
pub fn toggle_maximize_window(window: Window) -> CommandResult<()> {
    let is_maximized = window.is_maximized().map_err(|e| CommandError {
        message: e.to_string(),
        code: "WINDOW_ERROR".to_string(),
    })?;

    if is_maximized {
        window.unmaximize().map_err(|e| CommandError {
            message: e.to_string(),
            code: "WINDOW_ERROR".to_string(),
        })?;
    } else {
        window.maximize().map_err(|e| CommandError {
            message: e.to_string(),
            code: "WINDOW_ERROR".to_string(),
        })?;
    }

    Ok(())
}

#[tauri::command]
pub fn close_window(app: AppHandle, state: State<AppState>, window: Window) -> CommandResult<()> {
    if windowing::is_settings_window(window.label()) {
        windowing::close_settings_window(&app);
        return Ok(());
    }

    if windowing::is_entry_editor_window(window.label()) {
        let mut session = state.entry_editor_session.lock().map_err(|_| lock_error())?;
        *session = None;
        drop(session);

        windowing::close_entry_editor_window(&app);
        return Ok(());
    }

    window.hide().map_err(|error| CommandError {
        message: error.to_string(),
        code: "WINDOW_ERROR".to_string(),
    })?;
    Ok(())
}

// ==================== Dialog Commands ====================

#[tauri::command]
pub async fn open_file_dialog(app: tauri::AppHandle) -> CommandResult<Option<String>> {
    let file = app.dialog().file().blocking_pick_file();

    Ok(file.map(|f| f.to_string()))
}

#[tauri::command]
pub async fn open_directory_dialog(app: tauri::AppHandle) -> CommandResult<Option<String>> {
    let dir = app.dialog().file().blocking_pick_folder();

    Ok(dir.map(|d| d.to_string()))
}

#[tauri::command]
pub async fn save_file_dialog(
    app: tauri::AppHandle,
    default_name: String,
) -> CommandResult<Option<String>> {
    let file = app
        .dialog()
        .file()
        .set_file_name(&default_name)
        .blocking_save_file();

    Ok(file.map(|f| f.to_string()))
}

// ==================== File I/O Commands ====================

#[tauri::command]
pub fn write_text_file(path: String, contents: String) -> CommandResult<()> {
    std::fs::write(&path, &contents).map_err(|e| CommandError {
        message: format!("Failed to write file {}: {}", path, e),
        code: "WRITE_FILE_ERROR".to_string(),
    })
}

#[tauri::command]
pub fn read_text_file(path: String) -> CommandResult<String> {
    std::fs::read_to_string(&path).map_err(|e| CommandError {
        message: format!("Failed to read file {}: {}", path, e),
        code: "READ_FILE_ERROR".to_string(),
    })
}

// ==================== Window Spy ====================

#[tauri::command]
pub fn open_window_spy() -> CommandResult<()> {
    #[cfg(target_os = "windows")]
    {
        let ahk_exe = Path::new(r"C:\Program Files\AutoHotkey\v2\AutoHotkey.exe");
        let window_spy = Path::new(r"C:\Program Files\AutoHotkey\WindowSpy.ahk");

        if !ahk_exe.exists() {
            return Err(CommandError {
                message: format!("AutoHotkey.exe not found at {}", ahk_exe.display()),
                code: "WINDOW_SPY_MISSING".to_string(),
            });
        }

        if !window_spy.exists() {
            return Err(CommandError {
                message: format!("WindowSpy.ahk not found at {}", window_spy.display()),
                code: "WINDOW_SPY_MISSING".to_string(),
            });
        }

        Command::new(ahk_exe).arg(window_spy).spawn()?;
        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    {
        Err(CommandError {
            message: "WindowSpy is only available on Windows.".to_string(),
            code: "UNSUPPORTED_OS".to_string(),
        })
    }
}

// ==================== Security Commands ====================

#[tauri::command]
pub fn store_secure_credential(key: String, value: String) -> CommandResult<()> {
    security::store_credential(&key, &value)?;
    Ok(())
}

#[tauri::command]
pub fn get_secure_credential(key: String) -> CommandResult<String> {
    Ok(security::get_credential(&key)?)
}

#[tauri::command]
pub fn delete_secure_credential(key: String) -> CommandResult<()> {
    security::delete_credential(&key)?;
    Ok(())
}
