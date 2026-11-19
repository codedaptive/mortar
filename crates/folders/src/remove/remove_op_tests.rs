//! The remove op's every field, from literals, with and without a snapshot ref.

use gitio::Oid;
use oplog::{Op, RefMove};

use super::remove_op;

const TIP: &str = "0123456789abcdef0123456789abcdef01234567";
const SNAP: &str = "fedcba9876543210fedcba9876543210fedcba98";
const ZERO: &str = "0000000000000000000000000000000000000000";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

#[test]
fn deletes_the_branch_alone_with_no_snapshot_ref() {
    let op = remove_op(
        "op-1".to_string(),
        "agent-a".to_string(),
        1763030400,
        "refs/heads/req/folders/rf5",
        &oid(TIP),
        None,
    )
    .unwrap();
    assert_eq!(
        op,
        Op {
            op_id: "op-1".to_string(),
            key: "folders:remove:refs/heads/req/folders/rf5@0123456789abcdef0123456789abcdef01234567"
                .to_string(),
            kind: "folder-remove".to_string(),
            actor: "agent-a".to_string(),
            reason: "remove folder branch refs/heads/req/folders/rf5".to_string(),
            at: 1763030400,
            moves: vec![RefMove {
                refname: "refs/heads/req/folders/rf5".to_string(),
                old: Some(oid(TIP)),
                new: oid(ZERO),
            }],
        }
    );
}

#[test]
fn deletes_the_branch_then_the_snapshot_ref() {
    let op = remove_op(
        "op-2".to_string(),
        "agent-b".to_string(),
        7,
        "refs/heads/req/folders/r1",
        &oid(TIP),
        Some(("refs/mortar/snap/folders/r1", &oid(SNAP))),
    )
    .unwrap();
    assert_eq!(
        op,
        Op {
            op_id: "op-2".to_string(),
            key: "folders:remove:refs/heads/req/folders/r1@0123456789abcdef0123456789abcdef01234567".to_string(),
            kind: "folder-remove".to_string(),
            actor: "agent-b".to_string(),
            reason: "remove folder branch refs/heads/req/folders/r1".to_string(),
            at: 7,
            moves: vec![
                RefMove {
                    refname: "refs/heads/req/folders/r1".to_string(),
                    old: Some(oid(TIP)),
                    new: oid(ZERO),
                },
                RefMove {
                    refname: "refs/mortar/snap/folders/r1".to_string(),
                    old: Some(oid(SNAP)),
                    new: oid(ZERO),
                },
            ],
        }
    );
}

#[test]
fn a_sha256_tip_is_deleted_by_64_zeros() {
    let tip = oid("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef");
    let op = remove_op("op-3".to_string(), "agent-a".to_string(), 1, "refs/heads/req/a/b", &tip, None).unwrap();
    assert_eq!(op.moves[0].new, oid("0000000000000000000000000000000000000000000000000000000000000000"));
}
