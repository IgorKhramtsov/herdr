use std::io;
use std::path::Path;

use herdr_support::{
    install_integration, uninstall_integration, HostConfigChange, HostConfigRole,
    IntegrationContext, IntegrationInstallOutcome, IntegrationTarget, IntegrationUninstallOutcome,
};

use super::registry::integration_target_label;
use super::version::{agent_version_requirement, enforce_agent_version};
use super::KIMI_MIN_VERSION;

pub(crate) fn install_target(
    target: crate::api::schema::IntegrationTarget,
) -> io::Result<Vec<String>> {
    install_target_with(target, false)
}

pub(crate) fn install_target_with(
    target: crate::api::schema::IntegrationTarget,
    force: bool,
) -> io::Result<Vec<String>> {
    let result = install_target_inner(target, force);
    let outcome = if result.is_ok() { "ok" } else { "error" };
    crate::logging::integration_action("install", integration_target_label(target), outcome);
    result
}

fn install_target_inner(
    target: crate::api::schema::IntegrationTarget,
    force: bool,
) -> io::Result<Vec<String>> {
    let version_warning = match agent_version_requirement(target) {
        Some(requirement) => enforce_agent_version(&requirement)?,
        None => None,
    };

    let ctx = IntegrationContext::from_env()?;
    let outcome = install_integration(&ctx, target, force)?;
    let mut messages = install_messages(target, &outcome);
    if target == IntegrationTarget::Kimi {
        messages.push(format!("requires kimi code {KIMI_MIN_VERSION} or newer"));
    }
    if let Some(warning) = version_warning {
        messages.push(warning);
    }
    Ok(messages)
}

pub(crate) fn uninstall_target(
    target: crate::api::schema::IntegrationTarget,
) -> io::Result<Vec<String>> {
    uninstall_target_with(target, false)
}

pub(crate) fn uninstall_target_with(
    target: crate::api::schema::IntegrationTarget,
    force: bool,
) -> io::Result<Vec<String>> {
    let ctx = IntegrationContext::from_env()?;
    let outcome = uninstall_integration(&ctx, target, force)?;
    let messages = uninstall_messages(target, &outcome);
    crate::logging::integration_action("uninstall", integration_target_label(target), "ok");
    Ok(messages)
}

fn host(changes: &[HostConfigChange], role: HostConfigRole) -> Option<&HostConfigChange> {
    changes.iter().find(|change| change.role == role)
}

fn asset(outcome_paths: &[std::path::PathBuf], index: usize) -> &Path {
    outcome_paths
        .get(index)
        .map(Path::new)
        .unwrap_or_else(|| Path::new(""))
}

fn removed(paths: &[std::path::PathBuf], path: &Path) -> bool {
    paths.iter().any(|removed| removed == path)
}

fn install_messages(target: IntegrationTarget, outcome: &IntegrationInstallOutcome) -> Vec<String> {
    let path0 = asset(&outcome.paths, 0);
    let mut messages = match target {
        IntegrationTarget::Pi => vec![format!("installed pi integration to {}", path0.display())],
        IntegrationTarget::Omp => vec![format!("installed omp integration to {}", path0.display())],
        IntegrationTarget::Claude => vec![
            format!("installed claude integration hook to {}", path0.display()),
            format!(
                "ensured claude settings at {}",
                host(&outcome.host, HostConfigRole::Settings)
                    .map(|change| change.path.display().to_string())
                    .unwrap_or_default()
            ),
        ],
        IntegrationTarget::Codex => vec![
            format!("installed codex integration hook to {}", path0.display()),
            format!(
                "ensured codex hooks at {}",
                host(&outcome.host, HostConfigRole::Hooks)
                    .map(|change| change.path.display().to_string())
                    .unwrap_or_default()
            ),
            format!(
                "ensured codex config at {}",
                host(&outcome.host, HostConfigRole::Config)
                    .map(|change| change.path.display().to_string())
                    .unwrap_or_default()
            ),
        ],
        IntegrationTarget::Copilot => vec![
            format!("installed copilot integration hook to {}", path0.display()),
            format!(
                "ensured copilot settings at {}",
                host(&outcome.host, HostConfigRole::Settings)
                    .map(|change| change.path.display().to_string())
                    .unwrap_or_default()
            ),
        ],
        IntegrationTarget::Devin => vec![
            format!("installed devin integration hook to {}", path0.display()),
            format!(
                "ensured devin settings at {}",
                host(&outcome.host, HostConfigRole::Settings)
                    .map(|change| change.path.display().to_string())
                    .unwrap_or_default()
            ),
        ],
        IntegrationTarget::Kimi => vec![
            format!("installed kimi integration hook to {}", path0.display()),
            format!(
                "ensured kimi config at {}",
                host(&outcome.host, HostConfigRole::Config)
                    .map(|change| change.path.display().to_string())
                    .unwrap_or_default()
            ),
        ],
        IntegrationTarget::Droid => vec![
            format!("installed droid integration hook to {}", path0.display()),
            format!(
                "ensured droid hooks at {}",
                host(&outcome.host, HostConfigRole::Settings)
                    .map(|change| change.path.display().to_string())
                    .unwrap_or_default()
            ),
        ],
        IntegrationTarget::Opencode => vec![
            format!(
                "installed opencode integration plugin to {}",
                path0.display()
            ),
            format!(
                "installed opencode tui integration plugin to {}",
                asset(&outcome.paths, 1).display()
            ),
            format!(
                "ensured opencode tui plugin config at {}",
                host(&outcome.host, HostConfigRole::TuiConfig)
                    .map(|change| change.path.display().to_string())
                    .unwrap_or_default()
            ),
        ],
        IntegrationTarget::Kilo => vec![format!(
            "installed kilo integration plugin to {}",
            path0.display()
        )],
        IntegrationTarget::Hermes => vec![
            format!(
                "installed hermes integration plugin to {}",
                path0.parent().unwrap_or(path0).display()
            ),
            format!(
                "enabled hermes plugin in {}",
                host(&outcome.host, HostConfigRole::Config)
                    .map(|change| change.path.display().to_string())
                    .unwrap_or_default()
            ),
        ],
        IntegrationTarget::Qodercli => vec![
            format!("installed qodercli integration hook to {}", path0.display()),
            format!(
                "ensured qodercli settings at {}",
                host(&outcome.host, HostConfigRole::Settings)
                    .map(|change| change.path.display().to_string())
                    .unwrap_or_default()
            ),
        ],
        IntegrationTarget::Qwen => vec![
            format!("installed qwen integration hook to {}", path0.display()),
            format!(
                "ensured qwen settings at {}",
                host(&outcome.host, HostConfigRole::Settings)
                    .map(|change| change.path.display().to_string())
                    .unwrap_or_default()
            ),
        ],
        IntegrationTarget::Cursor => vec![
            format!("installed cursor integration hook to {}", path0.display()),
            format!(
                "updated cursor hooks at {}",
                host(&outcome.host, HostConfigRole::Hooks)
                    .map(|change| change.path.display().to_string())
                    .unwrap_or_default()
            ),
        ],
        IntegrationTarget::Mastracode => vec![
            format!(
                "installed mastracode integration hook to {}",
                path0.display()
            ),
            format!(
                "ensured mastracode hooks at {}",
                host(&outcome.host, HostConfigRole::Hooks)
                    .map(|change| change.path.display().to_string())
                    .unwrap_or_default()
            ),
        ],
        IntegrationTarget::AntigravityCli => vec![
            format!(
                "installed antigravity-cli integration hook to {}",
                path0.display()
            ),
            format!(
                "ensured antigravity-cli hooks at {}",
                host(&outcome.host, HostConfigRole::Hooks)
                    .map(|change| change.path.display().to_string())
                    .unwrap_or_default()
            ),
        ],
        IntegrationTarget::Grok => vec![
            format!("installed grok integration hook to {}", path0.display()),
            format!(
                "registered grok hook config at {}",
                host(&outcome.host, HostConfigRole::Config)
                    .map(|change| change.path.display().to_string())
                    .unwrap_or_default()
            ),
        ],
    };
    if target == IntegrationTarget::Omp {
        let mut ordered = outcome.extras.clone();
        ordered.append(&mut messages);
        messages = ordered;
    } else {
        messages.extend(outcome.extras.iter().cloned());
    }
    messages
}

fn uninstall_messages(
    target: IntegrationTarget,
    outcome: &IntegrationUninstallOutcome,
) -> Vec<String> {
    let path0 = asset(&outcome.paths, 0);
    let removed0 = removed(&outcome.removed, path0);
    match target {
        IntegrationTarget::Pi => vec![if removed0 {
            format!("removed pi integration extension at {}", path0.display())
        } else {
            format!("no pi integration extension found at {}", path0.display())
        }],
        IntegrationTarget::Omp => vec![if removed0 {
            format!("removed omp integration extension at {}", path0.display())
        } else {
            format!("no omp integration extension found at {}", path0.display())
        }],
        IntegrationTarget::Claude => uninstall_hook_and_host(
            "claude hook",
            path0,
            removed0,
            "herdr claude hook entries",
            host(&outcome.host, HostConfigRole::Settings),
        ),
        IntegrationTarget::Codex => {
            let mut messages = uninstall_hook_and_host(
                "codex hook",
                path0,
                removed0,
                "herdr codex hook entries",
                host(&outcome.host, HostConfigRole::Hooks),
            );
            if let Some(config) = host(&outcome.host, HostConfigRole::Config) {
                messages.push(format!(
                    "left codex config unchanged at {}",
                    config.path.display()
                ));
            }
            messages
        }
        IntegrationTarget::Copilot => uninstall_hook_and_host(
            "copilot hook",
            path0,
            removed0,
            "herdr copilot hook entries",
            host(&outcome.host, HostConfigRole::Settings),
        ),
        IntegrationTarget::Devin => uninstall_hook_and_host(
            "devin hook",
            path0,
            removed0,
            "herdr devin hook entries",
            host(&outcome.host, HostConfigRole::Settings),
        ),
        IntegrationTarget::Kimi => uninstall_hook_and_host(
            "kimi hook",
            path0,
            removed0,
            "herdr kimi hook entries",
            host(&outcome.host, HostConfigRole::Config),
        ),
        IntegrationTarget::Droid => {
            let mut messages = vec![if removed0 {
                format!("removed droid hook at {}", path0.display())
            } else {
                format!("no droid hook found at {}", path0.display())
            }];
            messages.extend(host_removed(
                "legacy herdr droid hook entries",
                host(&outcome.host, HostConfigRole::LegacyHooks),
            ));
            messages.extend(host_removed(
                "herdr droid hook entries",
                host(&outcome.host, HostConfigRole::Settings),
            ));
            messages
        }
        IntegrationTarget::Opencode => {
            let tui = asset(&outcome.paths, 1);
            let mut messages = vec![
                if removed0 {
                    format!("removed opencode integration plugin at {}", path0.display())
                } else {
                    format!(
                        "no opencode integration plugin found at {}",
                        path0.display()
                    )
                },
                if removed(&outcome.removed, tui) {
                    format!(
                        "removed opencode tui integration plugin at {}",
                        tui.display()
                    )
                } else {
                    format!(
                        "no opencode tui integration plugin found at {}",
                        tui.display()
                    )
                },
            ];
            if let Some(config) = host(&outcome.host, HostConfigRole::TuiConfig) {
                if config.changed {
                    messages.push(format!(
                        "removed herdr opencode plugin entry from {}",
                        config.path.display()
                    ));
                }
            }
            messages
        }
        IntegrationTarget::Kilo => vec![if removed0 {
            format!("removed kilo integration plugin at {}", path0.display())
        } else {
            format!("no kilo integration plugin found at {}", path0.display())
        }],
        IntegrationTarget::Hermes => {
            let plugin_dir = path0.parent().unwrap_or(path0);
            let removed_plugin = !outcome.removed.is_empty() || !outcome.extras.is_empty();
            let mut messages = vec![if removed_plugin {
                format!(
                    "removed hermes integration plugin at {}",
                    plugin_dir.display()
                )
            } else {
                format!(
                    "no hermes integration plugin found at {}",
                    plugin_dir.display()
                )
            }];
            messages.extend(host_toggle(
                "disabled hermes plugin in",
                "no hermes plugin entry found in",
                host(&outcome.host, HostConfigRole::Config),
            ));
            messages
        }
        IntegrationTarget::Qodercli => uninstall_hook_and_host(
            "qodercli hook",
            path0,
            removed0,
            "herdr qodercli hook entries",
            host(&outcome.host, HostConfigRole::Settings),
        ),
        IntegrationTarget::Qwen => uninstall_hook_and_host(
            "qwen hook",
            path0,
            removed0,
            "herdr qwen hook entries",
            host(&outcome.host, HostConfigRole::Settings),
        ),
        IntegrationTarget::Cursor => uninstall_hook_and_host(
            "cursor hook",
            path0,
            removed0,
            "herdr cursor hook entries",
            host(&outcome.host, HostConfigRole::Hooks),
        ),
        IntegrationTarget::Mastracode => uninstall_hook_and_host(
            "mastracode hook",
            path0,
            removed0,
            "herdr mastracode hook entries",
            host(&outcome.host, HostConfigRole::Hooks),
        ),
        IntegrationTarget::AntigravityCli => uninstall_hook_and_host(
            "antigravity-cli hook",
            path0,
            removed0,
            "herdr antigravity-cli hook entries",
            host(&outcome.host, HostConfigRole::Hooks),
        ),
        IntegrationTarget::Grok => {
            let mut messages = vec![if removed0 {
                format!("removed grok hook at {}", path0.display())
            } else {
                format!("no grok hook found at {}", path0.display())
            }];
            if let Some(config) = host(&outcome.host, HostConfigRole::Config) {
                messages.push(if config.changed {
                    format!("removed grok hook config at {}", config.path.display())
                } else {
                    format!("no grok hook config found at {}", config.path.display())
                });
            }
            messages
        }
    }
}

fn uninstall_hook_and_host(
    hook_label: &str,
    hook_path: &Path,
    removed_hook: bool,
    host_label: &str,
    host: Option<&HostConfigChange>,
) -> Vec<String> {
    let mut messages = vec![if removed_hook {
        format!("removed {hook_label} at {}", hook_path.display())
    } else {
        format!("no {hook_label} found at {}", hook_path.display())
    }];
    messages.extend(host_removed(host_label, host));
    messages
}

fn host_removed(label: &str, host: Option<&HostConfigChange>) -> Vec<String> {
    let Some(host) = host else {
        return Vec::new();
    };
    vec![if host.changed {
        format!("removed {label} from {}", host.path.display())
    } else {
        format!("no {label} found in {}", host.path.display())
    }]
}

fn host_toggle(changed: &str, absent: &str, host: Option<&HostConfigChange>) -> Vec<String> {
    let Some(host) = host else {
        return Vec::new();
    };
    vec![if host.changed {
        format!("{changed} {}", host.path.display())
    } else {
        format!("{absent} {}", host.path.display())
    }]
}
