//! Reads the commit the drive runs on. Real processes with no sibling test: run_drive (steps/run_drive.rs) calls it
//! once per drive, before the steps.

use std::path::Path;
use std::process::{Command, Stdio};

use super::commit_line;

/// `git rev-parse HEAD` and `git status --porcelain` in `workspace`, with the drive's own environment, made into
/// commit_line's text: the tree has local changes when the status prints anything.
pub fn read_commit(workspace: &Path) -> String {
    let head = git(workspace, &["rev-parse", "HEAD"]).map(|out| out.trim().to_string());
    let changed = git(workspace, &["status", "--porcelain"]).map(|out| !out.trim().is_empty());
    commit_line(&head, &changed)
}

/// The command's stdout when it exits 0; else why, naming the command: it did not start, or its exit and stderr.
fn git(workspace: &Path, args: &[&str]) -> Result<String, String> {
    let command = format!("git {}", args.join(" "));
    let ran = Command::new("git")
        .args(args)
        .current_dir(workspace)
        .stdin(Stdio::null())
        .output()
        .map_err(|error| format!("{command} did not start: {error}"))?;
    if !ran.status.success() {
        let code = ran.status.code().map_or("no exit code (a signal ended it)".to_string(), |code| format!("exit {code}"));
        let stderr = String::from_utf8_lossy(&ran.stderr);
        return Err(format!("{command} failed, {code}: {}", stderr.trim()));
    }
    Ok(String::from_utf8_lossy(&ran.stdout).into_owned())
}
