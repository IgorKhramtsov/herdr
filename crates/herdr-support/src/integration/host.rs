use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde_json::{json, Map, Value};

use super::claude_settings::{
    install as install_claude_settings, uninstall as uninstall_claude_settings,
};
use super::command::{grok_hook_command, hook_command, mastracode_hook_command};
use super::config_edit::{
    build_codex_config_with_hooks, build_kimi_config_with_hooks, ensure_command_hook,
    ensure_direct_command_hook, ensure_flat_command_hook, ensure_hermes_plugin_enabled,
    ensure_hooks_object, ensure_simple_command_hook, hooks_object_if_present,
    is_matching_command_hook, is_matching_direct_command_entry, remove_direct_hook_commands,
    remove_flat_command_hook, remove_hermes_plugin_enabled, remove_hook_commands,
    remove_kimi_config_block, remove_simple_command_hook,
};
use super::consts::{
    ANTIGRAVITY_CLI_HOOK_BLOCK_NAME, ANTIGRAVITY_CLI_HOOK_EVENTS, ANTIGRAVITY_CLI_HOOK_TIMEOUT_SEC,
    COPILOT_HOOK_EVENTS, COPILOT_REMOVED_LIFECYCLE_HOOK_EVENTS, DEVIN_HOOK_EVENTS,
    DEVIN_REMOVED_LIFECYCLE_HOOK_EVENTS, DROID_HOOK_EVENTS, DROID_REMOVED_LIFECYCLE_HOOK_EVENTS,
    GROK_HOOK_CONFIG_INSTALL_NAME, KIMI_CONFIG_BLOCK_BEGIN, KIMI_HOOK_EVENTS,
    MASTRACODE_HOOK_EVENTS, MASTRACODE_HOOK_TIMEOUT_MS, MASTRACODE_REMOVED_HOOK_EVENTS,
    OPENCODE_TUI_PLUGIN_SPEC, QODERCLI_HOOK_EVENTS, QODERCLI_REMOVED_LIFECYCLE_HOOK_EVENTS,
    QWEN_HOOK_EVENTS,
};
use super::layout::IntegrationLayout;
use super::opencode_config::{
    add_tui_plugin, remove_tui_plugin, tui_config_path, tui_plugin_is_configured,
    validate_tui_plugin_config,
};
use super::state::IntegrationFileState;
use super::{IntegrationFileStatus, IntegrationTarget};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostConfigRole {
    Settings,
    Hooks,
    Config,
    TuiConfig,
    LegacyHooks,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostConfigChange {
    pub path: PathBuf,
    pub role: HostConfigRole,
    pub changed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HostInstallResult {
    pub changes: Vec<HostConfigChange>,
    pub extras: Vec<String>,
}

pub fn grok_hook_config(hook_path: &Path) -> Value {
    let session_command = grok_hook_command(hook_path);
    json!({
        "hooks": {
            "SessionStart": [
                {
                    "hooks": [
                        {
                            "type": "command",
                            "command": session_command,
                            "timeout": 10,
                        }
                    ]
                }
            ]
        }
    })
}

pub(crate) fn host_status(layout: &IntegrationLayout) -> Vec<IntegrationFileStatus> {
    match layout.target {
        IntegrationTarget::Pi | IntegrationTarget::Omp | IntegrationTarget::Kilo => Vec::new(),
        IntegrationTarget::Claude => vec![merge_nested_status(
            &layout.root.join("settings.json"),
            &layout.files[0].path,
            &[("SessionStart", "session")],
        )],
        IntegrationTarget::Codex => vec![merge_nested_status(
            &layout.root.join("hooks.json"),
            &layout.files[0].path,
            &[("SessionStart", "session")],
        )],
        IntegrationTarget::Copilot => vec![merge_direct_status(
            &layout.root.join("settings.json"),
            &layout.files[0].path,
            COPILOT_HOOK_EVENTS.as_slice(),
        )],
        IntegrationTarget::Devin => vec![merge_nested_status(
            &layout.root.join("config.json"),
            &layout.files[0].path,
            &DEVIN_HOOK_EVENTS,
        )],
        IntegrationTarget::Droid => vec![merge_nested_status(
            &layout.root.join("settings.json"),
            &layout.files[0].path,
            &DROID_HOOK_EVENTS,
        )],
        IntegrationTarget::Kimi => vec![kimi_status(
            &layout.root.join("config.toml"),
            &layout.files[0].path,
        )],
        IntegrationTarget::Opencode => vec![opencode_status(&layout.root)],
        IntegrationTarget::Hermes => vec![hermes_status(&layout.root.join("config.yaml"))],
        IntegrationTarget::Qodercli => vec![merge_nested_status(
            &layout.root.join("settings.json"),
            &layout.files[0].path,
            &QODERCLI_HOOK_EVENTS,
        )],
        IntegrationTarget::Qwen => vec![merge_nested_status(
            &layout.root.join("settings.json"),
            &layout.files[0].path,
            &QWEN_HOOK_EVENTS,
        )],
        IntegrationTarget::Cursor => vec![cursor_status(
            &layout.root.join("hooks.json"),
            &layout.files[0].path,
        )],
        IntegrationTarget::Mastracode => vec![mastracode_status(
            &layout.root.join("hooks.json"),
            &layout.files[0].path,
        )],
        IntegrationTarget::AntigravityCli => vec![antigravity_status(
            &layout.root.join("hooks.json"),
            &layout.files[0].path,
        )],
        IntegrationTarget::Grok => vec![grok_status(
            &layout
                .root
                .join("hooks")
                .join(GROK_HOOK_CONFIG_INSTALL_NAME),
            &layout.files[0].path,
        )],
    }
}

pub(crate) fn install_host(layout: &IntegrationLayout) -> io::Result<HostInstallResult> {
    match layout.target {
        IntegrationTarget::Pi | IntegrationTarget::Omp | IntegrationTarget::Kilo => {
            Ok(HostInstallResult::default())
        }
        IntegrationTarget::Claude => install_claude(layout),
        IntegrationTarget::Codex => install_codex(layout),
        IntegrationTarget::Copilot => install_copilot(layout),
        IntegrationTarget::Devin => install_devin(layout),
        IntegrationTarget::Droid => install_droid(layout),
        IntegrationTarget::Kimi => install_kimi(layout),
        IntegrationTarget::Opencode => install_opencode(layout),
        IntegrationTarget::Hermes => install_hermes(layout),
        IntegrationTarget::Qodercli => install_qodercli(layout),
        IntegrationTarget::Qwen => install_qwen(layout),
        IntegrationTarget::Cursor => install_cursor(layout),
        IntegrationTarget::Mastracode => install_mastracode(layout),
        IntegrationTarget::AntigravityCli => install_antigravity(layout),
        IntegrationTarget::Grok => install_grok(layout),
    }
}

pub(crate) fn uninstall_host(
    layout: &IntegrationLayout,
    force: bool,
) -> io::Result<HostInstallResult> {
    match layout.target {
        IntegrationTarget::Pi | IntegrationTarget::Omp | IntegrationTarget::Kilo => {
            let _ = force;
            Ok(HostInstallResult::default())
        }
        IntegrationTarget::Claude => uninstall_claude(layout),
        IntegrationTarget::Codex => uninstall_codex(layout),
        IntegrationTarget::Copilot => uninstall_copilot(layout),
        IntegrationTarget::Devin => uninstall_devin(layout),
        IntegrationTarget::Droid => uninstall_droid(layout),
        IntegrationTarget::Kimi => uninstall_kimi(layout),
        IntegrationTarget::Opencode => uninstall_opencode(layout),
        IntegrationTarget::Hermes => uninstall_hermes(layout),
        IntegrationTarget::Qodercli => uninstall_qodercli(layout),
        IntegrationTarget::Qwen => uninstall_qwen(layout),
        IntegrationTarget::Cursor => uninstall_cursor(layout),
        IntegrationTarget::Mastracode => uninstall_mastracode(layout),
        IntegrationTarget::AntigravityCli => uninstall_antigravity(layout),
        IntegrationTarget::Grok => uninstall_grok(layout, force),
    }
}

fn hook_path(layout: &IntegrationLayout) -> &Path {
    &layout.files[0].path
}

fn change(path: PathBuf, role: HostConfigRole, changed: bool) -> HostConfigChange {
    HostConfigChange {
        path,
        role,
        changed,
    }
}

fn install_claude(layout: &IntegrationLayout) -> io::Result<HostInstallResult> {
    let settings_path = layout.root.join("settings.json");
    let existing = read_or(settings_path.as_path(), "{}")?;
    let updated = install_claude_settings(&existing, &settings_path, hook_path(layout))?;
    let changed = updated != existing;
    if changed {
        fs::write(&settings_path, updated)?;
    }
    Ok(HostInstallResult {
        changes: vec![change(settings_path, HostConfigRole::Settings, changed)],
        extras: Vec::new(),
    })
}

fn uninstall_claude(layout: &IntegrationLayout) -> io::Result<HostInstallResult> {
    let settings_path = layout.root.join("settings.json");
    let mut changed = false;
    if settings_path.is_file() {
        let existing = fs::read_to_string(&settings_path)?;
        let updated = uninstall_claude_settings(&existing, &settings_path, hook_path(layout))?;
        changed = updated != existing;
        if changed {
            fs::write(&settings_path, updated)?;
        }
    }
    Ok(HostInstallResult {
        changes: vec![change(settings_path, HostConfigRole::Settings, changed)],
        extras: Vec::new(),
    })
}

fn install_codex(layout: &IntegrationLayout) -> io::Result<HostInstallResult> {
    let hook_path = hook_path(layout);
    let hooks_path = layout.root.join("hooks.json");
    let mut hooks_file = load_json_object(&hooks_path, json!({}))?;
    let hooks = ensure_hooks_object(
        &mut hooks_file,
        &hooks_path,
        "codex hooks file",
        "codex hooks file hooks",
    )?;
    remove_hook_commands(hooks, "PermissionRequest", hook_path, Some("blocked"))?;
    remove_hook_commands(hooks, "SessionStart", hook_path, Some("idle"))?;
    remove_hook_commands(hooks, "UserPromptSubmit", hook_path, Some("working"))?;
    remove_hook_commands(hooks, "PreToolUse", hook_path, Some("working"))?;
    remove_hook_commands(hooks, "Stop", hook_path, Some("idle"))?;
    remove_hook_commands(hooks, "SessionStart", hook_path, Some("session"))?;
    ensure_command_hook(
        hooks,
        "SessionStart",
        hook_command(hook_path, Some("session")),
        10,
        None,
    )?;
    write_pretty_json(&hooks_path, &hooks_file)?;

    let config_path = layout.root.join("config.toml");
    let existing_config = read_or(&config_path, "")?;
    let new_config = build_codex_config_with_hooks(&existing_config);
    let config_changed = new_config != existing_config;
    if config_changed {
        fs::write(&config_path, new_config)?;
    }
    Ok(HostInstallResult {
        changes: vec![
            change(hooks_path, HostConfigRole::Hooks, true),
            change(config_path, HostConfigRole::Config, config_changed),
        ],
        extras: Vec::new(),
    })
}

fn uninstall_codex(layout: &IntegrationLayout) -> io::Result<HostInstallResult> {
    let hook_path = hook_path(layout);
    let hooks_path = layout.root.join("hooks.json");
    let config_path = layout.root.join("config.toml");
    let mut updated_hooks = false;
    if hooks_path.is_file() {
        let mut hooks_file = load_json_object(&hooks_path, json!({}))?;
        if let Some(hooks) = hooks_object_if_present(
            &mut hooks_file,
            &hooks_path,
            "codex hooks file",
            "codex hooks file hooks",
        )? {
            updated_hooks |= remove_hook_commands(hooks, "SessionStart", hook_path, Some("idle"))?;
            updated_hooks |=
                remove_hook_commands(hooks, "SessionStart", hook_path, Some("session"))?;
            updated_hooks |=
                remove_hook_commands(hooks, "UserPromptSubmit", hook_path, Some("working"))?;
            updated_hooks |= remove_hook_commands(hooks, "PreToolUse", hook_path, Some("working"))?;
            updated_hooks |=
                remove_hook_commands(hooks, "PermissionRequest", hook_path, Some("blocked"))?;
            updated_hooks |= remove_hook_commands(hooks, "Stop", hook_path, Some("idle"))?;
        }
        if updated_hooks {
            write_pretty_json(&hooks_path, &hooks_file)?;
        }
    }
    Ok(HostInstallResult {
        changes: vec![
            change(hooks_path, HostConfigRole::Hooks, updated_hooks),
            change(config_path, HostConfigRole::Config, false),
        ],
        extras: Vec::new(),
    })
}

fn install_kimi(layout: &IntegrationLayout) -> io::Result<HostInstallResult> {
    let config_path = layout.root.join("config.toml");
    let existing = read_or(&config_path, "")?;
    let new_config = build_kimi_config_with_hooks(&existing, hook_path(layout));
    let changed = new_config != existing;
    if changed {
        fs::write(&config_path, new_config)?;
    }
    Ok(HostInstallResult {
        changes: vec![change(config_path, HostConfigRole::Config, changed)],
        extras: Vec::new(),
    })
}

fn uninstall_kimi(layout: &IntegrationLayout) -> io::Result<HostInstallResult> {
    let config_path = layout.root.join("config.toml");
    let mut changed = false;
    if config_path.is_file() {
        let existing = fs::read_to_string(&config_path)?;
        let new_config = remove_kimi_config_block(&existing);
        if new_config != existing {
            fs::write(&config_path, new_config)?;
            changed = true;
        }
    }
    Ok(HostInstallResult {
        changes: vec![change(config_path, HostConfigRole::Config, changed)],
        extras: Vec::new(),
    })
}

fn install_copilot(layout: &IntegrationLayout) -> io::Result<HostInstallResult> {
    let hook_path = hook_path(layout);
    let settings_path = layout.root.join("settings.json");
    let mut settings = load_json_object(&settings_path, json!({}))?;
    let hooks = ensure_hooks_object(
        &mut settings,
        &settings_path,
        "copilot settings",
        "copilot settings hooks",
    )?;
    let command = hook_command(hook_path, None);
    for event in COPILOT_REMOVED_LIFECYCLE_HOOK_EVENTS {
        remove_direct_hook_commands(hooks, event, hook_path, None)?;
    }
    for event in COPILOT_HOOK_EVENTS {
        remove_direct_hook_commands(hooks, event, hook_path, None)?;
    }
    for event in COPILOT_HOOK_EVENTS {
        ensure_direct_command_hook(hooks, event, command.clone(), 10, None)?;
    }
    write_pretty_json(&settings_path, &settings)?;
    Ok(HostInstallResult {
        changes: vec![change(settings_path, HostConfigRole::Settings, true)],
        extras: Vec::new(),
    })
}

fn uninstall_copilot(layout: &IntegrationLayout) -> io::Result<HostInstallResult> {
    let hook_path = hook_path(layout);
    let settings_path = layout.root.join("settings.json");
    let mut changed = false;
    if settings_path.is_file() {
        let mut settings = load_json_object(&settings_path, json!({}))?;
        if let Some(hooks) = hooks_object_if_present(
            &mut settings,
            &settings_path,
            "copilot settings",
            "copilot settings hooks",
        )? {
            for event in COPILOT_HOOK_EVENTS {
                changed |= remove_direct_hook_commands(hooks, event, hook_path, None)?;
            }
            for event in COPILOT_REMOVED_LIFECYCLE_HOOK_EVENTS {
                changed |= remove_direct_hook_commands(hooks, event, hook_path, None)?;
            }
        }
        if changed {
            write_pretty_json(&settings_path, &settings)?;
        }
    }
    Ok(HostInstallResult {
        changes: vec![change(settings_path, HostConfigRole::Settings, changed)],
        extras: Vec::new(),
    })
}

fn install_devin(layout: &IntegrationLayout) -> io::Result<HostInstallResult> {
    let hook_path = hook_path(layout);
    let settings_path = layout.root.join("config.json");
    let mut settings = load_json_object(&settings_path, json!({}))?;
    let hooks = ensure_hooks_object(
        &mut settings,
        &settings_path,
        "devin settings",
        "devin settings hooks",
    )?;
    for (event, action) in DEVIN_REMOVED_LIFECYCLE_HOOK_EVENTS {
        remove_hook_commands(hooks, event, hook_path, Some(action))?;
    }
    for (event, action) in DEVIN_HOOK_EVENTS {
        remove_hook_commands(hooks, event, hook_path, Some(action))?;
    }
    for (event, action) in DEVIN_HOOK_EVENTS {
        ensure_command_hook(
            hooks,
            event,
            hook_command(hook_path, Some(action)),
            10,
            None,
        )?;
    }
    write_pretty_json(&settings_path, &settings)?;
    Ok(HostInstallResult {
        changes: vec![change(settings_path, HostConfigRole::Settings, true)],
        extras: Vec::new(),
    })
}

fn uninstall_devin(layout: &IntegrationLayout) -> io::Result<HostInstallResult> {
    let hook_path = hook_path(layout);
    let settings_path = layout.root.join("config.json");
    let mut changed = false;
    if settings_path.is_file() {
        let mut settings = load_json_object(&settings_path, json!({}))?;
        if let Some(hooks) = hooks_object_if_present(
            &mut settings,
            &settings_path,
            "devin settings",
            "devin settings hooks",
        )? {
            for (event, action) in DEVIN_REMOVED_LIFECYCLE_HOOK_EVENTS {
                changed |= remove_hook_commands(hooks, event, hook_path, Some(action))?;
            }
            for (event, action) in DEVIN_HOOK_EVENTS {
                changed |= remove_hook_commands(hooks, event, hook_path, Some(action))?;
            }
        }
        if changed {
            write_pretty_json(&settings_path, &settings)?;
        }
    }
    Ok(HostInstallResult {
        changes: vec![change(settings_path, HostConfigRole::Settings, changed)],
        extras: Vec::new(),
    })
}

fn install_droid(layout: &IntegrationLayout) -> io::Result<HostInstallResult> {
    let hook_path = hook_path(layout);
    let settings_path = layout.root.join("settings.json");
    let mut settings = load_json_object(&settings_path, json!({}))?;
    let hooks = ensure_hooks_object(
        &mut settings,
        &settings_path,
        "droid settings",
        "droid settings hooks",
    )?;
    remove_hook_commands(hooks, "SessionStart", hook_path, None)?;
    for (event, action) in DROID_REMOVED_LIFECYCLE_HOOK_EVENTS {
        remove_hook_commands(hooks, event, hook_path, Some(action))?;
    }
    for (event, action) in DROID_HOOK_EVENTS {
        remove_hook_commands(hooks, event, hook_path, Some(action))?;
    }
    for (event, action) in DROID_HOOK_EVENTS {
        ensure_command_hook(
            hooks,
            event,
            hook_command(hook_path, Some(action)),
            10,
            None,
        )?;
    }
    write_pretty_json(&settings_path, &settings)?;

    let hooks_path = layout.root.join("hooks.json");
    let mut updated_legacy_hooks = false;
    if hooks_path.is_file() {
        let mut hooks_file = load_json_object(&hooks_path, json!({}))?;
        if let Some(hooks) = hooks_object_if_present(
            &mut hooks_file,
            &hooks_path,
            "droid hooks file",
            "droid hooks file hooks",
        )? {
            updated_legacy_hooks = remove_hook_commands(hooks, "SessionStart", hook_path, None)?;
            for (event, action) in DROID_REMOVED_LIFECYCLE_HOOK_EVENTS {
                updated_legacy_hooks |=
                    remove_hook_commands(hooks, event, hook_path, Some(action))?;
            }
            for (event, action) in DROID_HOOK_EVENTS {
                updated_legacy_hooks |=
                    remove_hook_commands(hooks, event, hook_path, Some(action))?;
            }
        }
        if updated_legacy_hooks {
            write_pretty_json(&hooks_path, &hooks_file)?;
        }
    }

    let mut extras = Vec::new();
    if updated_legacy_hooks {
        extras.push(format!(
            "removed legacy herdr droid hook entries from {}",
            hooks_path.display()
        ));
    }
    Ok(HostInstallResult {
        changes: vec![
            change(settings_path, HostConfigRole::Settings, true),
            change(
                hooks_path,
                HostConfigRole::LegacyHooks,
                updated_legacy_hooks,
            ),
        ],
        extras,
    })
}

fn uninstall_droid(layout: &IntegrationLayout) -> io::Result<HostInstallResult> {
    let hook_path = hook_path(layout);
    let hooks_path = layout.root.join("hooks.json");
    let settings_path = layout.root.join("settings.json");
    let mut updated_hooks = false;
    let mut updated_settings = false;
    if hooks_path.is_file() {
        let mut hooks_file = load_json_object(&hooks_path, json!({}))?;
        if let Some(hooks) = hooks_object_if_present(
            &mut hooks_file,
            &hooks_path,
            "droid hooks file",
            "droid hooks file hooks",
        )? {
            updated_hooks |= remove_hook_commands(hooks, "SessionStart", hook_path, None)?;
            for (event, action) in DROID_REMOVED_LIFECYCLE_HOOK_EVENTS {
                updated_hooks |= remove_hook_commands(hooks, event, hook_path, Some(action))?;
            }
            for (event, action) in DROID_HOOK_EVENTS {
                updated_hooks |= remove_hook_commands(hooks, event, hook_path, Some(action))?;
            }
        }
        if updated_hooks {
            write_pretty_json(&hooks_path, &hooks_file)?;
        }
    }
    if settings_path.is_file() {
        let mut settings = load_json_object(&settings_path, json!({}))?;
        if let Some(hooks) = hooks_object_if_present(
            &mut settings,
            &settings_path,
            "droid settings",
            "droid settings hooks",
        )? {
            updated_settings = remove_hook_commands(hooks, "SessionStart", hook_path, None)?;
            for (event, action) in DROID_REMOVED_LIFECYCLE_HOOK_EVENTS {
                updated_settings |= remove_hook_commands(hooks, event, hook_path, Some(action))?;
            }
            for (event, action) in DROID_HOOK_EVENTS {
                updated_settings |= remove_hook_commands(hooks, event, hook_path, Some(action))?;
            }
        }
        if updated_settings {
            write_pretty_json(&settings_path, &settings)?;
        }
    }
    Ok(HostInstallResult {
        changes: vec![
            change(settings_path, HostConfigRole::Settings, updated_settings),
            change(hooks_path, HostConfigRole::LegacyHooks, updated_hooks),
        ],
        extras: Vec::new(),
    })
}

fn install_opencode(layout: &IntegrationLayout) -> io::Result<HostInstallResult> {
    validate_tui_plugin_config(&layout.root)?;
    let tui_config_path = add_tui_plugin(&layout.root, OPENCODE_TUI_PLUGIN_SPEC)?;
    Ok(HostInstallResult {
        changes: vec![change(tui_config_path, HostConfigRole::TuiConfig, true)],
        extras: Vec::new(),
    })
}

fn uninstall_opencode(layout: &IntegrationLayout) -> io::Result<HostInstallResult> {
    let path = tui_config_path(&layout.root);
    let changed = remove_tui_plugin(&layout.root, OPENCODE_TUI_PLUGIN_SPEC)?;
    Ok(HostInstallResult {
        changes: vec![change(path, HostConfigRole::TuiConfig, changed)],
        extras: Vec::new(),
    })
}

fn install_hermes(layout: &IntegrationLayout) -> io::Result<HostInstallResult> {
    let config_path = layout.root.join("config.yaml");
    let existing = read_or(&config_path, "")?;
    let new_config = ensure_hermes_plugin_enabled(&existing);
    let changed = new_config != existing;
    if changed {
        fs::write(&config_path, new_config)?;
    }
    Ok(HostInstallResult {
        changes: vec![change(config_path, HostConfigRole::Config, changed)],
        extras: Vec::new(),
    })
}

fn uninstall_hermes(layout: &IntegrationLayout) -> io::Result<HostInstallResult> {
    let config_path = layout.root.join("config.yaml");
    let mut changed = false;
    if config_path.is_file() {
        let existing = fs::read_to_string(&config_path)?;
        let new_config = remove_hermes_plugin_enabled(&existing);
        if new_config != existing {
            fs::write(&config_path, new_config)?;
            changed = true;
        }
    }
    Ok(HostInstallResult {
        changes: vec![change(config_path, HostConfigRole::Config, changed)],
        extras: Vec::new(),
    })
}

pub(crate) fn remove_empty_hermes_plugin_dir(layout: &IntegrationLayout) -> io::Result<bool> {
    let Some(plugin_dir) = layout.files.first().and_then(|file| file.path.parent()) else {
        return Ok(false);
    };
    if !plugin_dir.is_dir() {
        return Ok(false);
    }
    if fs::read_dir(plugin_dir)?.next().is_none() {
        fs::remove_dir(plugin_dir)?;
        return Ok(true);
    }
    Ok(false)
}

fn install_qodercli(layout: &IntegrationLayout) -> io::Result<HostInstallResult> {
    let hook_path = hook_path(layout);
    let settings_path = layout.root.join("settings.json");
    let mut settings = load_json_object(&settings_path, json!({}))?;
    let hooks = ensure_hooks_object(
        &mut settings,
        &settings_path,
        "qodercli settings",
        "qodercli settings hooks",
    )?;
    for (event, action) in QODERCLI_REMOVED_LIFECYCLE_HOOK_EVENTS {
        remove_hook_commands(hooks, event, hook_path, Some(action))?;
    }
    for (event, action) in QODERCLI_HOOK_EVENTS {
        remove_hook_commands(hooks, event, hook_path, Some(action))?;
    }
    for (event, action) in QODERCLI_HOOK_EVENTS {
        ensure_command_hook(
            hooks,
            event,
            hook_command(hook_path, Some(action)),
            10,
            Some("*"),
        )?;
    }
    write_pretty_json(&settings_path, &settings)?;
    Ok(HostInstallResult {
        changes: vec![change(settings_path, HostConfigRole::Settings, true)],
        extras: Vec::new(),
    })
}

fn uninstall_qodercli(layout: &IntegrationLayout) -> io::Result<HostInstallResult> {
    let hook_path = hook_path(layout);
    let settings_path = layout.root.join("settings.json");
    let mut changed = false;
    if settings_path.is_file() {
        let mut settings = load_json_object(&settings_path, json!({}))?;
        if let Some(hooks) = hooks_object_if_present(
            &mut settings,
            &settings_path,
            "qodercli settings",
            "qodercli settings hooks",
        )? {
            for (event, action) in QODERCLI_REMOVED_LIFECYCLE_HOOK_EVENTS {
                changed |= remove_hook_commands(hooks, event, hook_path, Some(action))?;
            }
            for (event, action) in QODERCLI_HOOK_EVENTS {
                changed |= remove_hook_commands(hooks, event, hook_path, Some(action))?;
            }
        }
        if changed {
            write_pretty_json(&settings_path, &settings)?;
        }
    }
    Ok(HostInstallResult {
        changes: vec![change(settings_path, HostConfigRole::Settings, changed)],
        extras: Vec::new(),
    })
}

fn install_qwen(layout: &IntegrationLayout) -> io::Result<HostInstallResult> {
    let hook_path = hook_path(layout);
    let settings_path = layout.root.join("settings.json");
    let mut settings = load_json_object(&settings_path, json!({}))?;
    let hooks = ensure_hooks_object(
        &mut settings,
        &settings_path,
        "qwen settings",
        "qwen settings hooks",
    )?;
    for (event, action) in QWEN_HOOK_EVENTS {
        remove_hook_commands(hooks, event, hook_path, Some(action))?;
        ensure_command_hook(
            hooks,
            event,
            hook_command(hook_path, Some(action)),
            10_000,
            Some("*"),
        )?;
    }
    write_pretty_json(&settings_path, &settings)?;
    Ok(HostInstallResult {
        changes: vec![change(settings_path, HostConfigRole::Settings, true)],
        extras: Vec::new(),
    })
}

fn uninstall_qwen(layout: &IntegrationLayout) -> io::Result<HostInstallResult> {
    let hook_path = hook_path(layout);
    let settings_path = layout.root.join("settings.json");
    let mut changed = false;
    if settings_path.is_file() {
        let mut settings = load_json_object(&settings_path, json!({}))?;
        if let Some(hooks) = hooks_object_if_present(
            &mut settings,
            &settings_path,
            "qwen settings",
            "qwen settings hooks",
        )? {
            for (event, action) in QWEN_HOOK_EVENTS {
                changed |= remove_hook_commands(hooks, event, hook_path, Some(action))?;
            }
        }
        if changed {
            write_pretty_json(&settings_path, &settings)?;
        }
    }
    Ok(HostInstallResult {
        changes: vec![change(settings_path, HostConfigRole::Settings, changed)],
        extras: Vec::new(),
    })
}

fn install_cursor(layout: &IntegrationLayout) -> io::Result<HostInstallResult> {
    let hook_path = hook_path(layout);
    let hooks_path = layout.root.join("hooks.json");
    let mut hooks_file = load_json_object(&hooks_path, json!({ "version": 1 }))?;
    if hooks_file.get("version").is_none() {
        hooks_file
            .as_object_mut()
            .ok_or_else(|| {
                io::Error::other(format!(
                    "cursor hooks file at {} must be a JSON object",
                    hooks_path.display()
                ))
            })?
            .insert("version".to_string(), json!(1));
    }
    let hooks = ensure_hooks_object(
        &mut hooks_file,
        &hooks_path,
        "cursor hooks file",
        "cursor hooks file hooks",
    )?;
    let session_command = hook_command(hook_path, Some("session"));
    remove_simple_command_hook(hooks, "beforeSubmitPrompt", &session_command)?;
    remove_simple_command_hook(hooks, "beforeShellExecution", &session_command)?;
    remove_simple_command_hook(hooks, "beforeMCPExecution", &session_command)?;
    remove_simple_command_hook(hooks, "stop", &session_command)?;
    remove_simple_command_hook(hooks, "sessionEnd", &session_command)?;
    ensure_simple_command_hook(hooks, "sessionStart", session_command)?;
    write_pretty_json(&hooks_path, &hooks_file)?;
    Ok(HostInstallResult {
        changes: vec![change(hooks_path, HostConfigRole::Hooks, true)],
        extras: Vec::new(),
    })
}

fn uninstall_cursor(layout: &IntegrationLayout) -> io::Result<HostInstallResult> {
    let hook_path = hook_path(layout);
    let hooks_path = layout.root.join("hooks.json");
    let mut changed = false;
    if hooks_path.is_file() {
        let mut hooks_file = load_json_object(&hooks_path, json!({}))?;
        if let Some(hooks) = hooks_object_if_present(
            &mut hooks_file,
            &hooks_path,
            "cursor hooks file",
            "cursor hooks file hooks",
        )? {
            let session_command = hook_command(hook_path, Some("session"));
            changed |= remove_simple_command_hook(hooks, "sessionStart", &session_command)?;
            changed |= remove_simple_command_hook(hooks, "beforeSubmitPrompt", &session_command)?;
            changed |= remove_simple_command_hook(hooks, "beforeShellExecution", &session_command)?;
            changed |= remove_simple_command_hook(hooks, "beforeMCPExecution", &session_command)?;
            changed |= remove_simple_command_hook(hooks, "stop", &session_command)?;
            changed |= remove_simple_command_hook(hooks, "sessionEnd", &session_command)?;
        }
        if changed {
            write_pretty_json(&hooks_path, &hooks_file)?;
        }
    }
    Ok(HostInstallResult {
        changes: vec![change(hooks_path, HostConfigRole::Hooks, changed)],
        extras: Vec::new(),
    })
}

fn install_mastracode(layout: &IntegrationLayout) -> io::Result<HostInstallResult> {
    let hook_path = hook_path(layout);
    let hooks_path = layout.root.join("hooks.json");
    let mut hooks_file = load_json_object(&hooks_path, json!({}))?;
    let hooks = require_object(&mut hooks_file, &hooks_path, "mastracode hooks file")?;
    for (event, action) in MASTRACODE_REMOVED_HOOK_EVENTS {
        remove_flat_command_hook(hooks, event, &hook_command(hook_path, Some(action)))?;
        remove_flat_command_hook(hooks, event, &mastracode_hook_command(hook_path, action))?;
    }
    for (event, action) in MASTRACODE_HOOK_EVENTS {
        remove_flat_command_hook(hooks, event, &hook_command(hook_path, Some(action)))?;
        ensure_flat_command_hook(
            hooks,
            event,
            mastracode_hook_command(hook_path, action),
            MASTRACODE_HOOK_TIMEOUT_MS,
        )?;
    }
    write_pretty_json(&hooks_path, &hooks_file)?;
    Ok(HostInstallResult {
        changes: vec![change(hooks_path, HostConfigRole::Hooks, true)],
        extras: Vec::new(),
    })
}

fn uninstall_mastracode(layout: &IntegrationLayout) -> io::Result<HostInstallResult> {
    let hook_path = hook_path(layout);
    let hooks_path = layout.root.join("hooks.json");
    let mut changed = false;
    if hooks_path.is_file() {
        let mut hooks_file = load_json_object(&hooks_path, json!({}))?;
        let hooks = require_object(&mut hooks_file, &hooks_path, "mastracode hooks file")?;
        for (event, action) in MASTRACODE_HOOK_EVENTS
            .into_iter()
            .chain(MASTRACODE_REMOVED_HOOK_EVENTS)
        {
            changed |=
                remove_flat_command_hook(hooks, event, &hook_command(hook_path, Some(action)))?;
            changed |= remove_flat_command_hook(
                hooks,
                event,
                &mastracode_hook_command(hook_path, action),
            )?;
        }
        if changed {
            write_pretty_json(&hooks_path, &hooks_file)?;
        }
    }
    Ok(HostInstallResult {
        changes: vec![change(hooks_path, HostConfigRole::Hooks, changed)],
        extras: Vec::new(),
    })
}

fn install_antigravity(layout: &IntegrationLayout) -> io::Result<HostInstallResult> {
    let hooks_path = layout.root.join("hooks.json");
    let mut hooks_file = load_json_object(&hooks_path, json!({}))?;
    let hooks = require_object(&mut hooks_file, &hooks_path, "antigravity cli hooks file")?;
    hooks.insert(
        ANTIGRAVITY_CLI_HOOK_BLOCK_NAME.to_string(),
        antigravity_cli_hook_block(hook_path(layout)),
    );
    write_pretty_json(&hooks_path, &hooks_file)?;
    Ok(HostInstallResult {
        changes: vec![change(hooks_path, HostConfigRole::Hooks, true)],
        extras: Vec::new(),
    })
}

fn uninstall_antigravity(layout: &IntegrationLayout) -> io::Result<HostInstallResult> {
    let hooks_path = layout.root.join("hooks.json");
    let mut changed = false;
    if hooks_path.is_file() {
        let mut hooks_file = load_json_object(&hooks_path, json!({}))?;
        let hooks = require_object(&mut hooks_file, &hooks_path, "antigravity cli hooks file")?;
        changed = hooks.remove(ANTIGRAVITY_CLI_HOOK_BLOCK_NAME).is_some();
        if changed {
            write_pretty_json(&hooks_path, &hooks_file)?;
        }
    }
    Ok(HostInstallResult {
        changes: vec![change(hooks_path, HostConfigRole::Hooks, changed)],
        extras: Vec::new(),
    })
}

fn antigravity_cli_hook_block(hook_path: &Path) -> Value {
    let mut block = Map::new();
    for (event, action) in ANTIGRAVITY_CLI_HOOK_EVENTS {
        let handler = json!({
            "type": "command",
            "command": hook_command(hook_path, Some(action)),
            "timeout": ANTIGRAVITY_CLI_HOOK_TIMEOUT_SEC,
        });
        block.insert(event.to_string(), json!([handler]));
    }
    Value::Object(block)
}

fn install_grok(layout: &IntegrationLayout) -> io::Result<HostInstallResult> {
    let config_path = layout
        .root
        .join("hooks")
        .join(GROK_HOOK_CONFIG_INSTALL_NAME);
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let expected = grok_hook_config(hook_path(layout));
    let existing = fs::read_to_string(&config_path).ok();
    let changed = existing
        .as_deref()
        .and_then(|content| serde_json::from_str::<Value>(content).ok())
        != Some(expected.clone());
    if changed {
        write_pretty_json(&config_path, &expected)?;
    }
    Ok(HostInstallResult {
        changes: vec![change(config_path, HostConfigRole::Config, changed)],
        extras: Vec::new(),
    })
}

fn uninstall_grok(layout: &IntegrationLayout, force: bool) -> io::Result<HostInstallResult> {
    let config_path = layout
        .root
        .join("hooks")
        .join(GROK_HOOK_CONFIG_INSTALL_NAME);
    let state = grok_status(&config_path, hook_path(layout)).state;
    let mut changed = false;
    if config_path.is_file() && (force || state != IntegrationFileState::Unowned) {
        match fs::remove_file(&config_path) {
            Ok(()) => changed = true,
            Err(err) if err.kind() == io::ErrorKind::NotFound => {}
            Err(err) => return Err(err),
        }
    }
    Ok(HostInstallResult {
        changes: vec![change(config_path, HostConfigRole::Config, changed)],
        extras: Vec::new(),
    })
}

fn merge_nested_status(
    path: &Path,
    hook_path: &Path,
    events: &[(&str, &str)],
) -> IntegrationFileStatus {
    match read_hooks_object(path) {
        None => missing(path),
        Some(Err(_)) => unowned(path),
        Some(Ok(hooks)) => {
            let present = events.iter().all(|(event, action)| {
                nested_has_command(&hooks, event, &hook_command(hook_path, Some(action)))
            });
            if present {
                current(path)
            } else if events.iter().any(|(event, action)| {
                nested_has_command(&hooks, event, &hook_command(hook_path, Some(action)))
            }) {
                outdated(path)
            } else {
                missing(path)
            }
        }
    }
}

fn merge_direct_status(path: &Path, hook_path: &Path, events: &[&str]) -> IntegrationFileStatus {
    let command = hook_command(hook_path, None);
    match read_hooks_object(path) {
        None => missing(path),
        Some(Err(_)) => unowned(path),
        Some(Ok(hooks)) => {
            let present = events
                .iter()
                .all(|event| direct_has_command(&hooks, event, &command));
            if present {
                current(path)
            } else if events
                .iter()
                .any(|event| direct_has_command(&hooks, event, &command))
            {
                outdated(path)
            } else {
                missing(path)
            }
        }
    }
}

fn cursor_status(path: &Path, hook_path: &Path) -> IntegrationFileStatus {
    let command = hook_command(hook_path, Some("session"));
    match read_hooks_object(path) {
        None => missing(path),
        Some(Err(_)) => unowned(path),
        Some(Ok(hooks)) => {
            if simple_has_command(&hooks, "sessionStart", &command) {
                current(path)
            } else {
                missing(path)
            }
        }
    }
}

fn mastracode_status(path: &Path, hook_path: &Path) -> IntegrationFileStatus {
    match read_root_object(path) {
        None => missing(path),
        Some(Err(_)) => unowned(path),
        Some(Ok(hooks)) => {
            let present = MASTRACODE_HOOK_EVENTS.iter().all(|(event, action)| {
                flat_has_command(&hooks, event, &mastracode_hook_command(hook_path, action))
            });
            if present {
                current(path)
            } else {
                missing(path)
            }
        }
    }
}

fn antigravity_status(path: &Path, hook_path: &Path) -> IntegrationFileStatus {
    match read_root_object(path) {
        None => missing(path),
        Some(Err(_)) => unowned(path),
        Some(Ok(hooks)) => match hooks.get(ANTIGRAVITY_CLI_HOOK_BLOCK_NAME) {
            Some(block) if block == &antigravity_cli_hook_block(hook_path) => current(path),
            Some(_) => outdated(path),
            None => missing(path),
        },
    }
}

fn grok_status(path: &Path, hook_path: &Path) -> IntegrationFileStatus {
    if !path.is_file() {
        return missing(path);
    }
    let Ok(content) = fs::read_to_string(path) else {
        return unowned(path);
    };
    let Ok(value) = serde_json::from_str::<Value>(&content) else {
        return unowned(path);
    };
    let expected = grok_hook_config(hook_path);
    if value == expected {
        current(path)
    } else if value.get("hooks").is_some() {
        outdated(path)
    } else {
        unowned(path)
    }
}

fn kimi_status(path: &Path, hook_path: &Path) -> IntegrationFileStatus {
    if !path.is_file() {
        return missing(path);
    }
    let Ok(content) = fs::read_to_string(path) else {
        return unowned(path);
    };
    if !content.contains(KIMI_CONFIG_BLOCK_BEGIN) {
        return missing(path);
    }
    let registered = KIMI_HOOK_EVENTS
        .iter()
        .all(|(_, _, action)| content.contains(&hook_command(hook_path, Some(action))));
    if registered {
        current(path)
    } else {
        outdated(path)
    }
}

fn hermes_status(path: &Path) -> IntegrationFileStatus {
    if !path.is_file() {
        return missing(path);
    }
    let Ok(content) = fs::read_to_string(path) else {
        return unowned(path);
    };
    if remove_hermes_plugin_enabled(&content) != content {
        current(path)
    } else {
        missing(path)
    }
}

fn opencode_status(config_dir: &Path) -> IntegrationFileStatus {
    let path = tui_config_path(config_dir);
    if tui_plugin_is_configured(config_dir, OPENCODE_TUI_PLUGIN_SPEC) {
        current(&path)
    } else {
        missing(&path)
    }
}

fn nested_has_command(hooks: &Map<String, Value>, event: &str, command: &str) -> bool {
    hooks
        .get(event)
        .and_then(Value::as_array)
        .is_some_and(|entries| {
            entries.iter().any(|entry| {
                entry
                    .get("hooks")
                    .and_then(Value::as_array)
                    .is_some_and(|hook_entries| {
                        hook_entries
                            .iter()
                            .any(|hook| is_matching_command_hook(hook, command))
                    })
            })
        })
}

fn direct_has_command(hooks: &Map<String, Value>, event: &str, command: &str) -> bool {
    hooks
        .get(event)
        .and_then(Value::as_array)
        .is_some_and(|entries| {
            entries
                .iter()
                .any(|entry| is_matching_direct_command_entry(entry, command))
        })
}

fn simple_has_command(hooks: &Map<String, Value>, event: &str, command: &str) -> bool {
    hooks
        .get(event)
        .and_then(Value::as_array)
        .is_some_and(|entries| {
            entries
                .iter()
                .any(|entry| entry.get("command").and_then(Value::as_str) == Some(command))
        })
}

fn flat_has_command(hooks: &Map<String, Value>, event: &str, command: &str) -> bool {
    hooks
        .get(event)
        .and_then(Value::as_array)
        .is_some_and(|entries| {
            entries
                .iter()
                .any(|entry| is_matching_command_hook(entry, command))
        })
}

fn read_hooks_object(path: &Path) -> Option<io::Result<Map<String, Value>>> {
    read_root_object(path).map(|result| {
        result.and_then(|root| match root.get("hooks") {
            None => Ok(Map::new()),
            Some(Value::Object(hooks)) => Ok(hooks.clone()),
            Some(_) => Err(io::Error::other(format!(
                "hooks at {} must be a JSON object",
                path.display()
            ))),
        })
    })
}

fn read_root_object(path: &Path) -> Option<io::Result<Map<String, Value>>> {
    if !path.is_file() {
        return None;
    }
    Some((|| {
        let content = fs::read_to_string(path)?;
        let value: Value = serde_json::from_str(&content).map_err(|err| {
            io::Error::other(format!("failed to parse {}: {err}", path.display()))
        })?;
        value
            .as_object()
            .cloned()
            .ok_or_else(|| io::Error::other(format!("{} must be a JSON object", path.display())))
    })())
}

fn load_json_object(path: &Path, default: Value) -> io::Result<Value> {
    if !path.is_file() {
        return Ok(default);
    }
    let content = fs::read_to_string(path)?;
    let value: Value = serde_json::from_str(&content)
        .map_err(|err| io::Error::other(format!("failed to parse {}: {err}", path.display())))?;
    if !value.is_object() {
        return Err(io::Error::other(format!(
            "{} must be a JSON object",
            path.display()
        )));
    }
    Ok(value)
}

fn require_object<'a>(
    value: &'a mut Value,
    path: &Path,
    description: &str,
) -> io::Result<&'a mut Map<String, Value>> {
    value.as_object_mut().ok_or_else(|| {
        io::Error::other(format!(
            "{description} at {} must be a JSON object",
            path.display()
        ))
    })
}

fn write_pretty_json(path: &Path, value: &Value) -> io::Result<()> {
    let mut encoded = serde_json::to_string_pretty(value)?;
    encoded.push('\n');
    fs::write(path, encoded)
}

fn read_or(path: &Path, default: &str) -> io::Result<String> {
    match fs::read_to_string(path) {
        Ok(content) => Ok(content),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(default.to_string()),
        Err(err) => Err(err),
    }
}

fn current(path: &Path) -> IntegrationFileStatus {
    status(path, IntegrationFileState::Current)
}

fn missing(path: &Path) -> IntegrationFileStatus {
    status(path, IntegrationFileState::Missing)
}

fn outdated(path: &Path) -> IntegrationFileStatus {
    status(path, IntegrationFileState::Outdated)
}

fn unowned(path: &Path) -> IntegrationFileStatus {
    status(path, IntegrationFileState::Unowned)
}

fn status(path: &Path, state: IntegrationFileState) -> IntegrationFileStatus {
    IntegrationFileStatus {
        path: path.to_path_buf(),
        state,
        installed_version: None,
    }
}
