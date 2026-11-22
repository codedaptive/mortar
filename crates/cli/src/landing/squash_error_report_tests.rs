//! squash_error_report: every SquashError variant in the cli's own words, none of land's, gitio's or oplog's texts.

use gitio::{GitError, Oid};
use land::SquashError;
use oplog::{ChangeIdError, MortarTrailerError, OpPhase, OpRecord, OpResult, OplogError};

use super::squash_error_report;
use crate::report::{ExitCode, Report};

const A: &str = "1111111111111111111111111111111111111111";
const B: &str = "2222222222222222222222222222222222222222";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn record(phase: OpPhase) -> OpRecord {
    OpRecord {
        op_id: "op-7".to_string(),
        key: "land-squash:refs/mortar/squash/a:op-7".to_string(),
        kind: "land-squash".to_string(),
        actor: "42@host".to_string(),
        reason: "r".to_string(),
        at: 5,
        phase,
        moves: Vec::new(),
        result: Some(OpResult::Moved),
    }
}

fn refused(line: &str, owner: &str, next: &str) -> Report {
    Report {
        verb: "squash".to_string(),
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
        verb: "squash".to_string(),
        exit: ExitCode::Failure,
        outcome: "failed".to_string(),
        lines: vec![line.to_string()],
        owner: None,
        next: "repeat the same command".to_string(),
        fields: Vec::new(),
    }
}

#[test]
fn git_is_a_failure_in_the_clis_words() {
    let error = SquashError::Git(GitError::refusal(vec!["merge-base".to_string()], "fatal: gitio text"));
    assert_eq!(
        squash_error_report("squash", &error),
        failed("git failed while reading the branches or writing the squash commit")
    );
}

#[test]
fn oplog_is_a_failure_in_the_clis_words() {
    let error = SquashError::Oplog(OplogError::Malformed { line: Some(3), reason: "oplog text".to_string() });
    assert_eq!(squash_error_report("squash", &error), failed("the op log could not be read or written"));
}

#[test]
fn trailers_is_a_failure_in_the_clis_words() {
    let error = SquashError::Trailers(MortarTrailerError::Git(GitError::refusal(Vec::new(), "trailer text")));
    assert_eq!(
        squash_error_report("squash", &error),
        failed("the Mortar trailers of a squash or zone commit could not be read")
    );
}

#[test]
fn a_bad_change_id_is_refused_with_the_next_step_in_words() {
    let error = SquashError::ChangeId(ChangeIdError::Malformed { text: "abc".to_string(), reason: "r".to_string() });
    assert_eq!(
        squash_error_report("squash", &error),
        refused(
            "a zone commit carries a malformed Mortar-Change-Id, or no new change ID could be drawn",
            "the zone branch's commits",
            "read the zone's Mortar-Change-Id trailers with git log, mend the malformed one through a request, then \
             repeat the same command",
        )
    );
}

#[test]
fn a_missing_ref_is_refused_naming_it() {
    let error = SquashError::RefMissing { refname: "refs/heads/zone/b".to_string() };
    assert_eq!(
        squash_error_report("squash", &error),
        refused(
            "refs/heads/zone/b does not exist; nothing was written and no ref moved",
            "the repository's branches",
            "create refs/heads/zone/b, then repeat the same command",
        )
    );
}

#[test]
fn a_missing_attribute_source_is_refused_naming_the_commit() {
    let error = SquashError::AttrSourceMissing { commit: oid(A) };
    assert_eq!(
        squash_error_report("squash", &error),
        refused(
            "commit 1111111111111111111111111111111111111111, the merge's attribute source, does not exist; nothing \
             was written",
            "the repository's objects",
            "fetch or restore commit 1111111111111111111111111111111111111111, then repeat the same command",
        )
    );
}

#[test]
fn unrelated_histories_are_refused_naming_both_tips() {
    let error = SquashError::Unrelated { origin: oid(A), source: oid(B) };
    assert_eq!(
        squash_error_report("squash", &error),
        refused(
            "the zone tip 2222222222222222222222222222222222222222 shares no history with origin \
             1111111111111111111111111111111111111111; nothing was squashed",
            "the zone branch",
            "rebuild the zone branch from origin, then repeat the same command",
        )
    );
}

#[test]
fn a_failed_op_is_refused_naming_its_op_id() {
    let error = SquashError::OpFailed(record(OpPhase::Failed));
    assert_eq!(
        squash_error_report("squash", &error),
        refused(
            "op op-7 moved nothing: the squash ref was not at its expected value",
            "the squash ref",
            "repeat the same command; it reads the refs afresh",
        )
    );
}

#[test]
fn a_repeated_op_is_refused_naming_its_key() {
    let error = SquashError::OpRepeated(record(OpPhase::Done));
    assert_eq!(
        squash_error_report("squash", &error),
        refused(
            "op key land-squash:refs/mortar/squash/a:op-7 is already done in the op log, so nothing moved",
            "the op log",
            "repeat the same command",
        )
    );
}

#[test]
fn a_missing_trailer_is_refused_naming_the_commit_and_trailer() {
    let error = SquashError::TrailerMissing { commit: oid(A), trailer: "Mortar-Change-Id".to_string() };
    assert_eq!(
        squash_error_report("squash", &error),
        refused(
            "commit 1111111111111111111111111111111111111111 carries no Mortar-Change-Id trailer, so it is not a \
             squash Mortar wrote",
            "the commit 1111111111111111111111111111111111111111",
            "name the commit at refs/mortar/squash/<zone>, or make the squash again: mr squash <zone> --actor \
             <name>",
        )
    );
}

#[test]
fn a_malformed_trailer_is_refused_quoting_its_value() {
    let error = SquashError::TrailerMalformed {
        commit: oid(B),
        trailer: "Mortar-Squash-Of".to_string(),
        value: "not an id".to_string(),
    };
    assert_eq!(
        squash_error_report("squash", &error),
        refused(
            "commit 2222222222222222222222222222222222222222's Mortar-Squash-Of trailer holds \"not an id\", which is \
             not a change ID or a commit id",
            "the commit 2222222222222222222222222222222222222222",
            "make the squash again: mr squash <zone> --actor <name>",
        )
    );
}

#[test]
fn malformed_request_lines_are_refused_naming_the_commit() {
    let error = SquashError::RequestsMalformed { commit: oid(A) };
    assert_eq!(
        squash_error_report("squash", &error),
        refused(
            "commit 1111111111111111111111111111111111111111's message does not list its requests as squash reads \
             them: one \"- <request>: ...\" line each in the paragraph before the trailers, as many as a \"<zone>: \
             <n> requests\" subject counts",
            "the commit 1111111111111111111111111111111111111111",
            "make the squash again: mr squash <zone> --actor <name>",
        )
    );
}

#[test]
fn the_verb_given_is_the_reports_verb() {
    let error = SquashError::RefMissing { refname: "refs/heads/main".to_string() };
    assert_eq!(squash_error_report("publish", &error).verb, "publish");
}
