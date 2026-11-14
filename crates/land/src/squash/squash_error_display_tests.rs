//! Display for SquashError's refusals: each names what it carries and says what moved.
//! The Git, Oplog, Trailers and ChangeId texts embed gitio's or oplog's own failure text, so they are left to
//! Nagatha's cross-zone tests.

use super::SquashError;
use gitio::Oid;
use oplog::{OpPhase, OpRecord, OpResult};

/// The one helper building values only a constructor makes (Oid::parse).
fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn record(phase: OpPhase) -> OpRecord {
    OpRecord {
        op_id: "op-7".to_string(),
        key: "land-squash:refs/mortar/squash/land:none:k:op-7".to_string(),
        kind: "land-squash".to_string(),
        actor: "42@host".to_string(),
        reason: "r".to_string(),
        at: 9,
        phase,
        moves: Vec::new(),
        result: Some(OpResult::Moved),
    }
}

#[test]
fn ref_missing_names_the_ref_and_says_nothing_moved() {
    let text = SquashError::RefMissing { refname: "refs/heads/zone/land".to_string() }.to_string();
    assert_eq!(text, "ref refs/heads/zone/land does not exist; no commit was written and no ref was moved");
}

#[test]
fn attr_source_missing_names_the_commit_and_says_nothing_moved() {
    let commit = oid("1111111111111111111111111111111111111111");
    let text = SquashError::AttrSourceMissing { commit }.to_string();
    assert!(
        text.starts_with("attribute source commit 1111111111111111111111111111111111111111 does not exist"),
        "{text}"
    );
    assert!(text.ends_with("no commit was written and no ref was moved"), "{text}");
}

#[test]
fn unrelated_names_both_tips_and_says_nothing_moved() {
    let origin = oid("1111111111111111111111111111111111111111");
    let source = oid("2222222222222222222222222222222222222222");
    let text = SquashError::Unrelated { origin, source }.to_string();
    assert!(
        text.starts_with(
            "zone tip 2222222222222222222222222222222222222222 shares no history with origin \
             1111111111111111111111111111111111111111"
        ),
        "{text}"
    );
    assert!(text.contains("no ref was moved"), "{text}");
}

#[test]
fn op_failed_names_the_op_and_its_key_and_says_nothing_moved() {
    let text = SquashError::OpFailed(record(OpPhase::Failed)).to_string();
    assert!(text.contains("op op-7 (key land-squash:refs/mortar/squash/land:none:k:op-7) failed"), "{text}");
    assert!(text.contains("not at its expected value"), "{text}");
    assert!(text.contains("nothing was moved"), "{text}");
}

#[test]
fn op_repeated_names_the_key_and_says_nothing_moved() {
    let text = SquashError::OpRepeated(record(OpPhase::Done)).to_string();
    assert!(text.contains("op key land-squash:refs/mortar/squash/land:none:k:op-7 is already done"), "{text}");
    assert!(text.contains("this squash moved nothing"), "{text}");
}
