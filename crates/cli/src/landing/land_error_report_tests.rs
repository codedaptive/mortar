//! land_error_report: every LandError variant in the cli's own words, none of land's, gitio's or oplog's texts.

use std::path::PathBuf;

use gitio::{GitError, Oid};
use land::{LandError, LandingStateError, SquashError};
use oplog::{ChangeIdError, MortarTrailerError, OpPhase, OpRecord, OpResult, OplogError};

use super::land_error_report;
use crate::report::{ExitCode, Report};

const A: &str = "1111111111111111111111111111111111111111";
const B: &str = "2222222222222222222222222222222222222222";
const RESTORE: &str = "find the op that moved it with mr log --ref refs/heads/integ, then mr restore <op id> \
                       refs/heads/integ --actor <name>";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn record(phase: OpPhase) -> OpRecord {
    OpRecord {
        op_id: "op-9".to_string(),
        key: "land:refs/heads/integ:op-9".to_string(),
        kind: "land".to_string(),
        actor: "42@host".to_string(),
        reason: "r".to_string(),
        at: 5,
        phase,
        moves: Vec::new(),
        result: Some(OpResult::Moved),
    }
}

fn report(error: &LandError) -> Report {
    land_error_report("back-out", error, "integ")
}

fn refused(line: &str, owner: &str, next: &str) -> Report {
    Report {
        verb: "back-out".to_string(),
        exit: ExitCode::Refused,
        outcome: "refused".to_string(),
        lines: vec![line.to_string()],
        owner: Some(owner.to_string()),
        next: next.to_string(),
        fields: Vec::new(),
    }
}

fn failed(line: &str) -> Report {
    Report {
        verb: "back-out".to_string(),
        exit: ExitCode::Failure,
        outcome: "failed".to_string(),
        lines: vec![line.to_string()],
        owner: None,
        next: "repeat the same command".to_string(),
        fields: Vec::new(),
    }
}

#[test]
fn a_squash_error_is_squashs_report() {
    let error = LandError::Squash(SquashError::RefMissing { refname: "refs/heads/main".to_string() });
    assert_eq!(
        report(&error),
        refused(
            "refs/heads/main does not exist; nothing was written and no ref moved",
            "the repository's branches",
            "create refs/heads/main, then repeat the same command",
        )
    );
}

#[test]
fn a_state_error_is_the_landing_states_report() {
    let path = PathBuf::from("/r/.git/mortar/landing.json");
    let error = LandError::State(LandingStateError::Read { path, source: std::io::Error::other("gone") });
    assert_eq!(report(&error), failed("reading /r/.git/mortar/landing.json failed"));
}

#[test]
fn git_oplog_and_trailers_are_failures_in_the_clis_words() {
    let git = LandError::Git(GitError::refusal(Vec::new(), "gitio text"));
    assert_eq!(report(&git), failed("git failed while reading or moving integration"));
    let oplog = LandError::Oplog(OplogError::Malformed { line: None, reason: "oplog text".to_string() });
    assert_eq!(report(&oplog), failed("the op log could not be read or written"));
    let malformed = ChangeIdError::Malformed { text: "x".to_string(), reason: "r".to_string() };
    let trailers = LandError::Trailers(MortarTrailerError::ChangeId(malformed));
    assert_eq!(report(&trailers), failed("the squash commit's Mortar trailers could not be read"));
}

#[test]
fn a_squash_commit_with_no_zone_is_refused_naming_it() {
    assert_eq!(
        report(&LandError::NoZone { commit: oid(A) }),
        refused(
            "squash commit 1111111111111111111111111111111111111111 carries no Mortar-Zone trailer",
            "the squash commit",
            "make the squash again: mr squash <zone> --actor <name>",
        )
    );
}

#[test]
fn integration_off_origin_with_no_landing_state_is_refused_with_the_restore_steps() {
    assert_eq!(
        report(&LandError::NoLandingState { origin: oid(A), integration: oid(B) }),
        refused(
            "integration is at 2222222222222222222222222222222222222222, not at origin \
             1111111111111111111111111111111111111111, and no landing state names a landing in flight",
            "refs/heads/integ",
            RESTORE,
        )
    );
}

#[test]
fn integration_elsewhere_is_refused_with_the_restore_steps() {
    assert_eq!(
        report(&LandError::IntegrationElsewhere { expected: oid(A), actual: oid(B) }),
        refused(
            "integration is at 2222222222222222222222222222222222222222, at neither origin nor the landed commit \
             1111111111111111111111111111111111111111",
            "refs/heads/integ",
            RESTORE,
        )
    );
}

#[test]
fn a_failed_op_is_refused_naming_its_op_id() {
    assert_eq!(
        report(&LandError::OpFailed(record(OpPhase::Failed))),
        refused(
            "op op-9 moved nothing: a ref was not at its expected value",
            "the landing's refs",
            "repeat the same command; it reads the refs afresh",
        )
    );
}

#[test]
fn a_repeated_op_is_refused_naming_its_key() {
    assert_eq!(
        report(&LandError::OpRepeated(record(OpPhase::Done))),
        refused(
            "op key land:refs/heads/integ:op-9 is already done in the op log, so nothing moved",
            "the op log",
            "repeat the same command",
        )
    );
}
