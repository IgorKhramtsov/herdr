use std::path::PathBuf;

use herdr_support::{integration_specs, IntegrationTarget};

use super::registry::{
    integration_target_command, integration_target_command_names, integration_target_label,
    integration_update_instructions,
};
use super::types::{IntegrationRecommendation, IntegrationStatusKind};
use super::version::{agent_version_requirement, extract_version_triple};

#[test]
fn version_parser_accepts_agent_output_forms() {
    assert_eq!(extract_version_triple("0.14.0"), Some((0, 14, 0)));
    assert_eq!(extract_version_triple("v1.2.3"), Some((1, 2, 3)));
    assert_eq!(
        extract_version_triple("kimi code 0.14.0-beta.1"),
        Some((0, 14, 0))
    );
    assert_eq!(extract_version_triple("version 2.7"), Some((2, 7, 0)));
    assert_eq!(extract_version_triple("unknown"), None);
}

#[test]
fn only_kimi_has_an_agent_version_requirement() {
    let requirement = agent_version_requirement(IntegrationTarget::Kimi).unwrap();
    assert_eq!(requirement.binary, "kimi");
    assert_eq!(requirement.args, ["--version"]);
    assert_eq!(requirement.min_version, "0.14.0");

    for spec in integration_specs() {
        if spec.target != IntegrationTarget::Kimi {
            assert!(agent_version_requirement(spec.target).is_none());
        }
    }
}

#[test]
fn app_registry_labels_and_commands_follow_support_registry() {
    for spec in integration_specs() {
        assert_eq!(integration_target_label(spec.target), spec.label);
        assert_eq!(
            integration_target_command_names(spec.target),
            spec.command_names
        );
        assert_eq!(
            integration_target_command(spec.target),
            spec.command_names[0]
        );
    }
}

#[test]
fn update_instructions_cover_empty_single_and_multiple_targets() {
    assert_eq!(integration_update_instructions(&[]), "");
    assert_eq!(
        integration_update_instructions(&[IntegrationTarget::Claude]),
        "run `herdr integration install claude`"
    );
    assert_eq!(
        integration_update_instructions(&[IntegrationTarget::Claude, IntegrationTarget::Codex]),
        "run `herdr integration install claude` and `herdr integration install codex`"
    );
}

#[test]
fn recommendation_labels_distinguish_availability_and_ownership() {
    let recommendation = |available, state| IntegrationRecommendation {
        target: IntegrationTarget::Claude,
        label: "claude",
        command: "claude",
        available,
        path: PathBuf::from("/tmp/herdr-test"),
        state,
    };

    assert_eq!(
        recommendation(true, IntegrationStatusKind::NotInstalled).status_label(),
        "available"
    );
    assert_eq!(
        recommendation(false, IntegrationStatusKind::NotInstalled).status_label(),
        "not found"
    );
    assert!(recommendation(true, IntegrationStatusKind::NotInstalled).needs_install());
    assert!(!recommendation(false, IntegrationStatusKind::NotInstalled).needs_install());
    assert_eq!(
        recommendation(true, IntegrationStatusKind::Outdated).status_label(),
        "update available"
    );
    assert!(recommendation(false, IntegrationStatusKind::Outdated).needs_install());
    assert_eq!(
        recommendation(true, IntegrationStatusKind::Modified).status_label(),
        "modified"
    );
    assert_eq!(
        recommendation(true, IntegrationStatusKind::Unowned).status_label(),
        "unowned"
    );
    assert_eq!(
        recommendation(true, IntegrationStatusKind::Current).status_label(),
        "installed"
    );
}
