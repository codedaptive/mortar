use super::repeat_head_refusal;
use crate::report::{ExitCode, Report};

#[test]
fn repeat_head_refusal_is_the_head_refusal_whose_next_is_the_checkout_and_a_repeat() {
    assert_eq!(
        repeat_head_refusal(
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
