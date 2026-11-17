//! The pid a Mortar lock file records.

/// The number on the file's `pid=<n>` line (oplog writes four `name=value` lines: pid, host, key, since); None when
/// there is no such line or its value is not a pid.
pub fn lock_pid(text: &str) -> Option<u32> {
    text.lines().find_map(|line| line.strip_prefix("pid=")).and_then(|pid| pid.trim().parse().ok())
}
