use crate::snapshot_op::snapshot_op;
use gitio::Oid;
use oplog::{Op, RefMove};

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

#[test]
fn first_snapshot_moves_a_ref_that_must_not_exist() {
    let commit = oid("dddddddddddddddddddddddddddddddddddddddd");
    let op = snapshot_op("refs/mortar/snap/snapshot/r1", None, &commit, "agent-a", "op-1".to_string(), 1700000000);
    assert_eq!(
        op,
        Op {
            op_id: "op-1".to_string(),
            key: "snapshot refs/mortar/snap/snapshot/r1 op-1".to_string(),
            kind: "snapshot".to_string(),
            actor: "agent-a".to_string(),
            reason: "snapshot".to_string(),
            at: 1700000000,
            moves: vec![RefMove {
                refname: "refs/mortar/snap/snapshot/r1".to_string(),
                old: None,
                new: oid("dddddddddddddddddddddddddddddddddddddddd"),
            }],
        }
    );
}

#[test]
fn later_snapshot_moves_the_ref_from_the_previous_commit() {
    let previous = oid("eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee");
    let commit = oid("dddddddddddddddddddddddddddddddddddddddd");
    let op = snapshot_op("refs/mortar/snap/z/r", Some(&previous), &commit, "agent-b", "op-2".to_string(), 42);
    assert_eq!(
        op,
        Op {
            op_id: "op-2".to_string(),
            key: "snapshot refs/mortar/snap/z/r op-2".to_string(),
            kind: "snapshot".to_string(),
            actor: "agent-b".to_string(),
            reason: "snapshot".to_string(),
            at: 42,
            moves: vec![RefMove {
                refname: "refs/mortar/snap/z/r".to_string(),
                old: Some(oid("eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee")),
                new: oid("dddddddddddddddddddddddddddddddddddddddd"),
            }],
        }
    );
}
