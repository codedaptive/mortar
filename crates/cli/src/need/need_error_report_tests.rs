//! need_error_report's Plan and NotAFolder arms: each is exit 1 "refused", owned by this request, with its own line
//! and next step. The Lock and fallback arms are pure and can also be covered by ZUTs (need_error_report.rs names them).

use std::path::PathBuf;

use folders::plan::PlanError;
use folders::FolderError;

use super::need_error_report;
use crate::report::{ExitCode, Report};

fn refused(line: &str, next: &str) -> Report {
    Report {
        verb: "need".to_string(),
        exit: ExitCode::Refused,
        outcome: "refused".to_string(),
        lines: vec![line.to_string()],
        owner: Some("this request".to_string()),
        next: next.to_string(),
        fields: Vec::new(),
    }
}

#[test]
fn a_plan_refusal_is_refused_with_the_plan_reason() {
    let error = FolderError::Plan(PlanError { reason: "sparse path \"../x\" has a bad component".to_string() });
    assert_eq!(
        need_error_report(&error),
        refused("sparse path \"../x\" has a bad component", "mr need <a path inside the repository>"),
    );
}

#[test]
fn a_folder_without_a_record_is_refused_naming_the_folder_and_why() {
    let error = FolderError::NotAFolder {
        path: PathBuf::from("/work/red-one"),
        reason: "a worktree with no Mortar folder record".to_string(),
    };
    assert_eq!(
        need_error_report(&error),
        refused(
            "/work/red-one is not a Mortar folder: a worktree with no Mortar folder record",
            "make the request folder again, then mr need",
        ),
    );
}
