//! withdraw_error_report: every WithdrawError variant in the cli's own words, none of land's, gitio's or oplog's
//! texts; Published and Landing refused with nothing moved and their next steps.

use std::path::PathBuf;

use gitio::{GitError, Oid};
use land::{LandingStateError, SquashError, WithdrawError};
use oplog::{ChangeId, OpPhase, OpRecord, OpResult, OplogError};

use super::withdraw_error_report;
use crate::report::{ExitCode, Report};

const A: &str = "1111111111111111111111111111111111111111";
const B: &str = "2222222222222222222222222222222222222222";
const CHANGE: &str = "kkkkllllmmmmnnnnooooppppqqqqrrrr";

fn record(phase: OpPhase) -> OpRecord {
    OpRecord {
        op_id: "op-7".to_string(),
        key: "land-withdraw:refs/mortar/squash/core:op-7".to_string(),
        kind: "land-withdraw".to_string(),
        actor: "42@host".to_string(),
        reason: "r".to_string(),
        at: 5,
        phase,
        moves: Vec::new(),
        result: Some(OpResult::Moved),
    }
}

fn refused(lines: &[&str], owner: &str, next: &str) -> Report {
    Report {
        verb: "withdraw".to_string(),
        exit: ExitCode::Refused,
        outcome: "refused".to_string(),
        lines: lines.iter().map(|line| line.to_string()).collect(),
        owner: Some(owner.to_string()),
        next: next.to_string(),
        fields: Vec::new(),
    }
}

fn failed(line: &str) -> Report {
    Report {
        verb: "withdraw".to_string(),
        exit: ExitCode::Failure,
        outcome: "failed".to_string(),
        lines: vec![line.to_string()],
        owner: None,
        next: "repeat the same command".to_string(),
        fields: Vec::new(),
    }
}

#[test]
fn each_error_is_its_report() {
    let rows = [
        (
            WithdrawError::Squash(SquashError::RefMissing { refname: "refs/heads/main".to_string() }),
            refused(
                &["refs/heads/main does not exist; nothing was written and no ref moved"],
                "the repository's branches",
                "create refs/heads/main, then repeat the same command",
            ),
        ),
        (
            WithdrawError::Git(GitError::refusal(vec!["rev-parse".to_string()], "fatal: gitio text")),
            failed(
                "git failed while reading the repository's common dir or building the op that deletes the squash \
                 ref",
            ),
        ),
        (
            WithdrawError::Oplog(OplogError::Malformed { line: Some(3), reason: "oplog text".to_string() }),
            failed("the op log could not be read or written"),
        ),
        (
            WithdrawError::State(LandingStateError::Read {
                path: PathBuf::from("/r/.git/mortar/landing.json"),
                source: std::io::Error::other("disk gone"),
            }),
            failed("reading /r/.git/mortar/landing.json failed"),
        ),
        (
            WithdrawError::Published {
                zone: "core".to_string(),
                commit: Oid::parse(A).unwrap(),
                origin: Oid::parse(B).unwrap(),
            },
            refused(
                &[
                    "squash 1111111111111111111111111111111111111111 of zone core is already in origin (at \
                     2222222222222222222222222222222222222222): it was published and cannot be withdrawn",
                    "nothing was moved",
                ],
                "the squash ref",
                "commit new work on zone core, then mr squash core --actor <name> starts a new change",
            ),
        ),
        (
            WithdrawError::Landing {
                zone: "core".to_string(),
                commit: Oid::parse(A).unwrap(),
                change_id: ChangeId::parse(CHANGE).unwrap(),
            },
            refused(
                &[
                    "squash 1111111111111111111111111111111111111111 of zone core is the landing in flight (change \
                     kkkkllllmmmmnnnnooooppppqqqqrrrr), so it cannot be withdrawn",
                    "nothing was moved",
                ],
                "the landing in flight",
                "mr back-out --actor <name>, then mr withdraw core --actor <name>",
            ),
        ),
        (
            WithdrawError::OpFailed(record(OpPhase::Failed)),
            refused(
                &["op op-7 moved nothing: the squash ref was not at its expected value"],
                "the squash ref",
                "repeat the same command; it reads the refs afresh",
            ),
        ),
        (
            WithdrawError::OpRepeated(record(OpPhase::Done)),
            refused(
                &["op key land-withdraw:refs/mortar/squash/core:op-7 is already done in the op log, so nothing moved"],
                "the op log",
                "repeat the same command",
            ),
        ),
    ];
    for (error, expected) in rows {
        assert_eq!(withdraw_error_report("withdraw", &error), expected, "{error:?}");
    }
}
