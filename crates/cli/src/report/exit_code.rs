//! The fixed exit codes of the binary.
//! A bare type; render_json_tests exercises its numbers, args/usage_report_tests the usage code.

/// `code as i32` is the process exit code. Done covers a repeat of a finished command; Busy means repeat later (a
/// held lock, a ref that moved while the command ran), except a landing in flight, which is finished first (mr publish
/// or mr back-out) and then mr land; Failure means a step could not be done; the report gives the next step, usually a
/// repeat, and names the work left behind that it knows of. A failed install names what it placed: what it recorded,
/// for uninstall to remove, or, when the record could not be written, each path for a person to remove.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitCode {
    Done = 0,
    Refused = 1,
    Conflict = 2,
    Busy = 3,
    Usage = 4,
    Failure = 5,
}
