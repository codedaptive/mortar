//! A git failure inside a refused restore of a paused join: restore_why gives git's own text, and the reports of
//! restore_check_report (exit 5, a failure to repeat) and restore_failed_report (exit 1, a refusal naming both ops)
//! carry it. RIT (S-101, S-135): the expected lines are gitio's GitError Display, run for real; moved here under
//! S-136 (1), Nagatha's from landing.

use ::join::JoinRunError;
use gitio::GitError;

use cli::report::{ExitCode, OutValue, Report};
use cli::undo::{restore_check_report, restore_failed_report, restore_why};

const FIRST: &str = "op op-7 was undone by op op-12; the paused join was not put back";

fn why(error: JoinRunError) -> Vec<String> {
    restore_why(&error)
}

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
fn a_git_failure_gives_git_s_own_text() {
    let error = JoinRunError::Git(GitError {
        args: vec!["rev-parse".to_string(), "x".to_string()],
        code: Some(128),
        stderr: "fatal: bad\n".to_string(),
    });
    assert_eq!(why(error), vec!["git failed: git rev-parse x: exit code 128: fatal: bad"]);
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
fn a_git_failure_is_a_refusal_too() {
    let error = JoinRunError::Git(GitError { args: vec!["show".to_string()], code: Some(1), stderr: "no".to_string() });
    assert_eq!(failed(error), refused(&["git failed: git show: exit code 1: no"]));
}
