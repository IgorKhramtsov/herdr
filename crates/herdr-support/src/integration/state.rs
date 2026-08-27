use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntegrationFileState {
    Missing,
    Current,
    Outdated,
    Modified,
    Unowned,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntegrationFileStatus {
    pub path: PathBuf,
    pub state: IntegrationFileState,
    pub installed_version: Option<u32>,
}
