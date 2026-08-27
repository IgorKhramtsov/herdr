use std::io;
use std::path::{Path, PathBuf};

use herdr_support::{
    install_integration, uninstall_integration, HostConfigChange, HostConfigRole,
    IntegrationContext, IntegrationInstallOutcome, IntegrationTarget, IntegrationUninstallOutcome,
};

use super::types::*;

fn ctx() -> io::Result<IntegrationContext> {
    IntegrationContext::from_env()
}

fn install(target: IntegrationTarget, force: bool) -> io::Result<IntegrationInstallOutcome> {
    install_integration(&ctx()?, target, force)
}

fn uninstall(target: IntegrationTarget, force: bool) -> io::Result<IntegrationUninstallOutcome> {
    uninstall_integration(&ctx()?, target, force)
}

fn path_at(paths: &[PathBuf], index: usize) -> io::Result<PathBuf> {
    paths.get(index).cloned().ok_or_else(|| {
        io::Error::other("integration produced no destination path")
    })
}

fn host<'a>(changes: &'a [HostConfigChange], role: HostConfigRole) -> Option<&'a HostConfigChange> {
    changes.iter().find(|change| change.role == role)
}

fn host_path(changes: &[HostConfigChange], role: HostConfigRole) -> io::Result<PathBuf> {
    host(changes, role)
        .map(|change| change.path.clone())
        .ok_or_else(|| io::Error::other("integration produced no host config path"))
}

fn host_changed(changes: &[HostConfigChange], role: HostConfigRole) -> bool {
    host(changes, role).is_some_and(|change| change.changed)
}

fn removed(paths: &[PathBuf], path: &Path) -> bool {
    paths.iter().any(|candidate| candidate == path)
}

fn plugin_dir(path: &Path) -> io::Result<PathBuf> {
    path.parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| io::Error::other("integration produced no plugin directory"))
}

pub(crate) fn install_pi() -> io::Result<PathBuf> {
    install_pi_with(false)
}

pub(crate) fn install_pi_with(force: bool) -> io::Result<PathBuf> {
    path_at(&install(IntegrationTarget::Pi, force)?.paths, 0)
}

pub(crate) fn uninstall_pi() -> io::Result<PiUninstallResult> {
    uninstall_pi_with(false)
}

pub(crate) fn uninstall_pi_with(force: bool) -> io::Result<PiUninstallResult> {
    let outcome = uninstall(IntegrationTarget::Pi, force)?;
    let extension_path = path_at(&outcome.paths, 0)?;
    Ok(PiUninstallResult {
        removed_extension: removed(&outcome.removed, &extension_path),
        extension_path,
    })
}

pub(crate) fn install_omp() -> io::Result<OmpInstallPaths> {
    install_omp_with(false)
}

pub(crate) fn install_omp_with(force: bool) -> io::Result<OmpInstallPaths> {
    let outcome = install(IntegrationTarget::Omp, force)?;
    Ok(OmpInstallPaths {
        extension_path: path_at(&outcome.paths, 0)?,
        removed_legacy_pi_extension: outcome
            .extras
            .iter()
            .any(|extra| extra.contains("removed legacy pi")),
    })
}

pub(crate) fn uninstall_omp() -> io::Result<OmpUninstallResult> {
    uninstall_omp_with(false)
}

pub(crate) fn uninstall_omp_with(force: bool) -> io::Result<OmpUninstallResult> {
    let outcome = uninstall(IntegrationTarget::Omp, force)?;
    let extension_path = path_at(&outcome.paths, 0)?;
    Ok(OmpUninstallResult {
        removed_extension: removed(&outcome.removed, &extension_path),
        extension_path,
    })
}

pub(crate) fn install_claude() -> io::Result<ClaudeInstallPaths> {
    install_claude_with(false)
}

pub(crate) fn install_claude_with(force: bool) -> io::Result<ClaudeInstallPaths> {
    let outcome = install(IntegrationTarget::Claude, force)?;
    Ok(ClaudeInstallPaths {
        hook_path: path_at(&outcome.paths, 0)?,
        settings_path: host_path(&outcome.host, HostConfigRole::Settings)?,
    })
}

pub(crate) fn uninstall_claude() -> io::Result<ClaudeUninstallResult> {
    uninstall_claude_with(false)
}

pub(crate) fn uninstall_claude_with(force: bool) -> io::Result<ClaudeUninstallResult> {
    let outcome = uninstall(IntegrationTarget::Claude, force)?;
    let hook_path = path_at(&outcome.paths, 0)?;
    Ok(ClaudeUninstallResult {
        settings_path: host_path(&outcome.host, HostConfigRole::Settings)?,
        removed_hook_file: removed(&outcome.removed, &hook_path),
        updated_settings: host_changed(&outcome.host, HostConfigRole::Settings),
        hook_path,
    })
}

pub(crate) fn install_codex() -> io::Result<CodexInstallPaths> {
    install_codex_with(false)
}

pub(crate) fn install_codex_with(force: bool) -> io::Result<CodexInstallPaths> {
    let outcome = install(IntegrationTarget::Codex, force)?;
    Ok(CodexInstallPaths {
        hook_path: path_at(&outcome.paths, 0)?,
        hooks_path: host_path(&outcome.host, HostConfigRole::Hooks)?,
        config_path: host_path(&outcome.host, HostConfigRole::Config)?,
    })
}

pub(crate) fn uninstall_codex() -> io::Result<CodexUninstallResult> {
    uninstall_codex_with(false)
}

pub(crate) fn uninstall_codex_with(force: bool) -> io::Result<CodexUninstallResult> {
    let outcome = uninstall(IntegrationTarget::Codex, force)?;
    let hook_path = path_at(&outcome.paths, 0)?;
    Ok(CodexUninstallResult {
        hooks_path: host_path(&outcome.host, HostConfigRole::Hooks)?,
        config_path: host_path(&outcome.host, HostConfigRole::Config)?,
        removed_hook_file: removed(&outcome.removed, &hook_path),
        updated_hooks: host_changed(&outcome.host, HostConfigRole::Hooks),
        hook_path,
    })
}

pub(crate) fn install_kimi() -> io::Result<KimiInstallPaths> {
    install_kimi_with(false)
}

pub(crate) fn install_kimi_with(force: bool) -> io::Result<KimiInstallPaths> {
    let outcome = install(IntegrationTarget::Kimi, force)?;
    Ok(KimiInstallPaths {
        hook_path: path_at(&outcome.paths, 0)?,
        config_path: host_path(&outcome.host, HostConfigRole::Config)?,
    })
}

pub(crate) fn uninstall_kimi() -> io::Result<KimiUninstallResult> {
    uninstall_kimi_with(false)
}

pub(crate) fn uninstall_kimi_with(force: bool) -> io::Result<KimiUninstallResult> {
    let outcome = uninstall(IntegrationTarget::Kimi, force)?;
    let hook_path = path_at(&outcome.paths, 0)?;
    Ok(KimiUninstallResult {
        config_path: host_path(&outcome.host, HostConfigRole::Config)?,
        removed_hook_file: removed(&outcome.removed, &hook_path),
        updated_config: host_changed(&outcome.host, HostConfigRole::Config),
        hook_path,
    })
}

pub(crate) fn install_copilot() -> io::Result<CopilotInstallPaths> {
    install_copilot_with(false)
}

pub(crate) fn install_copilot_with(force: bool) -> io::Result<CopilotInstallPaths> {
    let outcome = install(IntegrationTarget::Copilot, force)?;
    Ok(CopilotInstallPaths {
        hook_path: path_at(&outcome.paths, 0)?,
        settings_path: host_path(&outcome.host, HostConfigRole::Settings)?,
    })
}

pub(crate) fn uninstall_copilot() -> io::Result<CopilotUninstallResult> {
    uninstall_copilot_with(false)
}

pub(crate) fn uninstall_copilot_with(force: bool) -> io::Result<CopilotUninstallResult> {
    let outcome = uninstall(IntegrationTarget::Copilot, force)?;
    let hook_path = path_at(&outcome.paths, 0)?;
    Ok(CopilotUninstallResult {
        settings_path: host_path(&outcome.host, HostConfigRole::Settings)?,
        removed_hook_file: removed(&outcome.removed, &hook_path),
        updated_settings: host_changed(&outcome.host, HostConfigRole::Settings),
        hook_path,
    })
}

pub(crate) fn install_devin() -> io::Result<DevinInstallPaths> {
    install_devin_with(false)
}

pub(crate) fn install_devin_with(force: bool) -> io::Result<DevinInstallPaths> {
    let outcome = install(IntegrationTarget::Devin, force)?;
    Ok(DevinInstallPaths {
        hook_path: path_at(&outcome.paths, 0)?,
        settings_path: host_path(&outcome.host, HostConfigRole::Settings)?,
    })
}

pub(crate) fn uninstall_devin() -> io::Result<DevinUninstallResult> {
    uninstall_devin_with(false)
}

pub(crate) fn uninstall_devin_with(force: bool) -> io::Result<DevinUninstallResult> {
    let outcome = uninstall(IntegrationTarget::Devin, force)?;
    let hook_path = path_at(&outcome.paths, 0)?;
    Ok(DevinUninstallResult {
        settings_path: host_path(&outcome.host, HostConfigRole::Settings)?,
        removed_hook_file: removed(&outcome.removed, &hook_path),
        updated_settings: host_changed(&outcome.host, HostConfigRole::Settings),
        hook_path,
    })
}

pub(crate) fn install_droid() -> io::Result<DroidInstallPaths> {
    install_droid_with(false)
}

pub(crate) fn install_droid_with(force: bool) -> io::Result<DroidInstallPaths> {
    let outcome = install(IntegrationTarget::Droid, force)?;
    Ok(DroidInstallPaths {
        hook_path: path_at(&outcome.paths, 0)?,
        hooks_path: host_path(&outcome.host, HostConfigRole::LegacyHooks)?,
        settings_path: host_path(&outcome.host, HostConfigRole::Settings)?,
        updated_legacy_hooks: host_changed(&outcome.host, HostConfigRole::LegacyHooks),
    })
}

pub(crate) fn uninstall_droid() -> io::Result<DroidUninstallResult> {
    uninstall_droid_with(false)
}

pub(crate) fn uninstall_droid_with(force: bool) -> io::Result<DroidUninstallResult> {
    let outcome = uninstall(IntegrationTarget::Droid, force)?;
    let hook_path = path_at(&outcome.paths, 0)?;
    Ok(DroidUninstallResult {
        hooks_path: host_path(&outcome.host, HostConfigRole::LegacyHooks)?,
        settings_path: host_path(&outcome.host, HostConfigRole::Settings)?,
        removed_hook_file: removed(&outcome.removed, &hook_path),
        updated_hooks: host_changed(&outcome.host, HostConfigRole::LegacyHooks),
        updated_settings: host_changed(&outcome.host, HostConfigRole::Settings),
        hook_path,
    })
}

pub(crate) fn install_opencode() -> io::Result<OpenCodeInstallPaths> {
    install_opencode_with(false)
}

pub(crate) fn install_opencode_with(force: bool) -> io::Result<OpenCodeInstallPaths> {
    let outcome = install(IntegrationTarget::Opencode, force)?;
    Ok(OpenCodeInstallPaths {
        plugin_path: path_at(&outcome.paths, 0)?,
        tui_plugin_path: path_at(&outcome.paths, 1)?,
        tui_config_path: host_path(&outcome.host, HostConfigRole::TuiConfig)?,
    })
}

pub(crate) fn uninstall_opencode() -> io::Result<OpenCodeUninstallResult> {
    uninstall_opencode_with(false)
}

pub(crate) fn uninstall_opencode_with(force: bool) -> io::Result<OpenCodeUninstallResult> {
    let outcome = uninstall(IntegrationTarget::Opencode, force)?;
    let plugin_path = path_at(&outcome.paths, 0)?;
    let tui_plugin_path = path_at(&outcome.paths, 1)?;
    Ok(OpenCodeUninstallResult {
        removed_plugin: removed(&outcome.removed, &plugin_path),
        removed_tui_plugin: removed(&outcome.removed, &tui_plugin_path),
        tui_config_path: host_path(&outcome.host, HostConfigRole::TuiConfig)?,
        updated_tui_config: host_changed(&outcome.host, HostConfigRole::TuiConfig),
        plugin_path,
        tui_plugin_path,
    })
}

pub(crate) fn install_kilo() -> io::Result<KiloInstallPaths> {
    install_kilo_with(false)
}

pub(crate) fn install_kilo_with(force: bool) -> io::Result<KiloInstallPaths> {
    Ok(KiloInstallPaths {
        plugin_path: path_at(&install(IntegrationTarget::Kilo, force)?.paths, 0)?,
    })
}

pub(crate) fn uninstall_kilo() -> io::Result<KiloUninstallResult> {
    uninstall_kilo_with(false)
}

pub(crate) fn uninstall_kilo_with(force: bool) -> io::Result<KiloUninstallResult> {
    let outcome = uninstall(IntegrationTarget::Kilo, force)?;
    let plugin_path = path_at(&outcome.paths, 0)?;
    Ok(KiloUninstallResult {
        removed_plugin: removed(&outcome.removed, &plugin_path),
        plugin_path,
    })
}

pub(crate) fn install_hermes() -> io::Result<HermesInstallPaths> {
    install_hermes_with(false)
}

pub(crate) fn install_hermes_with(force: bool) -> io::Result<HermesInstallPaths> {
    let outcome = install(IntegrationTarget::Hermes, force)?;
    Ok(HermesInstallPaths {
        plugin_dir: plugin_dir(&path_at(&outcome.paths, 0)?)?,
        config_path: host_path(&outcome.host, HostConfigRole::Config)?,
    })
}

pub(crate) fn uninstall_hermes() -> io::Result<HermesUninstallResult> {
    uninstall_hermes_with(false)
}

pub(crate) fn uninstall_hermes_with(force: bool) -> io::Result<HermesUninstallResult> {
    let outcome = uninstall(IntegrationTarget::Hermes, force)?;
    let plugin_dir = plugin_dir(&path_at(&outcome.paths, 0)?)?;
    Ok(HermesUninstallResult {
        updated_config: host_changed(&outcome.host, HostConfigRole::Config),
        removed_plugin_dir: outcome
            .extras
            .iter()
            .any(|extra| extra.contains("removed empty hermes plugin directory")),
        config_path: host_path(&outcome.host, HostConfigRole::Config)?,
        plugin_dir,
    })
}

pub(crate) fn install_qodercli() -> io::Result<QodercliInstallPaths> {
    install_qodercli_with(false)
}

pub(crate) fn install_qodercli_with(force: bool) -> io::Result<QodercliInstallPaths> {
    let outcome = install(IntegrationTarget::Qodercli, force)?;
    Ok(QodercliInstallPaths {
        hook_path: path_at(&outcome.paths, 0)?,
        settings_path: host_path(&outcome.host, HostConfigRole::Settings)?,
    })
}

pub(crate) fn uninstall_qodercli() -> io::Result<QodercliUninstallResult> {
    uninstall_qodercli_with(false)
}

pub(crate) fn uninstall_qodercli_with(force: bool) -> io::Result<QodercliUninstallResult> {
    let outcome = uninstall(IntegrationTarget::Qodercli, force)?;
    let hook_path = path_at(&outcome.paths, 0)?;
    Ok(QodercliUninstallResult {
        settings_path: host_path(&outcome.host, HostConfigRole::Settings)?,
        removed_hook_file: removed(&outcome.removed, &hook_path),
        updated_settings: host_changed(&outcome.host, HostConfigRole::Settings),
        hook_path,
    })
}

pub(crate) fn install_qwen() -> io::Result<QwenInstallPaths> {
    install_qwen_with(false)
}

pub(crate) fn install_qwen_with(force: bool) -> io::Result<QwenInstallPaths> {
    let outcome = install(IntegrationTarget::Qwen, force)?;
    Ok(QwenInstallPaths {
        hook_path: path_at(&outcome.paths, 0)?,
        settings_path: host_path(&outcome.host, HostConfigRole::Settings)?,
    })
}

pub(crate) fn uninstall_qwen() -> io::Result<QwenUninstallResult> {
    uninstall_qwen_with(false)
}

pub(crate) fn uninstall_qwen_with(force: bool) -> io::Result<QwenUninstallResult> {
    let outcome = uninstall(IntegrationTarget::Qwen, force)?;
    let hook_path = path_at(&outcome.paths, 0)?;
    Ok(QwenUninstallResult {
        settings_path: host_path(&outcome.host, HostConfigRole::Settings)?,
        removed_hook_file: removed(&outcome.removed, &hook_path),
        updated_settings: host_changed(&outcome.host, HostConfigRole::Settings),
        hook_path,
    })
}

pub(crate) fn install_cursor() -> io::Result<CursorInstallPaths> {
    install_cursor_with(false)
}

pub(crate) fn install_cursor_with(force: bool) -> io::Result<CursorInstallPaths> {
    let outcome = install(IntegrationTarget::Cursor, force)?;
    Ok(CursorInstallPaths {
        hook_path: path_at(&outcome.paths, 0)?,
        hooks_path: host_path(&outcome.host, HostConfigRole::Hooks)?,
    })
}

pub(crate) fn uninstall_cursor() -> io::Result<CursorUninstallResult> {
    uninstall_cursor_with(false)
}

pub(crate) fn uninstall_cursor_with(force: bool) -> io::Result<CursorUninstallResult> {
    let outcome = uninstall(IntegrationTarget::Cursor, force)?;
    let hook_path = path_at(&outcome.paths, 0)?;
    Ok(CursorUninstallResult {
        hooks_path: host_path(&outcome.host, HostConfigRole::Hooks)?,
        removed_hook_file: removed(&outcome.removed, &hook_path),
        updated_hooks: host_changed(&outcome.host, HostConfigRole::Hooks),
        hook_path,
    })
}

pub(crate) fn install_mastracode() -> io::Result<MastracodeInstallPaths> {
    install_mastracode_with(false)
}

pub(crate) fn install_mastracode_with(force: bool) -> io::Result<MastracodeInstallPaths> {
    let outcome = install(IntegrationTarget::Mastracode, force)?;
    Ok(MastracodeInstallPaths {
        hook_path: path_at(&outcome.paths, 0)?,
        hooks_path: host_path(&outcome.host, HostConfigRole::Hooks)?,
    })
}

pub(crate) fn uninstall_mastracode() -> io::Result<MastracodeUninstallResult> {
    uninstall_mastracode_with(false)
}

pub(crate) fn uninstall_mastracode_with(force: bool) -> io::Result<MastracodeUninstallResult> {
    let outcome = uninstall(IntegrationTarget::Mastracode, force)?;
    let hook_path = path_at(&outcome.paths, 0)?;
    Ok(MastracodeUninstallResult {
        hooks_path: host_path(&outcome.host, HostConfigRole::Hooks)?,
        removed_hook_file: removed(&outcome.removed, &hook_path),
        updated_hooks: host_changed(&outcome.host, HostConfigRole::Hooks),
        hook_path,
    })
}

pub(crate) fn install_antigravity_cli() -> io::Result<AntigravityCliInstallPaths> {
    install_antigravity_cli_with(false)
}

pub(crate) fn install_antigravity_cli_with(force: bool) -> io::Result<AntigravityCliInstallPaths> {
    let outcome = install(IntegrationTarget::AntigravityCli, force)?;
    Ok(AntigravityCliInstallPaths {
        hook_path: path_at(&outcome.paths, 0)?,
        hooks_path: host_path(&outcome.host, HostConfigRole::Hooks)?,
    })
}

pub(crate) fn uninstall_antigravity_cli() -> io::Result<AntigravityCliUninstallResult> {
    uninstall_antigravity_cli_with(false)
}

pub(crate) fn uninstall_antigravity_cli_with(
    force: bool,
) -> io::Result<AntigravityCliUninstallResult> {
    let outcome = uninstall(IntegrationTarget::AntigravityCli, force)?;
    let hook_path = path_at(&outcome.paths, 0)?;
    Ok(AntigravityCliUninstallResult {
        hooks_path: host_path(&outcome.host, HostConfigRole::Hooks)?,
        removed_hook_file: removed(&outcome.removed, &hook_path),
        updated_hooks: host_changed(&outcome.host, HostConfigRole::Hooks),
        hook_path,
    })
}

pub(crate) fn install_grok() -> io::Result<GrokInstallPaths> {
    install_grok_with(false)
}

pub(crate) fn install_grok_with(force: bool) -> io::Result<GrokInstallPaths> {
    let outcome = install(IntegrationTarget::Grok, force)?;
    Ok(GrokInstallPaths {
        hook_path: path_at(&outcome.paths, 0)?,
        config_path: host_path(&outcome.host, HostConfigRole::Config)?,
    })
}

pub(crate) fn uninstall_grok() -> io::Result<GrokUninstallResult> {
    uninstall_grok_with(false)
}

pub(crate) fn uninstall_grok_with(force: bool) -> io::Result<GrokUninstallResult> {
    let outcome = uninstall(IntegrationTarget::Grok, force)?;
    let hook_path = path_at(&outcome.paths, 0)?;
    Ok(GrokUninstallResult {
        config_path: host_path(&outcome.host, HostConfigRole::Config)?,
        removed_hook_file: removed(&outcome.removed, &hook_path),
        removed_config_file: host_changed(&outcome.host, HostConfigRole::Config),
        hook_path,
    })
}
