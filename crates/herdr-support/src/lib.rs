//! Shared Herdr detection, catalogs, integration lifecycle, and newline-JSON reports.
//!
//! Detection remains an evidence producer: callers supply process snapshots,
//! screen text, and OSC strings they already own. Integration install/status/
//! uninstall owns bundled assets plus host-config registration for each harness.
//! The crate does not apply lifecycle precedence, report expiry, done/seen
//! presentation, reconnect, or any other reducer. It has no PTY, TUI, server,
//! persistence, or remote updater.

mod detect;
mod integration;
mod process;
mod report;

pub use detect::manifest::DetectionInput;
pub use detect::{
    agent_label, detect_agent_optional, detect_agent_with_osc, full_lifecycle_hook_authority,
    identify_agent_in_job, identify_agent_process, interactive_agent_executable, parse_agent_label,
    parse_canonical_agent_label, session_identity_only_integration, should_skip_state_update,
    Agent, AgentDetection, AgentState, DetectedAgent,
};
pub use detect::{
    manifest, set_manifest_roots, AgentRemoteStatus, ManifestRoots, ManifestUpdateStatus,
    ManifestVersion, MANIFEST_ENGINE_VERSION,
};
pub use integration::{
    bundled_integration_files, grok_hook_config, hook_command, install_integration,
    integration_asset, integration_layout, integration_root, integration_spec, integration_specs,
    integration_status, integration_statuses, integration_targets, mastracode_hook_command,
    parse_integration_id, parse_integration_version, shell_single_quote, uninstall_integration,
    HostConfigChange, HostConfigRole, IntegrationAsset, IntegrationContext, IntegrationEnv,
    IntegrationFile, IntegrationFileState, IntegrationFileStatus, IntegrationInstallOutcome,
    IntegrationLayout, IntegrationLocatedFile, IntegrationSpec, IntegrationStatus,
    IntegrationTarget, IntegrationUninstallOutcome, ANTIGRAVITY_CLI_CONFIG_DIR_ENV,
    ANTIGRAVITY_CLI_HOOK_BLOCK_NAME, ANTIGRAVITY_CLI_HOOK_EVENTS, ANTIGRAVITY_CLI_HOOK_TIMEOUT_SEC,
    CLAUDE_CONFIG_DIR_ENV, CODEX_HOME_ENV, COPILOT_HOME_ENV, COPILOT_REMOVED_LIFECYCLE_HOOK_EVENTS,
    CURSOR_CONFIG_DIR_ENV, DEVIN_HOOK_EVENTS, DEVIN_REMOVED_LIFECYCLE_HOOK_EVENTS,
    DROID_HOOK_EVENTS, GROK_CONFIG_DIR_ENV, GROK_HOME_ENV, GROK_HOOK_CONFIG_INSTALL_NAME,
    HERMES_HOME_ENV, HOME_ENV, KIMI_ASK_USER_QUESTION_MATCHER, KIMI_CODE_HOME_ENV,
    KIMI_CONFIG_BLOCK_BEGIN, KIMI_CONFIG_BLOCK_END, KIMI_HOOK_EVENTS, KIMI_OTHER_TOOL_MATCHER,
    LOCAL_APP_DATA_ENV, MASTRACODE_HOOK_EVENTS, MASTRACODE_HOOK_TIMEOUT_MS,
    OPENCODE_TUI_PLUGIN_SPEC, PI_CODING_AGENT_DIR_ENV, PI_CONFIG_DIR_ENV, QODERCLI_HOOK_EVENTS,
    QODER_CONFIG_DIR_ENV, QWEN_HOME_ENV, USER_PROFILE_ENV, XDG_CONFIG_HOME_ENV,
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
    fn catalog_classifies_lifecycle_and_session_only_sources() {
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
        assert!(!detection.visible_idle);
        assert!(!detection.visible_blocker);
        assert!(!detection.skip_state_update);
    }
}
