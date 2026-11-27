//! resolve_move_refusal_report: edits on paths the zone changed are exit 3 owned by the folder, naming the paths and
//! the snapshot that keeps them; anything else is move_refusal_report's report under the verb resolve.
//! Unit tests: join/move_refusal_report is pure and used directly.

use std::path::PathBuf;

use folders::FolderError;
use gitio::Oid;

use super::resolve_move_refusal_report;
use crate::report::{ExitCode, Report};

const OLD: &str = "1111111111111111111111111111111111111111";
const NEW: &str = "2222222222222222222222222222222222222222";
const ELSE: &str = "3333333333333333333333333333333333333333";
const SNAP: &str = "4444444444444444444444444444444444444444";
const SNAP_REF: &str = "refs/mortar/snap/core/fix-1";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn before() -> Vec<String> {
    vec!["rebased: refs/heads/req/core/fix-1 moved".to_string()]
}

#[test]
fn edits_on_zone_changed_paths_are_named_with_the_snapshot_that_keeps_them() {
    let error = FolderError::MoveRefused {
        path: PathBuf::from("/w/fix-1"),
        paths: vec![PathBuf::from("src/a.rs"), PathBuf::from("b.txt")],
    };
    assert_eq!(
        resolve_move_refusal_report(&error, SNAP_REF, &oid(SNAP), before(), None),
        Report {
            verb: "resolve".to_string(),
            exit: ExitCode::Refused,
            outcome: "refused".to_string(),
            lines: vec![
                "rebased: refs/heads/req/core/fix-1 moved".to_string(),
                "the folder could not move; edits on paths the zone changed stand in the way:".to_string(),
                "  src/a.rs".to_string(),
                "  b.txt".to_string(),
                "they are kept in refs/mortar/snap/core/fix-1 at 4444444444444444444444444444444444444444".to_string(),
            ],
            owner: Some("this request's folder".to_string()),
            next: "put those files back to the folder's version, then mr join".to_string(),
            fields: Vec::new(),
        }
    );
}

#[test]
fn a_move_refusal_puts_the_preceding_snapshot_move_first() {
    let error = FolderError::MoveRefused {
        path: PathBuf::from("/w/fix-1"),
        paths: vec![PathBuf::from("src/a.rs")],
    };
    assert_eq!(
        resolve_move_refusal_report(
            &error, SNAP_REF, &oid(SNAP), before(), Some("the snapshot ref moved as the work was saved first"),
        ),
        Report {
            verb: "resolve".to_string(),
            exit: ExitCode::Refused,
            outcome: "refused".to_string(),
            lines: vec![
                "the snapshot ref moved as the work was saved first".to_string(),
                "rebased: refs/heads/req/core/fix-1 moved".to_string(),
                "the folder could not move; edits on paths the zone changed stand in the way:".to_string(),
                "  src/a.rs".to_string(),
                "they are kept in refs/mortar/snap/core/fix-1 at 4444444444444444444444444444444444444444".to_string(),
            ],
            owner: Some("this request's folder".to_string()),
            next: "put those files back to the folder's version, then mr join".to_string(),
            fields: Vec::new(),
        }
    );
}

#[test]
fn a_branch_elsewhere_is_joins_move_refusal_under_resolve() {
    let error = FolderError::BranchElsewhere {
        branch: "refs/heads/req/core/fix-1".to_string(),
        at: Some(oid(ELSE)),
        old: oid(OLD),
        new: oid(NEW),
    };
    assert_eq!(
        resolve_move_refusal_report(&error, SNAP_REF, &oid(SNAP), before(), None),
        Report {
            verb: "resolve".to_string(),
            exit: ExitCode::Refused,
            outcome: "refused".to_string(),
            lines: vec![
                "rebased: refs/heads/req/core/fix-1 moved".to_string(),
                "branch refs/heads/req/core/fix-1 is at 3333333333333333333333333333333333333333, neither the move's \
                 old tip 1111111111111111111111111111111111111111 nor its new tip \
                 2222222222222222222222222222222222222222"
                    .to_string(),
            ],
            owner: Some("this request's folder".to_string()),
            next: "put the request branch back where mortar left it, then mr join".to_string(),
            fields: Vec::new(),
        }
    );
}

#[test]
fn anything_else_is_joins_move_refusal_under_resolve_after_the_lines_before() {
    let error = FolderError::NotSaved { path: PathBuf::from("/w/fix-1") };
    assert_eq!(
        resolve_move_refusal_report(&error, SNAP_REF, &oid(SNAP), before(), None),
        Report {
            verb: "resolve".to_string(),
            exit: ExitCode::Refused,
            outcome: "refused".to_string(),
            lines: vec![
                "rebased: refs/heads/req/core/fix-1 moved".to_string(),
                "/w/fix-1 holds work that is not saved".to_string(),
            ],
            owner: Some("this request's folder".to_string()),
            next: "repeat once the folder's work is saved".to_string(),
            fields: Vec::new(),
        }
    );
}
