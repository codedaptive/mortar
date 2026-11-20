//! landing_refusal: exit 1 "refused" with the lines, owner and next given, and no fields.

use super::landing_refusal;
use crate::report::{ExitCode, Report};

#[test]
fn a_refusal_carries_its_lines_owner_and_next_with_no_fields() {
    let lines = vec!["one".to_string(), "two".to_string()];
    assert_eq!(
        landing_refusal("publish", lines, "the op log", "repeat the same command".to_string()),
        Report {
            verb: "publish".to_string(),
            exit: ExitCode::Refused,
            outcome: "refused".to_string(),
            lines: vec!["one".to_string(), "two".to_string()],
            owner: Some("the op log".to_string()),
            next: "repeat the same command".to_string(),
            fields: Vec::new(),
        }
    );
}
