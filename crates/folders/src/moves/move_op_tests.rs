//! The move op's every field, from literals.

use gitio::Oid;
use oplog::{Op, RefMove};

use super::move_op;

const OLD: &str = "0123456789abcdef0123456789abcdef01234567";
const NEW: &str = "fedcba9876543210fedcba9876543210fedcba98";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

#[test]
fn builds_every_field() {
    let op = move_op(
        "op-1".to_string(),
        "bilby".to_string(),
        1763030400,
        "refs/heads/req/folders/WO-folders-4",
        &oid(OLD),
        &oid(NEW),
    );
    assert_eq!(
        op,
        Op {
            op_id: "op-1".to_string(),
            key: "folders:move:refs/heads/req/folders/WO-folders-4@0123456789abcdef0123456789abcdef01234567..\
                  fedcba9876543210fedcba9876543210fedcba98"
                .to_string(),
            kind: "folder-move".to_string(),
            actor: "bilby".to_string(),
            reason: "move folder branch refs/heads/req/folders/WO-folders-4 to \
                     fedcba9876543210fedcba9876543210fedcba98"
                .to_string(),
            at: 1763030400,
            moves: vec![RefMove {
                refname: "refs/heads/req/folders/WO-folders-4".to_string(),
                old: Some(oid(OLD)),
                new: oid(NEW),
            }],
        }
    );
}

#[test]
fn another_new_tip_gives_another_key() {
    let other = oid("1111111111111111111111111111111111111111");
    let op = move_op("op-2".to_string(), "simms".to_string(), 7, "refs/heads/zone/folders", &oid(OLD), &other);
    assert_eq!(
        op.key,
        "folders:move:refs/heads/zone/folders@0123456789abcdef0123456789abcdef01234567..\
         1111111111111111111111111111111111111111"
    );
}
