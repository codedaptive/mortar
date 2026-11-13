//! Shared libtest subprocess selection; tests/support.rs launches a real ignored worker (S-5).

use std::io;
use std::process::Command;

/// Builds a child of the current test executable, selecting exactly one ignored entry point.
/// Callers supply their own environment and inspect the outcome; no process-global state changes.
pub fn test_child(entry_point: &str) -> io::Result<Command> {
    let mut command = Command::new(std::env::current_exe()?);
    command.args(["--exact", entry_point, "--ignored", "--nocapture"]);
    Ok(command)
}
