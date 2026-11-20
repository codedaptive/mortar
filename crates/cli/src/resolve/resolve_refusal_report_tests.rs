//! resolve_refusal_report: no paused join is exit 0; markers left are exit 2 with each file's marker lines; a stale
//! state, a conflicted path that is a folder or refused, and a corrupt state are exit 1 owned by this request; the
//! rest are join_refusal_report's reports under the verb resolve. The expected lines are join's own refusal texts.
//! Unit tests: the errors are built as data; their lines come from join's exported, pure Display impls,
//! and join_refusal_report and the report module are pure and used directly.

use std::io::Error;
use std::path::PathBuf;

use ::join::{JoinRunError, MarkedFile};
use gitio::Oid;

use super::resolve_refusal_report;
use crate::report::{ExitCode, Report};

const ONE: &str = "1111111111111111111111111111111111111111";
const TWO: &str = "2222222222222222222222222222222222222222";
const STATE: &str = "/r/.git/worktrees/fix-1/mortar-join.json";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn refused(lines: &[&str], next: &str) -> Report {
    Report {
        verb: "resolve".to_string(),
        exit: ExitCode::Refused,
        outcome: "refused".to_string(),
        lines: lines.iter().map(ToString::to_string).collect(),
        owner: Some("this request".to_string()),
        next: next.to_string(),
        fields: Vec::new(),
    }
}

#[test]
fn no_paused_join_is_nothing_to_resolve() {
    assert_eq!(
        resolve_refusal_report(&JoinRunError::NothingToResolve { state: PathBuf::from(STATE) }, None),
        Report {
            verb: "resolve".to_string(),
            exit: ExitCode::Done,
            outcome: "nothing-to-resolve".to_string(),
            lines: vec!["no join is paused in this folder; nothing to resolve".to_string()],
            owner: None,
            next: "mr join".to_string(),
            fields: Vec::new(),
        }
    );
}

#[test]
fn no_paused_join_names_the_preceding_move() {
    assert_eq!(
        resolve_refusal_report(
            &JoinRunError::NothingToResolve { state: PathBuf::from(STATE) },
            Some("the snapshot ref moved as the work was saved first"),
        ),
        Report {
            verb: "resolve".to_string(),
            exit: ExitCode::Done,
            outcome: "nothing-to-resolve".to_string(),
            lines: vec![
                "the snapshot ref moved as the work was saved first".to_string(),
                "no join is paused in this folder; nothing to resolve".to_string(),
            ],
            owner: None,
            next: "mr join".to_string(),
            fields: Vec::new(),
        }
    );
}

#[test]
fn markers_left_name_each_file_and_its_lines() {
    let files = vec![
        MarkedFile { path: PathBuf::from("src/a.rs"), lines: vec![3, 7, 12] },
        MarkedFile { path: PathBuf::from("b.txt"), lines: vec![1] },
    ];
    assert_eq!(
        resolve_refusal_report(&JoinRunError::StillMarked(files), None),
        Report {
            verb: "resolve".to_string(),
            exit: ExitCode::Conflict,
            outcome: "still-marked".to_string(),
            lines: vec![
                "conflict markers remain in these files; no commit was made and no ref moved:".to_string(),
                "  src/a.rs (lines 3, 7, 12)".to_string(),
                "  b.txt (lines 1)".to_string(),
            ],
            owner: None,
            next: "fix the files, then mr resolve".to_string(),
            fields: Vec::new(),
        }
    );
}

#[test]
fn markers_left_name_the_preceding_move() {
    let files = vec![
        MarkedFile { path: PathBuf::from("src/a.rs"), lines: vec![3, 7, 12] },
        MarkedFile { path: PathBuf::from("b.txt"), lines: vec![1] },
    ];
    assert_eq!(
        resolve_refusal_report(&JoinRunError::StillMarked(files), Some("the snapshot ref moved as the work was saved first")),
        Report {
            verb: "resolve".to_string(), exit: ExitCode::Conflict, outcome: "still-marked".to_string(),
            lines: vec![
                "conflict markers remain in these files; the snapshot ref moved as the work was saved first; no other commit was made and no other ref moved:".to_string(),
                "  src/a.rs (lines 3, 7, 12)".to_string(),
                "  b.txt (lines 1)".to_string(),
            ],
            owner: None, next: "fix the files, then mr resolve".to_string(), fields: Vec::new(),
        }
    );
}

#[test]
fn conflicted_folder_puts_the_preceding_move_first() {
    let report = resolve_refusal_report(
        &JoinRunError::ConflictedPathIsFolder { path: PathBuf::from("src/dir") },
        Some("the snapshot ref moved as the work was saved first"),
    );
    assert_eq!(report.lines.len(), 2);
    assert_eq!(report.lines[0], "the snapshot ref moved as the work was saved first");
    assert_eq!(
        report.lines[1],
        "conflicted path src/dir names a folder; join moved no ref. A conflicted path \
         must be a file or removed: make it one, then run `mr resolve` again",
    );
    assert_eq!(report.verb, "resolve");
    assert_eq!(report.exit, ExitCode::Refused);
    assert_eq!(report.outcome, "refused");
    assert_eq!(report.owner.as_deref(), Some("this request"));
    assert_eq!(report.next, "put a file at that path, then mr resolve");
    assert!(report.fields.is_empty());
}

#[test]
fn missing_request_fallback_puts_the_preceding_move_first_once() {
    assert_eq!(
        resolve_refusal_report(
            &JoinRunError::RequestMissing("refs/heads/req/core/fix-1".to_string()),
            Some("the snapshot ref moved as the work was saved first"),
        ),
        refused(
            &[
                "the snapshot ref moved as the work was saved first",
                "request ref refs/heads/req/core/fix-1 does not exist; nothing to join",
            ],
            "make the request folder again, then mr join",
        )
    );
}

#[test]
fn a_stale_state_puts_the_request_back() {
    let error = JoinRunError::StateStale {
        request_ref: "refs/heads/req/core/fix-1".to_string(),
        expected: oid(ONE),
        actual: Some(oid(TWO)),
        state: PathBuf::from(STATE),
    };
    assert_eq!(
        resolve_refusal_report(&error, None),
        refused(
            &["request ref refs/heads/req/core/fix-1 is at 2222222222222222222222222222222222222222, not at \
               1111111111111111111111111111111111111111 where the paused join left it, so \
               /r/.git/worktrees/fix-1/mortar-join.json no longer describes the request; the state file was kept and \
               join moved no ref. To start over: remove /r/.git/worktrees/fix-1/mortar-join.json, then run `mr \
               join` to replay the request afresh"],
            "put refs/heads/req/core/fix-1 back at 1111111111111111111111111111111111111111, then mr resolve",
        )
    );
}

#[test]
fn a_conflicted_path_that_is_a_folder_needs_a_file() {
    assert_eq!(
        resolve_refusal_report(&JoinRunError::ConflictedPathIsFolder { path: PathBuf::from("src/dir") }, None),
        refused(
            &["conflicted path src/dir names a folder; join moved no ref. A conflicted path \
               must be a file or removed: make it one, then run `mr resolve` again"],
            "put a file at that path, then mr resolve",
        )
    );
}

#[test]
fn a_refused_conflicted_path_needs_a_file() {
    let error =
        JoinRunError::ConflictedPathRefused { path: Some(PathBuf::from("src/x.rs")), reason: "bad mode".to_string() };
    assert_eq!(
        resolve_refusal_report(&error, None),
        refused(
            &["conflicted path src/x.rs was refused: bad mode; join moved no ref"],
            "put a file at that path, then mr resolve",
        )
    );
}

#[test]
fn refused_conflicted_paths_with_no_path_named_need_a_file() {
    let error = JoinRunError::ConflictedPathRefused { path: None, reason: "bad mode".to_string() };
    assert_eq!(
        resolve_refusal_report(&error, None),
        refused(
            &["the conflicted paths were refused: bad mode; join moved no ref"],
            "put a file at that path, then mr resolve",
        )
    );
}

#[test]
fn a_corrupt_state_is_kept_and_mended_by_hand() {
    let error = JoinRunError::StateCorrupt { path: PathBuf::from(STATE), reason: "no commit field".to_string() };
    assert_eq!(
        resolve_refusal_report(&error, None),
        refused(
            &[
                "/r/.git/worktrees/fix-1/mortar-join.json is not a state join wrote: no commit field; join moved no ref",
                "the file is kept",
            ],
            "mend or remove that file by hand, then mr join",
        )
    );
}

#[test]
fn a_missing_request_is_joins_refusal_under_resolve() {
    assert_eq!(
        resolve_refusal_report(&JoinRunError::RequestMissing("refs/heads/req/core/fix-1".to_string()), None),
        refused(
            &["request ref refs/heads/req/core/fix-1 does not exist; nothing to join"],
            "make the request folder again, then mr join",
        )
    );
}

#[test]
fn an_unreadable_state_is_joins_failure_under_resolve() {
    let error = JoinRunError::StateRead { path: PathBuf::from(STATE), source: Error::other("denied") };
    assert_eq!(
        resolve_refusal_report(&error, None),
        Report {
            verb: "resolve".to_string(),
            exit: ExitCode::Failure,
            outcome: "failed".to_string(),
            lines: vec![
                "/r/.git/worktrees/fix-1/mortar-join.json: denied; the join state could not be read and join moved \
                 no ref"
                    .to_string(),
            ],
            owner: None,
            next: "repeat the same command".to_string(),
            fields: Vec::new(),
        }
    );
}
