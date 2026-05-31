use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use include_dir::{include_dir, Dir, DirEntry};

use crate::catalog::repo_root_from_option;

const CACHE_DIR_NAME: &str = "stacc";
const BUNDLES_DIR_NAME: &str = "bundles";
const BUNDLE_ROOT_ENV: &str = "STACC_BUNDLE_ROOT";
const CONFIGS_DIR_NAME: &str = "configs";
const INSTALL_SCRIPT_NAME: &str = "install.sh";
const README_FILE_NAME: &str = "README.md";
const MACOS_METADATA_FILE_NAME: &str = ".DS_Store";

static CONFIGS_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/configs");
static INSTALL_SCRIPT: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/install.sh"));
static README: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"));

pub fn resolve_runtime_root(explicit_root: Option<PathBuf>) -> Result<PathBuf> {
    if explicit_root.is_some() {
        return repo_root_from_option(explicit_root);
    }

    let current_dir = std::env::current_dir().context("failed to read current directory")?;
    if is_stacc_root(&current_dir) {
        return Ok(current_dir);
    }

    let root = bundled_root()?;
    materialize_bundle(&root)?;
    Ok(root)
}

pub fn materialize_bundle(root: &Path) -> Result<()> {
    fs::create_dir_all(root).with_context(|| format!("failed to create {}", root.display()))?;
    write_if_changed(&root.join(INSTALL_SCRIPT_NAME), INSTALL_SCRIPT.as_bytes())?;
    make_executable(&root.join(INSTALL_SCRIPT_NAME))?;
    write_if_changed(&root.join(README_FILE_NAME), README.as_bytes())?;
    materialize_dir(&CONFIGS_DIR, &root.join(CONFIGS_DIR_NAME))?;
    Ok(())
}

fn is_stacc_root(path: &Path) -> bool {
    path.join(CONFIGS_DIR_NAME).is_dir() && path.join(INSTALL_SCRIPT_NAME).is_file()
}

fn bundled_root() -> Result<PathBuf> {
    if let Some(path) = std::env::var_os(BUNDLE_ROOT_ENV) {
        return Ok(PathBuf::from(path));
    }

    let cache_root = std::env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".cache")))
        .unwrap_or_else(std::env::temp_dir);

    Ok(cache_root
        .join(CACHE_DIR_NAME)
        .join(BUNDLES_DIR_NAME)
        .join(bundle_id()))
}

fn bundle_id() -> String {
    let git_hash = option_env!("STACC_BUILD_GIT_HASH").unwrap_or("unknown");
    format!("{}-{git_hash}", env!("CARGO_PKG_VERSION"))
}

fn materialize_dir(source: &Dir<'_>, destination: &Path) -> Result<()> {
    fs::create_dir_all(destination)
        .with_context(|| format!("failed to create {}", destination.display()))?;

    for entry in source.entries() {
        match entry {
            DirEntry::Dir(directory) => {
                let name = directory
                    .path()
                    .file_name()
                    .context("embedded directory had no file name")?;
                if name == std::ffi::OsStr::new(MACOS_METADATA_FILE_NAME) {
                    continue;
                }
                materialize_dir(directory, &destination.join(name))?;
            }
            DirEntry::File(file) => {
                let name = file
                    .path()
                    .file_name()
                    .context("embedded file had no file name")?;
                if name == std::ffi::OsStr::new(MACOS_METADATA_FILE_NAME) {
                    continue;
                }
                write_if_changed(&destination.join(name), file.contents())?;
            }
        }
    }

    Ok(())
}

fn write_if_changed(path: &Path, contents: &[u8]) -> Result<()> {
    if path.is_file() && fs::read(path).is_ok_and(|existing| existing == contents) {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    fs::write(path, contents).with_context(|| format!("failed to write {}", path.display()))
}

#[cfg(unix)]
fn make_executable(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let mut permissions = fs::metadata(path)
        .with_context(|| format!("failed to read metadata for {}", path.display()))?
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions)
        .with_context(|| format!("failed to chmod {}", path.display()))
}

#[cfg(not(unix))]
fn make_executable(_path: &Path) -> Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundle_id_contains_package_version() {
        assert!(bundle_id().starts_with(env!("CARGO_PKG_VERSION")));
    }
}
