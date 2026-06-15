use std::ffi::{OsStr, OsString};
use std::process::{Command, Stdio};

use anyhow::{Context, Result};

pub const DEFAULT_REPO_URL: &str = "https://github.com/heyAyushh/stacc.git";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BootstrapOptions {
    pub repo_url: String,
    pub dry_run: bool,
}

pub fn default_bootstrap_options() -> BootstrapOptions {
    BootstrapOptions {
        repo_url: DEFAULT_REPO_URL.to_string(),
        dry_run: false,
    }
}

pub fn run_bootstrap(options: &BootstrapOptions) -> Result<()> {
    validate_repo_url(&options.repo_url)?;
    let args = cargo_install_args(options);
    println!("==> {}", shell_command(OsStr::new("cargo"), &args));
    if options.dry_run {
        return Ok(());
    }

    let status = Command::new("cargo")
        .args(&args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .context("failed to run cargo install")?;
    if !status.success() {
        anyhow::bail!("cargo install failed with status {:?}", status.code());
    }
    Ok(())
}

fn cargo_install_args(options: &BootstrapOptions) -> Vec<OsString> {
    vec![
        OsString::from("install"),
        OsString::from("--git"),
        OsString::from(&options.repo_url),
        OsString::from("--locked"),
        OsString::from("--force"),
    ]
}

fn validate_repo_url(repo_url: &str) -> Result<()> {
    let normalized = repo_url.strip_suffix(".git").unwrap_or(repo_url);
    if normalized.starts_with("https://github.com/") {
        return Ok(());
    }
    anyhow::bail!("bootstrap repo URL must be an https://github.com/... URL");
}

fn shell_command(program: &OsStr, args: &[OsString]) -> String {
    let mut parts = Vec::with_capacity(args.len() + 1);
    parts.push(shell_quote(program));
    parts.extend(args.iter().map(|arg| shell_quote(arg)));
    parts.join(" ")
}

fn shell_quote(value: &OsStr) -> String {
    let value = value.to_string_lossy();
    if value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '/' | '.' | '-' | '_' | ':' | '='))
    {
        return value.into_owned();
    }
    format!("'{}'", value.replace('\'', "'\\''"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_https_github_urls_only() {
        assert!(validate_repo_url("https://github.com/heyAyushh/stacc.git").is_ok());
        assert!(validate_repo_url("https://github.com/heyAyushh/stacc").is_ok());
        assert!(validate_repo_url("git@github.com:heyAyushh/stacc.git").is_err());
        assert!(validate_repo_url("https://example.com/heyAyushh/stacc").is_err());
    }

    #[test]
    fn bootstrap_command_uses_locked_force_install() {
        let args = cargo_install_args(&BootstrapOptions {
            repo_url: DEFAULT_REPO_URL.to_string(),
            dry_run: true,
        });
        assert_eq!(
            args,
            vec![
                OsString::from("install"),
                OsString::from("--git"),
                OsString::from(DEFAULT_REPO_URL),
                OsString::from("--locked"),
                OsString::from("--force"),
            ]
        );
    }
}
