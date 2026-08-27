use std::path::Path;
#[cfg(windows)]
use std::path::PathBuf;

pub fn shell_single_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

pub fn hook_command(hook_path: &Path, action: Option<&str>) -> String {
    let path = hook_path.display().to_string();
    #[cfg(windows)]
    {
        let mut command = format!(
            "powershell -NoProfile -ExecutionPolicy Bypass -File {}",
            windows_command_quote(&path)
        );
        if let Some(action) = action {
            command.push(' ');
            command.push_str(action);
        }
        command
    }

    #[cfg(not(windows))]
    {
        let mut command = format!("bash {}", shell_single_quote(&path));
        if let Some(action) = action {
            command.push(' ');
            command.push_str(action);
        }
        command
    }
}

pub(crate) fn legacy_bash_hook_command(hook_path: &Path, action: Option<&str>) -> String {
    let mut command = format!(
        "bash {}",
        shell_single_quote(&hook_path.display().to_string())
    );
    if let Some(action) = action {
        command.push(' ');
        command.push_str(action);
    }
    command
}

#[cfg(windows)]
pub(crate) fn legacy_bash_hook_path(hook_path: &Path) -> PathBuf {
    hook_path.with_file_name("herdr-agent-state.sh")
}

#[cfg(windows)]
fn windows_command_quote(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\\\""))
}

#[cfg(not(windows))]
pub(crate) fn grok_hook_command(hook_path: &Path) -> String {
    format!(
        "sh {} session",
        shell_single_quote(&hook_path.display().to_string())
    )
}

#[cfg(windows)]
pub(crate) fn grok_hook_command(hook_path: &Path) -> String {
    hook_command(hook_path, Some("session"))
}

pub fn mastracode_hook_command(hook_path: &Path, action: &str) -> String {
    #[cfg(windows)]
    {
        use base64::Engine;

        let path = hook_path.display().to_string().replace('\'', "''");
        let script = format!("& '{path}' {action}");
        let encoded_script = script
            .encode_utf16()
            .flat_map(u16::to_le_bytes)
            .collect::<Vec<_>>();
        let encoded = base64::engine::general_purpose::STANDARD.encode(encoded_script);
        format!("powershell -NoProfile -ExecutionPolicy Bypass -EncodedCommand {encoded}")
    }
    #[cfg(not(windows))]
    {
        hook_command(hook_path, Some(action))
    }
}
