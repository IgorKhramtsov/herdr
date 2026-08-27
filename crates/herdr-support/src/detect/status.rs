use std::{collections::BTreeMap, fs};

use serde::{Deserialize, Serialize};

use super::{agent_label, roots, Agent};

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ManifestUpdateStatus {
    pub last_check_unix: Option<u64>,
    pub last_result: Option<String>,
    #[serde(default)]
    pub agents: BTreeMap<String, AgentRemoteStatus>,
}

impl ManifestUpdateStatus {
    pub fn agent_status(&self, agent: Agent) -> Option<AgentRemoteStatus> {
        self.agents.get(agent_label(agent)).cloned()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentRemoteStatus {
    pub cached_version: Option<String>,
    pub attempted_version: Option<String>,
    pub last_checked_unix: Option<u64>,
    pub last_result: String,
    pub last_error: Option<String>,
}

pub fn load_status() -> ManifestUpdateStatus {
    let Some(path) = roots::status_path() else {
        return ManifestUpdateStatus::default();
    };
    let Ok(content) = fs::read_to_string(&path) else {
        return ManifestUpdateStatus::default();
    };
    toml::from_str(&content).unwrap_or_default()
}
