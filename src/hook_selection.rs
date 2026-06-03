use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::catalog::Editor;

const CONFIGS_DIR: &str = "configs";
const CURSOR_PLUGINS_DIR: &str = "cursor-plugins";
const HOOKS_DIR: &str = "hooks";

pub fn selected_hook_packages(
    root: &Path,
    editor: Editor,
    selected_hooks: &[String],
) -> Result<Vec<(String, PathBuf)>> {
    let available = available_hook_packages(root, editor)?;
    let selected = normalized_selected_names(selected_hooks);
    if selected.is_empty() {
        return Ok(available);
    }

    let available_names = available
        .iter()
        .map(|(name, _)| name.clone())
        .collect::<Vec<_>>();
    let unknown = unknown_names(&selected, &available_names);
    if !unknown.is_empty() {
        anyhow::bail!(
            "unknown hook package(s): {}. Available: {}",
            unknown.join(","),
            available_names.join(",")
        );
    }

    Ok(available
        .into_iter()
        .filter(|(name, _)| selected.iter().any(|selected_name| selected_name == name))
        .collect())
}

fn available_hook_packages(root: &Path, editor: Editor) -> Result<Vec<(String, PathBuf)>> {
    let mut packages = Vec::new();
    append_hook_packages(&mut packages, &root.join(CONFIGS_DIR).join(HOOKS_DIR))?;
    if editor == Editor::Cursor {
        append_hook_packages(
            &mut packages,
            &root
                .join(CONFIGS_DIR)
                .join(CURSOR_PLUGINS_DIR)
                .join(HOOKS_DIR),
        )?;
    }
    packages.sort_by(|left, right| left.0.cmp(&right.0).then_with(|| left.1.cmp(&right.1)));
    Ok(packages)
}

fn append_hook_packages(packages: &mut Vec<(String, PathBuf)>, parent: &Path) -> Result<()> {
    if !parent.is_dir() {
        return Ok(());
    }

    for entry in
        fs::read_dir(parent).with_context(|| format!("failed to read {}", parent.display()))?
    {
        let entry =
            entry.with_context(|| format!("failed to read entry under {}", parent.display()))?;
        if entry
            .file_type()
            .with_context(|| format!("failed to read file type for {}", entry.path().display()))?
            .is_dir()
        {
            packages.push((
                entry.file_name().to_string_lossy().into_owned(),
                entry.path(),
            ));
        }
    }

    Ok(())
}

fn normalized_selected_names(values: &[String]) -> Vec<String> {
    let mut names = Vec::new();
    for value in values {
        let name = value.trim();
        if name.is_empty() || names.iter().any(|existing| existing == name) {
            continue;
        }
        names.push(name.to_string());
    }
    names
}

fn unknown_names(selected: &[String], available: &[String]) -> Vec<String> {
    selected
        .iter()
        .filter(|name| {
            !available
                .iter()
                .any(|available_name| available_name == *name)
        })
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    struct TestDir {
        path: PathBuf,
    }

    impl TestDir {
        fn new(name: &str) -> Self {
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("test clock should be valid")
                .as_nanos();
            let path = std::env::temp_dir().join(format!("stacc-hooks-{name}-{unique}"));
            fs::create_dir_all(&path).expect("test dir should be created");
            Self { path }
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    fn write_test_file(path: &Path, contents: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("test parent should be created");
        }
        fs::write(path, contents).expect("test file should be written");
    }

    #[test]
    fn selected_hook_packages_reject_unknown_values() {
        let root = TestDir::new("unknown");
        write_test_file(
            &root
                .path
                .join(CONFIGS_DIR)
                .join(CURSOR_PLUGINS_DIR)
                .join(HOOKS_DIR)
                .join("continual-learning")
                .join("hooks.json"),
            "{}",
        );

        let error = selected_hook_packages(
            &root.path,
            Editor::Cursor,
            &["continual-learning".to_string(), "missing".to_string()],
        )
        .expect_err("unknown hook should fail");

        let message = error.to_string();
        assert!(message.contains("unknown hook package(s): missing"));
        assert!(message.contains("Available: continual-learning"));
    }

    #[test]
    fn selected_hook_packages_uses_cursor_plugin_hooks_for_cursor() {
        let root = TestDir::new("cursor");
        let hook_file = root
            .path
            .join(CONFIGS_DIR)
            .join(CURSOR_PLUGINS_DIR)
            .join(HOOKS_DIR)
            .join("continual-learning")
            .join("hooks.json");
        write_test_file(&hook_file, "{}");

        let selected = selected_hook_packages(
            &root.path,
            Editor::Cursor,
            &["continual-learning".to_string()],
        )
        .expect("hook should be selected");

        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].0, "continual-learning");
        assert_eq!(
            selected[0].1,
            hook_file
                .parent()
                .expect("hook dir should exist")
                .to_path_buf()
        );
    }
}
