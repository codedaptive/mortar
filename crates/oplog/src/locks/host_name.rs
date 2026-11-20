//! This host's name, read by running `hostname`; parse_host_name judges its output.
//! No unit test: it runs a real program. Covered by the crate's integration tests (tests/oplog/locks.rs):
//! acquire on a fresh dir records this host, and stale takeover of a dead pid on this host.

use std::process::Command;

use super::{parse_host_name, LockError};

/// The name `hostname` prints, or HostUnknown when it cannot be run, fails, or prints no clean word.
pub fn host_name() -> Result<String, LockError> {
    let out = Command::new("hostname")
        .output()
        .map_err(|e| LockError::HostUnknown(format!("cannot run hostname: {e}")))?;
    if !out.status.success() {
        return Err(LockError::HostUnknown(format!(
            "hostname exited with {}: {}",
            out.status,
            String::from_utf8_lossy(&out.stderr).trim()
        )));
    }
    parse_host_name(&out.stdout)
}
