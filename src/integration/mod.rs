mod actions;
mod env;
mod file_ops;
mod registry;
mod types;
mod version;

pub(crate) use actions::{
    install_target, install_target_with, uninstall_target, uninstall_target_with,
};
#[cfg(test)]
pub(crate) use env::integration_env_lock;
pub(crate) use env::{
    apply_pane_base_env, HERDR_PANE_ID_ENV_VAR, HERDR_TAB_ID_ENV_VAR, HERDR_WORKSPACE_ID_ENV_VAR,
};
pub(crate) use registry::{
    installed_integration_statuses, integration_recommendations, integration_target_label,
    print_outdated_update_notice,
};
pub(crate) use types::{IntegrationRecommendation, IntegrationStatus, IntegrationStatusKind};

use herdr_support::{integration_spec, IntegrationTarget};

const PI_EXTENSION_INSTALL_NAME: &str =
    integration_spec(IntegrationTarget::Pi).files[0].install_name;
const PI_EXTENSION_ASSET: &str = integration_spec(IntegrationTarget::Pi).files[0].contents;
const PI_INTEGRATION_VERSION: u32 = integration_spec(IntegrationTarget::Pi).version;
const OMP_EXTENSION_INSTALL_NAME: &str =
    integration_spec(IntegrationTarget::Omp).files[0].install_name;
const OMP_EXTENSION_ASSET: &str = integration_spec(IntegrationTarget::Omp).files[0].contents;
const OMP_INTEGRATION_VERSION: u32 = integration_spec(IntegrationTarget::Omp).version;
const CLAUDE_HOOK_INSTALL_NAME: &str =
    integration_spec(IntegrationTarget::Claude).files[0].install_name;
const CLAUDE_HOOK_ASSET: &str = integration_spec(IntegrationTarget::Claude).files[0].contents;
const CLAUDE_INTEGRATION_VERSION: u32 = integration_spec(IntegrationTarget::Claude).version;
const CODEX_HOOK_INSTALL_NAME: &str =
    integration_spec(IntegrationTarget::Codex).files[0].install_name;
const CODEX_HOOK_ASSET: &str = integration_spec(IntegrationTarget::Codex).files[0].contents;
const CODEX_INTEGRATION_VERSION: u32 = integration_spec(IntegrationTarget::Codex).version;
const KIMI_HOOK_INSTALL_NAME: &str =
    integration_spec(IntegrationTarget::Kimi).files[0].install_name;
const KIMI_HOOK_ASSET: &str = integration_spec(IntegrationTarget::Kimi).files[0].contents;
const KIMI_INTEGRATION_VERSION: u32 = integration_spec(IntegrationTarget::Kimi).version;
const KIMI_MIN_VERSION: &str = "0.14.0";
const COPILOT_HOOK_INSTALL_NAME: &str =
    integration_spec(IntegrationTarget::Copilot).files[0].install_name;
const COPILOT_HOOK_ASSET: &str = integration_spec(IntegrationTarget::Copilot).files[0].contents;
const COPILOT_INTEGRATION_VERSION: u32 = integration_spec(IntegrationTarget::Copilot).version;
const DEVIN_HOOK_INSTALL_NAME: &str =
    integration_spec(IntegrationTarget::Devin).files[0].install_name;
const DEVIN_HOOK_ASSET: &str = integration_spec(IntegrationTarget::Devin).files[0].contents;
const DEVIN_INTEGRATION_VERSION: u32 = integration_spec(IntegrationTarget::Devin).version;
const DROID_HOOK_INSTALL_NAME: &str =
    integration_spec(IntegrationTarget::Droid).files[0].install_name;
const DROID_HOOK_ASSET: &str = integration_spec(IntegrationTarget::Droid).files[0].contents;
const DROID_INTEGRATION_VERSION: u32 = integration_spec(IntegrationTarget::Droid).version;
const OPENCODE_PLUGIN_INSTALL_NAME: &str =
    integration_spec(IntegrationTarget::Opencode).files[0].install_name;
const OPENCODE_PLUGIN_ASSET: &str = integration_spec(IntegrationTarget::Opencode).files[0].contents;
const OPENCODE_TUI_PLUGIN_INSTALL_NAME: &str =
    integration_spec(IntegrationTarget::Opencode).files[1].install_name;
const OPENCODE_TUI_PLUGIN_SPEC: &str = "./herdr-tui-session.js";
const OPENCODE_TUI_PLUGIN_ASSET: &str =
    integration_spec(IntegrationTarget::Opencode).files[1].contents;
const OPENCODE_INTEGRATION_VERSION: u32 = integration_spec(IntegrationTarget::Opencode).version;
const KILO_PLUGIN_INSTALL_NAME: &str =
    integration_spec(IntegrationTarget::Kilo).files[0].install_name;
const KILO_PLUGIN_ASSET: &str = integration_spec(IntegrationTarget::Kilo).files[0].contents;
const KILO_INTEGRATION_VERSION: u32 = integration_spec(IntegrationTarget::Kilo).version;
const HERMES_PLUGIN_INSTALL_NAME: &str = "herdr-agent-state";
const HERMES_PLUGIN_MANIFEST_INSTALL_NAME: &str =
    integration_spec(IntegrationTarget::Hermes).files[0].install_name;
const HERMES_PLUGIN_INIT_INSTALL_NAME: &str =
    integration_spec(IntegrationTarget::Hermes).files[1].install_name;
const HERMES_PLUGIN_MANIFEST_ASSET: &str =
    integration_spec(IntegrationTarget::Hermes).files[0].contents;
const HERMES_PLUGIN_INIT_ASSET: &str =
    integration_spec(IntegrationTarget::Hermes).files[1].contents;
const HERMES_INTEGRATION_VERSION: u32 = integration_spec(IntegrationTarget::Hermes).version;
const QODERCLI_HOOK_INSTALL_NAME: &str =
    integration_spec(IntegrationTarget::Qodercli).files[0].install_name;
const QODERCLI_HOOK_ASSET: &str = integration_spec(IntegrationTarget::Qodercli).files[0].contents;
const QODERCLI_INTEGRATION_VERSION: u32 = integration_spec(IntegrationTarget::Qodercli).version;
const QWEN_HOOK_INSTALL_NAME: &str =
    integration_spec(IntegrationTarget::Qwen).files[0].install_name;
const QWEN_HOOK_ASSET: &str = integration_spec(IntegrationTarget::Qwen).files[0].contents;
const QWEN_INTEGRATION_VERSION: u32 = integration_spec(IntegrationTarget::Qwen).version;
const CURSOR_HOOK_INSTALL_NAME: &str =
    integration_spec(IntegrationTarget::Cursor).files[0].install_name;
const CURSOR_HOOK_ASSET: &str = integration_spec(IntegrationTarget::Cursor).files[0].contents;
const CURSOR_INTEGRATION_VERSION: u32 = integration_spec(IntegrationTarget::Cursor).version;
const ANTIGRAVITY_CLI_HOOK_INSTALL_NAME: &str =
    integration_spec(IntegrationTarget::AntigravityCli).files[0].install_name;
const ANTIGRAVITY_CLI_HOOK_ASSET: &str =
    integration_spec(IntegrationTarget::AntigravityCli).files[0].contents;
const ANTIGRAVITY_CLI_INTEGRATION_VERSION: u32 =
    integration_spec(IntegrationTarget::AntigravityCli).version;
const MASTRACODE_HOOK_INSTALL_NAME: &str =
    integration_spec(IntegrationTarget::Mastracode).files[0].install_name;
const MASTRACODE_HOOK_ASSET: &str =
    integration_spec(IntegrationTarget::Mastracode).files[0].contents;
const MASTRACODE_INTEGRATION_VERSION: u32 = integration_spec(IntegrationTarget::Mastracode).version;
const GROK_HOOK_INSTALL_NAME: &str =
    integration_spec(IntegrationTarget::Grok).files[0].install_name;
const GROK_HOOK_CONFIG_INSTALL_NAME: &str = "herdr.json";
const GROK_HOOK_ASSET: &str = integration_spec(IntegrationTarget::Grok).files[0].contents;
const GROK_INTEGRATION_VERSION: u32 = integration_spec(IntegrationTarget::Grok).version;

pub(crate) const INSTALL_WARNING_PREFIX: &str = "warning:";

#[cfg(test)]
pub(crate) use herdr_support::{
    grok_hook_config, hook_command, mastracode_hook_command, shell_single_quote,
    ANTIGRAVITY_CLI_HOOK_BLOCK_NAME, ANTIGRAVITY_CLI_HOOK_EVENTS, ANTIGRAVITY_CLI_HOOK_TIMEOUT_SEC,
    COPILOT_REMOVED_LIFECYCLE_HOOK_EVENTS, DEVIN_HOOK_EVENTS, DEVIN_REMOVED_LIFECYCLE_HOOK_EVENTS,
    DROID_HOOK_EVENTS, KIMI_ASK_USER_QUESTION_MATCHER, KIMI_CONFIG_BLOCK_BEGIN,
    KIMI_CONFIG_BLOCK_END, KIMI_HOOK_EVENTS, KIMI_OTHER_TOOL_MATCHER, MASTRACODE_HOOK_EVENTS,
    MASTRACODE_HOOK_TIMEOUT_MS, QODERCLI_HOOK_EVENTS,
};

#[cfg(test)]
mod adapters;

#[cfg(test)]
mod tests;
