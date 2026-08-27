//! Shared Herdr detection, catalogs, integration assets, and newline-JSON reports.
//!
//! This crate has no PTY, TUI, server, persistence, or remote updater.

mod detect;
mod integration;
mod process;
mod report;

pub use detect::manifest::DetectionInput;
pub use detect::{
    agent_label, detect_agent_optional, detect_agent_with_osc, full_lifecycle_hook_authority,
    identify_agent_in_job, identify_agent_process, interactive_agent_executable, parse_agent_label,
    session_identity_only_integration, should_skip_state_update, Agent, AgentDetection, AgentState,
    DetectedAgent,
};
pub use detect::{
    manifest, set_manifest_roots, AgentRemoteStatus, ManifestRoots, ManifestUpdateStatus,
    ManifestVersion, MANIFEST_ENGINE_VERSION,
};
pub use integration::{
    bundled_integration_files, integration_asset, integration_spec, integration_specs,
    integration_targets, IntegrationAsset, IntegrationFile, IntegrationSpec, IntegrationTarget,
};
pub use process::{ForegroundJob, ForegroundProcess};
pub use report::{
    PaneAgentState, PaneReportAgentParams, PaneReportAgentSessionParams, PingParams, ReportMethod,
    ReportRequest,
};

pub fn identify_agent(job: &ForegroundJob) -> Option<DetectedAgent> {
    identify_agent_in_job(job).map(|(agent, process_name)| DetectedAgent {
        agent,
        process_name,
    })
}

pub fn detect_agent(agent: Agent, input: DetectionInput<'_>) -> AgentDetection {
    detect::manifest::detect_with_osc(agent, input)
}

pub fn bundled_manifests() -> &'static [(&'static str, &'static str)] {
    manifest::bundled_manifests()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_lists_every_agent_and_screen_manifest() {
        assert_eq!(Agent::ALL.len(), 22);
        assert_eq!(Agent::SCREEN_MANIFEST_AGENTS.len(), 20);
        assert!(!Agent::SCREEN_MANIFEST_AGENTS.contains(&Agent::Omp));
        assert!(!Agent::SCREEN_MANIFEST_AGENTS.contains(&Agent::Mastracode));
        let bundled: Vec<_> = bundled_manifests().iter().map(|(id, _)| *id).collect();
        assert_eq!(bundled.len(), 20);
        for agent in Agent::SCREEN_MANIFEST_AGENTS {
            assert!(
                bundled.contains(&agent_label(agent)),
                "missing bundled manifest for {}",
                agent_label(agent)
            );
        }
    }

    #[test]
    fn bundled_manifests_parse() {
        for (id, content) in bundled_manifests() {
            manifest::parse_manifest(content).unwrap_or_else(|err| panic!("{id}: {err}"));
        }
    }

    #[test]
    fn authority_policy_covers_lifecycle_and_session_only_sources() {
        assert!(full_lifecycle_hook_authority("herdr:pi", "pi"));
        assert!(full_lifecycle_hook_authority("herdr:omp", "omp"));
        assert!(full_lifecycle_hook_authority(
            "herdr:mastracode",
            "mastracode"
        ));
        assert!(full_lifecycle_hook_authority("herdr:opencode", "opencode"));
        assert!(full_lifecycle_hook_authority("herdr:kilo", "kilo"));
        assert!(full_lifecycle_hook_authority("herdr:kimi", "kimi"));
        assert!(session_identity_only_integration("herdr:hermes", "hermes"));
        assert!(session_identity_only_integration("herdr:qwen", "qwen"));
        assert!(session_identity_only_integration(
            "herdr:antigravity_cli",
            "agy"
        ));
        assert!(!full_lifecycle_hook_authority("herdr:hermes", "hermes"));
        assert!(!session_identity_only_integration("herdr:pi", "pi"));
    }

    #[test]
    fn facade_identifies_job_and_detects_screen_state() {
        let job = ForegroundJob {
            process_group_id: 7,
            processes: vec![ForegroundProcess {
                pid: 7,
                name: "node".into(),
                argv0: None,
                argv: Some(vec!["node".into(), "/usr/bin/codex".into()]),
                cmdline: Some("node /usr/bin/codex".into()),
            }],
        };
        let detected = identify_agent(&job).expect("codex");
        assert_eq!(detected.agent, Agent::Codex);
        assert_eq!(detected.process_name, "codex");

        let detection = detect_agent(
            Agent::Pi,
            DetectionInput {
                screen: "Working...",
                osc_title: "",
                osc_progress: "",
            },
        );
        assert_eq!(detection.state, AgentState::Working);
        assert!(detection.visible_working);
    }
}
