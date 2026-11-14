//! move_refusal_report: files in the way and a branch elsewhere are exit 1 owned by the folder, after the lines
//! of what was already done; anything else is a failure with those lines first.

use std::path::PathBuf;

use folders::FolderError;
use gitio::Oid;

use super::move_refusal_report;
use crate::report::{ExitCode, Report};

const OLD: &str = "1111111111111111111111111111111111111111";
const NEW: &str = "2222222222222222222222222222222222222222";
const ELSE: &str = "3333333333333333333333333333333333333333";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn before() -> Vec<String> {
    vec!["rebased: refs/heads/req/core/fix-1 moved".to_string()]
}

#[test]
fn files_in_the_way_are_listed_and_the_next_join_finishes() {
    let error = FolderError::MoveRefused {
        path: PathBuf::from("/w/fix-1"),
        paths: vec![PathBuf::from("src/a.rs"), PathBuf::from("b.txt")],
    };
    assert_eq!(
        move_refusal_report(&error, before()),
        Report {
            verb: "join".to_string(),
            exit: ExitCode::Refused,
            outcome: "refused".to_string(),
            lines: vec![
                "rebased: refs/heads/req/core/fix-1 moved".to_string(),
                "the folder could not move; these files stand in the way:".to_string(),
                "  src/a.rs".to_string(),
                "  b.txt".to_string(),
                "the request branch already stands at the new tip; the next mortar join finishes the move".to_string(),
            ],
            owner: Some("this request's folder".to_string()),
            next: "move those files out of the way, then mortar join".to_string(),
            fields: Vec::new(),
        }
    );
}

#[test]
fn a_branch_elsewhere_is_put_back() {
    let error = FolderError::BranchElsewhere {
        branch: "refs/heads/req/core/fix-1".to_string(),
        at: Some(oid(ELSE)),
        old: oid(OLD),
        new: oid(NEW),
    };
    assert_eq!(
        move_refusal_report(&error, Vec::new()),
        Report {
            verb: "join".to_string(),
            exit: ExitCode::Refused,
            outcome: "refused".to_string(),
            lines: vec!["branch refs/heads/req/core/fix-1 is at 3333333333333333333333333333333333333333, neither \
                         the move's old tip 1111111111111111111111111111111111111111 nor its new tip \
                         2222222222222222222222222222222222222222"
                .to_string()],
            owner: Some("this request's folder".to_string()),
            next: "put the request branch back where mortar left it, then mortar join".to_string(),
            fields: Vec::new(),
        }
    );
}

#[test]
fn anything_else_is_a_failure_after_the_lines_before() {
    let error = FolderError::NotSaved { path: PathBuf::from("/w/fix-1") };
    assert_eq!(
        move_refusal_report(&error, before()),
        Report {
            verb: "join".to_string(),
            exit: ExitCode::Failure,
            outcome: "failed".to_string(),
            lines: vec![
                "rebased: refs/heads/req/core/fix-1 moved".to_string(),
                "/w/fix-1 holds work that is not saved; the folder was not moved; next step: mortar save".to_string(),
                "nothing half-done is left".to_string(),
            ],
            owner: None,
            next: "repeat the same command".to_string(),
            fields: Vec::new(),
        }
    );
}
