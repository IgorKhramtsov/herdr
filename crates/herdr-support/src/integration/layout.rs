use std::io;
use std::path::{Path, PathBuf};

use super::{integration_spec, IntegrationFile, IntegrationTarget};

pub const PI_CODING_AGENT_DIR_ENV: &str = "PI_CODING_AGENT_DIR";
pub const PI_CONFIG_DIR_ENV: &str = "PI_CONFIG_DIR";
pub const CLAUDE_CONFIG_DIR_ENV: &str = "CLAUDE_CONFIG_DIR";
pub const CODEX_HOME_ENV: &str = "CODEX_HOME";
pub const KIMI_CODE_HOME_ENV: &str = "KIMI_CODE_HOME";
pub const COPILOT_HOME_ENV: &str = "COPILOT_HOME";
pub const QODER_CONFIG_DIR_ENV: &str = "QODER_CONFIG_DIR";
pub const QWEN_HOME_ENV: &str = "QWEN_HOME";
pub const CURSOR_CONFIG_DIR_ENV: &str = "CURSOR_CONFIG_DIR";
pub const ANTIGRAVITY_CLI_CONFIG_DIR_ENV: &str = "ANTIGRAVITY_CLI_CONFIG_DIR";
pub const GROK_CONFIG_DIR_ENV: &str = "GROK_CONFIG_DIR";
pub const GROK_HOME_ENV: &str = "GROK_HOME";
pub const HERMES_HOME_ENV: &str = "HERMES_HOME";
pub const XDG_CONFIG_HOME_ENV: &str = "XDG_CONFIG_HOME";
pub const LOCAL_APP_DATA_ENV: &str = "LOCALAPPDATA";
pub const USER_PROFILE_ENV: &str = "USERPROFILE";
pub const HOME_ENV: &str = "HOME";
pub const HOMEDRIVE_ENV: &str = "HOMEDRIVE";
pub const HOMEPATH_ENV: &str = "HOMEPATH";

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct IntegrationEnv {
    pub pi_coding_agent_dir: Option<PathBuf>,
    pub pi_config_dir: Option<PathBuf>,
    pub claude_config_dir: Option<PathBuf>,
    pub codex_home: Option<PathBuf>,
    pub kimi_code_home: Option<PathBuf>,
    pub copilot_home: Option<PathBuf>,
    pub xdg_config_home: Option<PathBuf>,
    pub qoder_config_dir: Option<PathBuf>,
    pub qwen_home: Option<PathBuf>,
    pub cursor_config_dir: Option<PathBuf>,
    pub antigravity_cli_config_dir: Option<PathBuf>,
    pub grok_config_dir: Option<PathBuf>,
    pub grok_home: Option<PathBuf>,
    pub hermes_home: Option<PathBuf>,
    pub local_app_data: Option<PathBuf>,
    pub user_profile: Option<PathBuf>,
    pub home_env: Option<PathBuf>,
    pub homedrive: Option<PathBuf>,
    pub homepath: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntegrationContext {
    pub home: PathBuf,
    pub windows: bool,
    pub env: IntegrationEnv,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntegrationLocatedFile {
    pub install_name: &'static str,
    pub contents: &'static str,
    pub path: PathBuf,
    pub executable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntegrationLayout {
    pub target: IntegrationTarget,
    pub root: PathBuf,
    pub files: Vec<IntegrationLocatedFile>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RootPolicy {
    RequireRoot,
    DestOrParent,
    CreateAlways,
}

impl IntegrationContext {
    /// Caller-owned home, current process platform, and empty override env.
    /// `xdg_config_home` / agent dir fields on `env` are the config roots.
    pub fn new(home: impl Into<PathBuf>) -> Self {
        Self {
            home: home.into(),
            windows: cfg!(windows),
            env: IntegrationEnv::default(),
        }
    }

    /// Snapshot process environment into home, platform, and config-root overrides.
    pub fn from_env() -> io::Result<Self> {
        let env = IntegrationEnv {
            pi_coding_agent_dir: env_path(PI_CODING_AGENT_DIR_ENV),
            pi_config_dir: env_path(PI_CONFIG_DIR_ENV),
            claude_config_dir: env_path(CLAUDE_CONFIG_DIR_ENV),
            codex_home: env_path(CODEX_HOME_ENV),
            kimi_code_home: env_path(KIMI_CODE_HOME_ENV),
            copilot_home: env_path(COPILOT_HOME_ENV),
            xdg_config_home: env_path(XDG_CONFIG_HOME_ENV),
            qoder_config_dir: env_path(QODER_CONFIG_DIR_ENV),
            qwen_home: env_path(QWEN_HOME_ENV),
            cursor_config_dir: env_path(CURSOR_CONFIG_DIR_ENV),
            antigravity_cli_config_dir: env_path(ANTIGRAVITY_CLI_CONFIG_DIR_ENV),
            grok_config_dir: env_path(GROK_CONFIG_DIR_ENV),
            grok_home: env_path(GROK_HOME_ENV),
            hermes_home: env_path(HERMES_HOME_ENV),
            local_app_data: env_path(LOCAL_APP_DATA_ENV),
            user_profile: env_path(USER_PROFILE_ENV),
            home_env: env_path(HOME_ENV),
            homedrive: env_path(HOMEDRIVE_ENV),
            homepath: env_path(HOMEPATH_ENV),
        };
        let windows = cfg!(windows);
        let home = resolve_home(windows, &env)?;
        Ok(Self { home, windows, env })
    }
}

pub fn integration_layout(
    ctx: &IntegrationContext,
    target: IntegrationTarget,
) -> io::Result<IntegrationLayout> {
    let spec = integration_spec(target);
    let root = agent_root(ctx, target)?;
    let files = spec
        .files
        .iter()
        .map(|file| IntegrationLocatedFile {
            install_name: file.install_name,
            contents: file.contents,
            path: join_relative(&root, file.relative_dir, file.install_name),
            executable: file.executable,
        })
        .collect();
    Ok(IntegrationLayout {
        target,
        root,
        files,
    })
}

pub fn integration_root(
    ctx: &IntegrationContext,
    target: IntegrationTarget,
) -> io::Result<PathBuf> {
    Ok(integration_layout(ctx, target)?.root)
}

pub(crate) fn agent_root(
    ctx: &IntegrationContext,
    target: IntegrationTarget,
) -> io::Result<PathBuf> {
    match target {
        IntegrationTarget::Pi => pi_agent_root(ctx),
        IntegrationTarget::Omp => omp_agent_root(ctx),
        IntegrationTarget::Claude => {
            env_or_home(ctx, ctx.env.claude_config_dir.as_deref(), &[".claude"])
        }
        IntegrationTarget::Codex => env_or_home(ctx, ctx.env.codex_home.as_deref(), &[".codex"]),
        IntegrationTarget::Copilot => {
            env_or_home(ctx, ctx.env.copilot_home.as_deref(), &[".copilot"])
        }
        IntegrationTarget::Devin => {
            if let Some(xdg) = ctx.env.xdg_config_home.as_deref() {
                return Ok(expand_tilde(&ctx.home, xdg)?.join("devin"));
            }
            Ok(join_segments(require_home(ctx)?, &[".config", "devin"]))
        }
        IntegrationTarget::Droid => Ok(require_home(ctx)?.join(".factory")),
        IntegrationTarget::Kimi => {
            env_or_home(ctx, ctx.env.kimi_code_home.as_deref(), &[".kimi-code"])
        }
        IntegrationTarget::Opencode => {
            Ok(join_segments(require_home(ctx)?, &[".config", "opencode"]))
        }
        IntegrationTarget::Kilo => Ok(join_segments(require_home(ctx)?, &[".config", "kilo"])),
        IntegrationTarget::Hermes => hermes_root(ctx),
        IntegrationTarget::Qodercli => {
            env_or_home(ctx, ctx.env.qoder_config_dir.as_deref(), &[".qoder"])
        }
        IntegrationTarget::Qwen => env_or_home(ctx, ctx.env.qwen_home.as_deref(), &[".qwen"]),
        IntegrationTarget::Cursor => {
            env_or_home(ctx, ctx.env.cursor_config_dir.as_deref(), &[".cursor"])
        }
        IntegrationTarget::Mastracode => Ok(require_home(ctx)?.join(".mastracode")),
        IntegrationTarget::AntigravityCli => env_or_home(
            ctx,
            ctx.env.antigravity_cli_config_dir.as_deref(),
            &[".gemini", "config"],
        ),
        IntegrationTarget::Grok => grok_root(ctx),
    }
}

pub(crate) fn root_policy(target: IntegrationTarget) -> RootPolicy {
    match target {
        IntegrationTarget::Pi | IntegrationTarget::Omp => RootPolicy::DestOrParent,
        IntegrationTarget::Mastracode => RootPolicy::CreateAlways,
        _ => RootPolicy::RequireRoot,
    }
}

pub(crate) fn missing_root_error(target: IntegrationTarget, path: &Path) -> io::Error {
    let (subject, hint) = match target {
        IntegrationTarget::Pi => ("pi extension directory", "install pi first"),
        IntegrationTarget::Omp => ("omp extension directory", "install omp first"),
        IntegrationTarget::Claude => ("claude directory", "install claude code first"),
        IntegrationTarget::Codex => ("codex config directory", "install codex first"),
        IntegrationTarget::Copilot => (
            "copilot config directory",
            "install github copilot cli first",
        ),
        IntegrationTarget::Devin => ("devin config directory", "install devin cli first"),
        IntegrationTarget::Droid => ("droid config directory", "install droid first"),
        IntegrationTarget::Kimi => ("kimi code config directory", "install kimi code first"),
        IntegrationTarget::Opencode => ("opencode config directory", "install opencode first"),
        IntegrationTarget::Kilo => ("kilo config directory", "install kilo first"),
        IntegrationTarget::Hermes => ("hermes config directory", "install hermes agent first"),
        IntegrationTarget::Qodercli => ("qodercli config directory", "install qodercli first"),
        IntegrationTarget::Qwen => ("qwen code config directory", "install qwen code first"),
        IntegrationTarget::Cursor => ("cursor config directory", "install cursor agent cli first"),
        IntegrationTarget::Mastracode => {
            ("mastracode config directory", "install mastracode first")
        }
        IntegrationTarget::AntigravityCli => (
            "antigravity cli config directory",
            "install antigravity cli first",
        ),
        IntegrationTarget::Grok => ("grok config directory", "install grok cli first"),
    };
    io::Error::other(format!("{subject} not found at {}. {hint}", path.display()))
}

pub(crate) fn pi_extension_dir(ctx: &IntegrationContext) -> io::Result<PathBuf> {
    Ok(pi_agent_root(ctx)?.join("extensions"))
}

pub(crate) fn omp_extension_dir(ctx: &IntegrationContext) -> io::Result<PathBuf> {
    Ok(omp_agent_root(ctx)?.join("extensions"))
}

fn pi_agent_root(ctx: &IntegrationContext) -> io::Result<PathBuf> {
    env_or_home(
        ctx,
        ctx.env.pi_coding_agent_dir.as_deref(),
        &[".pi", "agent"],
    )
}

fn omp_agent_root(ctx: &IntegrationContext) -> io::Result<PathBuf> {
    if let Some(dir) = ctx.env.pi_coding_agent_dir.as_deref() {
        return expand_tilde(&ctx.home, dir);
    }
    let config_dir = ctx
        .env
        .pi_config_dir
        .clone()
        .unwrap_or_else(|| PathBuf::from(".omp"));
    Ok(require_home(ctx)?.join(config_dir).join("agent"))
}

fn hermes_root(ctx: &IntegrationContext) -> io::Result<PathBuf> {
    if let Some(dir) = ctx.env.hermes_home.as_deref() {
        return expand_tilde(&ctx.home, dir);
    }
    if ctx.windows {
        if let Some(home) = ctx
            .env
            .home_env
            .as_ref()
            .filter(|home| ctx.env.user_profile.as_ref() != Some(*home))
        {
            return Ok(home.join(".hermes"));
        }
        if let Some(local_app_data) = ctx.env.local_app_data.as_ref() {
            return Ok(local_app_data.join("hermes"));
        }
    }
    Ok(require_home(ctx)?.join(".hermes"))
}

fn grok_root(ctx: &IntegrationContext) -> io::Result<PathBuf> {
    if let Some(dir) = ctx.env.grok_config_dir.as_deref() {
        return expand_tilde(&ctx.home, dir);
    }
    env_or_home(ctx, ctx.env.grok_home.as_deref(), &[".grok"])
}

fn env_or_home(
    ctx: &IntegrationContext,
    override_dir: Option<&Path>,
    home_segments: &[&str],
) -> io::Result<PathBuf> {
    if let Some(dir) = override_dir.filter(|dir| !dir.as_os_str().is_empty()) {
        return expand_tilde(&ctx.home, dir);
    }
    Ok(join_segments(require_home(ctx)?, home_segments))
}

fn require_home(ctx: &IntegrationContext) -> io::Result<&Path> {
    if ctx.home.as_os_str().is_empty() {
        return Err(io::Error::other(
            "home directory is not set; cannot locate home directory",
        ));
    }
    Ok(&ctx.home)
}

fn resolve_home(windows: bool, env: &IntegrationEnv) -> io::Result<PathBuf> {
    if let Some(home) = env
        .home_env
        .clone()
        .filter(|home| !home.as_os_str().is_empty())
    {
        return Ok(home);
    }
    if windows {
        if let Some(profile) = env
            .user_profile
            .clone()
            .filter(|profile| !profile.as_os_str().is_empty())
        {
            return Ok(profile);
        }
        if let (Some(drive), Some(path)) = (
            env.homedrive
                .as_ref()
                .filter(|drive| !drive.as_os_str().is_empty()),
            env.homepath
                .as_ref()
                .filter(|path| !path.as_os_str().is_empty()),
        ) {
            let mut home = drive.clone();
            home.push(path);
            return Ok(home);
        }
    }
    Err(io::Error::other(
        "home directory is not set; cannot locate home directory",
    ))
}

fn env_path(name: &str) -> Option<PathBuf> {
    std::env::var_os(name)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

pub(crate) fn expand_tilde(home: &Path, path: &Path) -> io::Result<PathBuf> {
    let Some(raw) = path.to_str() else {
        return Ok(path.to_path_buf());
    };
    if raw == "~" || raw.starts_with("~/") || raw.starts_with("~\\") || raw.starts_with('~') {
        if home.as_os_str().is_empty() {
            return Err(io::Error::other(
                "home directory is not set; cannot locate home directory",
            ));
        }
        if raw == "~" {
            return Ok(home.to_path_buf());
        }
        if let Some(rest) = raw
            .strip_prefix("~/")
            .or_else(|| raw.strip_prefix("~\\"))
            .or_else(|| raw.strip_prefix('~'))
        {
            return Ok(home.join(rest));
        }
    }
    Ok(path.to_path_buf())
}

fn join_segments(root: &Path, segments: &[&str]) -> PathBuf {
    let mut path = root.to_path_buf();
    for segment in segments {
        path.push(segment);
    }
    path
}

fn join_relative(root: &Path, relative_dir: &str, name: &str) -> PathBuf {
    let mut path = root.to_path_buf();
    if !relative_dir.is_empty() {
        for segment in relative_dir.split('/') {
            if !segment.is_empty() {
                path.push(segment);
            }
        }
    }
    path.push(name);
    path
}

pub(crate) fn file_parent(file: &IntegrationLocatedFile) -> io::Result<PathBuf> {
    file.path.parent().map(Path::to_path_buf).ok_or_else(|| {
        io::Error::other(format!(
            "integration file {} has no parent directory",
            file.path.display()
        ))
    })
}

impl IntegrationLayout {
    pub fn primary_path(&self) -> &Path {
        if self.target == IntegrationTarget::Hermes {
            if let Some(file) = self
                .files
                .iter()
                .find(|file| file.install_name == "__init__.py")
            {
                return &file.path;
            }
        }
        &self.files[0].path
    }

    pub fn spec_files(&self) -> &'static [IntegrationFile] {
        integration_spec(self.target).files
    }
}

#[cfg(test)]
mod tests {
    use super::IntegrationTarget;
    use super::*;
    use std::path::PathBuf;

    fn unix_ctx(home: &str) -> IntegrationContext {
        IntegrationContext {
            home: PathBuf::from(home),
            windows: false,
            env: IntegrationEnv::default(),
        }
    }

    fn join(home: &str, parts: &[&str]) -> PathBuf {
        let mut path = PathBuf::from(home);
        for part in parts {
            path.push(part);
        }
        path
    }

    #[test]
    fn new_owns_home_without_process_env() {
        let ctx = IntegrationContext::new("/tmp/herdr-home");
        assert_eq!(ctx.home, PathBuf::from("/tmp/herdr-home"));
        assert_eq!(ctx.windows, cfg!(windows));
        assert_eq!(ctx.env, IntegrationEnv::default());
    }

    #[test]
    fn unix_destinations_match_historical_herdr_layout() {
        let home = "/home/me";
        let ctx = unix_ctx(home);
        let cases: &[(IntegrationTarget, &[&str])] = &[
            (
                IntegrationTarget::Pi,
                &[".pi", "agent", "extensions", "herdr-agent-state.ts"],
            ),
            (
                IntegrationTarget::Omp,
                &[".omp", "agent", "extensions", "herdr-omp-agent-state.ts"],
            ),
            (IntegrationTarget::Claude, &[".claude", "hooks"]),
            (IntegrationTarget::Codex, &[".codex"]),
            (IntegrationTarget::Copilot, &[".copilot", "hooks"]),
            (IntegrationTarget::Devin, &[".config", "devin"]),
            (IntegrationTarget::Droid, &[".factory", "hooks"]),
            (IntegrationTarget::Kimi, &[".kimi-code", "hooks"]),
            (
                IntegrationTarget::Opencode,
                &[".config", "opencode", "plugins", "herdr-agent-state.js"],
            ),
            (
                IntegrationTarget::Kilo,
                &[".config", "kilo", "plugin", "herdr-agent-state.js"],
            ),
            (
                IntegrationTarget::Hermes,
                &[".hermes", "plugins", "herdr-agent-state", "plugin.yaml"],
            ),
            (IntegrationTarget::Qodercli, &[".qoder", "hooks"]),
            (IntegrationTarget::Qwen, &[".qwen", "hooks"]),
            (IntegrationTarget::Cursor, &[".cursor"]),
            (IntegrationTarget::Mastracode, &[".mastracode", "hooks"]),
            (
                IntegrationTarget::AntigravityCli,
                &[".gemini", "config", "hooks"],
            ),
            (IntegrationTarget::Grok, &[".grok", "hooks"]),
        ];
        for (target, parts) in cases {
            let layout = integration_layout(&ctx, *target).unwrap();
            let expected = if matches!(
                target,
                IntegrationTarget::Pi
                    | IntegrationTarget::Omp
                    | IntegrationTarget::Opencode
                    | IntegrationTarget::Kilo
                    | IntegrationTarget::Hermes
            ) {
                join(home, parts)
            } else {
                join(home, parts).join(layout.files[0].install_name)
            };
            assert_eq!(layout.files[0].path, expected, "{target:?}");
        }
        let opencode = integration_layout(&ctx, IntegrationTarget::Opencode).unwrap();
        assert_eq!(opencode.files.len(), 2);
        assert_eq!(
            opencode.files[1].path,
            join(home, &[".config", "opencode", "herdr-tui-session.js"])
        );
        let hermes = integration_layout(&ctx, IntegrationTarget::Hermes).unwrap();
        assert_eq!(hermes.files.len(), 2);
        assert_eq!(
            hermes.primary_path(),
            join(
                home,
                &[".hermes", "plugins", "herdr-agent-state", "__init__.py"]
            )
            .as_path()
        );
    }

    #[test]
    fn windows_hermes_uses_local_app_data_when_home_matches_profile() {
        let home = PathBuf::from(r"C:\Users\me");
        let local = PathBuf::from(r"C:\Users\me\AppData\Local");
        let ctx = IntegrationContext {
            home: home.clone(),
            windows: true,
            env: IntegrationEnv {
                local_app_data: Some(local.clone()),
                user_profile: Some(home.clone()),
                ..Default::default()
            },
        };
        let layout = integration_layout(&ctx, IntegrationTarget::Hermes).unwrap();
        assert_eq!(layout.root, local.join("hermes"));
    }

    #[test]
    fn windows_hermes_uses_explicit_home_when_distinct_from_profile() {
        let ctx = IntegrationContext {
            home: PathBuf::from(r"C:\Users\me"),
            windows: true,
            env: IntegrationEnv {
                home_env: Some(PathBuf::from(r"C:\explicit-home")),
                user_profile: Some(PathBuf::from(r"C:\Users\me")),
                local_app_data: Some(PathBuf::from(r"C:\Users\me\AppData\Local")),
                ..Default::default()
            },
        };
        let layout = integration_layout(&ctx, IntegrationTarget::Hermes).unwrap();
        assert_eq!(
            layout.root,
            PathBuf::from(r"C:\explicit-home").join(".hermes")
        );
    }

    #[test]
    fn claude_config_dir_override_expands_tilde() {
        let ctx = IntegrationContext {
            home: PathBuf::from("/home/me"),
            windows: false,
            env: IntegrationEnv {
                claude_config_dir: Some(PathBuf::from("~/.custom-claude")),
                ..Default::default()
            },
        };
        let layout = integration_layout(&ctx, IntegrationTarget::Claude).unwrap();
        assert_eq!(layout.root, PathBuf::from("/home/me/.custom-claude"));
    }
}
