//! resolve_report: the first run's files written is exit 2 with the files, skipped paths and submodules; a rebase
//! reports as join does plus the snapshot's ref and commit; a conflict on a later commit reports as join does;
//! a finished folder move comes first; a joined answer, which resolve never gives, is a failure.
//! ZUT: join's answer_part and finished_move_part and report::failure_report are pure and used directly.

use std::path::PathBuf;

use ::join::JoinAnswer;
use gitio::Oid;

use super::{resolve_report, ResolveOutcome};
use crate::join::{MoveCause, PendingMove};
use crate::report::{ExitCode, OutValue, Report};

const ONE: &str = "1111111111111111111111111111111111111111";
const TWO: &str = "2222222222222222222222222222222222222222";
const THREE: &str = "3333333333333333333333333333333333333333";
const SNAP: &str = "4444444444444444444444444444444444444444";
const OLD: &str = "5555555555555555555555555555555555555555";
const NEW: &str = "6666666666666666666666666666666666666666";
const REQUEST_REF: &str = "refs/heads/req/core/fix-1";
const SNAP_REF: &str = "refs/mortar/snap/core/fix-1";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn paths(names: &[&str]) -> Vec<PathBuf> {
    names.iter().map(PathBuf::from).collect()
}

fn text(name: &str, value: &str) -> (String, OutValue) {
    (name.to_string(), OutValue::Text(value.to_string()))
}

fn texts(name: &str, values: &[&str]) -> (String, OutValue) {
    (name.to_string(), OutValue::Texts(values.iter().map(ToString::to_string).collect()))
}

fn null(name: &str) -> (String, OutValue) {
    (name.to_string(), OutValue::Null)
}

fn outcome(finished_move: Option<PendingMove>, answer: JoinAnswer) -> ResolveOutcome {
    ResolveOutcome {
        request_ref: REQUEST_REF.to_string(),
        finished_move,
        snapshot_ref: SNAP_REF.to_string(),
        snapshot_written: false,
        snapshot_commit: oid(SNAP),
        answer,
    }
}

#[test]
fn the_first_run_lists_files_written_skipped_paths_and_submodules() {
    let answer = JoinAnswer::FilesWritten {
        commit: oid(ONE),
        files: paths(&["src/a.rs", "b.txt"]),
        skipped: paths(&["moved.rs"]),
        gitlinks: paths(&["vendor/lib"]),
    };
    assert_eq!(
        resolve_report(&outcome(None, answer)),
        Report {
            verb: "resolve".to_string(),
            exit: ExitCode::Conflict,
            outcome: "files-written".to_string(),
            lines: vec![
                "files written: the conflicted versions of commit 1111111111111111111111111111111111111111, markers \
                 included; no ref moved"
                    .to_string(),
                "  src/a.rs".to_string(),
                "  b.txt".to_string(),
                "skipped (no version in the conflicted tree): moved.rs".to_string(),
                "submodule vendor/lib: set its commit in the folder, then mr resolve".to_string(),
            ],
            owner: None,
            next: "fix the files, then mr resolve".to_string(),
            fields: vec![
                null("finished_move_from"),
                null("finished_move_to"),
                text("request_ref", REQUEST_REF),
                text("commit", ONE),
                texts("files", &["src/a.rs", "b.txt"]),
                texts("skipped", &["moved.rs"]),
                texts("gitlinks", &["vendor/lib"]),
            ],
        }
    );
}

#[test]
fn a_rebase_reports_as_join_does_with_the_snapshot_after_a_finished_move() {
    let finished = PendingMove {
        request_ref: REQUEST_REF.to_string(),
        old: oid(OLD),
        new: oid(NEW),
        op_id: "op-1".to_string(),
        folder_tree: None,
        cause: MoveCause::Rebase,
    };
    let answer = JoinAnswer::Rebased {
        request_ref: REQUEST_REF.to_string(),
        from: oid(ONE),
        to: oid(TWO),
        folder_tree: Some(oid(THREE)),
    };
    assert_eq!(
        resolve_report(&outcome(Some(finished), answer)),
        Report {
            verb: "resolve".to_string(),
            exit: ExitCode::Done,
            outcome: "rebased".to_string(),
            lines: vec![
                "the folder stands at the last rebase's tip: 5555555555555555555555555555555555555555 -> \
                 6666666666666666666666666666666666666666"
                    .to_string(),
                "rebased: refs/heads/req/core/fix-1 moved from 1111111111111111111111111111111111111111 to \
                 2222222222222222222222222222222222222222, replayed onto the zone; the folder moved with it"
                    .to_string(),
                "the unit tests must run again".to_string(),
            ],
            owner: None,
            next: "mr join".to_string(),
            fields: vec![
                text("finished_move_from", OLD),
                text("finished_move_to", NEW),
                text("request_ref", REQUEST_REF),
                text("from", ONE),
                text("to", TWO),
                text("snapshot_ref", SNAP_REF),
                text("snapshot_commit", SNAP),
            ],
        }
    );
}

#[test]
fn a_conflict_on_a_later_commit_reports_as_join_does() {
    let answer = JoinAnswer::Conflicted {
        commit: oid(TWO),
        files: paths(&["src/c.rs"]),
        state: PathBuf::from("/r/.git/worktrees/fix-1/mortar-join.json"),
    };
    assert_eq!(
        resolve_report(&outcome(None, answer)),
        Report {
            verb: "resolve".to_string(),
            exit: ExitCode::Conflict,
            outcome: "conflicted".to_string(),
            lines: vec![
                "conflicted: commit 2222222222222222222222222222222222222222 did not apply; no ref moved and the \
                 folder stays at the request tip"
                    .to_string(),
                "  src/c.rs".to_string(),
                "mr resolve writes both sides into these files with markers; fix them, then run mr resolve again"
                    .to_string(),
            ],
            owner: None,
            next: "mr resolve".to_string(),
            fields: vec![
                null("finished_move_from"),
                null("finished_move_to"),
                text("request_ref", REQUEST_REF),
                text("commit", TWO),
                texts("files", &["src/c.rs"]),
                text("state", "/r/.git/worktrees/fix-1/mortar-join.json"),
            ],
        }
    );
}

#[test]
fn conflicted_names_the_snapshot_move() {
    let answer = JoinAnswer::Conflicted {
        commit: oid(TWO), files: paths(&["src/c.rs"]),
        state: PathBuf::from("/r/.git/worktrees/fix-1/mortar-join.json"),
    };
    let mut result = outcome(None, answer);
    result.snapshot_written = true;
    assert_eq!(resolve_report(&result), Report {
        verb: "resolve".to_string(), exit: ExitCode::Conflict, outcome: "conflicted".to_string(),
        lines: vec![
            "conflicted: commit 2222222222222222222222222222222222222222 did not apply; the snapshot ref moved as the work was saved first; no other ref moved and the folder stays at the request tip".to_string(),
            "  src/c.rs".to_string(),
            "mr resolve writes both sides into these files with markers; fix them, then run mr resolve again".to_string(),
        ],
        owner: None, next: "mr resolve".to_string(),
        fields: vec![
            ("finished_move_from".to_string(), OutValue::Null), ("finished_move_to".to_string(), OutValue::Null),
            ("request_ref".to_string(), OutValue::Text("refs/heads/req/core/fix-1".to_string())),
            ("commit".to_string(), OutValue::Text("2222222222222222222222222222222222222222".to_string())),
            ("files".to_string(), OutValue::Texts(vec!["src/c.rs".to_string()])),
            ("state".to_string(), OutValue::Text("/r/.git/worktrees/fix-1/mortar-join.json".to_string())),
        ],
    });
}

#[test]
fn files_written_names_the_snapshot_move() {
    let answer = JoinAnswer::FilesWritten {
        commit: oid(ONE), files: paths(&["src/a.rs"]), skipped: paths(&["moved.rs"]), gitlinks: paths(&["vendor/lib"]),
    };
    let mut result = outcome(None, answer);
    result.snapshot_written = true;
    assert_eq!(resolve_report(&result), Report {
        verb: "resolve".to_string(), exit: ExitCode::Conflict, outcome: "files-written".to_string(),
        lines: vec![
            "files written: the conflicted versions of commit 1111111111111111111111111111111111111111, markers included; the snapshot ref moved as the work was saved first; no other ref moved".to_string(),
            "  src/a.rs".to_string(),
            "skipped (no version in the conflicted tree): moved.rs".to_string(),
            "submodule vendor/lib: set its commit in the folder, then mr resolve".to_string(),
        ],
        owner: None, next: "fix the files, then mr resolve".to_string(),
        fields: vec![
            ("finished_move_from".to_string(), OutValue::Null), ("finished_move_to".to_string(), OutValue::Null),
            ("request_ref".to_string(), OutValue::Text("refs/heads/req/core/fix-1".to_string())),
            ("commit".to_string(), OutValue::Text("1111111111111111111111111111111111111111".to_string())),
            ("files".to_string(), OutValue::Texts(vec!["src/a.rs".to_string()])),
            ("skipped".to_string(), OutValue::Texts(vec!["moved.rs".to_string()])),
            ("gitlinks".to_string(), OutValue::Texts(vec!["vendor/lib".to_string()])),
        ],
    });
}

fn never_given() -> Report {
    Report {
        verb: "resolve".to_string(),
        exit: ExitCode::Failure,
        outcome: "failed".to_string(),
        lines: vec![
            "join resolve answered joined, which it never gives".to_string(),
        ],
        owner: None,
        next: "repeat the same command".to_string(),
        fields: Vec::new(),
    }
}

#[test]
fn a_joined_answer_is_a_failure() {
    let answer = JoinAnswer::Joined { zone_ref: "refs/heads/zone/core".to_string(), request_tip: oid(ONE) };
    assert_eq!(resolve_report(&outcome(None, answer)), never_given());
}

#[test]
fn a_fast_forward_is_a_failure() {
    let answer =
        JoinAnswer::FastForwarded { zone_ref: "refs/heads/zone/core".to_string(), from: oid(ONE), to: oid(TWO) };
    assert_eq!(resolve_report(&outcome(None, answer)), never_given());
}
