// Database layer for Opener

use crate::models::*;
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[allow(dead_code)]
    #[error("Invalid data: {0}")]
    InvalidData(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub type DbResult<T> = Result<T, DatabaseError>;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(path: &Path) -> DbResult<Self> {
        let conn = Connection::open(path)?;
        let db = Self { conn };
        db.init_schema()?;
        db.seed_default_data()?;
        Ok(db)
    }

    /// Initialize database schema
    fn init_schema(&self) -> DbResult<()> {
        self.conn.execute_batch(
            r#"
            -- Entries table
            CREATE TABLE IF NOT EXISTS entries (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                type TEXT NOT NULL,
                target TEXT NOT NULL,
                args TEXT,
                workdir TEXT,
                icon_path TEXT,
                tags TEXT,
                description TEXT,
                enabled INTEGER NOT NULL DEFAULT 1,
                confirm_before_run INTEGER,
                show_terminal INTEGER,
                wsl_distro TEXT,
                ssh_host TEXT,
                ssh_user TEXT,
                ssh_port INTEGER,
                ssh_key_id TEXT,
                env_vars TEXT,
                hotkey_filter TEXT,
                hotkey_position TEXT,
                hotkey_detect_hidden INTEGER,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                last_used_at TEXT,
                use_count INTEGER NOT NULL DEFAULT 0
            );

            -- Hotkeys table
            CREATE TABLE IF NOT EXISTS hotkeys (
                id TEXT PRIMARY KEY,
                entry_id TEXT NOT NULL,
                accelerator TEXT NOT NULL,
                scope TEXT NOT NULL DEFAULT 'app',
                enabled INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (entry_id) REFERENCES entries(id) ON DELETE CASCADE
            );

            -- Settings table
            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            -- Script templates table
            CREATE TABLE IF NOT EXISTS script_templates (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                language TEXT NOT NULL,
                template_content TEXT NOT NULL,
                variables_json TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            -- Indexes for faster search
            CREATE INDEX IF NOT EXISTS idx_entries_name ON entries(name);
            CREATE INDEX IF NOT EXISTS idx_entries_target ON entries(target);
            CREATE INDEX IF NOT EXISTS idx_entries_enabled ON entries(enabled);
            CREATE INDEX IF NOT EXISTS idx_entries_type ON entries(type);
            CREATE INDEX IF NOT EXISTS idx_hotkeys_entry_id ON hotkeys(entry_id);
            CREATE INDEX IF NOT EXISTS idx_hotkeys_accelerator ON hotkeys(accelerator);
            "#,
        )?;
        self.ensure_entry_columns()?;
        Ok(())
    }

    fn ensure_entry_columns(&self) -> DbResult<()> {
        let mut stmt = self.conn.prepare("PRAGMA table_info(entries)")?;
        let existing: std::collections::HashSet<String> = stmt
            .query_map([], |row| row.get::<_, String>(1))?
            .collect::<Result<_, _>>()?;

        let columns = [
            ("hotkey_filter", "TEXT"),
            ("hotkey_position", "TEXT"),
            ("hotkey_detect_hidden", "INTEGER"),
        ];

        for (name, column_type) in columns {
            if !existing.contains(name) {
                let sql = format!("ALTER TABLE entries ADD COLUMN {} {}", name, column_type);
                self.conn.execute(&sql, [])?;
            }
        }

        Ok(())
    }

    /// Seed default data if tables are empty
    fn seed_default_data(&self) -> DbResult<()> {
        // Check if settings exist
        let settings_count: i32 = self.conn.query_row(
            "SELECT COUNT(*) FROM settings",
            [],
            |row| row.get(0),
        )?;

        if settings_count == 0 {
            let default_settings = Settings::default();
            self.save_settings(&default_settings)?;
        }

        // Check if templates exist
        let template_count: i32 = self.conn.query_row(
            "SELECT COUNT(*) FROM script_templates",
            [],
            |row| row.get(0),
        )?;

        if template_count == 0 {
            self.seed_default_templates()?;
        }

        self.ensure_hotkey_template()?;

        Ok(())
    }

    /// Seed default script templates
    fn seed_default_templates(&self) -> DbResult<()> {
        let templates = vec![
            ScriptTemplate {
                id: uuid::Uuid::new_v4().to_string(),
                name: "Open Project".to_string(),
                description: Some("Open a project directory in terminal".to_string()),
                language: "powershell".to_string(),
                template_content: "cd \"{{path}}\"\n{{command}}".to_string(),
                variables: vec![
                    VariableDefinition {
                        name: "path".to_string(),
                        var_type: VariableType::Path,
                        label: "Project Path".to_string(),
                        default_value: None,
                        required: true,
                        choices: None,
                        validation_regex: None,
                    },
                    VariableDefinition {
                        name: "command".to_string(),
                        var_type: VariableType::String,
                        label: "Command to run".to_string(),
                        default_value: Some("code .".to_string()),
                        required: false,
                        choices: None,
                        validation_regex: None,
                    },
                ],
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            ScriptTemplate {
                id: uuid::Uuid::new_v4().to_string(),
                name: "Git Pull".to_string(),
                description: Some("Pull latest changes from a git repository".to_string()),
                language: "shell".to_string(),
                template_content: "cd \"{{repo_path}}\" && git checkout {{branch}} && git pull origin {{branch}}".to_string(),
                variables: vec![
                    VariableDefinition {
                        name: "repo_path".to_string(),
                        var_type: VariableType::Path,
                        label: "Repository Path".to_string(),
                        default_value: None,
                        required: true,
                        choices: None,
                        validation_regex: None,
                    },
                    VariableDefinition {
                        name: "branch".to_string(),
                        var_type: VariableType::String,
                        label: "Branch".to_string(),
                        default_value: Some("main".to_string()),
                        required: true,
                        choices: None,
                        validation_regex: None,
                    },
                ],
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            ScriptTemplate {
                id: uuid::Uuid::new_v4().to_string(),
                name: "Start Dev Server".to_string(),
                description: Some("Start a development server".to_string()),
                language: "shell".to_string(),
                template_content: "cd \"{{workdir}}\" && {{package_manager}} run dev --port {{port}}".to_string(),
                variables: vec![
                    VariableDefinition {
                        name: "workdir".to_string(),
                        var_type: VariableType::Path,
                        label: "Working Directory".to_string(),
                        default_value: None,
                        required: true,
                        choices: None,
                        validation_regex: None,
                    },
                    VariableDefinition {
                        name: "port".to_string(),
                        var_type: VariableType::Number,
                        label: "Port".to_string(),
                        default_value: Some("3000".to_string()),
                        required: true,
                        choices: None,
                        validation_regex: Some(r"^\d+$".to_string()),
                    },
                    VariableDefinition {
                        name: "package_manager".to_string(),
                        var_type: VariableType::Choice,
                        label: "Package Manager".to_string(),
                        default_value: Some("npm".to_string()),
                        required: true,
                        choices: Some(vec!["npm".to_string(), "yarn".to_string(), "pnpm".to_string(), "bun".to_string()]),
                        validation_regex: None,
                    },
                ],
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            ScriptTemplate {
                id: uuid::Uuid::new_v4().to_string(),
                name: "SSH Connect".to_string(),
                description: Some("Connect to a remote server via SSH".to_string()),
                language: "ssh".to_string(),
                template_content: "ssh {{user}}@{{host}} -p {{port}}".to_string(),
                variables: vec![
                    VariableDefinition {
                        name: "host".to_string(),
                        var_type: VariableType::String,
                        label: "Host".to_string(),
                        default_value: None,
                        required: true,
                        choices: None,
                        validation_regex: None,
                    },
                    VariableDefinition {
                        name: "user".to_string(),
                        var_type: VariableType::String,
                        label: "Username".to_string(),
                        default_value: Some("root".to_string()),
                        required: true,
                        choices: None,
                        validation_regex: None,
                    },
                    VariableDefinition {
                        name: "port".to_string(),
                        var_type: VariableType::Number,
                        label: "Port".to_string(),
                        default_value: Some("22".to_string()),
                        required: true,
                        choices: None,
                        validation_regex: Some(r"^\d+$".to_string()),
                    },
                ],
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            ScriptTemplate {
                id: uuid::Uuid::new_v4().to_string(),
                name: "WSL Execute".to_string(),
                description: Some("Execute a command in WSL".to_string()),
                language: "wsl".to_string(),
                template_content: "wsl -d {{distro}} -- {{command}}".to_string(),
                variables: vec![
                    VariableDefinition {
                        name: "distro".to_string(),
                        var_type: VariableType::Choice,
                        label: "Distribution".to_string(),
                        default_value: Some("Ubuntu".to_string()),
                        required: true,
                        choices: Some(vec!["Ubuntu".to_string(), "Debian".to_string(), "openSUSE-Leap".to_string(), "kali-linux".to_string()]),
                        validation_regex: None,
                    },
                    VariableDefinition {
                        name: "command".to_string(),
                        var_type: VariableType::String,
                        label: "Command".to_string(),
                        default_value: None,
                        required: true,
                        choices: None,
                        validation_regex: None,
                    },
                ],
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            Self::hotkey_app_template(),
        ];

        for template in templates {
            self.create_template(&template)?;
        }

        Ok(())
    }

    fn ensure_hotkey_template(&self) -> DbResult<()> {
        let count: i32 = self.conn.query_row(
            "SELECT COUNT(*) FROM script_templates WHERE name = ?1 AND language = ?2",
            params!["hotkey应用", "ahk"],
            |row| row.get(0),
        )?;

        if count == 0 {
            let template = Self::hotkey_app_template();
            self.create_template(&template)?;
        }

        Ok(())
    }

    fn hotkey_app_template() -> ScriptTemplate {
        ScriptTemplate {
            id: uuid::Uuid::new_v4().to_string(),
            name: "hotkey应用".to_string(),
            description: Some("通过快捷键启动或聚焦应用（AutoHotkey）".to_string()),
            language: "ahk".to_string(),
            template_content: r#"#Requires AutoHotkey v2.0
#SingleInstance Force
SetTitleMatchMode 2

ShowTrayTip(title, message) {
    TrayTip(message, title)
}

ApplyPosition(filter, position, left, top, width, height) {
    if (position = "left") {
        try WinMove left, top, width / 2, height, filter
    } else if (position = "right") {
        try WinMove left + width / 2, top, width / 2, height, filter
    } else if (position = "max") {
        try WinMaximize filter
    }
}

DoRunApp(name, executable, workdir, filter, position := "max", detectHidden := true) {
    prevDetect := A_DetectHiddenWindows
    DetectHiddenWindows detectHidden

    SplitPath executable, &exeName
    if (!filter) {
        filter := "ahk_exe " . exeName
    }

    MonitorGetWorkArea(&left, &top, &right, &bottom)
    workWidth := right - left
    workHeight := bottom - top

    try {
        if WinExist(filter) {
            WinRestore filter
            if WinWait(filter, , 15) {
                WinActivate filter
            } else {
                ShowTrayTip "错误", "打开失败: " . executable
                return
            }
            ApplyPosition filter, position, left, top, workWidth, workHeight
            ShowTrayTip "聚焦应用", "聚焦应用: " . name
            return
        }

        Run(executable, workdir)
        if WinWait(filter, , 15) {
            try WinActivate filter
        } else {
            ShowTrayTip "错误", "打开失败: " . executable
            return
        }

        ApplyPosition filter, position, left, top, workWidth, workHeight
        ShowTrayTip "打开应用", "打开应用: " . name
    } finally {
        DetectHiddenWindows prevDetect
    }
}

{{hotkey}}::DoRunApp("{{app_name}}", "{{executable}}", "{{workdir}}", "{{filter}}", "{{position}}", {{detect_hidden}})
"#.to_string(),
            variables: vec![
                VariableDefinition {
                    name: "hotkey".to_string(),
                    var_type: VariableType::String,
                    label: "快捷键".to_string(),
                    default_value: Some("^!1".to_string()),
                    required: true,
                    choices: None,
                    validation_regex: None,
                },
                VariableDefinition {
                    name: "app_name".to_string(),
                    var_type: VariableType::String,
                    label: "应用名称".to_string(),
                    default_value: None,
                    required: true,
                    choices: None,
                    validation_regex: None,
                },
                VariableDefinition {
                    name: "executable".to_string(),
                    var_type: VariableType::Path,
                    label: "可执行文件".to_string(),
                    default_value: None,
                    required: true,
                    choices: None,
                    validation_regex: None,
                },
                VariableDefinition {
                    name: "workdir".to_string(),
                    var_type: VariableType::Path,
                    label: "工作目录".to_string(),
                    default_value: None,
                    required: false,
                    choices: None,
                    validation_regex: None,
                },
                VariableDefinition {
                    name: "filter".to_string(),
                    var_type: VariableType::String,
                    label: "窗口匹配 (WinTitle)".to_string(),
                    default_value: None,
                    required: false,
                    choices: None,
                    validation_regex: None,
                },
                VariableDefinition {
                    name: "position".to_string(),
                    var_type: VariableType::Choice,
                    label: "窗口位置".to_string(),
                    default_value: Some("max".to_string()),
                    required: true,
                    choices: Some(vec![
                        "left".to_string(),
                        "right".to_string(),
                        "max".to_string(),
                    ]),
                    validation_regex: None,
                },
                VariableDefinition {
                    name: "detect_hidden".to_string(),
                    var_type: VariableType::Boolean,
                    label: "检测隐藏窗口".to_string(),
                    default_value: Some("true".to_string()),
                    required: false,
                    choices: None,
                    validation_regex: None,
                },
            ],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    // ==================== Entry CRUD ====================

    pub fn get_all_entries(&self) -> DbResult<Vec<Entry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, type, target, args, workdir, icon_path, tags, description,
                    enabled, confirm_before_run, show_terminal, wsl_distro, ssh_host,
                    ssh_user, ssh_port, ssh_key_id, env_vars, hotkey_filter,
                    hotkey_position, hotkey_detect_hidden, created_at, updated_at,
                    last_used_at, use_count
             FROM entries ORDER BY name"
        )?;

        let entries = stmt.query_map([], |row| self.row_to_entry(row))?;
        entries.collect::<Result<Vec<_>, _>>().map_err(DatabaseError::from)
    }

    pub fn get_entry(&self, id: &str) -> DbResult<Entry> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, type, target, args, workdir, icon_path, tags, description,
                    enabled, confirm_before_run, show_terminal, wsl_distro, ssh_host,
                    ssh_user, ssh_port, ssh_key_id, env_vars, hotkey_filter,
                    hotkey_position, hotkey_detect_hidden, created_at, updated_at,
                    last_used_at, use_count
             FROM entries WHERE id = ?"
        )?;

        stmt.query_row([id], |row| self.row_to_entry(row))
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => DatabaseError::NotFound(format!("Entry with id {} not found", id)),
                _ => DatabaseError::from(e),
            })
    }

    pub fn create_entry(&self, input: &CreateEntryInput) -> DbResult<Entry> {
        let now = Utc::now();
        let id = uuid::Uuid::new_v4().to_string();
        let entry_type_str = input.entry_type.as_str();
        let now_str = now.to_rfc3339();

        self.conn.execute(
            "INSERT INTO entries (id, name, type, target, args, workdir, icon_path, tags,
                                  description, enabled, confirm_before_run, show_terminal,
                                  wsl_distro, ssh_host, ssh_user, ssh_port, ssh_key_id,
                                  env_vars, hotkey_filter, hotkey_position, hotkey_detect_hidden,
                                  created_at, updated_at, use_count)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, 0)",
            params![
                id,
                input.name,
                entry_type_str,
                input.target,
                input.args,
                input.workdir,
                input.icon_path,
                input.tags,
                input.description,
                input.enabled.unwrap_or(true),
                input.confirm_before_run,
                input.show_terminal,
                input.wsl_distro,
                input.ssh_host,
                input.ssh_user,
                input.ssh_port,
                input.ssh_key_id,
                input.env_vars,
                input.hotkey_filter,
                input.hotkey_position,
                input.hotkey_detect_hidden,
                now_str,
                now_str,
            ],
        )?;

        self.get_entry(&id)
    }

    pub fn update_entry(&self, id: &str, input: &UpdateEntryInput) -> DbResult<Entry> {
        // First check if entry exists
        let _existing = self.get_entry(id)?;
        let now = Utc::now().to_rfc3339();

        // Build dynamic update query
        let mut updates = Vec::new();
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(ref name) = input.name {
            updates.push("name = ?");
            params_vec.push(Box::new(name.clone()));
        }
        if let Some(ref entry_type) = input.entry_type {
            updates.push("type = ?");
            params_vec.push(Box::new(entry_type.as_str().to_string()));
        }
        if let Some(ref target) = input.target {
            updates.push("target = ?");
            params_vec.push(Box::new(target.clone()));
        }
        if let Some(ref args) = input.args {
            updates.push("args = ?");
            params_vec.push(Box::new(args.clone()));
        }
        if let Some(ref workdir) = input.workdir {
            updates.push("workdir = ?");
            params_vec.push(Box::new(workdir.clone()));
        }
        if let Some(ref icon_path) = input.icon_path {
            updates.push("icon_path = ?");
            params_vec.push(Box::new(icon_path.clone()));
        }
        if let Some(ref tags) = input.tags {
            updates.push("tags = ?");
            params_vec.push(Box::new(tags.clone()));
        }
        if let Some(ref description) = input.description {
            updates.push("description = ?");
            params_vec.push(Box::new(description.clone()));
        }
        if let Some(enabled) = input.enabled {
            updates.push("enabled = ?");
            params_vec.push(Box::new(enabled));
        }
        if let Some(confirm) = input.confirm_before_run {
            updates.push("confirm_before_run = ?");
            params_vec.push(Box::new(confirm));
        }
        if let Some(show_terminal) = input.show_terminal {
            updates.push("show_terminal = ?");
            params_vec.push(Box::new(show_terminal));
        }
        if let Some(ref wsl_distro) = input.wsl_distro {
            updates.push("wsl_distro = ?");
            params_vec.push(Box::new(wsl_distro.clone()));
        }
        if let Some(ref ssh_host) = input.ssh_host {
            updates.push("ssh_host = ?");
            params_vec.push(Box::new(ssh_host.clone()));
        }
        if let Some(ref ssh_user) = input.ssh_user {
            updates.push("ssh_user = ?");
            params_vec.push(Box::new(ssh_user.clone()));
        }
        if let Some(ssh_port) = input.ssh_port {
            updates.push("ssh_port = ?");
            params_vec.push(Box::new(ssh_port));
        }
        if let Some(ref ssh_key_id) = input.ssh_key_id {
            updates.push("ssh_key_id = ?");
            params_vec.push(Box::new(ssh_key_id.clone()));
        }
        if let Some(ref env_vars) = input.env_vars {
            updates.push("env_vars = ?");
            params_vec.push(Box::new(env_vars.clone()));
        }
        if let Some(ref hotkey_filter) = input.hotkey_filter {
            updates.push("hotkey_filter = ?");
            params_vec.push(Box::new(hotkey_filter.clone()));
        }
        if let Some(ref hotkey_position) = input.hotkey_position {
            updates.push("hotkey_position = ?");
            params_vec.push(Box::new(hotkey_position.clone()));
        }
        if let Some(hotkey_detect_hidden) = input.hotkey_detect_hidden {
            updates.push("hotkey_detect_hidden = ?");
            params_vec.push(Box::new(hotkey_detect_hidden));
        }

        updates.push("updated_at = ?");
        params_vec.push(Box::new(now));
        params_vec.push(Box::new(id.to_string()));

        if !updates.is_empty() {
            let sql = format!(
                "UPDATE entries SET {} WHERE id = ?",
                updates.join(", ")
            );
            let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|b| b.as_ref()).collect();
            self.conn.execute(&sql, params_refs.as_slice())?;
        }

        self.get_entry(id)
    }

    pub fn delete_entry(&self, id: &str) -> DbResult<()> {
        let rows = self.conn.execute("DELETE FROM entries WHERE id = ?", [id])?;
        if rows == 0 {
            return Err(DatabaseError::NotFound(format!("Entry with id {} not found", id)));
        }
        // Also delete associated hotkeys
        self.conn.execute("DELETE FROM hotkeys WHERE entry_id = ?", [id])?;
        Ok(())
    }

    pub fn toggle_entry(&self, id: &str, enabled: bool) -> DbResult<Entry> {
        let now = Utc::now().to_rfc3339();
        let rows = self.conn.execute(
            "UPDATE entries SET enabled = ?, updated_at = ? WHERE id = ?",
            params![enabled, now, id],
        )?;
        if rows == 0 {
            return Err(DatabaseError::NotFound(format!("Entry with id {} not found", id)));
        }
        self.get_entry(id)
    }

    pub fn record_entry_usage(&self, id: &str) -> DbResult<()> {
        let now = Utc::now().to_rfc3339();
        let rows = self.conn.execute(
            "UPDATE entries SET last_used_at = ?, use_count = use_count + 1, updated_at = ? WHERE id = ?",
            params![now, now, id],
        )?;
        if rows == 0 {
            return Err(DatabaseError::NotFound(format!("Entry with id {} not found", id)));
        }
        Ok(())
    }

    /// Search entries with fuzzy matching
    pub fn search_entries(&self, query: &str, settings: &Settings) -> DbResult<Vec<SearchResult>> {
        use fuzzy_matcher::FuzzyMatcher;
        use fuzzy_matcher::skim::SkimMatcherV2;

        let matcher = SkimMatcherV2::default();
        let all_entries = self.get_all_entries()?;

        let mut results: Vec<SearchResult> = all_entries
            .into_iter()
            .filter(|e| e.enabled)
            .filter_map(|entry| {
                // Match against name
                let name_score = matcher.fuzzy_match(&entry.name, query).unwrap_or(0);
                // Match against target
                let target_score = matcher.fuzzy_match(&entry.target, query).unwrap_or(0);
                // Match against tags
                let tags_score = entry.tags.as_ref()
                    .map(|t| matcher.fuzzy_match(t, query).unwrap_or(0))
                    .unwrap_or(0);

                let max_score = name_score.max(target_score).max(tags_score);

                if max_score > 0 || query.is_empty() {
                    Some(SearchResult {
                        entry,
                        score: max_score,
                    })
                } else {
                    None
                }
            })
            .collect();

        // Sort based on strategy
        match settings.sort_strategy {
            SortStrategy::Relevance => {
                results.sort_by(|a, b| b.score.cmp(&a.score));
            }
            SortStrategy::Name => {
                results.sort_by(|a, b| a.entry.name.to_lowercase().cmp(&b.entry.name.to_lowercase()));
            }
            SortStrategy::RecentlyUsed => {
                results.sort_by(|a, b| {
                    let a_time = a.entry.last_used_at.unwrap_or(DateTime::<Utc>::MIN_UTC);
                    let b_time = b.entry.last_used_at.unwrap_or(DateTime::<Utc>::MIN_UTC);
                    b_time.cmp(&a_time)
                });
            }
            SortStrategy::UseCount => {
                results.sort_by(|a, b| b.entry.use_count.cmp(&a.entry.use_count));
            }
        }

        // Limit results
        results.truncate(settings.max_results as usize);

        Ok(results)
    }

    fn row_to_entry(&self, row: &rusqlite::Row) -> rusqlite::Result<Entry> {
        let type_str: String = row.get(2)?;
        let created_at_str: String = row.get(21)?;
        let updated_at_str: String = row.get(22)?;
        let last_used_at_str: Option<String> = row.get(23)?;

        Ok(Entry {
            id: row.get(0)?,
            name: row.get(1)?,
            entry_type: EntryType::from_str(&type_str).unwrap_or(EntryType::App),
            target: row.get(3)?,
            args: row.get(4)?,
            workdir: row.get(5)?,
            icon_path: row.get(6)?,
            tags: row.get(7)?,
            description: row.get(8)?,
            enabled: row.get(9)?,
            confirm_before_run: row.get(10)?,
            show_terminal: row.get(11)?,
            wsl_distro: row.get(12)?,
            ssh_host: row.get(13)?,
            ssh_user: row.get(14)?,
            ssh_port: row.get(15)?,
            ssh_key_id: row.get(16)?,
            env_vars: row.get(17)?,
            hotkey_filter: row.get(18)?,
            hotkey_position: row.get(19)?,
            hotkey_detect_hidden: row.get(20)?,
            created_at: DateTime::parse_from_rfc3339(&created_at_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            updated_at: DateTime::parse_from_rfc3339(&updated_at_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            last_used_at: last_used_at_str.and_then(|s| {
                DateTime::parse_from_rfc3339(&s)
                    .map(|dt| dt.with_timezone(&Utc))
                    .ok()
            }),
            use_count: row.get(24)?,
        })
    }

    // ==================== Hotkey CRUD ====================

    pub fn get_all_hotkeys(&self) -> DbResult<Vec<Hotkey>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, entry_id, accelerator, scope, enabled, created_at, updated_at
             FROM hotkeys ORDER BY accelerator"
        )?;

        let hotkeys = stmt.query_map([], |row| {
            let scope_str: String = row.get(3)?;
            let created_at_str: String = row.get(5)?;
            let updated_at_str: String = row.get(6)?;

            Ok(Hotkey {
                id: row.get(0)?,
                entry_id: row.get(1)?,
                accelerator: row.get(2)?,
                scope: HotkeyScope::from_str(&scope_str).unwrap_or(HotkeyScope::App),
                enabled: row.get(4)?,
                created_at: DateTime::parse_from_rfc3339(&created_at_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                updated_at: DateTime::parse_from_rfc3339(&updated_at_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
            })
        })?;

        hotkeys.collect::<Result<Vec<_>, _>>().map_err(DatabaseError::from)
    }

    pub fn get_hotkey(&self, id: &str) -> DbResult<Hotkey> {
        let mut stmt = self.conn.prepare(
            "SELECT id, entry_id, accelerator, scope, enabled, created_at, updated_at
             FROM hotkeys WHERE id = ?"
        )?;

        stmt.query_row([id], |row| {
            let scope_str: String = row.get(3)?;
            let created_at_str: String = row.get(5)?;
            let updated_at_str: String = row.get(6)?;

            Ok(Hotkey {
                id: row.get(0)?,
                entry_id: row.get(1)?,
                accelerator: row.get(2)?,
                scope: HotkeyScope::from_str(&scope_str).unwrap_or(HotkeyScope::App),
                enabled: row.get(4)?,
                created_at: DateTime::parse_from_rfc3339(&created_at_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                updated_at: DateTime::parse_from_rfc3339(&updated_at_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
            })
        })
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => DatabaseError::NotFound(format!("Hotkey with id {} not found", id)),
            _ => DatabaseError::from(e),
        })
    }

    pub fn create_hotkey(&self, entry_id: &str, accelerator: &str, scope: HotkeyScope) -> DbResult<Hotkey> {
        // Verify entry exists
        let _entry = self.get_entry(entry_id)?;

        let now = Utc::now();
        let id = uuid::Uuid::new_v4().to_string();
        let now_str = now.to_rfc3339();

        self.conn.execute(
            "INSERT INTO hotkeys (id, entry_id, accelerator, scope, enabled, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, 1, ?5, ?6)",
            params![id, entry_id, accelerator, scope.as_str(), now_str, now_str],
        )?;

        self.get_hotkey(&id)
    }

    pub fn update_hotkey(&self, id: &str, accelerator: Option<&str>, scope: Option<HotkeyScope>, enabled: Option<bool>) -> DbResult<Hotkey> {
        let _existing = self.get_hotkey(id)?;
        let now = Utc::now().to_rfc3339();

        let mut updates = vec!["updated_at = ?"];
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(now)];

        if let Some(acc) = accelerator {
            updates.push("accelerator = ?");
            params_vec.push(Box::new(acc.to_string()));
        }
        if let Some(s) = scope {
            updates.push("scope = ?");
            params_vec.push(Box::new(s.as_str().to_string()));
        }
        if let Some(e) = enabled {
            updates.push("enabled = ?");
            params_vec.push(Box::new(e));
        }

        params_vec.push(Box::new(id.to_string()));

        let sql = format!(
            "UPDATE hotkeys SET {} WHERE id = ?",
            updates.join(", ")
        );
        let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|b| b.as_ref()).collect();
        self.conn.execute(&sql, params_refs.as_slice())?;

        self.get_hotkey(id)
    }

    pub fn delete_hotkey(&self, id: &str) -> DbResult<()> {
        let rows = self.conn.execute("DELETE FROM hotkeys WHERE id = ?", [id])?;
        if rows == 0 {
            return Err(DatabaseError::NotFound(format!("Hotkey with id {} not found", id)));
        }
        Ok(())
    }

    pub fn check_hotkey_conflict(&self, accelerator: &str, scope: &HotkeyScope, exclude_id: Option<&str>) -> DbResult<Option<Hotkey>> {
        let sql = match exclude_id {
            Some(_) => "SELECT id, entry_id, accelerator, scope, enabled, created_at, updated_at
                        FROM hotkeys WHERE accelerator = ? AND scope = ? AND id != ? AND enabled = 1",
            None => "SELECT id, entry_id, accelerator, scope, enabled, created_at, updated_at
                     FROM hotkeys WHERE accelerator = ? AND scope = ? AND enabled = 1",
        };

        let result = if let Some(excl) = exclude_id {
            self.conn.query_row(sql, params![accelerator, scope.as_str(), excl], |row| {
                let scope_str: String = row.get(3)?;
                let created_at_str: String = row.get(5)?;
                let updated_at_str: String = row.get(6)?;

                Ok(Hotkey {
                    id: row.get(0)?,
                    entry_id: row.get(1)?,
                    accelerator: row.get(2)?,
                    scope: HotkeyScope::from_str(&scope_str).unwrap_or(HotkeyScope::App),
                    enabled: row.get(4)?,
                    created_at: DateTime::parse_from_rfc3339(&created_at_str)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    updated_at: DateTime::parse_from_rfc3339(&updated_at_str)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                })
            })
        } else {
            self.conn.query_row(sql, params![accelerator, scope.as_str()], |row| {
                let scope_str: String = row.get(3)?;
                let created_at_str: String = row.get(5)?;
                let updated_at_str: String = row.get(6)?;

                Ok(Hotkey {
                    id: row.get(0)?,
                    entry_id: row.get(1)?,
                    accelerator: row.get(2)?,
                    scope: HotkeyScope::from_str(&scope_str).unwrap_or(HotkeyScope::App),
                    enabled: row.get(4)?,
                    created_at: DateTime::parse_from_rfc3339(&created_at_str)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    updated_at: DateTime::parse_from_rfc3339(&updated_at_str)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                })
            })
        };

        match result {
            Ok(hotkey) => Ok(Some(hotkey)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(DatabaseError::from(e)),
        }
    }

    // ==================== Settings ====================

    pub fn get_settings(&self) -> DbResult<Settings> {
        let mut stmt = self.conn.prepare("SELECT key, value FROM settings")?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;

        let mut settings = Settings::default();
        for row in rows {
            let (key, value) = row?;
            match key.as_str() {
                "icon_size" => settings.icon_size = value.parse().unwrap_or(24),
                "show_path" => settings.show_path = value == "true",
                "show_type_label" => settings.show_type_label = value == "true",
                "show_description" => settings.show_description = value == "true",
                "sort_strategy" => settings.sort_strategy = SortStrategy::from_str(&value),
                "max_results" => settings.max_results = value.parse().unwrap_or(50),
                "confirm_dangerous_commands" => settings.confirm_dangerous_commands = value == "true",
                "auto_launch" => settings.auto_launch = value == "true",
                "app_hotkey" => settings.app_hotkey = value,
                "theme" => settings.theme = value,
                "language" => settings.language = value,
                "search_debounce_ms" => settings.search_debounce_ms = value.parse().unwrap_or(150),
                _ => {}
            }
        }

        Ok(settings)
    }

    pub fn save_settings(&self, settings: &Settings) -> DbResult<()> {
        let pairs = vec![
            ("icon_size", settings.icon_size.to_string()),
            ("show_path", settings.show_path.to_string()),
            ("show_type_label", settings.show_type_label.to_string()),
            ("show_description", settings.show_description.to_string()),
            ("sort_strategy", settings.sort_strategy.as_str().to_string()),
            ("max_results", settings.max_results.to_string()),
            ("confirm_dangerous_commands", settings.confirm_dangerous_commands.to_string()),
            ("auto_launch", settings.auto_launch.to_string()),
            ("app_hotkey", settings.app_hotkey.clone()),
            ("theme", settings.theme.clone()),
            ("language", settings.language.clone()),
            ("search_debounce_ms", settings.search_debounce_ms.to_string()),
        ];

        for (key, value) in pairs {
            self.conn.execute(
                "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
                params![key, value],
            )?;
        }

        Ok(())
    }

    // ==================== Script Templates ====================

    pub fn get_all_templates(&self) -> DbResult<Vec<ScriptTemplate>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, description, language, template_content, variables_json, created_at, updated_at
             FROM script_templates ORDER BY name"
        )?;

        let templates = stmt.query_map([], |row| {
            let variables_json: String = row.get(5)?;
            let created_at_str: String = row.get(6)?;
            let updated_at_str: String = row.get(7)?;

            let variables: Vec<VariableDefinition> = serde_json::from_str(&variables_json)
                .unwrap_or_default();

            Ok(ScriptTemplate {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                language: row.get(3)?,
                template_content: row.get(4)?,
                variables,
                created_at: DateTime::parse_from_rfc3339(&created_at_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                updated_at: DateTime::parse_from_rfc3339(&updated_at_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
            })
        })?;

        templates.collect::<Result<Vec<_>, _>>().map_err(DatabaseError::from)
    }

    pub fn get_template(&self, id: &str) -> DbResult<ScriptTemplate> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, description, language, template_content, variables_json, created_at, updated_at
             FROM script_templates WHERE id = ?"
        )?;

        stmt.query_row([id], |row| {
            let variables_json: String = row.get(5)?;
            let created_at_str: String = row.get(6)?;
            let updated_at_str: String = row.get(7)?;

            let variables: Vec<VariableDefinition> = serde_json::from_str(&variables_json)
                .unwrap_or_default();

            Ok(ScriptTemplate {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                language: row.get(3)?,
                template_content: row.get(4)?,
                variables,
                created_at: DateTime::parse_from_rfc3339(&created_at_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                updated_at: DateTime::parse_from_rfc3339(&updated_at_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
            })
        })
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => DatabaseError::NotFound(format!("Template with id {} not found", id)),
            _ => DatabaseError::from(e),
        })
    }

    pub fn create_template(&self, template: &ScriptTemplate) -> DbResult<ScriptTemplate> {
        let variables_json = serde_json::to_string(&template.variables)?;
        let now_str = Utc::now().to_rfc3339();

        self.conn.execute(
            "INSERT INTO script_templates (id, name, description, language, template_content, variables_json, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                template.id,
                template.name,
                template.description,
                template.language,
                template.template_content,
                variables_json,
                now_str,
                now_str,
            ],
        )?;

        self.get_template(&template.id)
    }

    pub fn update_template(&self, id: &str, name: Option<&str>, description: Option<&str>,
                           language: Option<&str>, template_content: Option<&str>,
                           variables: Option<&Vec<VariableDefinition>>) -> DbResult<ScriptTemplate> {
        let _existing = self.get_template(id)?;
        let now = Utc::now().to_rfc3339();

        let mut updates = vec!["updated_at = ?"];
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(now)];

        if let Some(n) = name {
            updates.push("name = ?");
            params_vec.push(Box::new(n.to_string()));
        }
        if let Some(d) = description {
            updates.push("description = ?");
            params_vec.push(Box::new(d.to_string()));
        }
        if let Some(l) = language {
            updates.push("language = ?");
            params_vec.push(Box::new(l.to_string()));
        }
        if let Some(tc) = template_content {
            updates.push("template_content = ?");
            params_vec.push(Box::new(tc.to_string()));
        }
        if let Some(vars) = variables {
            updates.push("variables_json = ?");
            params_vec.push(Box::new(serde_json::to_string(vars).unwrap_or_default()));
        }

        params_vec.push(Box::new(id.to_string()));

        let sql = format!(
            "UPDATE script_templates SET {} WHERE id = ?",
            updates.join(", ")
        );
        let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|b| b.as_ref()).collect();
        self.conn.execute(&sql, params_refs.as_slice())?;

        self.get_template(id)
    }

    pub fn delete_template(&self, id: &str) -> DbResult<()> {
        let rows = self.conn.execute("DELETE FROM script_templates WHERE id = ?", [id])?;
        if rows == 0 {
            return Err(DatabaseError::NotFound(format!("Template with id {} not found", id)));
        }
        Ok(())
    }

    // ==================== Import/Export ====================

    pub fn export_all(&self) -> DbResult<ExportData> {
        Ok(ExportData {
            schema_version: ExportData::CURRENT_SCHEMA_VERSION.to_string(),
            exported_at: Utc::now(),
            entries: self.get_all_entries()?,
            hotkeys: self.get_all_hotkeys()?,
            settings: self.get_settings()?,
            templates: self.get_all_templates()?,
        })
    }

    pub fn import_data(&self, data: &ExportData, strategy: ImportStrategy) -> DbResult<ImportResult> {
        let mut result = ImportResult::default();

        match strategy {
            ImportStrategy::OverwriteAll => {
                // Clear existing data
                self.conn.execute("DELETE FROM hotkeys", [])?;
                self.conn.execute("DELETE FROM entries", [])?;
                self.conn.execute("DELETE FROM script_templates", [])?;

                // Import all entries
                for entry in &data.entries {
                    self.import_entry(entry)?;
                    result.entries_imported += 1;
                }

                // Import all hotkeys
                for hotkey in &data.hotkeys {
                    self.import_hotkey(hotkey)?;
                    result.hotkeys_imported += 1;
                }

                // Import settings
                self.save_settings(&data.settings)?;

                // Import templates
                for template in &data.templates {
                    self.import_template(template)?;
                    result.templates_imported += 1;
                }
            }
            ImportStrategy::AddOnly => {
                // Only add new entries (by id)
                let existing_entry_ids: std::collections::HashSet<String> =
                    self.get_all_entries()?.into_iter().map(|e| e.id).collect();

                for entry in &data.entries {
                    if !existing_entry_ids.contains(&entry.id) {
                        self.import_entry(entry)?;
                        result.entries_imported += 1;
                    } else {
                        result.entries_skipped += 1;
                    }
                }

                let existing_hotkey_ids: std::collections::HashSet<String> =
                    self.get_all_hotkeys()?.into_iter().map(|h| h.id).collect();

                for hotkey in &data.hotkeys {
                    if !existing_hotkey_ids.contains(&hotkey.id) {
                        self.import_hotkey(hotkey)?;
                        result.hotkeys_imported += 1;
                    } else {
                        result.hotkeys_skipped += 1;
                    }
                }

                let existing_template_ids: std::collections::HashSet<String> =
                    self.get_all_templates()?.into_iter().map(|t| t.id).collect();

                for template in &data.templates {
                    if !existing_template_ids.contains(&template.id) {
                        self.import_template(template)?;
                        result.templates_imported += 1;
                    } else {
                        result.templates_skipped += 1;
                    }
                }
            }
            ImportStrategy::MergeByName => {
                // Merge by name - update if exists, add if new
                let existing_entries: std::collections::HashMap<String, Entry> =
                    self.get_all_entries()?.into_iter().map(|e| (e.name.clone(), e)).collect();

                for entry in &data.entries {
                    if let Some(existing) = existing_entries.get(&entry.name) {
                        // Update existing
                        let mut updated = entry.clone();
                        updated.id = existing.id.clone();
                        self.import_entry_update(&updated)?;
                        result.entries_updated += 1;
                    } else {
                        self.import_entry(entry)?;
                        result.entries_imported += 1;
                    }
                }

                // For hotkeys and templates, use add-only logic
                let existing_hotkey_ids: std::collections::HashSet<String> =
                    self.get_all_hotkeys()?.into_iter().map(|h| h.id).collect();

                for hotkey in &data.hotkeys {
                    if !existing_hotkey_ids.contains(&hotkey.id) {
                        self.import_hotkey(hotkey)?;
                        result.hotkeys_imported += 1;
                    } else {
                        result.hotkeys_skipped += 1;
                    }
                }

                let existing_template_names: std::collections::HashMap<String, ScriptTemplate> =
                    self.get_all_templates()?.into_iter().map(|t| (t.name.clone(), t)).collect();

                for template in &data.templates {
                    if existing_template_names.contains_key(&template.name) {
                        result.templates_skipped += 1;
                    } else {
                        self.import_template(template)?;
                        result.templates_imported += 1;
                    }
                }
            }
        }

        Ok(result)
    }

    fn import_entry(&self, entry: &Entry) -> DbResult<()> {
        self.conn.execute(
            "INSERT INTO entries (id, name, type, target, args, workdir, icon_path, tags,
                                  description, enabled, confirm_before_run, show_terminal,
                                  wsl_distro, ssh_host, ssh_user, ssh_port, ssh_key_id,
                                  env_vars, hotkey_filter, hotkey_position, hotkey_detect_hidden,
                                  created_at, updated_at, last_used_at, use_count)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25)",
            params![
                entry.id,
                entry.name,
                entry.entry_type.as_str(),
                entry.target,
                entry.args,
                entry.workdir,
                entry.icon_path,
                entry.tags,
                entry.description,
                entry.enabled,
                entry.confirm_before_run,
                entry.show_terminal,
                entry.wsl_distro,
                entry.ssh_host,
                entry.ssh_user,
                entry.ssh_port,
                entry.ssh_key_id,
                entry.env_vars,
                entry.hotkey_filter,
                entry.hotkey_position,
                entry.hotkey_detect_hidden,
                entry.created_at.to_rfc3339(),
                entry.updated_at.to_rfc3339(),
                entry.last_used_at.map(|dt| dt.to_rfc3339()),
                entry.use_count,
            ],
        )?;
        Ok(())
    }

    fn import_entry_update(&self, entry: &Entry) -> DbResult<()> {
        self.conn.execute(
            "UPDATE entries SET name = ?2, type = ?3, target = ?4, args = ?5, workdir = ?6,
                    icon_path = ?7, tags = ?8, description = ?9, enabled = ?10,
                    confirm_before_run = ?11, show_terminal = ?12, wsl_distro = ?13,
                    ssh_host = ?14, ssh_user = ?15, ssh_port = ?16, ssh_key_id = ?17,
                    env_vars = ?18, hotkey_filter = ?19, hotkey_position = ?20,
                    hotkey_detect_hidden = ?21, updated_at = ?22
             WHERE id = ?1",
            params![
                entry.id,
                entry.name,
                entry.entry_type.as_str(),
                entry.target,
                entry.args,
                entry.workdir,
                entry.icon_path,
                entry.tags,
                entry.description,
                entry.enabled,
                entry.confirm_before_run,
                entry.show_terminal,
                entry.wsl_distro,
                entry.ssh_host,
                entry.ssh_user,
                entry.ssh_port,
                entry.ssh_key_id,
                entry.env_vars,
                entry.hotkey_filter,
                entry.hotkey_position,
                entry.hotkey_detect_hidden,
                Utc::now().to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    fn import_hotkey(&self, hotkey: &Hotkey) -> DbResult<()> {
        self.conn.execute(
            "INSERT INTO hotkeys (id, entry_id, accelerator, scope, enabled, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                hotkey.id,
                hotkey.entry_id,
                hotkey.accelerator,
                hotkey.scope.as_str(),
                hotkey.enabled,
                hotkey.created_at.to_rfc3339(),
                hotkey.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    fn import_template(&self, template: &ScriptTemplate) -> DbResult<()> {
        let variables_json = serde_json::to_string(&template.variables)?;
        self.conn.execute(
            "INSERT INTO script_templates (id, name, description, language, template_content, variables_json, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                template.id,
                template.name,
                template.description,
                template.language,
                template.template_content,
                variables_json,
                template.created_at.to_rfc3339(),
                template.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ImportResult {
    pub entries_imported: i32,
    pub entries_updated: i32,
    pub entries_skipped: i32,
    pub hotkeys_imported: i32,
    pub hotkeys_skipped: i32,
    pub templates_imported: i32,
    pub templates_skipped: i32,
}

use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_database_creation() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let db = Database::new(&db_path).unwrap();

        // Verify tables exist
        let entries = db.get_all_entries().unwrap();
        assert!(entries.is_empty());

        let settings = db.get_settings().unwrap();
        assert_eq!(settings.icon_size, 24);
        assert_eq!(settings.language, "zh-CN");
        assert!(settings.auto_launch);
    }

    #[test]
    fn test_entry_crud() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let db = Database::new(&db_path).unwrap();

        // Create
        let input = CreateEntryInput {
            name: "Test Entry".to_string(),
            entry_type: EntryType::App,
            target: "/path/to/app".to_string(),
            args: None,
            workdir: None,
            icon_path: None,
            tags: Some("test,demo".to_string()),
            description: Some("A test entry".to_string()),
            enabled: Some(true),
            confirm_before_run: None,
            show_terminal: None,
            wsl_distro: None,
            ssh_host: None,
            ssh_user: None,
            ssh_port: None,
            ssh_key_id: None,
            env_vars: None,
            hotkey_filter: None,
            hotkey_position: None,
            hotkey_detect_hidden: None,
        };

        let entry = db.create_entry(&input).unwrap();
        assert_eq!(entry.name, "Test Entry");
        assert_eq!(entry.entry_type, EntryType::App);

        // Read
        let fetched = db.get_entry(&entry.id).unwrap();
        assert_eq!(fetched.name, "Test Entry");

        // Update
        let update_input = UpdateEntryInput {
            name: Some("Updated Entry".to_string()),
            entry_type: None,
            target: None,
            args: None,
            workdir: None,
            icon_path: None,
            tags: None,
            description: None,
            enabled: None,
            confirm_before_run: None,
            show_terminal: None,
            wsl_distro: None,
            ssh_host: None,
            ssh_user: None,
            ssh_port: None,
            ssh_key_id: None,
            env_vars: None,
            hotkey_filter: None,
            hotkey_position: None,
            hotkey_detect_hidden: None,
        };

        let updated = db.update_entry(&entry.id, &update_input).unwrap();
        assert_eq!(updated.name, "Updated Entry");

        // Delete
        db.delete_entry(&entry.id).unwrap();
        let result = db.get_entry(&entry.id);
        assert!(result.is_err());
    }

    #[test]
    fn test_search() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let db = Database::new(&db_path).unwrap();

        // Create entries
        let entries = vec![
            ("Chrome", EntryType::App, "/usr/bin/chrome"),
            ("Firefox", EntryType::App, "/usr/bin/firefox"),
            ("Google", EntryType::Url, "https://google.com"),
        ];

        for (name, entry_type, target) in entries {
            let input = CreateEntryInput {
                name: name.to_string(),
                entry_type,
                target: target.to_string(),
                args: None,
                workdir: None,
                icon_path: None,
                tags: None,
                description: None,
                enabled: Some(true),
                confirm_before_run: None,
                show_terminal: None,
                wsl_distro: None,
                ssh_host: None,
                ssh_user: None,
                ssh_port: None,
                ssh_key_id: None,
                env_vars: None,
                hotkey_filter: None,
                hotkey_position: None,
                hotkey_detect_hidden: None,
            };
            db.create_entry(&input).unwrap();
        }

        let settings = db.get_settings().unwrap();

        // Search for "chrome"
        let results = db.search_entries("chrome", &settings).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].entry.name, "Chrome");

        // Search for "goo" should match Google
        let results = db.search_entries("goo", &settings).unwrap();
        assert!(!results.is_empty());
    }

    #[test]
    fn test_hotkey_conflict() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let db = Database::new(&db_path).unwrap();

        // Create an entry
        let input = CreateEntryInput {
            name: "Test Entry".to_string(),
            entry_type: EntryType::App,
            target: "/path/to/app".to_string(),
            args: None,
            workdir: None,
            icon_path: None,
            tags: None,
            description: None,
            enabled: Some(true),
            confirm_before_run: None,
            show_terminal: None,
            wsl_distro: None,
            ssh_host: None,
            ssh_user: None,
            ssh_port: None,
            ssh_key_id: None,
            env_vars: None,
            hotkey_filter: None,
            hotkey_position: None,
            hotkey_detect_hidden: None,
        };
        let entry = db.create_entry(&input).unwrap();

        // Create a hotkey
        let hotkey = db.create_hotkey(&entry.id, "Ctrl+Alt+T", HotkeyScope::App).unwrap();
        assert_eq!(hotkey.accelerator, "Ctrl+Alt+T");

        // Check for conflict
        let conflict = db.check_hotkey_conflict("Ctrl+Alt+T", &HotkeyScope::App, None).unwrap();
        assert!(conflict.is_some());

        // No conflict with different accelerator
        let no_conflict = db.check_hotkey_conflict("Ctrl+Alt+X", &HotkeyScope::App, None).unwrap();
        assert!(no_conflict.is_none());
    }

    #[test]
    fn test_import_export() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let db = Database::new(&db_path).unwrap();

        // Create some data
        let input = CreateEntryInput {
            name: "Export Test".to_string(),
            entry_type: EntryType::Url,
            target: "https://example.com".to_string(),
            args: None,
            workdir: None,
            icon_path: None,
            tags: None,
            description: None,
            enabled: Some(true),
            confirm_before_run: None,
            show_terminal: None,
            wsl_distro: None,
            ssh_host: None,
            ssh_user: None,
            ssh_port: None,
            ssh_key_id: None,
            env_vars: None,
            hotkey_filter: None,
            hotkey_position: None,
            hotkey_detect_hidden: None,
        };
        db.create_entry(&input).unwrap();

        // Export
        let export_data = db.export_all().unwrap();
        assert_eq!(export_data.entries.len(), 1);
        assert_eq!(export_data.schema_version, ExportData::CURRENT_SCHEMA_VERSION);

        // Create new database and import
        let dir2 = tempdir().unwrap();
        let db_path2 = dir2.path().join("test2.db");
        let db2 = Database::new(&db_path2).unwrap();

        let result = db2.import_data(&export_data, ImportStrategy::AddOnly).unwrap();
        assert_eq!(result.entries_imported, 1);

        let entries = db2.get_all_entries().unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "Export Test");
    }
}
