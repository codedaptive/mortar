use super::head_refusal;
use crate::report::{ExitCode, Report};

#[test]
fn head_refusal_names_the_head_as_owner_and_the_checkout_as_next() {
    assert_eq!(
        head_refusal(
            "snapshot",
            vec!["HEAD of the folder /w/a names no branch".to_string(), "nothing was snapshotted".to_string()],
        ),
        Report {
            verb: "snapshot".to_string(),
            exit: ExitCode::Refused,
            outcome: "refused".to_string(),
            lines: vec!["HEAD of the folder /w/a names no branch".to_string(), "nothing was snapshotted".to_string()],
            owner: Some("the folder's HEAD".to_string()),
            next: "check out the request's branch in the folder, then repeat the same command".to_string(),
            fields: vec![],
        }
    );
}

#[test]
fn head_refusal_keeps_the_verb_and_lines_given() {
    assert_eq!(
        head_refusal("join", vec!["HEAD of the folder /w/b has no commit".to_string()]),
        Report {
            verb: "join".to_string(),
            exit: ExitCode::Refused,
            outcome: "refused".to_string(),
            lines: vec!["HEAD of the folder /w/b has no commit".to_string()],
            owner: Some("the folder's HEAD".to_string()),
            next: "check out the request's branch in the folder, then repeat the same command".to_string(),
            fields: vec![],
        }
    );
}
