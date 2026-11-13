//! Reads the result of `kill -0 <pid>` as "is the process alive?", leaning to alive when unsure.

/// True unless `kill -0` failed saying there is no such process. Any other failure, such as
/// "Operation not permitted" (the process exists under another user), counts as alive.
pub fn kill_status_alive(exit_success: bool, stderr: &str) -> bool {
    exit_success || !stderr.to_ascii_lowercase().contains("no such process")
}
