use std::fs;
use std::io;
use std::path::Path;

use jsonc_parser::cst::{CstInputValue, CstRootNode};
use jsonc_parser::ParseOptions;

const EXTENSIONS_KEY: &str = "extensions";

pub(crate) fn install(settings_path: &Path, extension_path: &Path) -> io::Result<bool> {
    let content = if settings_path.is_file() {
        fs::read_to_string(settings_path)?
    } else {
        "{}\n".to_string()
    };
    let root = parse_root(&content, settings_path)?;
    let object = root_object(&root, settings_path)?;
    let extension = extension_path_string(extension_path)?;

    match object.get(EXTENSIONS_KEY) {
        Some(property) => {
            let extensions = property
                .array_value()
                .ok_or_else(|| invalid_extensions(settings_path))?;
            if extensions.elements().iter().any(|entry| {
                entry
                    .to_serde_value()
                    .is_some_and(|value| value.as_str() == Some(extension))
            }) {
                return Ok(false);
            }
            extensions.append(CstInputValue::String(extension.to_string()));
        }
        None => {
            object.append(
                EXTENSIONS_KEY,
                CstInputValue::Array(vec![CstInputValue::String(extension.to_string())]),
            );
        }
    }

    fs::write(settings_path, root.to_string())?;
    Ok(true)
}

pub(crate) fn uninstall(settings_path: &Path, extension_path: &Path) -> io::Result<bool> {
    if !settings_path.is_file() {
        return Ok(false);
    }

    let content = fs::read_to_string(settings_path)?;
    let root = parse_root(&content, settings_path)?;
    let object = root_object(&root, settings_path)?;
    let Some(property) = object.get(EXTENSIONS_KEY) else {
        return Ok(false);
    };
    let extensions = property
        .array_value()
        .ok_or_else(|| invalid_extensions(settings_path))?;
    let extension = extension_path_string(extension_path)?;
    let mut removed = false;
    for entry in extensions.elements() {
        if entry
            .to_serde_value()
            .is_some_and(|value| value.as_str() == Some(extension))
        {
            entry.remove();
            removed = true;
        }
    }
    if !removed {
        return Ok(false);
    }
    if extensions.elements().is_empty() {
        property.remove();
    }

    fs::write(settings_path, root.to_string())?;
    Ok(true)
}

pub(crate) fn is_configured(settings_path: &Path, extension_path: &Path) -> io::Result<bool> {
    if !settings_path.is_file() {
        return Ok(false);
    }

    let content = fs::read_to_string(settings_path)?;
    let root = parse_root(&content, settings_path)?;
    let object = root_object(&root, settings_path)?;
    let Some(property) = object.get(EXTENSIONS_KEY) else {
        return Ok(false);
    };
    let extensions = property
        .array_value()
        .ok_or_else(|| invalid_extensions(settings_path))?;
    let extension = extension_path_string(extension_path)?;
    Ok(extensions.elements().iter().any(|entry| {
        entry
            .to_serde_value()
            .is_some_and(|value| value.as_str() == Some(extension))
    }))
}

fn parse_root(content: &str, path: &Path) -> io::Result<CstRootNode> {
    CstRootNode::parse(content, &jsonc_parse_options()).map_err(|err| {
        io::Error::other(format!(
            "failed to parse OMP settings at {}: {err}",
            path.display()
        ))
    })
}

fn root_object(root: &CstRootNode, path: &Path) -> io::Result<jsonc_parser::cst::CstObject> {
    root.value()
        .and_then(|value| value.as_object())
        .ok_or_else(|| {
            io::Error::other(format!(
                "OMP settings at {} must be a JSON object",
                path.display()
            ))
        })
}

fn extension_path_string(path: &Path) -> io::Result<&str> {
    path.to_str().ok_or_else(|| {
        io::Error::other(format!(
            "OMP extension path at {} is not valid UTF-8",
            path.display()
        ))
    })
}

fn invalid_extensions(path: &Path) -> io::Error {
    io::Error::other(format!(
        "OMP settings extensions at {} must be an array",
        path.display()
    ))
}

fn jsonc_parse_options() -> ParseOptions {
    ParseOptions {
        allow_comments: true,
        allow_loose_object_property_names: false,
        allow_trailing_commas: true,
        allow_missing_commas: false,
        allow_single_quoted_strings: false,
        allow_hexadecimal_numbers: false,
        allow_unary_plus_numbers: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static SEQ: AtomicUsize = AtomicUsize::new(0);

    fn unique_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "herdr-omp-settings-{}-{}",
            std::process::id(),
            SEQ.fetch_add(1, Ordering::Relaxed)
        ));
        fs::remove_dir_all(&dir).ok();
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn install_and_uninstall_preserve_foreign_jsonc() {
        let dir = unique_dir();
        let settings = dir.join("settings.json");
        let extension = dir.join("extensions/herdr.ts");
        fs::write(
            &settings,
            "{\n  // retained\n  \"theme\": \"dark\",\n  \"extensions\": [\"/foreign.ts\",],\n}\n",
        )
        .unwrap();

        assert!(install(&settings, &extension).unwrap());
        let installed = fs::read_to_string(&settings).unwrap();
        assert!(installed.contains("// retained"));
        assert!(installed.contains("/foreign.ts"));
        assert!(installed.contains(extension.to_str().unwrap()));
        assert!(is_configured(&settings, &extension).unwrap());

        assert!(!install(&settings, &extension).unwrap());
        assert_eq!(fs::read_to_string(&settings).unwrap(), installed);

        assert!(uninstall(&settings, &extension).unwrap());
        let uninstalled = fs::read_to_string(&settings).unwrap();
        assert!(uninstalled.contains("// retained"));
        assert!(uninstalled.contains("/foreign.ts"));
        assert!(!uninstalled.contains(extension.to_str().unwrap()));
        assert!(!is_configured(&settings, &extension).unwrap());
        fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn uninstall_removes_empty_extensions_property() {
        let dir = unique_dir();
        let settings = dir.join("settings.json");
        let extension = dir.join("herdr.ts");

        assert!(install(&settings, &extension).unwrap());
        assert!(uninstall(&settings, &extension).unwrap());
        assert!(!fs::read_to_string(&settings)
            .unwrap()
            .contains(EXTENSIONS_KEY));
        fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn invalid_extensions_property_is_rejected() {
        let dir = unique_dir();
        let settings = dir.join("settings.json");
        let extension = dir.join("herdr.ts");
        fs::write(&settings, "{\"extensions\": true}\n").unwrap();

        let error = install(&settings, &extension).unwrap_err();
        assert!(error.to_string().contains("must be an array"));
        fs::remove_dir_all(dir).ok();
    }
}
