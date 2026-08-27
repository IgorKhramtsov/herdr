use std::io;
use std::path::PathBuf;
#[cfg(test)]
use std::sync::{Mutex, MutexGuard, OnceLock};

use herdr_support::{IntegrationContext, IntegrationTarget};
use portable_pty::CommandBuilder;

pub(crate) const HERDR_PANE_ID_ENV_VAR: &str = "HERDR_PANE_ID";
pub(crate) const HERDR_TAB_ID_ENV_VAR: &str = "HERDR_TAB_ID";
pub(crate) const HERDR_WORKSPACE_ID_ENV_VAR: &str = "HERDR_WORKSPACE_ID";

pub(crate) fn apply_pane_base_env(cmd: &mut CommandBuilder) {
    cmd.env(crate::api::SOCKET_PATH_ENV_VAR, crate::api::socket_path());
    if let Ok(executable) = std::env::current_exe() {
        cmd.env("HERDR_BIN_PATH", executable);
    }
    if let Ok(pane) = std::env::var(HERDR_PANE_ID_ENV_VAR) {
        cmd.env(HERDR_PANE_ID_ENV_VAR, pane);
    } else if let Ok(tmux_pane) = std::env::var("TMUX_PANE") {
        cmd.env(HERDR_PANE_ID_ENV_VAR, tmux_pane);
    }
    if let Ok(tab) = std::env::var(HERDR_TAB_ID_ENV_VAR) {
        cmd.env(HERDR_TAB_ID_ENV_VAR, tab);
    }
    if let Ok(workspace) = std::env::var(HERDR_WORKSPACE_ID_ENV_VAR) {
        cmd.env(HERDR_WORKSPACE_ID_ENV_VAR, workspace);
    }
}

fn ctx() -> io::Result<IntegrationContext> {
    IntegrationContext::from_env()
}

pub(crate) fn codex_dir() -> io::Result<PathBuf> {
    herdr_support::integration_root(&ctx()?, IntegrationTarget::Codex)
}

#[cfg(windows)]
pub(crate) fn hermes_dir() -> io::Result<PathBuf> {
    herdr_support::integration_root(&ctx()?, IntegrationTarget::Hermes)
}

#[cfg(test)]
pub(crate) fn integration_env_lock() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(())).lock().unwrap()
}
