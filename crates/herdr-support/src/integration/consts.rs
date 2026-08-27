pub const KIMI_CONFIG_BLOCK_BEGIN: &str = "# >>> herdr kimi integration";
pub const KIMI_CONFIG_BLOCK_END: &str = "# <<< herdr kimi integration";
pub const KIMI_ASK_USER_QUESTION_MATCHER: &str = "^AskUserQuestion$";
pub const KIMI_OTHER_TOOL_MATCHER: &str = "^(?!AskUserQuestion$).*$";
pub const KIMI_HOOK_EVENTS: [(&str, Option<&str>, &str); 12] = [
    ("SessionStart", None, "session"),
    ("UserPromptSubmit", None, "working"),
    ("PreToolUse", Some(KIMI_OTHER_TOOL_MATCHER), "working"),
    (
        "PreToolUse",
        Some(KIMI_ASK_USER_QUESTION_MATCHER),
        "blocked",
    ),
    (
        "PostToolUse",
        Some(KIMI_ASK_USER_QUESTION_MATCHER),
        "working",
    ),
    (
        "PostToolUseFailure",
        Some(KIMI_ASK_USER_QUESTION_MATCHER),
        "working",
    ),
    ("SubagentStart", None, "working"),
    ("PreCompact", None, "working"),
    ("PermissionRequest", None, "blocked"),
    ("PermissionResult", None, "working"),
    ("Stop", None, "idle"),
    ("Interrupt", None, "idle"),
];

pub const COPILOT_HOOK_EVENTS: [&str; 1] = ["SessionStart"];
pub const COPILOT_REMOVED_LIFECYCLE_HOOK_EVENTS: [&str; 9] = [
    "UserPromptSubmit",
    "PreToolUse",
    "PostToolUse",
    "PostToolUseFailure",
    "Stop",
    "agentStop",
    "SessionEnd",
    "notification",
    "sessionStart",
];

pub const DEVIN_HOOK_EVENTS: [(&str, &str); 6] = [
    ("SessionStart", "session"),
    ("UserPromptSubmit", "session"),
    ("PreToolUse", "session"),
    ("PostToolUse", "session"),
    ("PermissionRequest", "session"),
    ("Stop", "session"),
];
pub const DEVIN_REMOVED_LIFECYCLE_HOOK_EVENTS: [(&str, &str); 6] = [
    ("UserPromptSubmit", "working"),
    ("PreToolUse", "working"),
    ("PostToolUse", "working"),
    ("PermissionRequest", "blocked"),
    ("Stop", "idle"),
    ("SessionEnd", "release"),
];

pub const DROID_HOOK_EVENTS: [(&str, &str); 1] = [("SessionStart", "session")];
pub const DROID_REMOVED_LIFECYCLE_HOOK_EVENTS: [(&str, &str); 9] = [
    ("SessionStart", "idle"),
    ("UserPromptSubmit", "working"),
    ("PreToolUse", "working"),
    ("PostToolUse", "working"),
    ("Notification", "blocked"),
    ("Stop", "idle"),
    ("SubagentStop", "working"),
    ("PreCompact", "working"),
    ("SessionEnd", "release"),
];

pub const OPENCODE_TUI_PLUGIN_SPEC: &str = "./herdr-tui-session.js";
pub const HERMES_PLUGIN_INSTALL_NAME: &str = "herdr-agent-state";

pub const QODERCLI_HOOK_EVENTS: [(&str, &str); 1] = [("SessionStart", "session")];
pub const QWEN_HOOK_EVENTS: [(&str, &str); 1] = [("SessionStart", "session")];
pub const QODERCLI_REMOVED_LIFECYCLE_HOOK_EVENTS: [(&str, &str); 12] = [
    ("SessionStart", "idle"),
    ("UserPromptSubmit", "working"),
    ("PreToolUse", "working"),
    ("PostToolUse", "working"),
    ("PostToolUseFailure", "working"),
    ("SubagentStart", "working"),
    ("SubagentStop", "working"),
    ("PreCompact", "working"),
    ("Notification", "blocked"),
    ("PermissionRequest", "blocked"),
    ("Stop", "idle"),
    ("SessionEnd", "release"),
];

pub const ANTIGRAVITY_CLI_HOOK_BLOCK_NAME: &str = "herdr";
pub const ANTIGRAVITY_CLI_HOOK_TIMEOUT_SEC: u64 = 10;
pub const ANTIGRAVITY_CLI_HOOK_EVENTS: [(&str, &str); 1] = [("PreInvocation", "session")];

pub const MASTRACODE_HOOK_TIMEOUT_MS: u64 = 10_000;
pub const MASTRACODE_REMOVED_HOOK_EVENTS: [(&str, &str); 2] =
    [("SessionStart", "idle"), ("SessionEnd", "release")];
pub const MASTRACODE_HOOK_EVENTS: [(&str, &str); 11] = [
    ("SessionStart", "session"),
    ("UserPromptSubmit", "working"),
    ("AgentStart", "working"),
    ("PreToolUse", "working"),
    ("PermissionRequest", "blocked"),
    ("PermissionResult", "working"),
    ("SubagentStart", "working"),
    ("SubagentEnd", "working"),
    ("Interrupt", "idle"),
    ("AgentEnd", "idle"),
    ("Stop", "idle"),
];

pub const GROK_HOOK_CONFIG_INSTALL_NAME: &str = "herdr.json";
