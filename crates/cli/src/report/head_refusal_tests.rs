use super::head_refusal;
use crate::report::{ExitCode, Report};

#[test]
fn head_refusal_names_the_head_as_owner_and_keeps_the_next_given() {
    assert_eq!(
        head_refusal(
            "save",
            vec!["HEAD of the folder /w/a names no branch".to_string(), "no save commit was written".to_string()],
            "check out the request's branch in this folder, then mr save again",
        ),
        Report {
            verb: "save".to_string(),
            exit: ExitCode::Refused,
            outcome: "refused".to_string(),
            lines: vec![
                "HEAD of the folder /w/a names no branch".to_string(),
                "no save commit was written".to_string(),
            ],
            owner: Some("the folder's HEAD".to_string()),
            next: "check out the request's branch in this folder, then mr save again".to_string(),
            fields: vec![],
        }
    );
}

#[test]
fn head_refusal_keeps_the_verb_and_lines_given() {
    assert_eq!(
        head_refusal("join", vec!["HEAD of the folder /w/b has no commit".to_string()], "mend HEAD"),
        Report {
            verb: "join".to_string(),
            exit: ExitCode::Refused,
            outcome: "refused".to_string(),
            lines: vec!["HEAD of the folder /w/b has no commit".to_string()],
            owner: Some("the folder's HEAD".to_string()),
            next: "mend HEAD".to_string(),
            fields: vec![],
        }
    );
}
