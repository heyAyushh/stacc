use std::ffi::{OsStr, OsString};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::{Context, Result};
use serde_json::Value;

const BINARY_NAME: &str = "stacc";
const BUNDLE_DIR_NAME: &str = "bundle";
const CARGO_ROOT_DIR_NAME: &str = "cargo-root";
const CARGO_MANIFEST_FILE: &str = "Cargo.toml";
const CHECK_ROOT: &str = "target/stacc-check";
const INSTALL_SCRIPT: &str = "install.sh";
const SHELLCHECK_COMMAND: &str = "shellcheck";
const STACC_BUNDLE_ROOT_ENV: &str = "STACC_BUNDLE_ROOT";

const JSON_FILES: [&str; 3] = [
    "configs/mcps/mcp.json",
    "configs/stacc-panel.json",
    "configs/metadata/skills.lock.json",
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckOptions {
    pub root: PathBuf,
    pub require_shellcheck: bool,
}

pub fn run_checks(options: &CheckOptions) -> Result<()> {
    validate_checkout_root(&options.root)?;
    run_rust_checks(&options.root)?;
    run_installer_checks(&options.root, options.require_shellcheck)?;
    validate_json_files(&options.root)?;
    run_installed_binary_checks(&options.root)?;
    println!("\nAll checks passed.");
    Ok(())
}

fn validate_checkout_root(root: &Path) -> Result<()> {
    if !root.join(CARGO_MANIFEST_FILE).is_file() {
        anyhow::bail!(
            "check requires a local stacc checkout with {CARGO_MANIFEST_FILE}; run from the repo or pass --root <path>"
        );
    }
    if !root.join(INSTALL_SCRIPT).is_file() {
        anyhow::bail!("check requires {INSTALL_SCRIPT} under {}", root.display());
    }
    Ok(())
}

fn run_rust_checks(root: &Path) -> Result<()> {
    run_command(
        root,
        "cargo",
        string_args(&["fmt", "--all", "--", "--check"]),
    )?;
    run_command(root, "cargo", string_args(&["test"]))?;
    run_command(
        root,
        "cargo",
        string_args(&[
            "clippy",
            "--workspace",
            "--all-targets",
            "--all-features",
            "--",
            "-D",
            "warnings",
        ]),
    )
}

fn run_installer_checks(root: &Path, require_shellcheck: bool) -> Result<()> {
    run_command(root, "bash", string_args(&["-n", INSTALL_SCRIPT]))?;

    if command_exists(SHELLCHECK_COMMAND) {
        return run_command(
            root,
            SHELLCHECK_COMMAND,
            string_args(&["-x", INSTALL_SCRIPT]),
        );
    }

    if require_shellcheck {
        anyhow::bail!("{SHELLCHECK_COMMAND} not found; install it or omit --require-shellcheck");
    }

    println!("\n==> skipping {SHELLCHECK_COMMAND}; command not found");
    Ok(())
}

fn validate_json_files(root: &Path) -> Result<()> {
    for relative_path in JSON_FILES {
        let path = root.join(relative_path);
        println!("\n==> validate json {}", path.display());
        let contents =
            fs::read(&path).with_context(|| format!("failed to read {}", path.display()))?;
        serde_json::from_slice::<Value>(&contents)
            .with_context(|| format!("invalid JSON in {}", path.display()))?;
    }
    Ok(())
}

fn run_installed_binary_checks(root: &Path) -> Result<()> {
    let check_root = root.join(CHECK_ROOT);
    let cargo_root = check_root.join(CARGO_ROOT_DIR_NAME);
    let bundle_root = check_root.join(BUNDLE_DIR_NAME);
    fs::create_dir_all(&check_root)
        .with_context(|| format!("failed to create {}", check_root.display()))?;

    run_command(
        root,
        "cargo",
        vec![
            OsString::from("install"),
            OsString::from("--path"),
            OsString::from("."),
            OsString::from("--root"),
            cargo_root.as_os_str().to_os_string(),
            OsString::from("--locked"),
            OsString::from("--force"),
        ],
    )?;

    let binary = cargo_root.join("bin").join(BINARY_NAME);
    println!("\n==> installed binary smoke checks");
    run_command_with_env(
        &check_root,
        binary.as_os_str(),
        string_args(&["status", "--json"]),
        &[(STACC_BUNDLE_ROOT_ENV, bundle_root.as_os_str())],
        OutputMode::Quiet,
    )?;
    run_command_with_env(
        &check_root,
        binary.as_os_str(),
        string_args(&[
            "install",
            "--editor",
            "codex",
            "--scope",
            "global",
            "--category",
            "rules",
            "--category",
            "skills",
            "--category",
            "mcps",
            "--mcp-server",
            "github",
            "--dry-run",
            "--print-plan",
        ]),
        &[(STACC_BUNDLE_ROOT_ENV, bundle_root.as_os_str())],
        OutputMode::Quiet,
    )
}

fn run_command(current_dir: &Path, program: &str, args: Vec<OsString>) -> Result<()> {
    run_command_with_env(
        current_dir,
        OsStr::new(program),
        args,
        &[],
        OutputMode::Inherit,
    )
}

fn run_command_with_env(
    current_dir: &Path,
    program: &OsStr,
    args: Vec<OsString>,
    envs: &[(&str, &OsStr)],
    output_mode: OutputMode,
) -> Result<()> {
    print_command(program, &args);
    let mut command = Command::new(program);
    command.args(&args).current_dir(current_dir);
    for (key, value) in envs {
        command.env(key, value);
    }
    match output_mode {
        OutputMode::Inherit => {
            command
                .stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit());
        }
        OutputMode::Quiet => {
            command
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::inherit());
        }
    }

    let status = command
        .status()
        .with_context(|| format!("failed to run {}", shell_quote(program)))?;
    if !status.success() {
        anyhow::bail!(
            "{} failed with status {:?}",
            shell_quote(program),
            status.code()
        );
    }
    Ok(())
}

fn print_command(program: &OsStr, args: &[OsString]) {
    let mut parts = Vec::with_capacity(args.len() + 1);
    parts.push(shell_quote(program));
    parts.extend(args.iter().map(|arg| shell_quote(arg)));
    println!("\n==> {}", parts.join(" "));
}

fn shell_quote(value: &OsStr) -> String {
    let value = value.to_string_lossy();
    if value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '/' | '.' | '-' | '_' | ',' | '='))
    {
        return value.into_owned();
    }
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn string_args(values: &[&str]) -> Vec<OsString> {
    values.iter().map(OsString::from).collect()
}

fn command_exists(command: &str) -> bool {
    std::env::var_os("PATH")
        .map(|paths| std::env::split_paths(&paths).any(|path| path.join(command).is_file()))
        .unwrap_or(false)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum OutputMode {
    Inherit,
    Quiet,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shell_quote_leaves_simple_args_unquoted() {
        assert_eq!(shell_quote(OsStr::new("--category")), "--category");
        assert_eq!(
            shell_quote(OsStr::new("configs/mcps/mcp.json")),
            "configs/mcps/mcp.json"
        );
    }

    #[test]
    fn shell_quote_wraps_args_with_spaces() {
        assert_eq!(
            shell_quote(OsStr::new("path with spaces")),
            "'path with spaces'"
        );
    }
}
