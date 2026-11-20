use crate::save_op::save_op;
use gitio::Oid;
use oplog::{Op, RefMove};

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

#[test]
fn moves_the_branch_from_head_to_the_save_commit() {
    let head = oid("eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee");
    let commit = oid("dddddddddddddddddddddddddddddddddddddddd");
    let op = save_op("refs/heads/zone/snapshot", &head, &commit, "agent-a", "op-1".to_string(), 1700000000);
    assert_eq!(
        op,
        Op {
            op_id: "op-1".to_string(),
            key: "save refs/heads/zone/snapshot op-1".to_string(),
            kind: "save".to_string(),
            actor: "agent-a".to_string(),
            reason: "save".to_string(),
            at: 1700000000,
            moves: vec![RefMove {
                refname: "refs/heads/zone/snapshot".to_string(),
                old: Some(oid("eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee")),
                new: oid("dddddddddddddddddddddddddddddddddddddddd"),
            }],
        }
    );
}
