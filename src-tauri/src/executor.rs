// Entry executor module - handles execution of different entry types

use crate::models::{Entry, EntryType};
use std::process::Command;
use thiserror::Error;

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
        let terminal = std::env::var("TERMINAL")
            .unwrap_or_else(|_| "x-terminal-emulator".to_string());
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
    let host = entry.ssh_host.as_ref().ok_or_else(|| {
        ExecutorError::ExecutionFailed("SSH host not configured".to_string())
    })?;

    let user = entry.ssh_user.as_ref().map(|s| s.as_str()).unwrap_or("root");
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
            let terminal = std::env::var("TERMINAL")
                .unwrap_or_else(|_| "x-terminal-emulator".to_string());
            let ssh_cmd = format!("ssh {}", ssh_args.join(" "));
            Command::new(&terminal)
                .args(["-e", "sh", "-c", &ssh_cmd])
                .spawn()?;
        } else {
            Command::new("ssh")
                .args(&ssh_args)
                .spawn()?;
        }
    }

    Ok(())
}

/// Execute script
fn execute_script(entry: &Entry) -> ExecutorResult<()> {
    // Script content is in target field
    // Determine execution method based on content or extension

    #[cfg(windows)]
    {
        let show_terminal = entry.show_terminal.unwrap_or(false);

        // Write script to temp file
        let temp_dir = std::env::temp_dir();
        let script_path = temp_dir.join(format!("opener_script_{}.ps1", uuid::Uuid::new_v4()));
        std::fs::write(&script_path, &entry.target)?;

        let mut cmd = if show_terminal {
            let mut c = Command::new("cmd");
            let ps_cmd = format!("powershell -ExecutionPolicy Bypass -File \"{}\"", script_path.display());
            c.args(["/c", "start", "cmd", "/k", &ps_cmd]);
            c
        } else {
            let mut c = Command::new("powershell");
            c.args(["-ExecutionPolicy", "Bypass", "-File", &script_path.to_string_lossy()]);
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
        let show_terminal = entry.show_terminal.unwrap_or(false);

        // Write script to temp file
        let temp_dir = std::env::temp_dir();
        let script_path = temp_dir.join(format!("opener_script_{}.sh", uuid::Uuid::new_v4()));
        std::fs::write(&script_path, &entry.target)?;

        // Make executable
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&script_path)?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&script_path, perms)?;

        if show_terminal {
            let terminal = std::env::var("TERMINAL")
                .unwrap_or_else(|_| "x-terminal-emulator".to_string());
            Command::new(&terminal)
                .args(["-e", &script_path.to_string_lossy()])
                .spawn()?;
        } else {
            let mut cmd = Command::new("sh");
            cmd.arg(&script_path);
            if let Some(ref workdir) = entry.workdir {
                cmd.current_dir(workdir);
            }
            cmd.spawn()?;
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
fn execute_ahk(entry: &Entry, ahk_path: Option<&str>) -> ExecutorResult<()> {
    let ahk_exe = ahk_path.unwrap_or("AutoHotkey.exe");

    // Check if AHK exists in PATH or at specified location
    let ahk_found = std::path::Path::new(ahk_exe).exists() ||
        which::which(ahk_exe).is_ok();

    if !ahk_found {
        return Err(ExecutorError::AhkNotFound);
    }

    let path = std::path::Path::new(&entry.target);

    let mut cmd = if path.exists() && path.extension().map(|e| e == "ahk").unwrap_or(false) {
        // Execute existing .ahk file
        let mut c = Command::new(ahk_exe);
        c.arg(&entry.target);
        c
    } else {
        // Write content to temp file and execute
        let temp_dir = std::env::temp_dir();
        let script_path = temp_dir.join(format!("opener_ahk_{}.ahk", uuid::Uuid::new_v4()));
        std::fs::write(&script_path, &entry.target)?;

        let mut c = Command::new(ahk_exe);
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
pub fn render_template(template: &str, variables: &std::collections::HashMap<String, String>) -> String {
    let mut result = template.to_string();
    for (key, value) in variables {
        let placeholder = format!("{{{{{}}}}}", key);
        result = result.replace(&placeholder, value);
    }
    result
}

/// Get execution preview (what will be run)
pub fn get_execution_preview(entry: &Entry) -> String {
    match entry.entry_type {
        EntryType::Url => format!("Open URL: {}", entry.target),
        EntryType::File => format!("Open file: {}", entry.target),
        EntryType::Dir => format!("Open directory: {}", entry.target),
        EntryType::App => {
            let args = entry.args.as_ref().map(|a| format!(" {}", a)).unwrap_or_default();
            format!("Run: {}{}", entry.target, args)
        }
        EntryType::Cmd => format!("Execute command: {}", entry.target),
        EntryType::Wsl => {
            let distro = entry.wsl_distro.as_ref().map(|d| format!(" -d {}", d)).unwrap_or_default();
            format!("WSL{}: {}", distro, entry.target)
        }
        EntryType::Ssh => {
            let host = entry.ssh_host.as_ref().map(|h| h.as_str()).unwrap_or("unknown");
            let user = entry.ssh_user.as_ref().map(|u| u.as_str()).unwrap_or("root");
            let port = entry.ssh_port.unwrap_or(22);
            format!("SSH: {}@{}:{}", user, host, port)
        }
        EntryType::Script => {
            let preview = entry.target.lines().next().unwrap_or("(empty script)");
            format!("Execute script: {}...", preview)
        }
        EntryType::Shortcut => format!("Open shortcut: {}", entry.target),
        EntryType::Ahk => {
            let preview = entry.target.lines().next().unwrap_or("(empty script)");
            format!("Run AHK: {}...", preview)
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
}
