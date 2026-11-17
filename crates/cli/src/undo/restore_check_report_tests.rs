//! restore_check_report: check_restore_join_state refused before anything moved; each refusal says why, that nothing
//! was changed, and its one next step; git and the disk failing are failures to repeat.

use std::io;
use std::path::PathBuf;

use ::join::JoinRunError;
use gitio::GitError;

use super::restore_check_report;
use crate::report::{ExitCode, Report};

const STATE: &str = "/w/.git/mortar-join.json";

fn report(exit: ExitCode, outcome: &str, lines: &[&str], next: &str) -> Report {
    Report {
        verb: "undo".to_string(),
        exit,
        outcome: outcome.to_string(),
        lines: lines.iter().map(ToString::to_string).collect(),
        owner: None,
        next: next.to_string(),
        fields: Vec::new(),
    }
}

#[test]
fn an_op_that_records_no_paused_join_is_refused_with_a_fresh_join_next() {
    assert_eq!(
        restore_check_report(&JoinRunError::NotAResolveOp { op_id: "op-9".to_string() }),
        report(
            ExitCode::Refused,
            "refused",
            &["op op-9 records no paused join that undo can put back", "nothing was changed"],
            "change the files, mr save, then mr join",
        )
    );
}

#[test]
fn a_recorded_state_that_cannot_be_read_back_is_refused_with_a_fresh_join_next() {
    let error = JoinRunError::RecordedStateCorrupt { op_id: "op-9".to_string(), reason: "no files key".to_string() };
    assert_eq!(
        restore_check_report(&error),
        report(
            ExitCode::Refused,
            "refused",
            &["the paused join op op-9 recorded cannot be read back: no files key", "nothing was changed"],
            "change the files, mr save, then mr join",
        )
    );
}

#[test]
fn another_paused_join_is_refused_with_status_next() {
    let error = JoinRunError::StatePresent { state: PathBuf::from(STATE) };
    assert_eq!(
        restore_check_report(&error),
        report(
            ExitCode::Refused,
            "refused",
            &["/w/.git/mortar-join.json holds another paused join, which undo never overwrites", "nothing was changed"],
            "mr status",
        )
    );
}

#[test]
fn a_state_file_mortar_did_not_write_is_refused_and_mended_by_hand() {
    let error = JoinRunError::StateCorrupt { path: PathBuf::from(STATE), reason: "commit is not an oid".to_string() };
    assert_eq!(
        restore_check_report(&error),
        report(
            ExitCode::Refused,
            "refused",
            &[
                "/w/.git/mortar-join.json is not a paused-join file mortar wrote: commit is not an oid",
                "the file is kept",
                "nothing was changed",
            ],
            "mend or remove that file by hand, then mr undo",
        )
    );
}

#[test]
fn an_unreadable_state_file_is_a_failure_to_repeat() {
    let error = JoinRunError::StateRead { path: PathBuf::from(STATE), source: io::Error::other("boom") };
    assert_eq!(
        restore_check_report(&error),
        report(
            ExitCode::Failure,
            "failed",
            &["/w/.git/mortar-join.json could not be read: boom", "nothing was changed"],
            "repeat the same command",
        )
    );
}

#[test]
fn a_git_failure_is_a_failure_to_repeat() {
    let error = JoinRunError::Git(GitError { args: Vec::new(), code: None, stderr: "not a repository".to_string() });
    assert_eq!(
        restore_check_report(&error),
        report(
            ExitCode::Failure,
            "failed",
            &["git failed: git: no exit code: not a repository", "nothing was changed"],
            "repeat the same command",
        )
    );
}

#[test]
fn errors_the_check_never_gives_are_failures_to_repeat() {
    let never = "join answered an error it never gives for a restore of a paused join";
    for error in [
        JoinRunError::ZoneMissing("refs/heads/zone/cli".to_string()),
        JoinRunError::NothingToResolve { state: PathBuf::from(STATE) },
    ] {
        assert_eq!(
            restore_check_report(&error),
            report(ExitCode::Failure, "failed", &[never, "nothing was changed"], "repeat the same command")
        );
    }
}

#[test]
fn a_write_error_the_check_never_gives_is_a_failure_with_its_own_why() {
    let error = JoinRunError::StateWrite { path: PathBuf::from(STATE), source: io::Error::other("full") };
    assert_eq!(
        restore_check_report(&error),
        report(
            ExitCode::Failure,
            "failed",
            &["/w/.git/mortar-join.json could not be written: full", "nothing was changed"],
            "repeat the same command",
        )
    );
}
