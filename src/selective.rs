use std::fs;
use std::io::{self, IsTerminal, Write};
use std::path::Path;

use anyhow::{Context, Result};

use crate::catalog::ConflictMode;

pub fn prepare_existing_target_for_write(
    destination: &Path,
    conflict_mode: ConflictMode,
    backup: &Path,
) -> Result<bool> {
    if !destination.exists() || conflict_mode != ConflictMode::Selective {
        return Ok(true);
    }

    match prompt_write_conflict(destination)? {
        WriteConflictAction::Backup => {
            backup_existing_path(destination, backup)?;
            Ok(true)
        }
        WriteConflictAction::Overwrite => {
            if destination.is_dir() {
                fs::remove_dir_all(destination)
            } else {
                Ok(())
            }
            .with_context(|| format!("failed to remove {}", destination.display()))?;
            Ok(true)
        }
        WriteConflictAction::Skip => Ok(false),
    }
}

pub fn prepare_existing_target_for_append(
    destination: &Path,
    conflict_mode: ConflictMode,
) -> Result<bool> {
    if !destination.exists() || conflict_mode != ConflictMode::Selective {
        return Ok(true);
    }
    prompt_append_conflict(destination)
}

fn backup_existing_path(path: &Path, backup: &Path) -> Result<()> {
    ensure_parent(backup)?;
    fs::rename(path, backup).with_context(|| {
        format!(
            "failed to backup {} -> {}",
            path.display(),
            backup.display()
        )
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum WriteConflictAction {
    Backup,
    Overwrite,
    Skip,
}

fn prompt_write_conflict(destination: &Path) -> Result<WriteConflictAction> {
    ensure_interactive_selective()?;
    loop {
        print!(
            "conflict: {} [b]ackup/[o]verwrite/[s]kip: ",
            destination.display()
        );
        io::stdout().flush().context("failed to flush prompt")?;
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .context("failed to read conflict choice")?;
        if let Some(action) = parse_write_conflict_action(&input) {
            return Ok(action);
        }
    }
}

fn prompt_append_conflict(destination: &Path) -> Result<bool> {
    ensure_interactive_selective()?;
    loop {
        print!(
            "append to existing {}? [a]ppend/[s]kip: ",
            destination.display()
        );
        io::stdout().flush().context("failed to flush prompt")?;
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .context("failed to read append choice")?;
        match input.trim().to_ascii_lowercase().as_str() {
            "a" | "append" | "y" | "yes" => return Ok(true),
            "s" | "skip" | "n" | "no" => return Ok(false),
            _ => {}
        }
    }
}

fn ensure_interactive_selective() -> Result<()> {
    if io::stdin().is_terminal() && io::stdout().is_terminal() {
        return Ok(());
    }
    anyhow::bail!(
        "selective conflict mode requires an interactive terminal; use --dry-run first or choose backup, overwrite, or skip"
    );
}

fn parse_write_conflict_action(input: &str) -> Option<WriteConflictAction> {
    match input.trim().to_ascii_lowercase().as_str() {
        "b" | "backup" => Some(WriteConflictAction::Backup),
        "o" | "overwrite" => Some(WriteConflictAction::Overwrite),
        "s" | "skip" => Some(WriteConflictAction::Skip),
        _ => None,
    }
}

fn ensure_parent(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_selective_write_choices() {
        assert_eq!(
            parse_write_conflict_action("backup"),
            Some(WriteConflictAction::Backup)
        );
        assert_eq!(
            parse_write_conflict_action("o"),
            Some(WriteConflictAction::Overwrite)
        );
        assert_eq!(
            parse_write_conflict_action("skip"),
            Some(WriteConflictAction::Skip)
        );
        assert_eq!(parse_write_conflict_action("later"), None);
    }
}
