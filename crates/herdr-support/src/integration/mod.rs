use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum IntegrationTarget {
    Pi,
    Omp,
    Claude,
    Codex,
    Copilot,
    Devin,
    Droid,
    Kimi,
    Opencode,
    Kilo,
    Hermes,
    Qodercli,
    Qwen,
    Cursor,
    Mastracode,
    AntigravityCli,
    Grok,
}

impl IntegrationTarget {
    pub const ALL: [Self; 17] = [
        Self::Pi,
        Self::Omp,
        Self::Claude,
        Self::Codex,
        Self::Copilot,
        Self::Devin,
        Self::Droid,
        Self::Kimi,
        Self::Opencode,
        Self::Kilo,
        Self::Hermes,
        Self::Qodercli,
        Self::Qwen,
        Self::Cursor,
        Self::Mastracode,
        Self::AntigravityCli,
        Self::Grok,
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IntegrationFile {
    pub install_name: &'static str,
    pub contents: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IntegrationSpec {
    pub target: IntegrationTarget,
    pub label: &'static str,
    pub version: u32,
    pub command_names: &'static [&'static str],
    pub files: &'static [IntegrationFile],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IntegrationAsset {
    pub target: IntegrationTarget,
    pub version: u32,
    pub files: &'static [IntegrationFile],
}

const fn hook(sh: &'static str, ps1: &'static str) -> IntegrationFile {
    IntegrationFile {
        install_name: if cfg!(windows) {
            "herdr-agent-state.ps1"
        } else {
            "herdr-agent-state.sh"
        },
        contents: if cfg!(windows) { ps1 } else { sh },
    }
}

const fn session_hook(sh: &'static str, ps1: &'static str) -> IntegrationFile {
    IntegrationFile {
        install_name: if cfg!(windows) {
            "herdr-agent-session.ps1"
        } else {
            "herdr-agent-session.sh"
        },
        contents: if cfg!(windows) { ps1 } else { sh },
    }
}

const PI_FILES: &[IntegrationFile] = &[IntegrationFile {
    install_name: "herdr-agent-state.ts",
    contents: include_str!("assets/pi/herdr-agent-state.ts"),
}];
const PI: IntegrationSpec = IntegrationSpec {
    target: IntegrationTarget::Pi,
    label: "pi",
    version: 8,
    command_names: &["pi"],
    files: PI_FILES,
};

const OMP_FILES: &[IntegrationFile] = &[IntegrationFile {
    install_name: "herdr-omp-agent-state.ts",
    contents: include_str!("assets/omp/herdr-agent-state.ts"),
}];
const OMP: IntegrationSpec = IntegrationSpec {
    target: IntegrationTarget::Omp,
    label: "omp",
    version: 9,
    command_names: &["omp"],
    files: OMP_FILES,
};

const CLAUDE_FILES: &[IntegrationFile] = &[hook(
    include_str!("assets/claude/herdr-agent-state.sh"),
    include_str!("assets/claude/herdr-agent-state.ps1"),
)];
const CLAUDE: IntegrationSpec = IntegrationSpec {
    target: IntegrationTarget::Claude,
    label: "claude",
    version: 8,
    command_names: &["claude"],
    files: CLAUDE_FILES,
};

const CODEX_FILES: &[IntegrationFile] = &[hook(
    include_str!("assets/codex/herdr-agent-state.sh"),
    include_str!("assets/codex/herdr-agent-state.ps1"),
)];
const CODEX: IntegrationSpec = IntegrationSpec {
    target: IntegrationTarget::Codex,
    label: "codex",
    version: 8,
    command_names: &["codex"],
    files: CODEX_FILES,
};

const COPILOT_FILES: &[IntegrationFile] = &[hook(
    include_str!("assets/copilot/herdr-agent-state.sh"),
    include_str!("assets/copilot/herdr-agent-state.ps1"),
)];
const COPILOT: IntegrationSpec = IntegrationSpec {
    target: IntegrationTarget::Copilot,
    label: "copilot",
    version: 3,
    command_names: &["copilot"],
    files: COPILOT_FILES,
};

const DEVIN_FILES: &[IntegrationFile] = &[hook(
    include_str!("assets/devin/herdr-agent-state.sh"),
    include_str!("assets/devin/herdr-agent-state.ps1"),
)];
const DEVIN: IntegrationSpec = IntegrationSpec {
    target: IntegrationTarget::Devin,
    label: "devin",
    version: 2,
    command_names: &["devin"],
    files: DEVIN_FILES,
};

const DROID_FILES: &[IntegrationFile] = &[hook(
    include_str!("assets/droid/herdr-agent-state.sh"),
    include_str!("assets/droid/herdr-agent-state.ps1"),
)];
const DROID: IntegrationSpec = IntegrationSpec {
    target: IntegrationTarget::Droid,
    label: "droid",
    version: 3,
    command_names: &["droid"],
    files: DROID_FILES,
};

const KIMI_FILES: &[IntegrationFile] = &[hook(
    include_str!("assets/kimi/herdr-agent-state.sh"),
    include_str!("assets/kimi/herdr-agent-state.ps1"),
)];
const KIMI: IntegrationSpec = IntegrationSpec {
    target: IntegrationTarget::Kimi,
    label: "kimi",
    version: 7,
    command_names: &["kimi"],
    files: KIMI_FILES,
};

const OPENCODE_FILES: &[IntegrationFile] = &[
    IntegrationFile {
        install_name: "herdr-agent-state.js",
        contents: include_str!("assets/opencode/herdr-agent-state.js"),
    },
    IntegrationFile {
        install_name: "herdr-tui-session.js",
        contents: include_str!("assets/opencode/herdr-tui-session.js"),
    },
];
const OPENCODE: IntegrationSpec = IntegrationSpec {
    target: IntegrationTarget::Opencode,
    label: "opencode",
    version: 10,
    command_names: &["opencode"],
    files: OPENCODE_FILES,
};

const KILO_FILES: &[IntegrationFile] = &[IntegrationFile {
    install_name: "herdr-agent-state.js",
    contents: include_str!("assets/kilo/herdr-agent-state.js"),
}];
const KILO: IntegrationSpec = IntegrationSpec {
    target: IntegrationTarget::Kilo,
    label: "kilo",
    version: 4,
    command_names: &["kilo", "kilo-code"],
    files: KILO_FILES,
};

const HERMES_FILES: &[IntegrationFile] = &[
    IntegrationFile {
        install_name: "plugin.yaml",
        contents: include_str!("assets/hermes/plugin.yaml"),
    },
    IntegrationFile {
        install_name: "__init__.py",
        contents: include_str!("assets/hermes/__init__.py"),
    },
];
const HERMES: IntegrationSpec = IntegrationSpec {
    target: IntegrationTarget::Hermes,
    label: "hermes",
    version: 5,
    command_names: &["hermes"],
    files: HERMES_FILES,
};

#[cfg(windows)]
const QODERCLI_COMMANDS: &[&str] = &["qodercli", "qoder", "qoderclicn", "qodercn"];
#[cfg(not(windows))]
const QODERCLI_COMMANDS: &[&str] = &["qodercli"];

const QODERCLI_FILES: &[IntegrationFile] = &[hook(
    include_str!("assets/qodercli/herdr-agent-state.sh"),
    include_str!("assets/qodercli/herdr-agent-state.ps1"),
)];
const QODERCLI: IntegrationSpec = IntegrationSpec {
    target: IntegrationTarget::Qodercli,
    label: "qodercli",
    version: 3,
    command_names: QODERCLI_COMMANDS,
    files: QODERCLI_FILES,
};

const QWEN_FILES: &[IntegrationFile] = &[session_hook(
    include_str!("assets/qwen/herdr-agent-session.sh"),
    include_str!("assets/qwen/herdr-agent-session.ps1"),
)];
const QWEN: IntegrationSpec = IntegrationSpec {
    target: IntegrationTarget::Qwen,
    label: "qwen",
    version: 1,
    command_names: &["qwen"],
    files: QWEN_FILES,
};

const CURSOR_FILES: &[IntegrationFile] = &[hook(
    include_str!("assets/cursor/herdr-agent-state.sh"),
    include_str!("assets/cursor/herdr-agent-state.ps1"),
)];
const CURSOR: IntegrationSpec = IntegrationSpec {
    target: IntegrationTarget::Cursor,
    label: "cursor",
    version: 1,
    command_names: &["cursor-agent"],
    files: CURSOR_FILES,
};

const MASTRACODE_FILES: &[IntegrationFile] = &[hook(
    include_str!("assets/mastracode/herdr-agent-state.sh"),
    include_str!("assets/mastracode/herdr-agent-state.ps1"),
)];
const MASTRACODE: IntegrationSpec = IntegrationSpec {
    target: IntegrationTarget::Mastracode,
    label: "mastracode",
    version: 2,
    command_names: &["mastracode"],
    files: MASTRACODE_FILES,
};

const ANTIGRAVITY_CLI_FILES: &[IntegrationFile] = &[hook(
    include_str!("assets/antigravity_cli/herdr-agent-state.sh"),
    include_str!("assets/antigravity_cli/herdr-agent-state.ps1"),
)];
const ANTIGRAVITY_CLI: IntegrationSpec = IntegrationSpec {
    target: IntegrationTarget::AntigravityCli,
    label: "antigravity-cli",
    version: 2,
    command_names: &["agy"],
    files: ANTIGRAVITY_CLI_FILES,
};

const GROK_FILES: &[IntegrationFile] = &[hook(
    include_str!("assets/grok/herdr-agent-state.sh"),
    include_str!("assets/grok/herdr-agent-state.ps1"),
)];
const GROK: IntegrationSpec = IntegrationSpec {
    target: IntegrationTarget::Grok,
    label: "grok",
    version: 1,
    command_names: &["grok"],
    files: GROK_FILES,
};

const SPECS: [IntegrationSpec; 17] = [
    PI,
    OMP,
    CLAUDE,
    CODEX,
    COPILOT,
    DEVIN,
    DROID,
    KIMI,
    OPENCODE,
    KILO,
    HERMES,
    QODERCLI,
    QWEN,
    CURSOR,
    MASTRACODE,
    ANTIGRAVITY_CLI,
    GROK,
];

pub const fn integration_spec(target: IntegrationTarget) -> &'static IntegrationSpec {
    match target {
        IntegrationTarget::Pi => &PI,
        IntegrationTarget::Omp => &OMP,
        IntegrationTarget::Claude => &CLAUDE,
        IntegrationTarget::Codex => &CODEX,
        IntegrationTarget::Copilot => &COPILOT,
        IntegrationTarget::Devin => &DEVIN,
        IntegrationTarget::Droid => &DROID,
        IntegrationTarget::Kimi => &KIMI,
        IntegrationTarget::Opencode => &OPENCODE,
        IntegrationTarget::Kilo => &KILO,
        IntegrationTarget::Hermes => &HERMES,
        IntegrationTarget::Qodercli => &QODERCLI,
        IntegrationTarget::Qwen => &QWEN,
        IntegrationTarget::Cursor => &CURSOR,
        IntegrationTarget::Mastracode => &MASTRACODE,
        IntegrationTarget::AntigravityCli => &ANTIGRAVITY_CLI,
        IntegrationTarget::Grok => &GROK,
    }
}

pub fn integration_specs() -> &'static [IntegrationSpec] {
    &SPECS
}

pub fn integration_targets() -> &'static [IntegrationTarget] {
    &IntegrationTarget::ALL
}

pub fn integration_asset(target: IntegrationTarget) -> IntegrationAsset {
    let spec = integration_spec(target);
    IntegrationAsset {
        target: spec.target,
        version: spec.version,
        files: spec.files,
    }
}

pub fn bundled_integration_files() -> &'static [(&'static str, &'static str)] {
    &[
        (
            "herdr-agent-state.test.ts",
            include_str!("assets/herdr-agent-state.test.ts"),
        ),
        (
            "pi/herdr-agent-state.ts",
            include_str!("assets/pi/herdr-agent-state.ts"),
        ),
        (
            "omp/herdr-agent-state.ts",
            include_str!("assets/omp/herdr-agent-state.ts"),
        ),
        (
            "claude/herdr-agent-state.sh",
            include_str!("assets/claude/herdr-agent-state.sh"),
        ),
        (
            "claude/herdr-agent-state.ps1",
            include_str!("assets/claude/herdr-agent-state.ps1"),
        ),
        (
            "codex/herdr-agent-state.sh",
            include_str!("assets/codex/herdr-agent-state.sh"),
        ),
        (
            "codex/herdr-agent-state.ps1",
            include_str!("assets/codex/herdr-agent-state.ps1"),
        ),
        (
            "copilot/herdr-agent-state.sh",
            include_str!("assets/copilot/herdr-agent-state.sh"),
        ),
        (
            "copilot/herdr-agent-state.ps1",
            include_str!("assets/copilot/herdr-agent-state.ps1"),
        ),
        (
            "devin/herdr-agent-state.sh",
            include_str!("assets/devin/herdr-agent-state.sh"),
        ),
        (
            "devin/herdr-agent-state.ps1",
            include_str!("assets/devin/herdr-agent-state.ps1"),
        ),
        (
            "droid/herdr-agent-state.sh",
            include_str!("assets/droid/herdr-agent-state.sh"),
        ),
        (
            "droid/herdr-agent-state.ps1",
            include_str!("assets/droid/herdr-agent-state.ps1"),
        ),
        (
            "kimi/herdr-agent-state.sh",
            include_str!("assets/kimi/herdr-agent-state.sh"),
        ),
        (
            "kimi/herdr-agent-state.ps1",
            include_str!("assets/kimi/herdr-agent-state.ps1"),
        ),
        (
            "opencode/herdr-agent-state.js",
            include_str!("assets/opencode/herdr-agent-state.js"),
        ),
        (
            "opencode/herdr-tui-session.js",
            include_str!("assets/opencode/herdr-tui-session.js"),
        ),
        (
            "opencode/herdr-agent-state.test.ts",
            include_str!("assets/opencode/herdr-agent-state.test.ts"),
        ),
        (
            "opencode/herdr-tui-session.test.ts",
            include_str!("assets/opencode/herdr-tui-session.test.ts"),
        ),
        (
            "kilo/herdr-agent-state.js",
            include_str!("assets/kilo/herdr-agent-state.js"),
        ),
        (
            "hermes/plugin.yaml",
            include_str!("assets/hermes/plugin.yaml"),
        ),
        (
            "hermes/__init__.py",
            include_str!("assets/hermes/__init__.py"),
        ),
        (
            "qodercli/herdr-agent-state.sh",
            include_str!("assets/qodercli/herdr-agent-state.sh"),
        ),
        (
            "qodercli/herdr-agent-state.ps1",
            include_str!("assets/qodercli/herdr-agent-state.ps1"),
        ),
        (
            "qwen/herdr-agent-session.sh",
            include_str!("assets/qwen/herdr-agent-session.sh"),
        ),
        (
            "qwen/herdr-agent-session.ps1",
            include_str!("assets/qwen/herdr-agent-session.ps1"),
        ),
        (
            "cursor/herdr-agent-state.sh",
            include_str!("assets/cursor/herdr-agent-state.sh"),
        ),
        (
            "cursor/herdr-agent-state.ps1",
            include_str!("assets/cursor/herdr-agent-state.ps1"),
        ),
        (
            "mastracode/herdr-agent-state.sh",
            include_str!("assets/mastracode/herdr-agent-state.sh"),
        ),
        (
            "mastracode/herdr-agent-state.ps1",
            include_str!("assets/mastracode/herdr-agent-state.ps1"),
        ),
        (
            "antigravity_cli/herdr-agent-state.sh",
            include_str!("assets/antigravity_cli/herdr-agent-state.sh"),
        ),
        (
            "antigravity_cli/herdr-agent-state.ps1",
            include_str!("assets/antigravity_cli/herdr-agent-state.ps1"),
        ),
        (
            "grok/herdr-agent-state.sh",
            include_str!("assets/grok/herdr-agent-state.sh"),
        ),
        (
            "grok/herdr-agent-state.ps1",
            include_str!("assets/grok/herdr-agent-state.ps1"),
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_exposes_every_integration_target_and_asset() {
        assert_eq!(IntegrationTarget::ALL.len(), 17);
        assert_eq!(integration_targets().len(), 17);
        assert_eq!(integration_specs().len(), 17);
        for (target, catalog) in IntegrationTarget::ALL.iter().zip(integration_specs()) {
            assert_eq!(catalog.target, *target);
            let spec = integration_spec(*target);
            let asset = integration_asset(*target);
            assert_eq!(spec, catalog);
            assert_eq!(spec.target, *target);
            assert_eq!(asset.target, *target);
            assert_eq!(spec.version, asset.version);
            assert!(!spec.label.is_empty());
            assert!(!spec.command_names.is_empty());
            assert!(!spec.files.is_empty());
            assert_eq!(spec.files, asset.files);
            for file in spec.files {
                assert!(!file.install_name.is_empty());
                assert!(!file.contents.is_empty());
            }
        }
    }

    #[test]
    fn every_bundled_integration_file_is_present() {
        let files = bundled_integration_files();
        assert_eq!(files.len(), 34);
        for (path, contents) in files {
            assert!(!contents.is_empty(), "{path}");
        }
    }
}
