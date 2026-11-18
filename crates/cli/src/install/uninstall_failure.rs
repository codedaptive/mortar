//! uninstall_failure: The report of an uninstall the disk failed (spec 07, "mr uninstall").
//! Pure: builds the report from its arguments; uninstall_failure_tests pins its text.

use std::path::PathBuf;

use crate::report::{ExitCode, Report};

/// Exit Failure, outcome "failed": `error`, then each of `left`, the folders still on disk that install's record no
/// longer lists. With none, the record still lists what is left, so the next step is the same command again; else a
/// repeat cannot see them, so the next step is to remove the empty ones by hand.
pub fn uninstall_failure(error: &str, left: &[PathBuf]) -> Report {
    let (lines, next): (Vec<String>, _) = if left.is_empty() {
        (
            vec!["install's record is still in place, so a repeat finds what is left".to_string()],
            "repeat the same command",
        )
    } else {
        (
            left.iter().map(|path| format!("left, and no longer in install's record: {}", path.display())).collect(),
            "remove each folder listed that is empty; a repeat no longer sees them",
        )
    };
    Report {
        verb: "uninstall".to_string(),
        exit: ExitCode::Failure,
        outcome: "failed".to_string(),
        lines: [error.to_string()].into_iter().chain(lines).collect(),
        owner: None,
        next: next.to_string(),
        fields: Vec::new(),
    }
}
