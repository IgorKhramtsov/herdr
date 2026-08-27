pub use herdr_support::{
    agent_label, full_lifecycle_hook_authority, interactive_agent_executable, parse_agent_label,
    parse_canonical_agent_label, session_identity_only_integration, Agent, AgentDetection,
    AgentState,
};

pub mod manifest_update;

pub(crate) fn bind_manifest_roots() {
    herdr_support::set_manifest_roots(herdr_support::ManifestRoots {
        config_dir: Some(crate::config::config_dir()),
        state_dir: Some(crate::config::state_dir()),
    });
}

pub mod manifest {
    pub use herdr_support::manifest::*;

    pub fn reload_manifests() -> Vec<herdr_support::manifest::AgentManifestSummary> {
        super::bind_manifest_roots();
        herdr_support::manifest::reload_manifests()
    }

    pub fn reload_manifests_for_agents(agents: &[super::Agent]) {
        super::bind_manifest_roots();
        herdr_support::manifest::reload_manifests_for_agents(agents);
    }

    pub fn manifest_summaries() -> Vec<herdr_support::manifest::AgentManifestSummary> {
        super::bind_manifest_roots();
        herdr_support::manifest::manifest_summaries()
    }

    #[cfg(test)]
    pub fn explain(
        agent: super::Agent,
        screen_content: &str,
    ) -> herdr_support::manifest::DetectionExplain {
        super::bind_manifest_roots();
        herdr_support::manifest::explain(agent, screen_content)
    }

    pub fn explain_with_input(
        agent: super::Agent,
        input: herdr_support::DetectionInput<'_>,
    ) -> herdr_support::manifest::DetectionExplain {
        super::bind_manifest_roots();
        herdr_support::manifest::explain_with_input(agent, input)
    }

    pub fn explain_for_label(
        agent_label: &str,
        screen_content: &str,
    ) -> herdr_support::manifest::DetectionExplain {
        super::bind_manifest_roots();
        herdr_support::manifest::explain_for_label(agent_label, screen_content)
    }
}

#[cfg(windows)]
pub fn identify_agent(process_name: &str) -> Option<Agent> {
    herdr_support::identify_agent_process(process_name)
}

pub fn identify_agent_in_job(job: &crate::platform::ForegroundJob) -> Option<(Agent, String)> {
    herdr_support::identify_agent_in_job(job)
}

pub fn detect_agent_with_osc(
    agent: Option<Agent>,
    screen_content: &str,
    osc_title: &str,
    osc_progress: &str,
) -> AgentDetection {
    bind_manifest_roots();
    herdr_support::detect_agent_optional(
        agent,
        herdr_support::DetectionInput {
            screen: screen_content,
            osc_title,
            osc_progress,
        },
    )
}

pub fn should_skip_state_update(agent: Option<Agent>, screen_content: &str) -> bool {
    bind_manifest_roots();
    herdr_support::should_skip_state_update(agent, screen_content)
}

pub fn foreground_job(child_pid: u32) -> Option<crate::platform::ForegroundJob> {
    crate::platform::foreground_job(child_pid)
}

pub fn foreground_group_leader_job(
    process_group_id: u32,
) -> Option<crate::platform::ForegroundJob> {
    crate::platform::foreground_group_leader_job(process_group_id)
}

pub fn foreground_process_group_id(child_pid: u32) -> Option<u32> {
    crate::platform::foreground_process_group_id(child_pid)
}

#[cfg(test)]
mod tests {
    #[cfg(target_os = "linux")]
    use super::*;

    #[cfg(target_os = "linux")]
    fn open_test_pty() -> portable_pty::PtyPair {
        portable_pty::native_pty_system()
            .openpty(portable_pty::PtySize {
                rows: 24,
                cols: 80,
                pixel_width: 0,
                pixel_height: 0,
            })
            .expect("failed to open pty")
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn foreground_job_detects_sleep() {
        use portable_pty::CommandBuilder;

        let pair = open_test_pty();

        let mut cmd = CommandBuilder::new("sleep");
        cmd.arg("999");
        let mut child = pair.slave.spawn_command(cmd).expect("failed to spawn");
        let pid = child.process_id().expect("no pid");

        std::thread::sleep(std::time::Duration::from_millis(50));

        let job = foreground_job(pid).expect("expected foreground job");
        assert!(
            job.processes.iter().any(|p| p.name == "sleep"),
            "expected sleep in {job:?}"
        );
        assert_eq!(
            identify_agent_in_job(&job),
            None,
            "sleep should not map to an agent"
        );

        child.kill().ok();
        child.wait().ok();
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn foreground_job_detects_shell_running_command() {
        use portable_pty::CommandBuilder;
        use std::io::Write;

        let pair = open_test_pty();

        let cmd = CommandBuilder::new("sh");
        let mut child = pair.slave.spawn_command(cmd).expect("failed to spawn");
        let pid = child.process_id().expect("no pid");

        let mut writer = pair.master.take_writer().expect("no writer");
        writer.write_all(b"exec sleep 999\n").ok();
        drop(writer);

        std::thread::sleep(std::time::Duration::from_millis(100));

        let job = foreground_job(pid).expect("expected foreground job");
        assert!(
            job.processes.iter().any(|p| p.name == "sleep"),
            "expected sleep in {job:?}"
        );
        assert_eq!(
            identify_agent_in_job(&job),
            None,
            "sleep should not map to an agent"
        );

        child.kill().ok();
        child.wait().ok();
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn foreground_job_detects_agent_behind_shell_wrapper() {
        use portable_pty::CommandBuilder;

        let pair = open_test_pty();

        let mut cmd = CommandBuilder::new("bash");
        cmd.arg("-c");
        cmd.arg("bash -c 'exec -a codex sleep 999' & wait");
        let mut child = pair.slave.spawn_command(cmd).expect("failed to spawn");
        let pid = child.process_id().expect("no pid");
        std::thread::sleep(std::time::Duration::from_millis(100));

        let job = foreground_job(pid);
        let process_group_id = job.as_ref().map(|job| job.process_group_id).unwrap_or(pid);
        unsafe {
            libc::kill(-(process_group_id as i32), libc::SIGKILL);
        }
        child.wait().ok();

        let job = job.expect("expected foreground job");
        assert!(
            job.processes.iter().any(|process| process.name == "bash")
                && job.processes.iter().any(|process| {
                    process.name == "sleep"
                        && process
                            .argv
                            .as_deref()
                            .and_then(|argv| argv.first())
                            .is_some_and(|argv0| argv0 == "codex")
                }),
            "expected wrapper and agent child in {job:?}"
        );
        assert_eq!(
            identify_agent_in_job(&job),
            Some((Agent::Codex, "codex".to_string()))
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn proc_stat_parsing_handles_spaces_in_comm() {
        let pid = std::process::id();
        let stat = std::fs::read_to_string(format!("/proc/{pid}/stat")).unwrap();

        let close_paren = stat.rfind(')').expect("should have closing paren");
        let rest = &stat[close_paren + 2..];
        let fields: Vec<&str> = rest.split_whitespace().collect();

        assert!(
            fields.len() >= 6,
            "not enough fields in stat: {}",
            fields.len()
        );

        let state = fields[0];
        assert!(
            ["S", "R", "D", "Z", "T", "t", "W", "X", "I"].contains(&state),
            "unexpected state: {state}"
        );

        let tpgid: i32 = fields[5].parse().expect("tpgid should be a number");
        let _ = tpgid;
    }
}
