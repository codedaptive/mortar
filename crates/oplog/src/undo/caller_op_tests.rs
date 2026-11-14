//! caller_op: every field of the op comes from the caller or the arguments, unchanged.

use gitio::Oid;

use super::{caller_op, OpCaller};
use crate::store::{Op, RefMove};

#[test]
fn fields_are_copied_as_given() {
    let caller = OpCaller {
        key: "undo:zone-a:7".to_string(),
        actor: "simms".to_string(),
        reason: "bad squash".to_string(),
    };
    let moves = vec![RefMove {
        refname: "refs/heads/landing".to_string(),
        old: Some(Oid::parse("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb").unwrap()),
        new: Oid::parse("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").unwrap(),
    }];
    let op = caller_op(&caller, "undo", "op-9".to_string(), 1763030400, moves);
    assert_eq!(
        op,
        Op {
            op_id: "op-9".to_string(),
            key: "undo:zone-a:7".to_string(),
            kind: "undo".to_string(),
            actor: "simms".to_string(),
            reason: "bad squash".to_string(),
            at: 1763030400,
            moves: vec![RefMove {
                refname: "refs/heads/landing".to_string(),
                old: Some(Oid::parse("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb").unwrap()),
                new: Oid::parse("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").unwrap(),
            }],
        }
    );
}
