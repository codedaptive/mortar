//! joined_unknown_reason: one line as it is; two lines joined with "; ".

use super::joined_unknown_reason;
use crate::report::{failure_report, Report};

#[test]
fn one_line_is_the_reason() {
    assert_eq!(
        joined_unknown_reason(&failure_report("disk", "the branch refs/heads/req/core/fix-1 does not exist")),
        "the branch refs/heads/req/core/fix-1 does not exist"
    );
}

#[test]
fn two_lines_are_joined_with_a_semicolon() {
    let refusal = Report {
        lines: vec!["HEAD of the folder /w/fix-1 has no commit".to_string(), "no save commit was written".to_string()],
        ..failure_report("disk", "")
    };
    assert_eq!(
        joined_unknown_reason(&refusal),
        "HEAD of the folder /w/fix-1 has no commit; no save commit was written"
    );
}
