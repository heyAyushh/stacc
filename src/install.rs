use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::catalog::{Category, ConflictMode, Editor, Scope};

const INSTALL_SCRIPT: &str = "install.sh";

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct InstallRequest {
    pub root: PathBuf,
    pub editors: Vec<Editor>,
    pub scope: Scope,
    pub categories: Vec<Category>,
    pub stacks: Vec<String>,
    pub mcp_servers: Vec<String>,
    pub conflict_mode: ConflictMode,
    pub yes: bool,
    pub dry_run: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct InstallCommandPlan {
    pub editor: Editor,
    pub args: Vec<String>,
    pub categories: Vec<Category>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct InstallRunResult {
    pub editor: Editor,
    pub status_code: Option<i32>,
    pub command: Vec<String>,
}

impl InstallRequest {
    pub fn validate(&self) -> Result<()> {
        if self.editors.is_empty() {
            anyhow::bail!("install needs at least one editor");
        }
        if self.categories.is_empty() {
            anyhow::bail!("install needs at least one category");
        }
        if !self.dry_run && !self.yes {
            anyhow::bail!("review dry-run first, then pass --yes for writes");
        }
        if !self.root.join(INSTALL_SCRIPT).is_file() {
            anyhow::bail!("{} missing under {}", INSTALL_SCRIPT, self.root.display());
        }
        Ok(())
    }
}

pub fn build_install_plan(request: &InstallRequest) -> Result<Vec<InstallCommandPlan>> {
    request.validate()?;

    let mut plans = Vec::new();
    for editor in &request.editors {
        let categories = filtered_categories(request, *editor);
        if categories.is_empty() {
            continue;
        }

        let mut args = vec![
            request.root.join(INSTALL_SCRIPT).display().to_string(),
            "--root".to_string(),
            request.root.display().to_string(),
            editor.install_flag().to_string(),
            request.scope.install_flag().to_string(),
            "--categories".to_string(),
            join_categories(&categories),
            "--conflict".to_string(),
            request.conflict_mode.install_value().to_string(),
        ];

        if categories.contains(&Category::Stack) && !request.stacks.is_empty() {
            args.push("--stacks".to_string());
            args.push(request.stacks.join(","));
        }

        if categories.contains(&Category::Mcps) && !request.mcp_servers.is_empty() {
            args.push("--mcp-servers".to_string());
            args.push(request.mcp_servers.join(","));
        }

        if request.yes {
            args.push("--yes".to_string());
        }

        if request.dry_run {
            args.push("--dry-run".to_string());
        }

        plans.push(InstallCommandPlan {
            editor: *editor,
            args,
            categories,
        });
    }

    if plans.is_empty() {
        anyhow::bail!("selected categories are unsupported for selected editors/scope");
    }

    Ok(plans)
}

pub fn execute_install_request(request: &InstallRequest) -> Result<Vec<InstallRunResult>> {
    let plans = build_install_plan(request)?;
    let mut results = Vec::with_capacity(plans.len());

    for plan in plans {
        let (program, arguments) = plan
            .args
            .split_first()
            .context("install command plan had no program")?;
        let status = Command::new(program)
            .args(arguments)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .with_context(|| format!("failed to run {}", program))?;

        let result = InstallRunResult {
            editor: plan.editor,
            status_code: status.code(),
            command: plan.args,
        };
        if !status.success() {
            anyhow::bail!(
                "install failed for {} with status {:?}",
                plan.editor,
                result.status_code
            );
        }
        results.push(result);
    }

    Ok(results)
}

pub fn print_plan(plans: &[InstallCommandPlan]) {
    for plan in plans {
        println!("{}:", plan.editor);
        println!("  {}", shell_display(&plan.args));
    }
}

fn filtered_categories(request: &InstallRequest, editor: Editor) -> Vec<Category> {
    request
        .categories
        .iter()
        .copied()
        .filter(|category| category.is_supported_for(editor, request.scope, &request.root))
        .collect()
}

fn join_categories(categories: &[Category]) -> String {
    categories
        .iter()
        .map(|category| category.install_value())
        .collect::<Vec<_>>()
        .join(",")
}

fn shell_display(args: &[String]) -> String {
    args.iter()
        .map(|arg| shell_quote(arg))
        .collect::<Vec<_>>()
        .join(" ")
}

fn shell_quote(value: &str) -> String {
    if value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '/' | '.' | '-' | '_' | ',' | '='))
    {
        return value.to_string();
    }
    format!("'{}'", value.replace('\'', "'\\''"))
}

#[allow(dead_code)]
pub fn install_script_path(root: &Path) -> PathBuf {
    root.join(INSTALL_SCRIPT)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filters_codex_project_mcp_category() {
        let request = InstallRequest {
            root: PathBuf::from("."),
            editors: vec![Editor::Codex],
            scope: Scope::Project,
            categories: vec![Category::Skills, Category::Mcps],
            stacks: Vec::new(),
            mcp_servers: Vec::new(),
            conflict_mode: ConflictMode::Backup,
            yes: true,
            dry_run: true,
        };

        let categories = filtered_categories(&request, Editor::Codex);
        assert_eq!(categories, vec![Category::Skills]);
    }

    #[test]
    fn quotes_paths_with_spaces() {
        let args = vec!["tool".to_string(), "path with spaces".to_string()];
        assert_eq!(shell_display(&args), "tool 'path with spaces'");
    }
}
