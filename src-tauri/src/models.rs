// Data models for Opener

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Entry type enum representing different launcher item types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum EntryType {
    App,
    Url,
    File,
    Dir,
    Cmd,
    Wsl,
    Ssh,
    Script,
    Shortcut,
    Ahk,
    #[serde(rename = "hotkey_app")]
    HotkeyApp,
}

impl EntryType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "app" => Some(EntryType::App),
            "url" => Some(EntryType::Url),
            "file" => Some(EntryType::File),
            "dir" => Some(EntryType::Dir),
            "cmd" => Some(EntryType::Cmd),
            "wsl" => Some(EntryType::Wsl),
            "ssh" => Some(EntryType::Ssh),
            "script" => Some(EntryType::Script),
            "shortcut" => Some(EntryType::Shortcut),
            "ahk" => Some(EntryType::Ahk),
            "hotkey_app" | "hotkeyapp" => Some(EntryType::HotkeyApp),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            EntryType::App => "app",
            EntryType::Url => "url",
            EntryType::File => "file",
            EntryType::Dir => "dir",
            EntryType::Cmd => "cmd",
            EntryType::Wsl => "wsl",
            EntryType::Ssh => "ssh",
            EntryType::Script => "script",
            EntryType::Shortcut => "shortcut",
            EntryType::Ahk => "ahk",
            EntryType::HotkeyApp => "hotkey_app",
        }
    }

    /// Returns true if this entry type requires security confirmation
    #[allow(dead_code)]
    pub fn requires_confirmation(&self) -> bool {
        matches!(
            self,
            EntryType::Cmd
                | EntryType::Wsl
                | EntryType::Ssh
                | EntryType::Script
                | EntryType::Ahk
                | EntryType::HotkeyApp
        )
    }
}

/// Main entry model representing a launcher item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub entry_type: EntryType,
    pub target: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workdir: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confirm_before_run: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_terminal: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wsl_distro: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ssh_host: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ssh_user: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ssh_port: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ssh_key_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env_vars: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hotkey_filter: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hotkey_position: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hotkey_detect_hidden: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub script_content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub script_type: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_used_at: Option<DateTime<Utc>>,
    pub use_count: i32,
}

impl Entry {
    #[allow(dead_code)]
    pub fn new(name: String, entry_type: EntryType, target: String) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            entry_type,
            target,
            args: None,
            workdir: None,
            icon_path: None,
            tags: None,
            description: None,
            enabled: true,
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
            script_content: None,
            script_type: None,
            created_at: now,
            updated_at: now,
            last_used_at: None,
            use_count: 0,
        }
    }
}

/// Input structure for creating a new entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEntryInput {
    pub name: String,
    #[serde(rename = "type")]
    pub entry_type: EntryType,
    pub target: String,
    pub args: Option<String>,
    pub workdir: Option<String>,
    pub icon_path: Option<String>,
    pub tags: Option<String>,
    pub description: Option<String>,
    pub enabled: Option<bool>,
    pub confirm_before_run: Option<bool>,
    pub show_terminal: Option<bool>,
    pub wsl_distro: Option<String>,
    pub ssh_host: Option<String>,
    pub ssh_user: Option<String>,
    pub ssh_port: Option<i32>,
    pub ssh_key_id: Option<String>,
    pub env_vars: Option<String>,
    pub hotkey_filter: Option<String>,
    pub hotkey_position: Option<String>,
    pub hotkey_detect_hidden: Option<bool>,
    pub script_content: Option<String>,
    pub script_type: Option<String>,
}

/// Input structure for updating an entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateEntryInput {
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub entry_type: Option<EntryType>,
    pub target: Option<String>,
    pub args: Option<String>,
    pub workdir: Option<String>,
    pub icon_path: Option<String>,
    pub tags: Option<String>,
    pub description: Option<String>,
    pub enabled: Option<bool>,
    pub confirm_before_run: Option<bool>,
    pub show_terminal: Option<bool>,
    pub wsl_distro: Option<String>,
    pub ssh_host: Option<String>,
    pub ssh_user: Option<String>,
    pub ssh_port: Option<i32>,
    pub ssh_key_id: Option<String>,
    pub env_vars: Option<String>,
    pub hotkey_filter: Option<String>,
    pub hotkey_position: Option<String>,
    pub hotkey_detect_hidden: Option<bool>,
    pub script_content: Option<String>,
    pub script_type: Option<String>,
}

/// Hotkey scope enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum HotkeyScope {
    Global,
    App,
}

impl HotkeyScope {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "global" => Some(HotkeyScope::Global),
            "app" => Some(HotkeyScope::App),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            HotkeyScope::Global => "global",
            HotkeyScope::App => "app",
        }
    }
}

/// Hotkey binding model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hotkey {
    pub id: String,
    pub entry_id: String,
    pub accelerator: String,
    pub scope: HotkeyScope,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Hotkey {
    #[allow(dead_code)]
    pub fn new(entry_id: String, accelerator: String, scope: HotkeyScope) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            entry_id,
            accelerator,
            scope,
            enabled: true,
            created_at: now,
            updated_at: now,
        }
    }
}

/// Sort strategy enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SortStrategy {
    Relevance,
    Name,
    RecentlyUsed,
    UseCount,
}

impl SortStrategy {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "name" => SortStrategy::Name,
            "recently_used" => SortStrategy::RecentlyUsed,
            "use_count" => SortStrategy::UseCount,
            _ => SortStrategy::Relevance,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            SortStrategy::Relevance => "relevance",
            SortStrategy::Name => "name",
            SortStrategy::RecentlyUsed => "recently_used",
            SortStrategy::UseCount => "use_count",
        }
    }
}

/// Application settings model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub icon_size: i32,
    pub show_path: bool,
    pub show_type_label: bool,
    pub show_description: bool,
    pub sort_strategy: SortStrategy,
    pub max_results: i32,
    pub confirm_dangerous_commands: bool,
    #[serde(default)]
    pub auto_launch: bool,
    #[serde(default)]
    pub app_hotkey: String,
    pub theme: String,
    pub language: String,
    pub search_debounce_ms: i32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            icon_size: 24,
            show_path: true,
            show_type_label: true,
            show_description: true,
            sort_strategy: SortStrategy::Relevance,
            max_results: 50,
            confirm_dangerous_commands: true,
            auto_launch: true,
            app_hotkey: "Alt+R".to_string(),
            theme: "system".to_string(),
            language: "zh-CN".to_string(),
            search_debounce_ms: 150,
        }
    }
}

/// Variable type for script templates
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum VariableType {
    String,
    Number,
    Path,
    Choice,
    Boolean,
}

impl VariableType {
    #[allow(dead_code)]
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "number" => VariableType::Number,
            "path" => VariableType::Path,
            "choice" => VariableType::Choice,
            "boolean" => VariableType::Boolean,
            _ => VariableType::String,
        }
    }

    #[allow(dead_code)]
    pub fn as_str(&self) -> &'static str {
        match self {
            VariableType::String => "string",
            VariableType::Number => "number",
            VariableType::Path => "path",
            VariableType::Choice => "choice",
            VariableType::Boolean => "boolean",
        }
    }
}

/// Variable definition for script templates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableDefinition {
    pub name: String,
    #[serde(rename = "type")]
    pub var_type: VariableType,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_value: Option<String>,
    pub required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub choices: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validation_regex: Option<String>,
}

/// Script template model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptTemplate {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub language: String,
    pub template_content: String,
    pub variables: Vec<VariableDefinition>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ScriptTemplate {
    pub fn new(name: String, language: String, template_content: String) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            description: None,
            language,
            template_content,
            variables: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }
}

/// Import strategy enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ImportStrategy {
    OverwriteAll,
    AddOnly,
    MergeByName,
}

/// Export data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportData {
    pub schema_version: String,
    pub exported_at: DateTime<Utc>,
    pub entries: Vec<Entry>,
    pub hotkeys: Vec<Hotkey>,
    pub settings: Settings,
    pub templates: Vec<ScriptTemplate>,
}

impl ExportData {
    pub const CURRENT_SCHEMA_VERSION: &'static str = "1.0.0";
}

/// Search result with relevance score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub entry: Entry,
    pub score: i64,
}
