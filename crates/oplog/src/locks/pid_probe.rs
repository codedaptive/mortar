//! The question "is this process alive?", as a trait so the answer can come from the OS or a fake.
//! A bare trait; kill_status_alive_tests and decide_lock_staleness_tests cover the decisions it feeds.

/// Answers whether a process on this host is alive.
pub trait PidProbe {
    /// True when the process exists. When unsure, answer true: a lock is never taken over on doubt.
    fn pid_alive(&self, pid: u32) -> bool;
}
