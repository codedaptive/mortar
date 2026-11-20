//! The branch op's every field, from literals.

use gitio::Oid;
use oplog::{Op, RefMove};

use super::branch_op;

const COMMIT: &str = "0123456789abcdef0123456789abcdef01234567";

#[test]
fn builds_every_field() {
    let commit = Oid::parse(COMMIT).unwrap();
    let op = branch_op(
        "op-1".to_string(),
        "agent-a".to_string(),
        1763030400,
        "req/folders/rf1",
        &commit,
    );
    assert_eq!(
        op,
        Op {
            op_id: "op-1".to_string(),
            key: "folders:branch:req/folders/rf1@0123456789abcdef0123456789abcdef01234567".to_string(),
            kind: "folder-branch".to_string(),
            actor: "agent-a".to_string(),
            reason: "make folder branch req/folders/rf1".to_string(),
            at: 1763030400,
            moves: vec![RefMove {
                refname: "refs/heads/req/folders/rf1".to_string(),
                old: None,
                new: Oid::parse(COMMIT).unwrap(),
            }],
        }
    );
}

#[test]
fn another_commit_gives_another_key() {
    let other = Oid::parse("fedcba9876543210fedcba9876543210fedcba98").unwrap();
    let op = branch_op("op-2".to_string(), "agent-b".to_string(), 7, "zone/folders", &other);
    assert_eq!(op.key, "folders:branch:zone/folders@fedcba9876543210fedcba9876543210fedcba98");
}
