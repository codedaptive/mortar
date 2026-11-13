//! The OS answer to "is this process alive?": runs `kill -0 <pid>`; kill_status_alive reads the result.
//! No unit test: it runs a real program against a real process (S-5). Covered by Nagatha's integration
//! group: a live pid (this process) is alive, a dead pid is not, so stale takeover happens only on a
//! dead pid and refusal while held on a live one.

use std::process::Command;

use super::{kill_status_alive, PidProbe};

/// The pid probe that asks the OS.
pub struct KillPidProbe;

impl PidProbe for KillPidProbe {
    /// When `kill` cannot be run, answers alive: a lock is never taken over on doubt.
    fn pid_alive(&self, pid: u32) -> bool {
        match Command::new("kill").arg("-0").arg(pid.to_string()).output() {
            Ok(out) => kill_status_alive(out.status.success(), &String::from_utf8_lossy(&out.stderr)),
            Err(_) => true,
        }
    }
}
