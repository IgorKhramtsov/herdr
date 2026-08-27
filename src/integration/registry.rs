use std::fs;
use std::path::{Path, PathBuf};

use super::env::*;
use herdr_support::{IntegrationContext, IntegrationFileState};

pub(crate) fn integration_target_label(
    target: crate::api::schema::IntegrationTarget,
) -> &'static str {
    herdr_support::integration_spec(target).label
}

pub(crate) fn integration_target_command(
    target: crate::api::schema::IntegrationTarget,
) -> &'static str {
    integration_target_command_names(target)[0]
}

pub(crate) fn integration_target_command_names(
    target: crate::api::schema::IntegrationTarget,
) -> &'static [&'static str] {
    herdr_support::integration_spec(target).command_names
}

pub(crate) fn integration_target_supported(target: crate::api::schema::IntegrationTarget) -> bool {
    #[cfg(windows)]
    {
        matches!(
            target,
            crate::api::schema::IntegrationTarget::Pi
                | crate::api::schema::IntegrationTarget::Omp
                | crate::api::schema::IntegrationTarget::Claude
                | crate::api::schema::IntegrationTarget::Codex
                | crate::api::schema::IntegrationTarget::Copilot
                | crate::api::schema::IntegrationTarget::Opencode
                | crate::api::schema::IntegrationTarget::Kilo
                | crate::api::schema::IntegrationTarget::Droid
                | crate::api::schema::IntegrationTarget::Kimi
                | crate::api::schema::IntegrationTarget::Qodercli
                | crate::api::schema::IntegrationTarget::Qwen
                | crate::api::schema::IntegrationTarget::AntigravityCli
                | crate::api::schema::IntegrationTarget::Devin
                | crate::api::schema::IntegrationTarget::Hermes
                | crate::api::schema::IntegrationTarget::Cursor
                | crate::api::schema::IntegrationTarget::Mastracode
                | crate::api::schema::IntegrationTarget::Grok
        )
    }

    #[cfg(not(windows))]
    {
        let _ = target;
        true
    }
}

pub(crate) fn integration_target_available(target: crate::api::schema::IntegrationTarget) -> bool {
    if !integration_target_supported(target) {
        return false;
    }

    integration_target_command_names(target)
        .iter()
        .any(|command| command_available(command))
        || integration_target_install_layout_available(target)
}

pub(crate) fn integration_target_install_layout_available(
    target: crate::api::schema::IntegrationTarget,
) -> bool {
    match target {
        crate::api::schema::IntegrationTarget::Codex => codex_standalone_binary_available(),
        crate::api::schema::IntegrationTarget::Hermes => hermes_install_layout_available(),
        _ => false,
    }
}

pub(crate) fn command_available(command: &str) -> bool {
    let Some(paths) = std::env::var_os("PATH") else {
        return false;
    };
    std::env::split_paths(&paths).any(|dir| {
        command_path_candidates(&dir, command)
            .into_iter()
            .any(|path| executable_file_exists(&path))
    })
}

pub(crate) fn command_path_candidates(dir: &Path, command: &str) -> Vec<PathBuf> {
    let base = dir.join(command);

    #[cfg(not(windows))]
    {
        vec![base]
    }

    #[cfg(windows)]
    {
        if Path::new(command).extension().is_some() {
            return vec![base];
        }

        let mut candidates = vec![base];
        for extension in [".exe", ".cmd", ".bat", ".ps1"] {
            candidates.push(dir.join(format!("{command}{extension}")));
        }
        candidates
    }
}

pub(crate) fn executable_file_exists(path: &Path) -> bool {
    let Ok(metadata) = path.metadata() else {
        return false;
    };
    if !metadata.is_file() {
        return false;
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        metadata.permissions().mode() & 0o111 != 0
    }

    #[cfg(not(unix))]
    {
        true
    }
}

pub(crate) fn codex_standalone_binary_available() -> bool {
    let Ok(releases_dir) =
        codex_dir().map(|dir| dir.join("packages").join("standalone").join("releases"))
    else {
        return false;
    };
    let Ok(entries) = fs::read_dir(releases_dir) else {
        return false;
    };

    entries.filter_map(Result::ok).any(|entry| {
        executable_file_exists(&entry.path().join("bin").join(codex_executable_name()))
    })
}

pub(crate) fn codex_executable_name() -> &'static str {
    if cfg!(windows) {
        "codex.exe"
    } else {
        "codex"
    }
}

pub(crate) fn hermes_install_layout_available() -> bool {
    #[cfg(windows)]
    {
        let Ok(dir) = hermes_dir() else {
            return false;
        };
        [
            dir.join("hermes.exe"),
            dir.join("bin").join("hermes.exe"),
            dir.join("Scripts").join("hermes.exe"),
        ]
        .into_iter()
        .any(|path| executable_file_exists(&path))
    }

    #[cfg(not(windows))]
    {
        false
    }
}

fn map_file_state(state: IntegrationFileState) -> super::IntegrationStatusKind {
    match state {
        IntegrationFileState::Missing => super::IntegrationStatusKind::NotInstalled,
        IntegrationFileState::Current => super::IntegrationStatusKind::Current,
        IntegrationFileState::Outdated => super::IntegrationStatusKind::Outdated,
        IntegrationFileState::Modified => super::IntegrationStatusKind::Modified,
        IntegrationFileState::Unowned => super::IntegrationStatusKind::Unowned,
    }
}

fn map_support_status(status: herdr_support::IntegrationStatus) -> super::IntegrationStatus {
    super::IntegrationStatus {
        target: status.target,
        path: status.path,
        state: map_file_state(status.state),
        installed_version: status.installed_version,
        expected_version: status.expected_version,
    }
}

pub(crate) fn installed_integration_statuses() -> Vec<super::IntegrationStatus> {
    let Ok(ctx) = IntegrationContext::from_env() else {
        return Vec::new();
    };
    herdr_support::integration_statuses(&ctx)
        .into_iter()
        .filter(|status| integration_target_supported(status.target))
        .map(map_support_status)
        .collect()
}

pub(crate) fn integration_recommendations() -> Vec<super::IntegrationRecommendation> {
    installed_integration_statuses()
        .into_iter()
        .map(|status| super::IntegrationRecommendation {
            target: status.target,
            label: integration_target_label(status.target),
            command: integration_target_command(status.target),
            available: integration_target_available(status.target)
                || status.state != super::IntegrationStatusKind::NotInstalled,
            path: status.path,
            state: status.state,
        })
        .collect()
}

pub(crate) fn outdated_installed_integrations() -> Vec<super::IntegrationStatus> {
    installed_integration_statuses()
        .into_iter()
        .filter(|status| status.state == super::IntegrationStatusKind::Outdated)
        .collect()
}

pub(crate) fn integration_update_instructions(
    targets: &[crate::api::schema::IntegrationTarget],
) -> String {
    let commands: Vec<String> = targets
        .iter()
        .map(|target| {
            format!(
                "`herdr integration install {}`",
                integration_target_label(*target)
            )
        })
        .collect();

    match commands.as_slice() {
        [] => String::new(),
        [command] => format!("run {command}"),
        [rest @ .., last] => format!("run {} and {last}", rest.join(", ")),
    }
}

pub(crate) fn print_outdated_update_notice() -> bool {
    let outdated = outdated_installed_integrations();
    if outdated.is_empty() {
        return false;
    }

    let targets = outdated
        .iter()
        .map(|integration| integration.target)
        .collect::<Vec<_>>();
    eprintln!(
        "installed herdr integrations need updating; {}.",
        integration_update_instructions(&targets).replace('`', "")
    );
    true
}

pub(crate) fn parse_integration_version(content: &str) -> Option<u32> {
    herdr_support::parse_integration_version(content)
}

pub(crate) fn integration_status_at(
    target: crate::api::schema::IntegrationTarget,
    path: PathBuf,
    expected_version: u32,
) -> super::IntegrationStatus {
    let Ok(ctx) = IntegrationContext::from_env() else {
        return super::IntegrationStatus {
            target,
            path,
            state: super::IntegrationStatusKind::NotInstalled,
            installed_version: None,
            expected_version,
        };
    };
    match herdr_support::integration_status(&ctx, target) {
        Ok(status) => {
            let mut mapped = map_support_status(status);
            mapped.path = path;
            mapped.expected_version = expected_version;
            mapped
        }
        Err(_) => super::IntegrationStatus {
            target,
            path,
            state: super::IntegrationStatusKind::NotInstalled,
            installed_version: None,
            expected_version,
        },
    }
}
