use std::{path::PathBuf, sync::RwLock};

use super::{agent_label, Agent};

#[derive(Debug, Clone, Default)]
pub struct ManifestRoots {
    pub config_dir: Option<PathBuf>,
    pub state_dir: Option<PathBuf>,
}

static ROOTS: RwLock<ManifestRoots> = RwLock::new(ManifestRoots {
    config_dir: None,
    state_dir: None,
});

pub fn set_manifest_roots(roots: ManifestRoots) {
    *ROOTS
        .write()
        .unwrap_or_else(|poisoned| poisoned.into_inner()) = roots;
}

pub fn manifest_roots() -> ManifestRoots {
    ROOTS
        .read()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone()
}

pub fn override_path(agent: Agent) -> Option<PathBuf> {
    Some(
        manifest_roots()
            .config_dir?
            .join("agent-detection")
            .join(format!("{}.toml", agent_label(agent))),
    )
}

pub fn remote_manifest_path(agent: Agent) -> Option<PathBuf> {
    Some(
        manifest_roots()
            .state_dir?
            .join("agent-detection")
            .join("remote")
            .join(format!("{}.toml", agent_label(agent))),
    )
}

pub fn status_path() -> Option<PathBuf> {
    Some(
        manifest_roots()
            .state_dir?
            .join("agent-detection")
            .join("status.toml"),
    )
}

#[cfg(test)]
pub fn test_roots_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}
