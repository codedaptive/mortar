//! resolve_refusal_report: no paused join is exit 0; markers left are exit 2 with each file's marker lines; a stale
//! state, a conflicted path that is a folder or refused, and a corrupt state are exit 1 owned by this request; the
//! rest are join_refusal_report's reports under the verb resolve. The expected lines are join's own refusal texts.
//! The two join lines that name a command (the stale state, the folder path) accept today's text or the same text
//! with its backticked command as `mr` (S-133), so join's rename lands before or after the cli's.
//! ZIT (S-79 as replaced: the line is the file): it calls join_refusal_report for real.

use std::io::{Error, ErrorKind};
use std::path::PathBuf;

use ::join::{JoinRunError, MarkedFile};
use gitio::Oid;

use cli::{resolve_refusal_report, ExitCode, Report};

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

/// Today's join line with its one backticked command "`mortar " as "`mr "; anything but exactly one is a test bug.
fn mr_form(today: &str) -> String {
    assert_eq!(today.matches("`mortar ").count(), 1, "expected exactly one \"`mortar \" in {today:?}");
    today.replacen("`mortar ", "`mr ", 1)
}

/// True when `actual` is today's join line or the same line with the command renamed to `mr` (S-133).
fn join_line_matches(actual: &str, today: &str) -> bool {
    actual == today || actual == mr_form(today)
}

/// Asserts `actual` equals `today` except that its one join line may be today's or its `mr` form.
fn assert_join_report(actual: Report, today: Report) {
    let accepted = actual.lines.len() == 1
        && join_line_matches(&actual.lines[0], &today.lines[0])
        && Report { lines: today.lines.clone(), ..actual.clone() } == today;
    assert!(
        accepted,
        "report matches neither accepted form\n  actual: {actual:?}\n  today:  {today:?}\n  mr:     {:?}",
        mr_form(&today.lines[0])
    );
}

#[test]
fn todays_join_lines_match() {
    assert!(join_line_matches(
        "make it one, then run `mortar resolve` again",
        "make it one, then run `mortar resolve` again"
    ));
    assert!(join_line_matches(
        "remove /r/mortar-join.json, then run `mortar join` to replay the request afresh",
        "remove /r/mortar-join.json, then run `mortar join` to replay the request afresh"
    ));
}

#[test]
fn the_mr_form_written_by_hand_matches() {
    assert!(join_line_matches(
        "make it one, then run `mr resolve` again",
        "make it one, then run `mortar resolve` again"
    ));
    assert!(join_line_matches(
        "remove /r/mortar-join.json, then run `mr join` to replay the request afresh",
        "remove /r/mortar-join.json, then run `mortar join` to replay the request afresh"
    ));
}

#[test]
fn other_join_lines_do_not_match() {
    let today = "remove /r/mortar-join.json, then run `mortar join` to replay the request afresh";
    for actual in [
        "remove /r/mr-join.json, then run `mr join` to replay the request afresh",
        "remove /r/mortar-join.json, then run `mr join` to replay the request again",
        "remove /r/mortar-join.json, then run `mr save` to replay the request afresh",
        "",
    ] {
        assert!(!join_line_matches(actual, today), "{actual:?} must not match");
    }
}

#[test]
fn no_paused_join_is_nothing_to_resolve() {
    assert_eq!(
        resolve_refusal_report(&JoinRunError::NothingToResolve { state: PathBuf::from(STATE) }),
        Report {
            verb: "resolve".to_string(),
            exit: ExitCode::Done,
            outcome: "nothing-to-resolve".to_string(),
            lines: vec!["no join is paused in this folder; nothing to resolve".to_string()],
            owner: None,
            next: "mortar join".to_string(),
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
        resolve_refusal_report(&JoinRunError::StillMarked(files)),
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
            next: "fix the files, then mortar resolve".to_string(),
            fields: Vec::new(),
        }
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
    assert_join_report(
        resolve_refusal_report(&error),
        refused(
            &["request ref refs/heads/req/core/fix-1 is at 2222222222222222222222222222222222222222, not at \
               1111111111111111111111111111111111111111 where the paused join left it, so \
               /r/.git/worktrees/fix-1/mortar-join.json no longer describes the request; the state file was kept and \
               no ref was moved. To start over: remove /r/.git/worktrees/fix-1/mortar-join.json, then run `mortar \
               join` to replay the request afresh"],
            "put refs/heads/req/core/fix-1 back at 1111111111111111111111111111111111111111, then mortar resolve",
        )
    );
}

#[test]
fn a_conflicted_path_that_is_a_folder_needs_a_file() {
    assert_join_report(
        resolve_refusal_report(&JoinRunError::ConflictedPathIsFolder { path: PathBuf::from("src/dir") }),
        refused(
            &["conflicted path src/dir names a folder; nothing was committed and no ref was moved. A conflicted path \
               must be a file or removed: make it one, then run `mortar resolve` again"],
            "put a file at that path, then mortar resolve",
        )
    );
}

#[test]
fn a_refused_conflicted_path_needs_a_file() {
    let error =
        JoinRunError::ConflictedPathRefused { path: Some(PathBuf::from("src/x.rs")), reason: "bad mode".to_string() };
    assert_eq!(
        resolve_refusal_report(&error),
        refused(
            &["conflicted path src/x.rs was refused: bad mode; nothing was committed and no ref was moved"],
            "put a file at that path, then mortar resolve",
        )
    );
}

#[test]
fn refused_conflicted_paths_with_no_path_named_need_a_file() {
    let error = JoinRunError::ConflictedPathRefused { path: None, reason: "bad mode".to_string() };
    assert_eq!(
        resolve_refusal_report(&error),
        refused(
            &["the conflicted paths were refused: bad mode; nothing was committed and no ref was moved"],
            "put a file at that path, then mortar resolve",
        )
    );
}

#[test]
fn a_corrupt_state_is_kept_and_mended_by_hand() {
    let error = JoinRunError::StateCorrupt { path: PathBuf::from(STATE), reason: "no commit field".to_string() };
    assert_eq!(
        resolve_refusal_report(&error),
        refused(
            &[
                "/r/.git/worktrees/fix-1/mortar-join.json is not a state join wrote: no commit field; resolve refuses \
                 and no ref was moved",
                "the file is kept",
            ],
            "mend or remove that file by hand, then mortar join",
        )
    );
}

#[test]
fn a_missing_request_is_joins_refusal_under_resolve() {
    assert_eq!(
        resolve_refusal_report(&JoinRunError::RequestMissing("refs/heads/req/core/fix-1".to_string())),
        refused(
            &["request ref refs/heads/req/core/fix-1 does not exist; nothing to join"],
            "make the request folder again, then mortar join",
        )
    );
}

#[test]
fn an_unreadable_state_is_joins_failure_under_resolve() {
    let error = JoinRunError::StateRead { path: PathBuf::from(STATE), source: Error::new(ErrorKind::Other, "denied") };
    assert_eq!(
        resolve_refusal_report(&error),
        Report {
            verb: "resolve".to_string(),
            exit: ExitCode::Failure,
            outcome: "failed".to_string(),
            lines: vec![
                "/r/.git/worktrees/fix-1/mortar-join.json: denied; the join state could not be read and no ref was \
                 moved"
                    .to_string(),
                "nothing half-done is left".to_string(),
            ],
            owner: None,
            next: "repeat the same command".to_string(),
            fields: Vec::new(),
        }
    );
}
