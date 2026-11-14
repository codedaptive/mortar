//! The fixed exit codes of the binary (S-71).
//! A bare type (S-7); render_json_tests exercises its numbers, crates/cli/tests/usage_report_tests.rs the usage code.

/// `code as i32` is the process exit code. Done covers a repeat of a finished command; Busy means repeat
/// later; Failure means git or the disk failed, nothing half-done is left and a repeat is safe.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitCode {
    Done = 0,
    Refused = 1,
    Conflict = 2,
    Busy = 3,
    Usage = 4,
    Failure = 5,
}
