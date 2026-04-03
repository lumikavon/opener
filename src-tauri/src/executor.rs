// Entry executor module - handles execution of different entry types

use crate::models::{Entry, EntryType};
#[cfg(windows)]
use std::ffi::OsString;
#[cfg(windows)]
use std::os::windows::ffi::OsStringExt;
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;
#[cfg(windows)]
use std::path::Path;
#[cfg(windows)]
use std::path::PathBuf;
use std::process::Command;
#[cfg(windows)]
use std::ptr;
#[cfg(windows)]
use std::time::{Duration, Instant};
use thiserror::Error;
#[cfg(windows)]
use winapi::shared::minwindef::{BOOL, DWORD, LPARAM};
#[cfg(windows)]
use winapi::shared::windef::{HWND, RECT};
#[cfg(windows)]
use winapi::um::handleapi::CloseHandle;
#[cfg(windows)]
use winapi::um::processthreadsapi::OpenProcess;
#[cfg(windows)]
use winapi::um::psapi::GetModuleFileNameExW;
#[cfg(windows)]
use winapi::um::winnt::{PROCESS_QUERY_INFORMATION, PROCESS_VM_READ};
#[cfg(windows)]
use winapi::um::winuser::{
    BringWindowToTop, EnumWindows, GetWindowTextLengthW, GetWindowTextW, GetWindowThreadProcessId,
    IsWindowVisible, MoveWindow, SetForegroundWindow, ShowWindow, SystemParametersInfoW,
    SPI_GETWORKAREA, SW_MAXIMIZE, SW_RESTORE,
};

#[derive(Error, Debug)]
pub enum ExecutorError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Target not found: {0}")]
    TargetNotFound(String),
    #[error("Unsupported on this platform: {0}")]
    UnsupportedPlatform(String),
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
    #[allow(dead_code)]
    #[error("AHK not found. Please install AutoHotkey or configure its path in settings.")]
    AhkNotFound,
}

pub type ExecutorResult<T> = Result<T, ExecutorError>;

/// Execute an entry based on its type
pub fn execute_entry(entry: &Entry, ahk_path: Option<&str>) -> ExecutorResult<()> {
    match entry.entry_type {
        EntryType::Url => execute_url(entry),
        EntryType::File => execute_file(entry),
        EntryType::Dir => execute_directory(entry),
        EntryType::App => execute_app(entry),
        EntryType::Cmd => execute_cmd(entry),
        EntryType::Wsl => execute_wsl(entry),
        EntryType::Ssh => execute_ssh(entry),
        EntryType::Script => execute_script(entry),
        EntryType::Shortcut => execute_shortcut(entry),
        EntryType::Ahk => execute_ahk(entry, ahk_path),
        EntryType::HotkeyApp => execute_hotkey_app(entry),
    }
}

/// Open URL in default browser
fn execute_url(entry: &Entry) -> ExecutorResult<()> {
    open::that(&entry.target)?;
    Ok(())
}

/// Open file with default application
fn execute_file(entry: &Entry) -> ExecutorResult<()> {
    let path = std::path::Path::new(&entry.target);
    if !path.exists() {
        return Err(ExecutorError::TargetNotFound(entry.target.clone()));
    }
    open::that(&entry.target)?;
    Ok(())
}

/// Open directory in file manager
fn execute_directory(entry: &Entry) -> ExecutorResult<()> {
    let path = std::path::Path::new(&entry.target);
    if !path.exists() || !path.is_dir() {
        return Err(ExecutorError::TargetNotFound(entry.target.clone()));
    }
    open::that(&entry.target)?;
    Ok(())
}

/// Execute application with optional arguments
fn execute_app(entry: &Entry) -> ExecutorResult<()> {
    let path = std::path::Path::new(&entry.target);
    if !path.exists() {
        return Err(ExecutorError::TargetNotFound(entry.target.clone()));
    }

    let mut cmd = Command::new(&entry.target);

    // Add arguments if present
    if let Some(ref args) = entry.args {
        let args_vec: Vec<&str> = args.split_whitespace().collect();
        cmd.args(&args_vec);
    }

    // Set working directory if present
    if let Some(ref workdir) = entry.workdir {
        cmd.current_dir(workdir);
    }

    // Parse environment variables if present
    if let Some(ref env_vars) = entry.env_vars {
        for line in env_vars.lines() {
            if let Some((key, value)) = line.split_once('=') {
                cmd.env(key.trim(), value.trim());
            }
        }
    }

    cmd.spawn()?;
    Ok(())
}

/// Execute command in terminal
#[cfg(windows)]
fn execute_cmd(entry: &Entry) -> ExecutorResult<()> {
    let show_terminal = entry.show_terminal.unwrap_or(false);

    let mut cmd = if show_terminal {
        let mut c = Command::new("cmd");
        c.args(["/c", "start", "cmd", "/k", &entry.target]);
        c
    } else {
        let mut c = Command::new("cmd");
        c.args(["/c", &entry.target]);
        c.creation_flags(0x08000000); // CREATE_NO_WINDOW
        c
    };

    if let Some(ref workdir) = entry.workdir {
        cmd.current_dir(workdir);
    }

    if let Some(ref env_vars) = entry.env_vars {
        for line in env_vars.lines() {
            if let Some((key, value)) = line.split_once('=') {
                cmd.env(key.trim(), value.trim());
            }
        }
    }

    cmd.spawn()?;
    Ok(())
}

#[cfg(not(windows))]
fn execute_cmd(entry: &Entry) -> ExecutorResult<()> {
    let show_terminal = entry.show_terminal.unwrap_or(false);

    let mut cmd = if show_terminal {
        // Try to open in terminal emulator
        let terminal =
            std::env::var("TERMINAL").unwrap_or_else(|_| "x-terminal-emulator".to_string());
        let mut c = Command::new(&terminal);
        c.args(["-e", "sh", "-c", &entry.target]);
        c
    } else {
        let mut c = Command::new("sh");
        c.args(["-c", &entry.target]);
        c
    };

    if let Some(ref workdir) = entry.workdir {
        cmd.current_dir(workdir);
    }

    if let Some(ref env_vars) = entry.env_vars {
        for line in env_vars.lines() {
            if let Some((key, value)) = line.split_once('=') {
                cmd.env(key.trim(), value.trim());
            }
        }
    }

    cmd.spawn()?;
    Ok(())
}

/// Execute command in WSL
#[cfg(windows)]
fn execute_wsl(entry: &Entry) -> ExecutorResult<()> {
    let show_terminal = entry.show_terminal.unwrap_or(true);

    let mut args = Vec::new();

    if let Some(ref distro) = entry.wsl_distro {
        args.push("-d".to_string());
        args.push(distro.clone());
    }

    args.push("--".to_string());
    args.push(entry.target.clone());

    let mut cmd = if show_terminal {
        let mut c = Command::new("cmd");
        let wsl_cmd = format!("wsl {}", args.join(" "));
        c.args(["/c", "start", "cmd", "/k", &wsl_cmd]);
        c
    } else {
        let mut c = Command::new("wsl");
        c.args(&args);
        c
    };

    if let Some(ref workdir) = entry.workdir {
        cmd.current_dir(workdir);
    }

    cmd.spawn()?;
    Ok(())
}

#[cfg(not(windows))]
fn execute_wsl(_entry: &Entry) -> ExecutorResult<()> {
    Err(ExecutorError::UnsupportedPlatform(
        "WSL is only available on Windows".to_string(),
    ))
}

/// Execute SSH connection
fn execute_ssh(entry: &Entry) -> ExecutorResult<()> {
    let host = entry
        .ssh_host
        .as_ref()
        .ok_or_else(|| ExecutorError::ExecutionFailed("SSH host not configured".to_string()))?;

    let user = entry
        .ssh_user
        .as_ref()
        .map(|s| s.as_str())
        .unwrap_or("root");
    let port = entry.ssh_port.unwrap_or(22);

    let ssh_args = vec![
        format!("{}@{}", user, host),
        "-p".to_string(),
        port.to_string(),
    ];

    // Note: SSH key is handled through keyring, not directly here
    // The key path would be retrieved from secure storage

    #[cfg(windows)]
    {
        let show_terminal = entry.show_terminal.unwrap_or(true);
        let ssh_cmd = format!("ssh {}", ssh_args.join(" "));

        let mut cmd = if show_terminal {
            let mut c = Command::new("cmd");
            c.args(["/c", "start", "cmd", "/k", &ssh_cmd]);
            c
        } else {
            let mut c = Command::new("ssh");
            c.args(&ssh_args);
            c
        };

        cmd.spawn()?;
    }

    #[cfg(not(windows))]
    {
        let show_terminal = entry.show_terminal.unwrap_or(true);

        if show_terminal {
            let terminal =
                std::env::var("TERMINAL").unwrap_or_else(|_| "x-terminal-emulator".to_string());
            let ssh_cmd = format!("ssh {}", ssh_args.join(" "));
            Command::new(&terminal)
                .args(["-e", "sh", "-c", &ssh_cmd])
                .spawn()?;
        } else {
            Command::new("ssh").args(&ssh_args).spawn()?;
        }
    }

    Ok(())
}

/// Execute script
/// Priority: if target (script path) is non-empty and points to a file, execute it directly.
/// Otherwise, use script_content with script_type to determine the interpreter.
fn execute_script(entry: &Entry) -> ExecutorResult<()> {
    let target_path = entry.target.trim();
    let has_script_path = !target_path.is_empty() && std::path::Path::new(target_path).exists();

    if has_script_path {
        // Execute script file directly
        execute_script_file(entry, target_path)?;
    } else {
        // Execute inline script content
        let content = entry.script_content.as_deref().unwrap_or("").trim();
        if content.is_empty() {
            return Err(ExecutorError::ExecutionFailed(
                "No script path or script content provided".to_string(),
            ));
        }
        let script_type = entry.script_type.as_deref().unwrap_or("powershell");
        execute_script_content(entry, content, script_type)?;
    }

    Ok(())
}

/// Execute a script file by path
fn execute_script_file(entry: &Entry, path: &str) -> ExecutorResult<()> {
    let show_terminal = entry.show_terminal.unwrap_or(false);
    let ext = std::path::Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    #[cfg(windows)]
    {
        let mut cmd = match ext.as_str() {
            "ps1" => {
                if show_terminal {
                    let mut c = Command::new("cmd");
                    c.args([
                        "/c",
                        "start",
                        "",
                        "cmd",
                        "/k",
                        "pwsh",
                        "-ExecutionPolicy",
                        "Bypass",
                        "-File",
                        path,
                    ]);
                    c
                } else {
                    let mut c = Command::new("pwsh");
                    c.args(["-ExecutionPolicy", "Bypass", "-File", path]);
                    c.creation_flags(0x08000000);
                    c
                }
            }
            "py" => {
                if show_terminal {
                    let mut c = Command::new("cmd");
                    c.args(["/c", "start", "", "cmd", "/k", "python", path]);
                    c
                } else {
                    let mut c = Command::new("python");
                    c.arg(path);
                    c.creation_flags(0x08000000);
                    c
                }
            }
            "bat" | "cmd" => {
                if show_terminal {
                    let mut c = Command::new("cmd");
                    c.args(["/c", "start", "", "cmd", "/k", path]);
                    c
                } else {
                    let mut c = Command::new("cmd");
                    c.args(["/c", path]);
                    c.creation_flags(0x08000000);
                    c
                }
            }
            "sh" | "bash" => {
                if show_terminal {
                    let mut c = Command::new("cmd");
                    c.args(["/c", "start", "", "bash", path]);
                    c
                } else {
                    let mut c = Command::new("bash");
                    c.arg(path);
                    c.creation_flags(0x08000000);
                    c
                }
            }
            _ => {
                // Default: try powershell for unknown extensions
                if show_terminal {
                    let mut c = Command::new("cmd");
                    c.args([
                        "/c",
                        "start",
                        "",
                        "cmd",
                        "/k",
                        "powershell",
                        "-ExecutionPolicy",
                        "Bypass",
                        "-File",
                        path,
                    ]);
                    c
                } else {
                    let mut c = Command::new("powershell");
                    c.args(["-ExecutionPolicy", "Bypass", "-File", path]);
                    c.creation_flags(0x08000000);
                    c
                }
            }
        };

        if let Some(ref workdir) = entry.workdir {
            cmd.current_dir(workdir);
        }
        cmd.spawn()?;
    }

    #[cfg(not(windows))]
    {
        if show_terminal {
            let terminal =
                std::env::var("TERMINAL").unwrap_or_else(|_| "x-terminal-emulator".to_string());
            let mut cmd = Command::new(&terminal);
            cmd.args(["-e", path]);
            if let Some(ref workdir) = entry.workdir {
                cmd.current_dir(workdir);
            }
            cmd.spawn()?;
        } else {
            let interpreter = match ext.as_str() {
                "py" => "python3",
                "sh" | "bash" | "" => "bash",
                _ => "bash",
            };
            let mut cmd = Command::new(interpreter);
            cmd.arg(path);
            if let Some(ref workdir) = entry.workdir {
                cmd.current_dir(workdir);
            }
            cmd.spawn()?;
        }
    }

    Ok(())
}

/// Execute inline script content, writing to a temp file with appropriate extension
fn execute_script_content(entry: &Entry, content: &str, script_type: &str) -> ExecutorResult<()> {
    let show_terminal = entry.show_terminal.unwrap_or(false);
    let temp_dir = std::env::temp_dir();

    #[cfg(windows)]
    {
        let (ext, interpreter, interpreter_args): (&str, &str, Vec<&str>) = match script_type {
            "bash" => ("sh", "bash", vec![]),
            "python" => ("py", "python", vec![]),
            "cmd" => ("bat", "cmd", vec!["/c"]),
            _ => (
                "ps1",
                "powershell",
                vec!["-ExecutionPolicy", "Bypass", "-File"],
            ),
        };

        let script_path = temp_dir.join(format!("opener_script_{}.{}", uuid::Uuid::new_v4(), ext));
        std::fs::write(&script_path, content)?;
        let path_str = script_path.to_string_lossy().to_string();

        let mut cmd = if show_terminal {
            let mut c = Command::new("cmd");
            let mut args = vec!["/c", "start", "", "cmd", "/k", interpreter];
            args.extend_from_slice(&interpreter_args);
            args.push(&path_str);
            c.args(&args);
            c
        } else {
            let mut c = Command::new(interpreter);
            let mut args: Vec<&str> = interpreter_args.clone();
            args.push(&path_str);
            c.args(&args);
            c.creation_flags(0x08000000);
            c
        };

        if let Some(ref workdir) = entry.workdir {
            cmd.current_dir(workdir);
        }
        cmd.spawn()?;
    }

    #[cfg(not(windows))]
    {
        let (ext, interpreter): (&str, &str) = match script_type {
            "python" => ("py", "python3"),
            "cmd" | "bash" | _ => ("sh", "bash"),
        };

        let script_path = temp_dir.join(format!("opener_script_{}.{}", uuid::Uuid::new_v4(), ext));
        std::fs::write(&script_path, content)?;

        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&script_path)?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&script_path, perms)?;

        if show_terminal {
            let terminal =
                std::env::var("TERMINAL").unwrap_or_else(|_| "x-terminal-emulator".to_string());
            let mut cmd = Command::new(&terminal);
            cmd.args(["-e", &script_path.to_string_lossy()]);
            if let Some(ref workdir) = entry.workdir {
                cmd.current_dir(workdir);
            }
            cmd.spawn()?;
        } else {
            let mut cmd = Command::new(interpreter);
            cmd.arg(&script_path);
            if let Some(ref workdir) = entry.workdir {
                cmd.current_dir(workdir);
            }
            cmd.spawn()?;
        }
    }

    Ok(())
}

/// Execute hotkey application logic
#[cfg(windows)]
fn execute_hotkey_app(entry: &Entry) -> ExecutorResult<()> {
    let filter = build_window_filter(entry);
    let detect_hidden = entry.hotkey_detect_hidden.unwrap_or(true);
    let position = entry
        .hotkey_position
        .as_deref()
        .unwrap_or("keep")
        .to_lowercase();

    if let Some(hwnd) = find_window(&filter, detect_hidden) {
        restore_and_focus(hwnd);
        apply_position(hwnd, &position)?;
        return Ok(());
    }

    let (target, inline_args) = split_hotkey_target(&entry.target);
    let target_args = inline_args.or_else(|| entry.args.clone());
    let target_path = Path::new(&target);
    let executable = if target_path.exists() {
        target.clone()
    } else if let Ok(resolved) = which::which(&target) {
        resolved.to_string_lossy().to_string()
    } else {
        return Err(ExecutorError::TargetNotFound(target.clone()));
    };

    let mut cmd = Command::new(&executable);
    if let Some(ref args) = target_args {
        let args_vec: Vec<&str> = args.split_whitespace().collect();
        cmd.args(&args_vec);
    }
    if let Some(ref workdir) = entry.workdir {
        cmd.current_dir(workdir);
    }
    if let Some(ref env_vars) = entry.env_vars {
        for line in env_vars.lines() {
            if let Some((key, value)) = line.split_once('=') {
                cmd.env(key.trim(), value.trim());
            }
        }
    }

    cmd.spawn()?;

    if let Some(hwnd) = wait_for_window(&filter, detect_hidden, Duration::from_secs(15)) {
        restore_and_focus(hwnd);
        apply_position(hwnd, &position)?;
    } else {
        log::warn!("HotKey应用窗口未检测到，已跳过定位: {}", entry.target);
    }

    Ok(())
}

#[cfg(not(windows))]
fn execute_hotkey_app(_entry: &Entry) -> ExecutorResult<()> {
    Err(ExecutorError::UnsupportedPlatform(
        "HotKey应用 is only available on Windows".to_string(),
    ))
}

#[cfg(windows)]
fn strip_surrounding_quotes(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.len() >= 2 {
        let first = trimmed.chars().next().unwrap_or('"');
        let last = trimmed.chars().last().unwrap_or('"');
        if (first == '"' && last == '"') || (first == '\'' && last == '\'') {
            return trimmed[1..trimmed.len() - 1].to_string();
        }
    }
    trimmed.to_string()
}

#[cfg(windows)]
fn is_executable_candidate(value: &str) -> bool {
    let candidate = strip_surrounding_quotes(value);
    if candidate.is_empty() {
        return false;
    }
    if Path::new(&candidate).exists() {
        return true;
    }
    which::which(candidate).is_ok()
}

#[cfg(windows)]
fn split_hotkey_target(value: &str) -> (String, Option<String>) {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return (String::new(), None);
    }

    let first_char = trimmed.chars().next().unwrap_or('"');
    if first_char == '"' || first_char == '\'' {
        if let Some(end_offset) = trimmed[1..].find(first_char) {
            let end = end_offset + 1;
            let executable = trimmed[1..end].to_string();
            let rest = trimmed[end + 1..].trim();
            return (
                executable,
                if rest.is_empty() {
                    None
                } else {
                    Some(rest.to_string())
                },
            );
        }
    }

    let mut best_end = None;
    let mut best_exec = None;
    for (idx, ch) in trimmed.char_indices() {
        if ch.is_whitespace() {
            let candidate = trimmed[..idx].trim();
            if is_executable_candidate(candidate) {
                best_end = Some(idx);
                best_exec = Some(strip_surrounding_quotes(candidate));
            }
        }
    }

    if let Some(exec) = best_exec {
        let rest = trimmed[best_end.unwrap_or(trimmed.len())..].trim();
        return (
            exec,
            if rest.is_empty() {
                None
            } else {
                Some(rest.to_string())
            },
        );
    }

    if is_executable_candidate(trimmed) {
        return (strip_surrounding_quotes(trimmed), None);
    }

    if let Some(pos) = trimmed.find(|c: char| c.is_whitespace()) {
        let exec = strip_surrounding_quotes(trimmed[..pos].trim());
        let rest = trimmed[pos..].trim();
        return (
            exec,
            if rest.is_empty() {
                None
            } else {
                Some(rest.to_string())
            },
        );
    }

    (strip_surrounding_quotes(trimmed), None)
}

#[cfg(windows)]
enum WindowFilter {
    Title(String),
    Exe(String),
    TitleAndExe { title: String, exe: String },
}

#[cfg(windows)]
fn build_window_filter(entry: &Entry) -> WindowFilter {
    let raw = entry.hotkey_filter.as_deref().unwrap_or("").trim();
    if !raw.is_empty() {
        if let Some((title, exe)) = split_window_filter_parts(raw) {
            return match (title, exe) {
                (Some(title), Some(exe)) => WindowFilter::TitleAndExe { title, exe },
                (Some(title), None) => WindowFilter::Title(title),
                (None, Some(exe)) => WindowFilter::Exe(exe),
                (None, None) => WindowFilter::Title(raw.to_string()),
            };
        }
        return WindowFilter::Title(raw.to_string());
    }

    let (target, _) = split_hotkey_target(&entry.target);
    let resolved = which::which(&target).ok();
    let exe_name = resolved
        .as_ref()
        .and_then(|path| path.file_name())
        .or_else(|| Path::new(&target).file_name())
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| target);
    WindowFilter::Exe(exe_name)
}

#[cfg(windows)]
fn split_window_filter_parts(raw: &str) -> Option<(Option<String>, Option<String>)> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }

    let lower = trimmed.to_lowercase();
    let token = "ahk_exe";
    for (index, _) in lower.match_indices(token) {
        let before = &trimmed[..index];
        let after_start = index + token.len();
        let after = &trimmed[after_start..];
        let has_left_boundary = before.is_empty()
            || before
                .chars()
                .last()
                .map(|ch| ch.is_whitespace())
                .unwrap_or(false);
        let has_right_boundary = after.is_empty()
            || after
                .chars()
                .next()
                .map(|ch| ch.is_whitespace())
                .unwrap_or(false);

        if !has_left_boundary || !has_right_boundary {
            continue;
        }

        let title = before.trim();
        let exe = after.trim();
        if exe.is_empty() {
            return Some((Some(trimmed.to_string()), None));
        }

        return Some((
            (!title.is_empty()).then(|| title.to_string()),
            Some(exe.to_string()),
        ));
    }

    Some((Some(trimmed.to_string()), None))
}

#[cfg(windows)]
fn find_window(filter: &WindowFilter, detect_hidden: bool) -> Option<HWND> {
    let mut data = WindowSearch {
        filter,
        detect_hidden,
        hwnd: None,
    };

    unsafe {
        EnumWindows(Some(enum_windows_proc), &mut data as *mut _ as LPARAM);
    }

    data.hwnd
}

#[cfg(windows)]
fn wait_for_window(filter: &WindowFilter, detect_hidden: bool, timeout: Duration) -> Option<HWND> {
    let start = Instant::now();
    loop {
        if let Some(hwnd) = find_window(filter, detect_hidden) {
            return Some(hwnd);
        }
        if start.elapsed() >= timeout {
            return None;
        }
        std::thread::sleep(Duration::from_millis(200));
    }
}

#[cfg(windows)]
struct WindowSearch<'a> {
    filter: &'a WindowFilter,
    detect_hidden: bool,
    hwnd: Option<HWND>,
}

#[cfg(windows)]
unsafe extern "system" fn enum_windows_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let data = &mut *(lparam as *mut WindowSearch);

    if !data.detect_hidden && IsWindowVisible(hwnd) == 0 {
        return 1;
    }

    let matched = match data.filter {
        WindowFilter::Title(title) => match_window_title(hwnd, title),
        WindowFilter::Exe(exe) => match_window_exe(hwnd, exe),
        WindowFilter::TitleAndExe { title, exe } => {
            match_window_title(hwnd, title) && match_window_exe(hwnd, exe)
        }
    };

    if matched {
        data.hwnd = Some(hwnd);
        0
    } else {
        1
    }
}

#[cfg(windows)]
fn match_window_title(hwnd: HWND, title: &str) -> bool {
    let target = title.trim();
    if target.is_empty() {
        return false;
    }
    let window_title = get_window_title(hwnd);
    window_title
        .map(|text| text.to_lowercase().contains(&target.to_lowercase()))
        .unwrap_or(false)
}

#[cfg(windows)]
fn match_window_exe(hwnd: HWND, exe: &str) -> bool {
    let target = exe.trim();
    if target.is_empty() {
        return false;
    }
    let normalized_target = Path::new(target)
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| target.to_string());
    let normalized_lower = normalized_target.to_lowercase();
    let exe_lower = if normalized_lower.ends_with(".exe") {
        None
    } else {
        Some(format!("{}.exe", normalized_lower))
    };
    let process_exe = get_window_process_exe(hwnd);
    process_exe
        .map(|text| {
            let process_lower = text.to_lowercase();
            process_lower == normalized_lower
                || exe_lower
                    .as_ref()
                    .map(|exe| process_lower == *exe)
                    .unwrap_or(false)
        })
        .unwrap_or(false)
}

#[cfg(windows)]
fn get_window_title(hwnd: HWND) -> Option<String> {
    unsafe {
        let length = GetWindowTextLengthW(hwnd);
        if length == 0 {
            return None;
        }
        let mut buffer = vec![0u16; (length + 1) as usize];
        let read = GetWindowTextW(hwnd, buffer.as_mut_ptr(), buffer.len() as i32);
        if read == 0 {
            return None;
        }
        Some(
            OsString::from_wide(&buffer[..read as usize])
                .to_string_lossy()
                .to_string(),
        )
    }
}

#[cfg(windows)]
fn get_window_process_exe(hwnd: HWND) -> Option<String> {
    unsafe {
        let mut pid: DWORD = 0;
        GetWindowThreadProcessId(hwnd, &mut pid);
        if pid == 0 {
            return None;
        }

        let handle = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, 0, pid);
        if handle.is_null() {
            return None;
        }

        let mut buffer = vec![0u16; 260];
        let len = GetModuleFileNameExW(
            handle,
            ptr::null_mut(),
            buffer.as_mut_ptr(),
            buffer.len() as DWORD,
        );
        CloseHandle(handle);
        if len == 0 {
            return None;
        }

        let path = OsString::from_wide(&buffer[..len as usize])
            .to_string_lossy()
            .to_string();
        Path::new(&path)
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
    }
}

#[cfg(windows)]
fn restore_and_focus(hwnd: HWND) {
    unsafe {
        ShowWindow(hwnd, SW_RESTORE);
        SetForegroundWindow(hwnd);
        BringWindowToTop(hwnd);
    }
}

#[cfg(windows)]
fn apply_position(hwnd: HWND, position: &str) -> ExecutorResult<()> {
    unsafe {
        let mut rect: RECT = std::mem::zeroed();
        SystemParametersInfoW(SPI_GETWORKAREA, 0, &mut rect as *mut _ as *mut _, 0);
        let left = rect.left;
        let top = rect.top;
        let width = rect.right - rect.left;
        let height = rect.bottom - rect.top;

        match position {
            "left" => {
                MoveWindow(hwnd, left, top, width / 2, height, 1);
            }
            "right" => {
                MoveWindow(hwnd, left + width / 2, top, width / 2, height, 1);
            }
            "max" => {
                ShowWindow(hwnd, SW_MAXIMIZE);
            }
            "keep" => {}
            _ => {
                ShowWindow(hwnd, SW_MAXIMIZE);
            }
        }
    }

    Ok(())
}

/// Execute Windows shortcut (.lnk file)
#[cfg(windows)]
fn execute_shortcut(entry: &Entry) -> ExecutorResult<()> {
    let path = std::path::Path::new(&entry.target);
    if !path.exists() {
        return Err(ExecutorError::TargetNotFound(entry.target.clone()));
    }

    // Try to parse .lnk file
    match lnk::ShellLink::open(path) {
        Ok(lnk) => {
            if let Some(target_path) = lnk.link_info() {
                if let Some(local_base_path) = target_path.local_base_path() {
                    let mut cmd = Command::new(local_base_path);
                    if let Some(args) = lnk.arguments() {
                        let args_vec: Vec<&str> = args.split_whitespace().collect();
                        cmd.args(&args_vec);
                    }
                    if let Some(workdir) = lnk.working_dir() {
                        cmd.current_dir(workdir);
                    }
                    cmd.spawn()?;
                    return Ok(());
                }
            }
            // Fallback: just open the .lnk file
            open::that(&entry.target)?;
            Ok(())
        }
        Err(_) => {
            // Fallback: let Windows handle it
            open::that(&entry.target)?;
            Ok(())
        }
    }
}

#[cfg(not(windows))]
fn execute_shortcut(entry: &Entry) -> ExecutorResult<()> {
    // On non-Windows, just try to open the file
    let path = std::path::Path::new(&entry.target);
    if !path.exists() {
        return Err(ExecutorError::TargetNotFound(entry.target.clone()));
    }
    open::that(&entry.target)?;
    Ok(())
}

/// Execute AutoHotkey script
#[cfg(windows)]
fn resolve_ahk_exe(ahk_path: Option<&str>) -> Option<PathBuf> {
    if let Some(path) = ahk_path {
        let candidate = std::path::Path::new(path);
        if candidate.exists() {
            return Some(candidate.to_path_buf());
        }
        if let Ok(found) = which::which(path) {
            return Some(found);
        }
    } else if let Ok(found) = which::which("AutoHotkey.exe") {
        return Some(found);
    }

    let default_dir = std::path::Path::new(r"C:\Program Files\AutoHotkey\v2");
    for exe_name in ["AutoHotkey.exe", "AutoHotkey64.exe", "AutoHotkey32.exe"] {
        let candidate = default_dir.join(exe_name);
        if candidate.exists() {
            return Some(candidate);
        }
    }

    None
}

/// Execute AutoHotkey script
#[cfg(windows)]
fn execute_ahk(entry: &Entry, ahk_path: Option<&str>) -> ExecutorResult<()> {
    let ahk_exe = resolve_ahk_exe(ahk_path).ok_or(ExecutorError::AhkNotFound)?;

    let path = std::path::Path::new(&entry.target);

    let mut cmd = if path.exists() && path.extension().map(|e| e == "ahk").unwrap_or(false) {
        // Execute existing .ahk file
        let mut c = Command::new(&ahk_exe);
        c.arg(&entry.target);
        c
    } else {
        // Write content to temp file and execute
        let temp_dir = std::env::temp_dir();
        let script_path = temp_dir.join(format!("opener_ahk_{}.ahk", uuid::Uuid::new_v4()));
        std::fs::write(&script_path, &entry.target)?;

        let mut c = Command::new(&ahk_exe);
        c.arg(&script_path);
        c
    };

    if let Some(ref workdir) = entry.workdir {
        cmd.current_dir(workdir);
    }

    cmd.spawn()?;
    Ok(())
}

#[cfg(not(windows))]
fn execute_ahk(_entry: &Entry, _ahk_path: Option<&str>) -> ExecutorResult<()> {
    Err(ExecutorError::UnsupportedPlatform(
        "AutoHotkey is only available on Windows".to_string(),
    ))
}

/// Render template variables into content
pub fn render_template(
    template: &str,
    variables: &std::collections::HashMap<String, String>,
) -> String {
    let mut result = template.to_string();
    for (key, value) in variables {
        let placeholder = format!("{{{{{}}}}}", key);
        result = result.replace(&placeholder, value);
    }
    result
}

fn expand_env_placeholders(value: &str) -> String {
    let mut result = String::with_capacity(value.len());
    let mut rest = value;

    loop {
        let Some(start) = rest.find("${") else {
            result.push_str(rest);
            break;
        };

        result.push_str(&rest[..start]);
        let after = &rest[start + 2..];
        let Some(end) = after.find('}') else {
            result.push_str("${");
            result.push_str(after);
            break;
        };

        let name = &after[..end];
        if name.is_empty() {
            result.push_str("${}");
        } else if let Ok(value) = std::env::var(name) {
            result.push_str(&value);
        } else {
            result.push_str("${");
            result.push_str(name);
            result.push('}');
        }

        rest = &after[end + 1..];
    }

    result
}

fn expand_optional(value: &Option<String>) -> Option<String> {
    value.as_ref().map(|val| expand_env_placeholders(val))
}

pub fn resolve_entry_env(entry: &Entry) -> Entry {
    let mut resolved = entry.clone();
    let expand_target = matches!(
        entry.entry_type,
        EntryType::App
            | EntryType::File
            | EntryType::Dir
            | EntryType::Cmd
            | EntryType::Wsl
            | EntryType::Script
            | EntryType::Shortcut
            | EntryType::Ahk
            | EntryType::HotkeyApp
    );

    if expand_target {
        resolved.target = expand_env_placeholders(&entry.target);
    }

    resolved.workdir = expand_optional(&entry.workdir);
    resolved.ssh_host = expand_optional(&entry.ssh_host);
    resolved.ssh_user = expand_optional(&entry.ssh_user);
    resolved.hotkey_filter = expand_optional(&entry.hotkey_filter);
    resolved
}

/// Get execution preview (what will be run)
pub fn get_execution_preview(entry: &Entry) -> String {
    match entry.entry_type {
        EntryType::Url => format!("Open URL: {}", entry.target),
        EntryType::File => format!("Open file: {}", entry.target),
        EntryType::Dir => format!("Open directory: {}", entry.target),
        EntryType::App => {
            let args = entry
                .args
                .as_ref()
                .map(|a| format!(" {}", a))
                .unwrap_or_default();
            format!("Run: {}{}", entry.target, args)
        }
        EntryType::Cmd => format!("Execute command: {}", entry.target),
        EntryType::Wsl => {
            let distro = entry
                .wsl_distro
                .as_ref()
                .map(|d| format!(" -d {}", d))
                .unwrap_or_default();
            format!("WSL{}: {}", distro, entry.target)
        }
        EntryType::Ssh => {
            let host = entry
                .ssh_host
                .as_ref()
                .map(|h| h.as_str())
                .unwrap_or("unknown");
            let user = entry
                .ssh_user
                .as_ref()
                .map(|u| u.as_str())
                .unwrap_or("root");
            let port = entry.ssh_port.unwrap_or(22);
            format!("SSH: {}@{}:{}", user, host, port)
        }
        EntryType::Script => {
            let target = entry.target.trim();
            if !target.is_empty() {
                format!("Execute script file: {}", target)
            } else {
                let content = entry.script_content.as_deref().unwrap_or("");
                let script_type = entry.script_type.as_deref().unwrap_or("powershell");
                let preview = content.lines().next().unwrap_or("(empty script)");
                format!("Execute {} script: {}...", script_type, preview)
            }
        }
        EntryType::Shortcut => format!("Open shortcut: {}", entry.target),
        EntryType::Ahk => {
            let preview = entry.target.lines().next().unwrap_or("(empty script)");
            format!("Run AHK: {}...", preview)
        }
        EntryType::HotkeyApp => {
            let name = if entry.name.is_empty() {
                "HotKey应用"
            } else {
                entry.name.as_str()
            };
            format!("HotKey应用: {}", name)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_template() {
        let template = "cd {{path}} && {{command}}";
        let mut vars = std::collections::HashMap::new();
        vars.insert("path".to_string(), "/home/user/project".to_string());
        vars.insert("command".to_string(), "npm start".to_string());

        let result = render_template(template, &vars);
        assert_eq!(result, "cd /home/user/project && npm start");
    }

    #[test]
    fn test_execution_preview() {
        let mut entry = Entry::new(
            "Test".to_string(),
            EntryType::Url,
            "https://example.com".to_string(),
        );

        let preview = get_execution_preview(&entry);
        assert!(preview.contains("https://example.com"));

        entry.entry_type = EntryType::Ssh;
        entry.ssh_host = Some("server.example.com".to_string());
        entry.ssh_user = Some("admin".to_string());
        entry.ssh_port = Some(2222);

        let preview = get_execution_preview(&entry);
        assert!(preview.contains("admin@server.example.com:2222"));
    }

    #[cfg(windows)]
    #[test]
    fn test_build_window_filter_keeps_combined_title_and_exe_selector() {
        let mut entry = Entry::new(
            "Test".to_string(),
            EntryType::HotkeyApp,
            "C:\\Program Files\\WebContainer\\webcontainer.exe".to_string(),
        );
        entry.hotkey_filter = Some("OpenAI-ChatGPT-Web ahk_exe webcontainer.exe".to_string());

        match build_window_filter(&entry) {
            WindowFilter::TitleAndExe { title, exe } => {
                assert_eq!(title, "OpenAI-ChatGPT-Web");
                assert_eq!(exe, "webcontainer.exe");
            }
            WindowFilter::Title(_) => {
                panic!("combined title + ahk_exe selector was parsed as title-only")
            }
            WindowFilter::Exe(_) => {
                panic!("combined title + ahk_exe selector was parsed as exe-only")
            }
        }
    }
}
