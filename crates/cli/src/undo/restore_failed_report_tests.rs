//! restore_failed_report: restore_join_state refused after the oplog undo moved the ref (S-155 Q2); every error is a
//! refusal naming both ops, then why, with mr undo next to finish the restore.
//! ZUT (rule 11): restore_failed_report builds its why lines by calling undo::restore_why, which is pure on these
//! errors, directly. Git failing, a_git_failure_is_a_refusal_too, runs gitio's GitError Display, exported, pure and
//! below cli (S-171).

use std::io;
use std::path::PathBuf;

use ::join::JoinRunError;
use gitio::{GitError, Oid};

use super::restore_failed_report;
use crate::report::{ExitCode, OutValue, Report};

const STATE: &str = "/w/.git/mortar-join.json";
const FIRST: &str = "op op-7 was undone by op op-12; the paused join was not put back";

fn refused(why: &[&str]) -> Report {
    let mut lines = vec![FIRST.to_string()];
    lines.extend(why.iter().map(ToString::to_string));
    Report {
        verb: "undo".to_string(),
        exit: ExitCode::Refused,
        outcome: "refused".to_string(),
        lines,
        owner: None,
        next: "mr undo".to_string(),
        fields: vec![
            ("undone_op".to_string(), OutValue::Text("op-7".to_string())),
            ("undo_op".to_string(), OutValue::Text("op-12".to_string())),
        ],
    }
}

fn failed(error: JoinRunError) -> Report {
    restore_failed_report(&error, "op-7", "op-12")
}

#[test]
fn an_op_that_records_no_paused_join_is_refused_naming_both_ops() {
    assert_eq!(
        failed(JoinRunError::NotAResolveOp { op_id: "op-7".to_string() }),
        refused(&["op op-7 records no paused join that undo can put back"])
    );
}

#[test]
fn a_recorded_state_that_cannot_be_read_back_is_refused() {
    let error = JoinRunError::RecordedStateCorrupt { op_id: "op-7".to_string(), reason: "bad oid".to_string() };
    assert_eq!(failed(error), refused(&["the paused join op op-7 recorded cannot be read back: bad oid"]));
}

#[test]
fn another_paused_join_is_refused() {
    let error = JoinRunError::StatePresent { state: PathBuf::from(STATE) };
    assert_eq!(
        failed(error),
        refused(&["/w/.git/mortar-join.json holds another paused join, which undo never overwrites"])
    );
}

#[test]
fn a_state_file_mortar_did_not_write_is_refused_and_kept() {
    let error = JoinRunError::StateCorrupt { path: PathBuf::from(STATE), reason: "no commit".to_string() };
    assert_eq!(
        failed(error),
        refused(&["/w/.git/mortar-join.json is not a paused-join file mortar wrote: no commit", "the file is kept"])
    );
}

#[test]
fn disk_errors_are_refusals_too() {
    let read = JoinRunError::StateRead { path: PathBuf::from(STATE), source: io::Error::other("boom") };
    assert_eq!(failed(read), refused(&["/w/.git/mortar-join.json could not be read: boom"]));
    let write = JoinRunError::StateWrite {
        path: PathBuf::from(STATE),
        source: io::Error::new(io::ErrorKind::NotFound, "gone"),
    };
    assert_eq!(failed(write), refused(&["/w/.git/mortar-join.json could not be written: gone"]));
}

#[test]
fn a_non_utf8_conflicted_path_is_refused() {
    let error = JoinRunError::StatePath(PathBuf::from("src/a.rs"));
    assert_eq!(
        failed(error),
        refused(&["the conflicted path src/a.rs is not UTF-8, so the paused-join file cannot hold it"])
    );
}

#[test]
fn a_moved_request_ref_is_refused() {
    let error = JoinRunError::RestoreStale {
        request_ref: "refs/heads/req/a".to_string(),
        expected: Oid::parse(&"c".repeat(40)).expect("a valid oid"),
        actual: None,
    };
    assert_eq!(
        failed(error),
        refused(&[
            "refs/heads/req/a is at nothing, not at cccccccccccccccccccccccccccccccccccccccc, where the paused join stopped"
        ])
    );
}

#[test]
fn errors_a_restore_never_gives_are_refusals_with_the_shared_line() {
    let never = "join answered an error it never gives for a restore of a paused join";
    for error in [
        JoinRunError::ZoneMissing("refs/heads/zone/cli".to_string()),
        JoinRunError::NothingToResolve { state: PathBuf::from(STATE) },
    ] {
        assert_eq!(failed(error), refused(&[never]));
    }
}

#[test]
fn a_git_failure_is_a_refusal_too() {
    let error = JoinRunError::Git(GitError { args: vec!["show".to_string()], code: Some(1), stderr: "no".to_string() });
    assert_eq!(failed(error), refused(&["git failed: git show: exit code 1: no"]));
}
