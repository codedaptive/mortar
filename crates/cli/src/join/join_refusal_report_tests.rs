//! join_refusal_report: each refusal the caller can act on is exit 1 with its owner and next step; the rest are
//! failures. The expected lines are join's own refusal texts.
//! Unit tests: the errors are built as data; their lines come from join's exported, pure Display impls,
//! and report::failure_report is used directly.

use std::io::{Error, ErrorKind};
use std::path::PathBuf;

use ::join::{JoinError, JoinRunError};
use gitio::Oid;

use super::join_refusal_report;
use crate::report::{ExitCode, Report};

const COMMIT: &str = "1111111111111111111111111111111111111111";

fn refused(line: &str, owner: &str, next: &str) -> Report {
    Report {
        verb: "join".to_string(),
        exit: ExitCode::Refused,
        outcome: "refused".to_string(),
        lines: vec![line.to_string()],
        owner: Some(owner.to_string()),
        next: next.to_string(),
        fields: Vec::new(),
    }
}

fn replay(error: JoinError) -> JoinRunError {
    JoinRunError::Replay(error)
}

fn oid() -> Oid {
    Oid::parse(COMMIT).unwrap()
}

#[test]
fn a_missing_zone_is_the_zone_coordinators() {
    assert_eq!(
        join_refusal_report(&JoinRunError::ZoneMissing("refs/heads/zone/core".to_string()), None),
        refused(
            "zone ref refs/heads/zone/core does not exist; join does not create it: make the zone folder first",
            "the zone's coordinator",
            "have the zone made, then mr join",
        )
    );
}

#[test]
fn missing_zone_puts_the_preceding_move_first() {
    assert_eq!(
        join_refusal_report(
            &JoinRunError::ZoneMissing("refs/heads/zone/core".to_string()),
            Some("the snapshot ref moved as the work was saved first"),
        ),
        Report {
            verb: "join".to_string(), exit: ExitCode::Refused, outcome: "refused".to_string(),
            lines: vec![
                "the snapshot ref moved as the work was saved first".to_string(),
                "zone ref refs/heads/zone/core does not exist; join does not create it: make the zone folder first".to_string(),
            ],
            owner: Some("the zone's coordinator".to_string()),
            next: "have the zone made, then mr join".to_string(), fields: Vec::new(),
        }
    );
}

#[test]
fn state_write_failure_puts_the_preceding_move_first() {
    let error = JoinRunError::StateWrite {
        path: PathBuf::from("/r/.git/mortar-join.json.tmp"),
        source: Error::new(ErrorKind::PermissionDenied, "denied"),
    };
    assert_eq!(
        join_refusal_report(&error, Some("the snapshot ref moved as the work was saved first")),
        Report {
            verb: "join".to_string(), exit: ExitCode::Failure, outcome: "failed".to_string(),
            lines: vec![
                "the snapshot ref moved as the work was saved first".to_string(),
                "/r/.git/mortar-join.json.tmp: denied; mortar-join.json was not written and join moved no ref; check \
                 that folder and its permissions, then run the command again".to_string(),
            ],
            owner: None, next: "repeat the same command".to_string(), fields: Vec::new(),
        }
    );
}

#[test]
fn a_missing_request_is_this_requests() {
    assert_eq!(
        join_refusal_report(&JoinRunError::RequestMissing("refs/heads/req/core/fix-1".to_string()), None),
        refused(
            "request ref refs/heads/req/core/fix-1 does not exist; nothing to join",
            "this request",
            "make the request folder again, then mr join",
        )
    );
}

#[test]
fn an_unrelated_request_is_cut_again() {
    let error = JoinRunError::Unrelated {
        zone_ref: "refs/heads/zone/core".to_string(),
        request_ref: "refs/heads/req/core/fix-1".to_string(),
    };
    assert_eq!(
        join_refusal_report(&error, None),
        refused(
            "request ref refs/heads/req/core/fix-1 shares no history with zone ref refs/heads/zone/core; join \
             refuses and moved no ref",
            "this request",
            "cut the request again from the zone",
        )
    );
}

#[test]
fn a_state_path_is_renamed() {
    assert_eq!(
        join_refusal_report(&JoinRunError::StatePath(PathBuf::from("src/bad-name")), None),
        refused(
            "conflicted path src/bad-name is not UTF-8, so mortar-join.json cannot record it; join moved no ref",
            "this request",
            "rename that file to a UTF-8 name, then mr join",
        )
    );
}

#[test]
fn commits_replay_cannot_take_are_rewritten() {
    const NEXT: &str = "rewrite the request's commits as one-parent commits with author dates, then mr join";
    assert_eq!(
        join_refusal_report(&replay(JoinError::MergeCommit(oid())), None),
        refused(
            "join replay: commit 1111111111111111111111111111111111111111 is a merge commit; replay takes \
             one-parent commits only; join moved no ref",
            "this request",
            NEXT,
        )
    );
    assert_eq!(
        join_refusal_report(&replay(JoinError::RootCommit(oid())), None),
        refused(
            "join replay: commit 1111111111111111111111111111111111111111 is a root commit; replay takes \
             one-parent commits only; join moved no ref",
            "this request",
            NEXT,
        )
    );
    assert_eq!(
        join_refusal_report(&replay(JoinError::EmptyAuthorDate(oid())), None),
        refused(
            "join replay: commit 1111111111111111111111111111111111111111 has an empty author date; commit_tree \
             cannot hand an empty date back unchanged; join moved no ref",
            "this request",
            NEXT,
        )
    );
}

#[test]
fn other_replay_errors_are_failures() {
    let report = join_refusal_report(&replay(JoinError::OntoMissing(oid())), None);
    assert_eq!(report.exit, ExitCode::Failure);
    assert_eq!(report.outcome, "failed");
    assert_eq!(report.owner, None);
    assert_eq!(report.next, "repeat the same command");
}

#[test]
fn a_state_write_error_is_a_failure_naming_it() {
    let error = JoinRunError::StateWrite {
        path: PathBuf::from("/r/.git/mortar-join.json.tmp"),
        source: Error::new(ErrorKind::PermissionDenied, "denied"),
    };
    assert_eq!(
        join_refusal_report(&error, None),
        Report {
            verb: "join".to_string(),
            exit: ExitCode::Failure,
            outcome: "failed".to_string(),
            lines: vec![
                "/r/.git/mortar-join.json.tmp: denied; mortar-join.json was not written and join moved no ref; check \
                 that folder and its permissions, then run the command again"
                    .to_string(),
            ],
            owner: None,
            next: "repeat the same command".to_string(),
            fields: Vec::new(),
        }
    );
}
