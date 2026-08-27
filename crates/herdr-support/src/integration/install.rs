use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use super::host::{
    host_status, install_host, remove_empty_hermes_plugin_dir, uninstall_host, HostConfigChange,
};
use super::layout::{
    file_parent, integration_layout, missing_root_error, omp_extension_dir, pi_extension_dir,
    root_policy, IntegrationContext, IntegrationLayout, IntegrationLocatedFile, RootPolicy,
};
use super::opencode_config::validate_tui_plugin_config;
use super::state::{IntegrationFileState, IntegrationFileStatus};
use super::{integration_spec, IntegrationTarget};

const VERSION_MARKER: &str = "HERDR_INTEGRATION_VERSION=";
const ID_MARKER: &str = "HERDR_INTEGRATION_ID=";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntegrationStatus {
    pub target: IntegrationTarget,
    pub path: PathBuf,
    pub state: IntegrationFileState,
    pub files: Vec<IntegrationFileStatus>,
    pub host: Vec<IntegrationFileStatus>,
    pub installed_version: Option<u32>,
    pub expected_version: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntegrationInstallOutcome {
    pub paths: Vec<PathBuf>,
    pub written: Vec<PathBuf>,
    pub skipped: Vec<PathBuf>,
    pub host: Vec<HostConfigChange>,
    pub extras: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntegrationUninstallOutcome {
    pub paths: Vec<PathBuf>,
    pub removed: Vec<PathBuf>,
    pub skipped: Vec<PathBuf>,
    pub host: Vec<HostConfigChange>,
    pub extras: Vec<String>,
}

pub fn parse_integration_version(content: &str) -> Option<u32> {
    content
        .lines()
        .find_map(|line| marker_value(line, VERSION_MARKER)?.parse().ok())
}

pub fn parse_integration_id(content: &str) -> Option<&str> {
    content
        .lines()
        .find_map(|line| marker_value(line, ID_MARKER))
}

pub fn integration_status(
    ctx: &IntegrationContext,
    target: IntegrationTarget,
) -> io::Result<IntegrationStatus> {
    let spec = integration_spec(target);
    let layout = integration_layout(ctx, target)?;
    let files: Vec<IntegrationFileStatus> = layout
        .files
        .iter()
        .map(|file| file_status(file, spec.version))
        .collect();
    let host = host_status(&layout);
    let state = aggregate_states(files.iter().chain(host.iter()).map(|file| file.state));
    let installed_version = files.iter().find_map(|file| file.installed_version);
    Ok(IntegrationStatus {
        target,
        path: layout.primary_path().to_path_buf(),
        state,
        files,
        host,
        installed_version,
        expected_version: spec.version,
    })
}

pub fn integration_statuses(ctx: &IntegrationContext) -> Vec<IntegrationStatus> {
    IntegrationTarget::ALL
        .iter()
        .copied()
        .filter_map(|target| integration_status(ctx, target).ok())
        .collect()
}

pub fn install_integration(
    ctx: &IntegrationContext,
    target: IntegrationTarget,
    force: bool,
) -> io::Result<IntegrationInstallOutcome> {
    if target == IntegrationTarget::Omp {
        let pi_dir = pi_extension_dir(ctx)?;
        let omp_dir = omp_extension_dir(ctx)?;
        if pi_dir == omp_dir {
            return Err(io::Error::other(format!(
                "Pi and OMP resolve to the same extension directory at {}; configure separate agent directories before installing OMP",
                omp_dir.display()
            )));
        }
    }

    if target == IntegrationTarget::Opencode {
        let layout = integration_layout(ctx, target)?;
        validate_tui_plugin_config(&layout.root)?;
    }

    let status = integration_status(ctx, target)?;
    refuse_if_protected(target, &status, force, Action::Install)?;
    let layout = integration_layout(ctx, target)?;
    ensure_install_dirs(&layout)?;
    let mut extras = Vec::new();
    if target == IntegrationTarget::Omp {
        let omp_dir = omp_extension_dir(ctx)?;
        if remove_legacy_pi_extension_from_omp_dir(&omp_dir)? {
            let spec = integration_spec(IntegrationTarget::Pi);
            extras.push(format!(
                "removed legacy pi integration from omp extension directory at {}",
                omp_dir.join(spec.files[0].install_name).display()
            ));
        }
    }

    let mut written = Vec::new();
    let mut skipped = Vec::new();
    let mut paths = Vec::new();
    for file in &layout.files {
        paths.push(file.path.clone());
        let already_current = status
            .files
            .iter()
            .find(|entry| entry.path == file.path)
            .is_some_and(|entry| entry.state == IntegrationFileState::Current);
        if already_current && !force {
            skipped.push(file.path.clone());
            continue;
        }
        fs::write(&file.path, file.contents)?;
        if file.executable {
            make_executable(&file.path)?;
        }
        written.push(file.path.clone());
    }

    remove_legacy_bash_hook(&layout)?;
    let host = install_host(&layout)?;
    extras.extend(host.extras);

    Ok(IntegrationInstallOutcome {
        paths,
        written,
        skipped,
        host: host.changes,
        extras,
    })
}

pub fn uninstall_integration(
    ctx: &IntegrationContext,
    target: IntegrationTarget,
    force: bool,
) -> io::Result<IntegrationUninstallOutcome> {
    let status = integration_status(ctx, target)?;
    refuse_if_protected(target, &status, force, Action::Uninstall)?;
    let layout = integration_layout(ctx, target)?;

    let mut removed = Vec::new();
    let mut skipped = Vec::new();
    let mut paths = Vec::new();
    for file in &layout.files {
        paths.push(file.path.clone());
        match remove_file_if_exists(&file.path)? {
            true => removed.push(file.path.clone()),
            false => skipped.push(file.path.clone()),
        }
    }
    remove_legacy_bash_hook(&layout)?;
    let mut extras = Vec::new();
    if target == IntegrationTarget::Hermes && remove_empty_hermes_plugin_dir(&layout)? {
        extras.push("removed empty hermes plugin directory".to_string());
    }
    let host = uninstall_host(&layout, force)?;
    extras.extend(host.extras);

    Ok(IntegrationUninstallOutcome {
        paths,
        removed,
        skipped,
        host: host.changes,
        extras,
    })
}

#[derive(Clone, Copy)]
enum Action {
    Install,
    Uninstall,
}

fn refuse_if_protected(
    target: IntegrationTarget,
    status: &IntegrationStatus,
    force: bool,
    action: Action,
) -> io::Result<()> {
    if force {
        return Ok(());
    }
    let protected = status.files.iter().chain(status.host.iter()).find(|file| {
        matches!(
            file.state,
            IntegrationFileState::Modified | IntegrationFileState::Unowned
        )
    });
    let Some(file) = protected else {
        return Ok(());
    };
    let label = integration_spec(target).label;
    let kind = match file.state {
        IntegrationFileState::Modified => "modified",
        IntegrationFileState::Unowned => "unowned",
        _ => unreachable!(),
    };
    let message = match action {
        Action::Install => format!(
            "refusing to overwrite {kind} {label} integration at {}; reinstall with force to replace it",
            file.path.display()
        ),
        Action::Uninstall => format!(
            "refusing to remove {kind} {label} integration at {}; uninstall with force to delete it",
            file.path.display()
        ),
    };
    Err(io::Error::other(message))
}

fn file_status(file: &IntegrationLocatedFile, expected_version: u32) -> IntegrationFileStatus {
    if !file.path.is_file() {
        return IntegrationFileStatus {
            path: file.path.clone(),
            state: IntegrationFileState::Missing,
            installed_version: None,
        };
    }
    let on_disk = match fs::read(&file.path) {
        Ok(bytes) => bytes,
        Err(_) => {
            return IntegrationFileStatus {
                path: file.path.clone(),
                state: IntegrationFileState::Unowned,
                installed_version: None,
            };
        }
    };
    if on_disk == file.contents.as_bytes() {
        return IntegrationFileStatus {
            path: file.path.clone(),
            state: IntegrationFileState::Current,
            installed_version: parse_integration_version(file.contents),
        };
    }
    let text = String::from_utf8_lossy(&on_disk);
    let installed_version = parse_integration_version(&text);
    let bundled_id = parse_integration_id(file.contents);
    let disk_id = parse_integration_id(&text);
    let state = if bundled_id.is_some() && bundled_id == disk_id {
        if installed_version.is_some_and(|version| version < expected_version) {
            IntegrationFileState::Outdated
        } else {
            IntegrationFileState::Modified
        }
    } else {
        IntegrationFileState::Unowned
    };
    IntegrationFileStatus {
        path: file.path.clone(),
        state,
        installed_version,
    }
}

fn aggregate_states<I>(states: I) -> IntegrationFileState
where
    I: IntoIterator<Item = IntegrationFileState>,
{
    let states: Vec<IntegrationFileState> = states.into_iter().collect();
    if states
        .iter()
        .all(|state| *state == IntegrationFileState::Missing)
    {
        return IntegrationFileState::Missing;
    }
    if states
        .iter()
        .all(|state| *state == IntegrationFileState::Current)
    {
        return IntegrationFileState::Current;
    }
    if states
        .iter()
        .any(|state| *state == IntegrationFileState::Unowned)
    {
        return IntegrationFileState::Unowned;
    }
    if states
        .iter()
        .any(|state| *state == IntegrationFileState::Modified)
    {
        return IntegrationFileState::Modified;
    }
    IntegrationFileState::Outdated
}

fn ensure_install_dirs(layout: &IntegrationLayout) -> io::Result<()> {
    match root_policy(layout.target) {
        RootPolicy::RequireRoot => {
            if !layout.root.is_dir() {
                return Err(missing_root_error(layout.target, &layout.root));
            }
        }
        RootPolicy::DestOrParent => {
            let dest_dir = file_parent(&layout.files[0])?;
            if dest_dir.is_dir() {
                // already present
            } else if dest_dir.parent().is_some_and(Path::is_dir) {
                fs::create_dir_all(&dest_dir)?;
            } else {
                return Err(missing_root_error(layout.target, &dest_dir));
            }
        }
        RootPolicy::CreateAlways => {}
    }
    for file in &layout.files {
        if let Some(parent) = file.path.parent() {
            fs::create_dir_all(parent)?;
        }
    }
    Ok(())
}

fn make_executable(path: &Path) -> io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(path, perms)?;
    }
    let _ = path;
    Ok(())
}

fn remove_file_if_exists(path: &Path) -> io::Result<bool> {
    match fs::remove_file(path) {
        Ok(()) => Ok(true),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(err) => Err(err),
    }
}

fn remove_legacy_pi_extension_from_omp_dir(dir: &Path) -> io::Result<bool> {
    let spec = integration_spec(IntegrationTarget::Pi);
    let legacy_path = dir.join(spec.files[0].install_name);
    if !legacy_path.is_file() {
        return Ok(false);
    }
    let content = fs::read_to_string(&legacy_path)?;
    if parse_integration_id(&content) == Some("pi") {
        fs::remove_file(legacy_path)?;
        return Ok(true);
    }
    Ok(false)
}

fn remove_legacy_bash_hook(layout: &IntegrationLayout) -> io::Result<bool> {
    #[cfg(windows)]
    {
        let mut removed = false;
        for file in &layout.files {
            if file
                .path
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("ps1"))
            {
                let legacy_path = file.path.with_file_name("herdr-agent-state.sh");
                let content = match fs::read_to_string(&legacy_path) {
                    Ok(content) => content,
                    Err(err) if err.kind() == io::ErrorKind::NotFound => continue,
                    Err(err) => return Err(err),
                };
                if content.contains("HERDR_INTEGRATION_ID=") {
                    fs::remove_file(legacy_path)?;
                    removed = true;
                }
            }
        }
        return Ok(removed);
    }
    #[cfg(not(windows))]
    {
        let _ = layout;
        Ok(false)
    }
}

fn marker_value<'a>(line: &'a str, marker: &str) -> Option<&'a str> {
    let marker_line = line
        .trim()
        .trim_start_matches('/')
        .trim_start_matches('#')
        .trim();
    marker_line
        .strip_prefix(marker)
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::integration::bundled_integration_files;
    use crate::integration::layout::IntegrationContext;
    use crate::{integration_spec, integration_targets, IntegrationTarget};
    use std::collections::BTreeSet;
    use std::process::Command;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn test_ctx(home: &Path) -> IntegrationContext {
        let mut ctx = IntegrationContext::new(home);
        ctx.windows = false;
        ctx
    }

    fn unique_home() -> PathBuf {
        let base = std::env::temp_dir().join(format!(
            "herdr-support-integration-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&base).unwrap();
        base
    }

    fn language_family(path: &str) -> Option<&'static str> {
        if path.ends_with(".test.ts") || path.ends_with(".test.js") {
            return None;
        }
        if path.ends_with(".sh") {
            Some("sh")
        } else if path.ends_with(".ps1") {
            Some("ps1")
        } else if path.ends_with(".js") {
            Some("js")
        } else if path.ends_with(".ts") {
            Some("ts")
        } else if path.ends_with(".py") {
            Some("py")
        } else {
            None
        }
    }

    #[test]
    fn statuses_iterate_every_target_in_registry_order() {
        let home = unique_home();
        let ctx = test_ctx(&home);
        let statuses = integration_statuses(&ctx);
        let targets: Vec<_> = statuses.iter().map(|status| status.target).collect();
        assert_eq!(targets, integration_targets());
        assert_eq!(targets, IntegrationTarget::ALL);
        fs::remove_dir_all(home).ok();
    }

    #[test]
    fn pane_fallback_present_in_every_language_family() {
        let mut families = BTreeSet::new();
        for (path, contents) in bundled_integration_files() {
            let Some(family) = language_family(path) else {
                continue;
            };
            if !contents.contains("HERDR_PANE_ID") {
                continue;
            }
            families.insert(family);
            match family {
                "sh" => {
                    assert!(
                        contents.contains("${HERDR_PANE_ID:-${TMUX_PANE:-}}")
                            || contents.contains(
                                "os.environ.get(\"HERDR_PANE_ID\") or os.environ.get(\"TMUX_PANE\")"
                            ),
                        "{path} sh fallback"
                    );
                }
                "ps1" => {
                    assert!(contents.contains("$env:TMUX_PANE"), "{path} ps1 fallback");
                    assert!(contents.contains("$paneId"), "{path} resolved pane id");
                }
                "js" | "ts" => {
                    assert!(
                        contents.contains("process.env.HERDR_PANE_ID || process.env.TMUX_PANE"),
                        "{path} js/ts fallback"
                    );
                }
                "py" => {
                    assert!(
                        contents.contains("os.environ.get(\"TMUX_PANE\")"),
                        "{path} py fallback"
                    );
                }
                _ => {}
            }
        }
        assert!(families.contains("sh"));
        assert!(families.contains("ps1"));
        assert!(families.contains("js"));
        assert!(families.contains("ts"));
        assert!(families.contains("py"));
    }

    #[cfg(unix)]
    #[test]
    fn sh_pane_fallback_prefers_herdr_then_tmux() {
        let script = r#"
pane_id="${HERDR_PANE_ID:-${TMUX_PANE:-}}"
printf '%s' "$pane_id"
"#;
        let herdr = Command::new("sh")
            .arg("-c")
            .arg(script)
            .env("HERDR_PANE_ID", "%herdr")
            .env("TMUX_PANE", "%tmux")
            .output()
            .unwrap();
        assert_eq!(String::from_utf8_lossy(&herdr.stdout), "%herdr");
        let tmux = Command::new("sh")
            .arg("-c")
            .arg(script)
            .env_remove("HERDR_PANE_ID")
            .env("TMUX_PANE", "%tmux")
            .output()
            .unwrap();
        assert_eq!(String::from_utf8_lossy(&tmux.stdout), "%tmux");
    }

    #[cfg(unix)]
    #[test]
    fn python_pane_fallback_prefers_herdr_then_tmux() {
        if let Err(err) = Command::new("python3")
            .arg("-c")
            .arg("raise SystemExit(0)")
            .output()
        {
            if err.kind() == std::io::ErrorKind::NotFound {
                return;
            }
            panic!("{err}");
        }
        let script =
            "import os; print(os.environ.get('HERDR_PANE_ID') or os.environ.get('TMUX_PANE') or '', end='')";
        let herdr = Command::new("python3")
            .arg("-c")
            .arg(script)
            .env("HERDR_PANE_ID", "%herdr")
            .env("TMUX_PANE", "%tmux")
            .output()
            .unwrap();
        assert_eq!(String::from_utf8_lossy(&herdr.stdout), "%herdr");
        let tmux = Command::new("python3")
            .arg("-c")
            .arg(script)
            .env_remove("HERDR_PANE_ID")
            .env("TMUX_PANE", "%tmux")
            .output()
            .unwrap();
        assert_eq!(String::from_utf8_lossy(&tmux.stdout), "%tmux");
    }

    #[test]
    fn js_pane_fallback_prefers_herdr_then_tmux() {
        if let Err(err) = Command::new("node")
            .arg("-e")
            .arg("process.exit(0)")
            .output()
        {
            if err.kind() == std::io::ErrorKind::NotFound {
                return;
            }
            panic!("{err}");
        }
        let script =
            "process.stdout.write(process.env.HERDR_PANE_ID || process.env.TMUX_PANE || '')";
        let run = |herdr: Option<&str>, tmux: &str| {
            let mut cmd = Command::new("node");
            cmd.arg("-e").arg(script).env("TMUX_PANE", tmux);
            match herdr {
                Some(value) => {
                    cmd.env("HERDR_PANE_ID", value);
                }
                None => {
                    cmd.env_remove("HERDR_PANE_ID");
                }
            }
            String::from_utf8(cmd.output().unwrap().stdout).unwrap()
        };
        assert_eq!(run(Some("%herdr"), "%tmux"), "%herdr");
        assert_eq!(run(None, "%tmux"), "%tmux");
    }

    #[test]
    fn powershell_pane_fallback_source_resolves_tmux() {
        let scripts = bundled_integration_files()
            .iter()
            .filter(|(path, _)| path.ends_with(".ps1"))
            .collect::<Vec<_>>();
        assert!(!scripts.is_empty());
        for (path, contents) in scripts {
            assert!(
                contents.contains("$env:HERDR_PANE_ID") && contents.contains("$env:TMUX_PANE"),
                "{path} must fall back from HERDR_PANE_ID to TMUX_PANE"
            );
        }
    }

    fn prepare_pi(home: &Path) -> PathBuf {
        let dir = home.join(".pi/agent/extensions");
        fs::create_dir_all(dir.parent().unwrap()).unwrap();
        dir
    }

    fn prepare_hermes(home: &Path) {
        fs::create_dir_all(home.join(".hermes")).unwrap();
    }

    #[test]
    fn pi_status_current_outdated_missing_modified_unowned() {
        let home = unique_home();
        let ctx = test_ctx(&home);
        let spec = integration_spec(IntegrationTarget::Pi);
        assert_eq!(
            integration_status(&ctx, IntegrationTarget::Pi)
                .unwrap()
                .state,
            IntegrationFileState::Missing
        );

        let dir = prepare_pi(&home);
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join(spec.files[0].install_name);

        fs::write(&path, spec.files[0].contents).unwrap();
        let current = integration_status(&ctx, IntegrationTarget::Pi).unwrap();
        assert_eq!(current.state, IntegrationFileState::Current);
        assert_eq!(current.installed_version, Some(spec.version));

        fs::write(
            &path,
            "// HERDR_INTEGRATION_ID=pi\n// HERDR_INTEGRATION_VERSION=4\n",
        )
        .unwrap();
        assert_eq!(
            integration_status(&ctx, IntegrationTarget::Pi)
                .unwrap()
                .state,
            IntegrationFileState::Outdated
        );

        fs::write(
            &path,
            format!(
                "// HERDR_INTEGRATION_ID=pi\n// HERDR_INTEGRATION_VERSION={}\nedited\n",
                spec.version
            ),
        )
        .unwrap();
        assert_eq!(
            integration_status(&ctx, IntegrationTarget::Pi)
                .unwrap()
                .state,
            IntegrationFileState::Modified
        );

        fs::write(&path, "not a herdr file\n").unwrap();
        assert_eq!(
            integration_status(&ctx, IntegrationTarget::Pi)
                .unwrap()
                .state,
            IntegrationFileState::Unowned
        );

        fs::remove_dir_all(home).ok();
    }

    #[test]
    fn two_file_hermes_current_and_partial_is_outdated() {
        let home = unique_home();
        let ctx = test_ctx(&home);
        prepare_hermes(&home);
        let layout = integration_layout(&ctx, IntegrationTarget::Hermes).unwrap();
        assert_eq!(layout.files.len(), 2);

        assert_eq!(
            integration_status(&ctx, IntegrationTarget::Hermes)
                .unwrap()
                .state,
            IntegrationFileState::Missing
        );

        install_integration(&ctx, IntegrationTarget::Hermes, false).unwrap();
        let current = integration_status(&ctx, IntegrationTarget::Hermes).unwrap();
        assert_eq!(current.state, IntegrationFileState::Current);
        assert_eq!(current.files.len(), 2);
        assert!(current
            .files
            .iter()
            .all(|file| file.state == IntegrationFileState::Current));

        fs::remove_file(&layout.files[1].path).unwrap();
        assert_eq!(
            integration_status(&ctx, IntegrationTarget::Hermes)
                .unwrap()
                .state,
            IntegrationFileState::Outdated
        );

        fs::write(&layout.files[1].path, "user plugin\n").unwrap();
        assert_eq!(
            integration_status(&ctx, IntegrationTarget::Hermes)
                .unwrap()
                .state,
            IntegrationFileState::Unowned
        );

        fs::create_dir_all(home.join(".config/opencode")).unwrap();
        install_integration(&ctx, IntegrationTarget::Opencode, false).unwrap();
        let opencode = integration_layout(&ctx, IntegrationTarget::Opencode).unwrap();
        assert_eq!(opencode.files.len(), 2);
        assert_eq!(
            integration_status(&ctx, IntegrationTarget::Opencode)
                .unwrap()
                .state,
            IntegrationFileState::Current
        );
        fs::remove_file(&opencode.files[1].path).unwrap();
        assert_eq!(
            integration_status(&ctx, IntegrationTarget::Opencode)
                .unwrap()
                .state,
            IntegrationFileState::Outdated
        );

        fs::remove_dir_all(home).ok();
    }

    #[test]
    fn install_skips_current_and_force_overwrites_modified() {
        let home = unique_home();
        let ctx = test_ctx(&home);
        prepare_pi(&home);
        fs::create_dir_all(home.join(".pi/agent")).unwrap();

        let first = install_integration(&ctx, IntegrationTarget::Pi, false).unwrap();
        assert_eq!(first.written.len(), 1);
        let again = install_integration(&ctx, IntegrationTarget::Pi, false).unwrap();
        assert!(again.written.is_empty());
        assert_eq!(again.skipped.len(), 1);

        fs::write(&first.paths[0], "user edit\n").unwrap();
        let refused = install_integration(&ctx, IntegrationTarget::Pi, false).unwrap_err();
        assert!(refused.to_string().contains("unowned"));

        fs::write(
            &first.paths[0],
            format!(
                "// HERDR_INTEGRATION_ID=pi\n// HERDR_INTEGRATION_VERSION={}\nchanged\n",
                integration_spec(IntegrationTarget::Pi).version
            ),
        )
        .unwrap();
        let refused_modified = install_integration(&ctx, IntegrationTarget::Pi, false).unwrap_err();
        assert!(refused_modified.to_string().contains("modified"));

        let forced = install_integration(&ctx, IntegrationTarget::Pi, true).unwrap();
        assert_eq!(forced.written.len(), 1);
        assert_eq!(
            fs::read_to_string(&forced.paths[0]).unwrap(),
            integration_spec(IntegrationTarget::Pi).files[0].contents
        );

        fs::remove_dir_all(home).ok();
    }

    #[test]
    fn uninstall_byte_match_safety() {
        let home = unique_home();
        let ctx = test_ctx(&home);
        fs::create_dir_all(home.join(".pi/agent")).unwrap();
        install_integration(&ctx, IntegrationTarget::Pi, false).unwrap();
        let layout = integration_layout(&ctx, IntegrationTarget::Pi).unwrap();
        let path = &layout.files[0].path;

        let removed = uninstall_integration(&ctx, IntegrationTarget::Pi, false).unwrap();
        assert_eq!(removed.removed.len(), 1);
        assert!(!path.exists());

        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, "custom\n").unwrap();
        let refused = uninstall_integration(&ctx, IntegrationTarget::Pi, false).unwrap_err();
        assert!(refused.to_string().contains("unowned"));
        assert!(path.exists());

        uninstall_integration(&ctx, IntegrationTarget::Pi, true).unwrap();
        assert!(!path.exists());

        fs::remove_dir_all(home).ok();
    }

    #[test]
    fn outdated_managed_version_installs_without_force() {
        let home = unique_home();
        let ctx = test_ctx(&home);
        let spec = integration_spec(IntegrationTarget::Pi);
        let dir = prepare_pi(&home);
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join(spec.files[0].install_name),
            "// HERDR_INTEGRATION_ID=pi\n// HERDR_INTEGRATION_VERSION=1\n",
        )
        .unwrap();
        install_integration(&ctx, IntegrationTarget::Pi, false).unwrap();
        assert_eq!(
            integration_status(&ctx, IntegrationTarget::Pi)
                .unwrap()
                .state,
            IntegrationFileState::Current
        );
        fs::remove_dir_all(home).ok();
    }

    #[test]
    fn install_requires_existing_agent_root() {
        let home = unique_home();
        let ctx = test_ctx(&home);
        let err = install_integration(&ctx, IntegrationTarget::Pi, false).unwrap_err();
        assert!(err.to_string().contains("pi extension directory not found"));
        fs::remove_dir_all(home).ok();
    }

    #[test]
    fn every_target_completes_install_status_uninstall_lifecycle() {
        for target in IntegrationTarget::ALL {
            let home = unique_home();
            let ctx = test_ctx(&home);
            let layout = integration_layout(&ctx, target).unwrap();
            fs::create_dir_all(&layout.root).unwrap();

            let installed = install_integration(&ctx, target, false).unwrap_or_else(|err| {
                panic!("{} install failed: {err}", integration_spec(target).label)
            });
            assert_eq!(installed.paths.len(), integration_spec(target).files.len());
            assert_eq!(
                integration_status(&ctx, target).unwrap().state,
                IntegrationFileState::Current,
                "{} did not become current",
                integration_spec(target).label
            );

            uninstall_integration(&ctx, target, false).unwrap_or_else(|err| {
                panic!("{} uninstall failed: {err}", integration_spec(target).label)
            });
            assert_eq!(
                integration_status(&ctx, target).unwrap().state,
                IntegrationFileState::Missing,
                "{} did not become missing",
                integration_spec(target).label
            );

            fs::remove_dir_all(home).ok();
        }
    }

    #[test]
    fn platform_destinations_match_unix_layout() {
        let home = unique_home();
        let ctx = test_ctx(&home);
        let claude = integration_layout(&ctx, IntegrationTarget::Claude).unwrap();
        assert_eq!(
            claude.files[0].path,
            home.join(".claude/hooks")
                .join(claude.files[0].install_name)
        );
        let opencode = integration_layout(&ctx, IntegrationTarget::Opencode).unwrap();
        assert_eq!(opencode.files.len(), 2);
        assert_eq!(
            opencode.files[0].path,
            home.join(".config/opencode/plugins")
                .join(opencode.files[0].install_name)
        );
        assert_eq!(
            opencode.files[1].path,
            home.join(".config/opencode")
                .join(opencode.files[1].install_name)
        );
        let hermes = integration_layout(&ctx, IntegrationTarget::Hermes).unwrap();
        assert_eq!(hermes.files.len(), 2);
        assert!(hermes.primary_path().ends_with("__init__.py"));
        fs::remove_dir_all(home).ok();
    }
}
