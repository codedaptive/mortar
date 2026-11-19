//! after_publish_failure: exit 5 with the lines given, a repeat as next, and no line added.

use super::after_publish_failure;
use crate::report::{ExitCode, Report};

#[test]
fn the_lines_are_kept_exactly_with_a_repeat_as_next() {
    let lines = vec!["git failed while reading or replaying a zone".to_string(), "the second line".to_string()];
    assert_eq!(
        after_publish_failure(lines),
        Report {
            verb: "after-publish".to_string(),
            exit: ExitCode::Failure,
            outcome: "failed".to_string(),
            lines: vec!["git failed while reading or replaying a zone".to_string(), "the second line".to_string()],
            owner: None,
            next: "repeat the same command".to_string(),
            fields: Vec::new(),
        }
    );
}

#[test]
fn one_line_gets_no_line_added() {
    let report = after_publish_failure(vec!["the op log could not be read".to_string()]);
    assert_eq!(report.lines, vec!["the op log could not be read".to_string()]);
    assert!(!report.lines.iter().any(|line| line == "nothing half-done is left"));
}
